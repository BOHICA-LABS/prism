---
document_type: architecture-assessment
title: "SQL vs PrismQL — What It Takes to Drop PrismQL and Adopt Pure SQL"
version: "1.0"
created: 2026-09-18
authors: [architect]
status: draft
traces_to: ".factory/specs/architecture/ARCH-INDEX.md"
scope: assessment-only — no ADR, story, code, or spec amendment produced here
cycle: wave-5-e-demo-fidelity
---

# SQL vs PrismQL — Architecture Assessment

**ASSESSMENT ONLY.** No decision made, no implementation started, no ADR authored,
no story created. This document exists to give the human a grounded feasibility picture.

---

## A. Executive Summary

**Feasibility verdict:** Technically feasible with high engineering cost and non-trivial
risk. Not a simplification — a substitution of one complexity for another.

**The single biggest gain:** LLM agents (Claude in Claude Code) write standard SQL today
without any teaching surface. Dropping PrismQL's custom DSL removes the vocabulary gap
that ADR-041 built an entire 4-layer teaching mechanism to bridge.

**The single biggest risk:** The PrismQL parser is not just a query language — it is the
security perimeter enforcement layer. Dropping Chumsky and accepting raw SQL through
DataFusion's native SQL frontend expands the exploitable surface from a curated ~250 token
grammar to the full DataFusion SQL dialect (CTEs, window functions, arbitrary subqueries,
arbitrary function calls, arbitrary type casts). Every gate currently enforced at parse
time (size, depth, regex validation, CIDR validation, list item limits, aggregate-in-
predicate guard, temporal literal validation, table/column/UDF availability gates,
VP-014/VP-015 Kani-proven properties) must be re-implemented on the new surface —
against a parser whose internals are not under prism's control.

**The PARADOX:** The most honest statement about a "pure SQL" migration is this: to make
raw DataFusion SQL safe for agent-facing use in a multi-tenant MSSP product, you must
restrict it back down to a safe subset. That restricted subset has a substantial overlap
with what PrismQL's plan gates already enforce. The net simplification is real but
smaller than it appears from the outside.

**Hybrid recommendation framing:** A SQL-as-additional-frontend approach — accepting
standard SQL through the same PrismQL plan-gate stack — captures roughly 80% of the
gain (agent familiarity) at a fraction of the cost and risk of removing PrismQL.
The human's call.

---

## B. Current State — What PrismQL Is and Where It Sits vs DataFusion

### B.1 Architecture Position (verified in code)

PrismQL lives in `crates/prism-query/` and is invoked exclusively via
`PrismQlParser::parse` (exposed from `filter_parser.rs`). The public security entry
point enforces five pre-AST gates before any query work:

1. `check_query_size` — rejects inputs > 64KB (VP-014, Kani proven, commit f5212641)
2. `check_paren_depth` — rejects nesting depth > 64 (VP-015, Kani proven, commit
   f5212641)
3. Snapshotted `ParseLimits` thread-local — eliminates TOCTOU race between env-var reads
   (F-HIGH-001 / F-HIGH-002 / BC-2.11.006)
4. Mode detection — dispatches to `parse_filter`, `parse_sql`, or `parse_pipe`
5. Write-verb gating — routes pipe write-mode through `parse_pipe_with_write` and rejects
   write verbs in filter mode

Sub-parsers (`parse_filter`, `parse_sql`, `parse_pipe` and all their builder factories)
are `pub(crate)`. The compile-fail test at `tests/external/perimeter-violation/` enforces
this boundary — CI blocks any commit that accidentally promotes a sub-parser symbol to
`pub`. This boundary is `INV-SEC-PERIMETER-001`.

**Critical architectural fact:** PrismQL does NOT have its own execution engine.
All three modes lower to a DataFusion SQL string that is passed to
`SessionContext::sql()`. The flow is:

```
PrismQL string
  → PrismQlParser::parse  (security gates + Chumsky AST)
  → Ast enum (Filter / Sql / Pipe / SqlPipe)
  → plan gates (E-QUERY-037 through E-QUERY-045 chain, engine.rs)
  → pipe_sql_emitter::pipe_to_executable_sql / predicate_to_datafusion_sql  [pipe/filter]
  → session_ctx.sql(datafusion_sql_string)  [all modes]
  → DataFusion logical plan → execution → Arrow RecordBatches
```

PrismQL's SQL mode is already restricted SQL. It accepts a `SELECT ... FROM ... WHERE
... GROUP BY ... ORDER BY ... LIMIT` shape, parses it into a typed `SqlQuery` AST,
validates it through the plan-gate chain, and passes the result to DataFusion. The
question "drop PrismQL, adopt pure SQL" is therefore: remove the gating layer and
pass the query string directly to `session_ctx.sql()`.

### B.2 Four PrismQL Modes (CURRENT — verified in `ast.rs`)

| Mode | Entry | Example syntax | Lowers to |
|------|-------|----------------|-----------|
| Filter | `filter_parser.rs` | `severity = high AND time > 24h` | DataFusion WHERE clause via `predicate_to_datafusion_sql` |
| SQL | `sql_parser.rs` | `SELECT * FROM crowdstrike_detections WHERE severity_id >= 4` | DataFusion SQL via `pipe_sql_emitter` |
| Pipe | `pipe_parser.rs` | `FROM crowdstrike_detections \| where severity IEQ 'high' \| limit 50` | DataFusion SQL via `pipe_to_executable_sql` |
| SqlPipe | `pipe_parser.rs` | `SELECT * FROM t \| where severity IEQ 'high' \| limit 50` | DataFusion SQL via `sqlpipe_to_executable_sql` |

### B.3 Security Perimeter Inventory (CURRENT — verified in `security.rs`, `ast.rs`,
`engine.rs`)

**Parse-time gates (pure, synchronous, inside `PrismQlParser::parse`):**

| Gate | Constant | Module | VP |
|------|----------|--------|----|
| Query size ≤ 64KB | `PRISM_MAX_QUERY_SIZE = 65536` | `security.rs` | VP-014 (Kani proven) |
| Nesting depth ≤ 64 | `PRISM_MAX_NESTING_DEPTH = 64` | `security.rs` | VP-015 (Kani proven) |
| Pipe stages ≤ 32 | `PRISM_MAX_PIPE_STAGES = 32` | `security.rs` | — |
| Regex pattern ≤ 1024 bytes | `PRISM_MAX_REGEX_PATTERN_LEN = 1024` | `security.rs` | — |
| List items ≤ 1024 | `PRISM_MAX_LIST_ITEMS = 1024` | `security.rs` | — |
| Regex compile validation (CWE-1333) | finite automaton only | `ast.rs::RegexLiteral::new` | VP-021 (fuzz) |
| CIDR parse validation (CWE-20) | `ipnet::IpNet::from_str` | `ast.rs::CidrLiteral::new` | — |
| Integer overflow check | `DurationLiteral::new` | `ast.rs` | — |
| Parser no-panic | fuzz corpus | `vp021_parse_fuzz.rs` | VP-021 |
| ParseLimits TOCTOU elimination | snapshot before combinators | `security.rs` | — |

**Plan-time gates (sequential, inside `engine.rs::execute_inner`):**

| Gate | Error | Condition |
|------|-------|-----------|
| Aggregate-in-predicate | E-QUERY-001 (ADR-048) | `count(*)` used in WHERE position |
| Temporal literal validation | E-QUERY-041 / E-QUERY-042 | Invalid timestamp forms (ADR-052) |
| Table availability | E-QUERY-037 | Table not registered for org |
| Column availability | E-QUERY-038 | Column not in table schema |
| Enrich UDF availability | E-QUERY-039 | Unknown infusion name |
| json_extract literal-key | E-QUERY-045 | Non-literal key arg (ADR-066) |
| Audit table capability | E-QUERY-011 | prism_audit access without AuditRead capability |
| Org scoping | E-QUERY-032 | Sensor not registered for requesting org |
| Redundant limit | E-QUERY-040 | SQL LIMIT + pipe limit both present (ADR-043) |

**PrismQL-specific operators (no SQL equivalent, CWE-relevant):**

| Operator | Variant in AST | Notes |
|----------|---------------|-------|
| `CONTAINS` / `ICONTAINS` | `Predicate::StringOp` | case-insensitive variants |
| `STARTSWITH` / `ENDSWITH` and I* variants | `Predicate::StringOp` | |
| `HAS field` | `Predicate::Has` | field existence check |
| `MISSING field` | `Predicate::Missing` | field absence check |
| `field IN CIDR "10.0.0.0/8"` | `Predicate::Cidr` | CWE-20 validated at parse |
| `field =~ "regex"` / `MATCHES` | `Predicate::Regex` | CWE-1333 validated at parse |
| `time > 24h` | duration literal `DurationLiteral` | prism-specific duration type |
| `IEQ` / `INE` / `IIN` | `case_insensitive: bool` on Compare/In | ADR-047 |
| Wildcard promotion `field = "10.0.*"` | `Predicate::Wildcard` | auto-promoted |

**Composite virtual sources (CURRENT — grammar-only, not executing):**

`FROM EVENTS`, `FROM ALERTS`, `FROM DEVICES`, `FROM ASSETS`, `FROM SESSIONS` are
classified as `SourceRefKind::Composite` at parse time but fail with `E-QUERY-036`
(UnknownSourceTable) at execution time. The execution routing in `resolve_source_refs`
(verified in `federated-search-analysis.md` §B.1) treats the composite name as a
sensor prefix and fails when no sensor named "events" is registered. These composite
sources are today grammar promises, not implemented semantics.

**Virtual fields (CURRENT — `ast.rs`, BC-2.11.012):**

`_sensor`, `_client`, `_source_table`, `_source_type` — promoted from field paths to
typed `Expr::VirtualField` enum at parse time, injected into every RecordBatch during
fan-out materialization.

### B.4 What Is Attached to the PrismQL Surface (BC/VP count)

- **25 BCs** in the BC-2.11.NNN namespace, all tracing to PrismQL query subsystem:
  BC-2.11.001 (query MCP tool), BC-2.11.002..004 (three parser modes), BC-2.11.005
  (materialization), BC-2.11.006 (security limits), BC-2.11.007 (pushdown),
  BC-2.11.008..015 (alias subsystem — shares parser surface), BC-2.11.016..025
  (plan-time gates, case-insensitive operators, temporal grammar, etc.)
- **VPs in prism-query:** VP-014 (size, Kani), VP-015 (depth, Kani), VP-021 (no-panic,
  fuzz), VP-025 (cache key), VP-031 (required column), VP-037 (alias expansion),
  VP-162 (json_extract null safety)
- **~1501 tests** in `crates/prism-query/src/` (grep-counted `#[test]` / `#[tokio::test]`
  annotations); plus ~12,100 lines of inline test code in `engine.rs` (comment on file
  structure from CLAUDE.md §File size section)
- **ADR-041** — the entire 4-layer LLM teaching surface (prism_describe, prismql://reference
  resource, normalized_pql echo, E-QUERY pedagogical payloads) was designed specifically
  because PrismQL is not in any LLM's training corpus

---

## C. What "Adopt Pure SQL, Drop PrismQL" Concretely Requires

"Drop PrismQL and accept pure SQL" means: remove the Chumsky parser layer and pass the
query string from the MCP `query` tool directly to `SessionContext::sql()`. Below is the
work itemized by subsystem.

### C.1 Parser / Grammar Removal

**Remove:**
- `filter_parser.rs`, `sql_parser.rs`, `pipe_parser.rs`, `pipe_sql_emitter.rs`,
  `ast.rs`, `error.rs`, `error_recovery.rs`, `security.rs`, `visit.rs`
- Fuzz target `vp021_parse_fuzz.rs`
- Compile-fail test `tests/external/perimeter-violation/`
- Chumsky 0.12 dependency (one fewer compile-time dependency, meaningful build time
  win)

**Effort:** Small relative to total — removing code is faster than writing it.

### C.2 Security Perimeter Re-establishment on DataFusion SQL Surface

This is the expensive part. Each PrismQL gate must be reimplemented against DataFusion's
SQL AST (via `sqlparser-rs`) or as a LogicalPlan visitor AFTER DataFusion parses the
query.

| Current PrismQL gate | Re-implement on DataFusion SQL as | Complexity |
|---------------------|-----------------------------------|------------|
| 64KB size limit | Pre-parse byte length check on raw string | Trivial (same code) |
| Nesting depth 64 | Recursive AST visitor on sqlparser-rs `Statement` | Medium — no formal proof yet; VP-015 Kani proofs must be re-anchored to new target |
| Pipe stages ≤ 32 | N/A (pipe mode gone) | — |
| Regex length ≤ 1024 | Visitor on `Expr::Like` / `Expr::SimilarTo` | Medium — must find all LIKE/SIMILAR TO positions |
| List items ≤ 1024 | Visitor on `Expr::InList` | Medium |
| Regex compile validation (CWE-1333) | Extract pattern literals and call `regex::Regex::new` | Medium |
| CIDR validation (CWE-20) | `field IN CIDR` has no SQL equivalent — operator is gone or becomes a UDF call | Hard — either drop the operator or implement as UDF with validation in UDF body |
| Integer/duration overflow | Duration literals don't exist in standard SQL | Duration becomes explicit arithmetic: `timestamp > NOW() - INTERVAL '7' DAY` (standard); lose `time > 24h` sugar |
| TOCTOU ParseLimits snapshot | Still needed on the new gate layer | Medium |
| ParseLimits thread-local API restrictions | Remove perimeter-violation test; new equivalent for DataFusion AST visitor exports | Medium |

VP-014 and VP-015 Kani proofs are currently anchored to `PrismQlParser::parse` and
`check_query_size` / `check_paren_depth` in `security.rs`. After removing those
functions, both proofs become dead. New Kani proof harnesses must be written targeting
the replacement gate functions on the DataFusion sqlparser-rs AST. This is feasible
(the proof strategy is identical) but requires meaningful formal-verifier time.

**VP-021 (fuzz):** The fuzz target exercises `PrismQlParser::parse`. After removal, the
fuzz target must be retargeted at the new SQL-accepting entry point. Non-trivial: the
fuzz corpus accumulated against the Chumsky grammar is not valid input for a
sqlparser-rs-gated entry point.

### C.3 Plan-Gate Migration

The nine plan-time gates in `engine.rs::execute_inner` (E-QUERY-037 through E-QUERY-045
chain) do NOT depend on the PrismQL parser. They operate on:
- `TableRegistry` (for E-QUERY-037)
- Column metadata from sensor TOML specs (for E-QUERY-038)
- `InfusionRegistry` (for E-QUERY-039)
- `json_extract_string` arg AST inspection (for E-QUERY-045)

These gates must be reimplemented to inspect the DataFusion `LogicalPlan` instead of
the PrismQL AST. The table availability and column availability checks are
straightforward (DataFusion surfaces the FROM clause and SELECT column references in
the plan). The E-QUERY-045 literal-key gate requires walking the `LogicalPlan` to find
`json_extract_string(col, arg)` calls and verifying `arg` is a literal — equivalent
complexity to the current PrismQL AST walk.

**The aggregate-in-predicate gate (E-QUERY-001, ADR-048):** DataFusion itself would
catch many illegal-aggregate-in-WHERE conditions as planner errors, but the error
message would be DataFusion-internal, not a structured E-QUERY-001 pedagogical payload.
If pedagogical error payloads are to be preserved, a pre-plan validation pass is still
required.

### C.4 Semantic Gaps — Features That Cannot Survive the Migration Unchanged

| PrismQL feature | SQL equivalent | Impact |
|----------------|----------------|--------|
| `time > 24h` duration literal | `timestamp > NOW() - INTERVAL '24' HOUR` | Loss of ergonomics; SQL is more verbose |
| `HAS field` | `field IS NOT NULL` | Equivalent semantics; different syntax |
| `MISSING field` | `field IS NULL` | Equivalent semantics; different syntax |
| `field IN CIDR "10.0.0.0/8"` | `subnet_contains('10.0.0.0/8', field)` or custom operator | CIDR parse validation at parse time is gone; moves to UDF runtime |
| `field =~ "regex"` | `regexp_match(field, 'regex')` | Regex CWE-1333 validation at parse time is gone; moves to runtime |
| `IEQ` / `IIN` / `INE` | `lower(field) = lower('value')` or `ILIKE` | Different syntax; must document in teaching surface |
| Filter mode (bare predicates) | Requires `SELECT * FROM <table> WHERE ...` | Loss of shorthand; agents write more verbose queries |
| `FROM EVENTS` composite sources | Cannot work until composite-source routing is implemented | Same gap as today; SQL doesn't solve it |
| Pipe mode stages | SQL window functions / CTEs are more verbose equivalents | Loss of analyst-friendly pipe ergonomics |
| `normalized_pql` echo field | `normalized_sql` echo (same concept; DataFusion would emit the canonical SQL) | Neutral or positive — SQL echo is already familiar to the model |

### C.5 Error Taxonomy Migration (E-QUERY-NNN)

The error taxonomy in `prism-core/src/error.rs` contains ~30 E-QUERY-NNN variants.
Parser-specific errors (E-QUERY-001 parse failure) would be replaced by sqlparser-rs /
DataFusion error types, which are not structured. The 25 BCs in BC-2.11.NNN that cite
E-QUERY-NNN codes as postconditions would each need review: some codes survive (plan-time
gates), some disappear (parse-mode codes), some need SQL equivalents authored.

### C.6 MCP `query` Tool Surface

The `query` tool description in `server.rs` currently embeds PQL primer text (clause
vocabulary, pipe mode hint, three schema-agnostic skeletons). Under pure SQL, this
description simplifies: drop the PQL primer paragraph, change the parameter doc from
"PrismQL query string" to "SQL query string." The tool still needs the discovery
instructions ("call `prism_describe` first") and the error code table. Net change: tool
description becomes simpler — a win.

The `normalized_pql` field in the query response (BC-2.11.018, ADR-041 §Echo-Normalized-
PQL-Back) becomes `normalized_sql`: DataFusion can echo back its canonical SQL form.
Neutral change.

### C.7 Effort Magnitude Estimate

| Work stream | Effort estimate | Blocking on |
|------------|-----------------|-------------|
| Remove Chumsky parser crates | 1 sprint (decompose, retarget) | Nothing |
| Reimplement security gates on sqlparser-rs AST | 2-3 sprints | Gate design decisions |
| Re-anchor VP-014/015 Kani proofs | 1 sprint | formal-verifier |
| Retarget VP-021 fuzz target | 0.5 sprints | New entry point stabilized |
| Migrate engine.rs plan gates to LogicalPlan inspection | 2 sprints | DataFusion API study |
| Migrate ~1501 prism-query tests (parser tests become SQL parser tests) | 3-4 sprints | Migration guide |
| Update 25 BC-2.11.NNN contracts | 1 sprint | Product-owner + architect |
| Update ADRs: ADR-041, ADR-043..048, ADR-052, ADR-066 minimum | 1 sprint | Architect |
| ADR-041 4-layer teaching surface revision | 0.5 sprints | Simpler, not longer |

**Total honest estimate:** 12-15 sprints (story-weeks equivalent), touching ~25 BCs,
~10 ADRs, ~1500 tests, 2 Kani proofs, and 1 fuzz target.

---

## D. What We GAIN

### D.1 LLM / Agent SQL Familiarity (the primary gain — directly tied to ADR-041)

ADR-041 was written because "PrismQL (PQL) is a custom DSL that is not part of any LLM's
training corpus." The 4-layer teaching surface (L1 primer in tool description + L2
prism_describe discovery + L3 prismql://reference resource + L4 E-QUERY pedagogical
errors) exists entirely to bridge the gap between the model's SQL fluency and PrismQL's
custom vocabulary.

Under pure SQL, the 4-layer teaching surface collapses to 2 layers: L2 (schema
discovery — still needed, since prism tables are not in the model's corpus) and L4
(plan-time errors — still needed). L1 and L3 become dramatically simpler: instead of
embedding a grammar primer in the tool description, the tool description says "use
standard SQL."

**Quantified impact from ADR-041 research:** DIN-SQL lifted Spider SOTA +5.4pt;
MAC-SQL lifted BIRD +13.24pt; Self-Debugging lifted hardest queries +9pt. These gains
were achieved with SQL — a pretrained-known language. The baseline for PrismQL is lower
because the model must first learn the vocabulary. The research confirms the hypothesis:
a model writing SQL against a described schema is more accurate than a model learning a
custom DSL.

### D.2 Reduced Custom Parser Maintenance Burden

The Chumsky 0.12 parser is 10+ source files in `prism-query`. Every new operator or
clause (see ADR-044 temporal grammar, ADR-047 case-insensitive operators, ADR-048
aggregate predicates, ADR-052 temporal typing, ADR-056 pagination, ADR-057 push-down
grammar, ADR-066 json_extract) required extending the Chumsky grammar and adding
corresponding tests. SQL gets these features for free from DataFusion's SQL dialect.
Rough count: 7 grammar-extending ADRs since initial parser implementation. Under pure
SQL, these ADRs never need to be written.

### D.3 Standard Tooling / Ecosystem

- sqlparser-rs / DataFusion SQL are documented, tested, and maintained by the broader
  Apache Arrow community
- IDE syntax highlighting, query formatters, query validators for SQL exist and work
- The community knows how to debug DataFusion SQL errors; PrismQL errors are
  prism-internal

### D.4 DataFusion Alignment

PrismQL already lowers to DataFusion SQL. Removing the intermediate representation
eliminates a translation layer that can drift as DataFusion's SQL dialect evolves.
New DataFusion SQL features (CTEs, window functions, UNNEST, etc.) become available
immediately — though for the agent-facing surface these must be gated to prevent
resource exhaustion, which loops back to the security perimeter concern.

### D.5 Onboarding / Documentation Ease

A new Prism user or contributor who understands SQL can immediately write queries.
PrismQL requires reading ADR-041 + prismql-grammar.md + four BC-2.11.002..004 contracts
to understand the query surface. This is a real cost for human operator adoption.

---

## E. What We LOSE / RISKS

### E.1 Security Perimeter Re-derivation on SQL's Larger Surface (the primary risk)

DataFusion's SQL dialect accepts: CTEs, arbitrary subqueries, window functions, UNNEST,
arbitrary scalar functions by name, arbitrary aggregate functions, arbitrary type casts,
CASE expressions, ANY/ALL quantifiers, PIVOT, arbitrary CROSS JOINs without ON clause,
recursive CTEs (depending on DataFusion version), scalar subqueries, correlated
subqueries, and multi-join chains.

PrismQL's parser accepts NONE of these because they are not in the Chumsky grammar. The
security perimeter is enforced at parse time by what the grammar recognizes.

Under raw DataFusion SQL, the engine would plan and attempt to execute all of the above.
The GreedyMemoryPool (BC-2.11.006) and the 30s timeout gate resource-exhaust conditions
at execution time — but the attack surface for prompt injection via SQL injection is:

- **Arbitrary UDF calls:** `SELECT eval('DROP TABLE ...')` or equivalent — DataFusion
  would reject unknown UDFs, but this requires DataFusion's error, not prism's gate
- **Infinite CROSS JOIN:** `SELECT * FROM crowdstrike_detections CROSS JOIN armis_alerts`
  — produces up to 10K × 10K = 100M rows before the materialization cap; DataFusion's
  memory pool would eventually fire but only after significant CPU work
- **Recursive CTE / deep subquery chains:** Resource-exhaustion denial-of-service;
  depth limit becomes harder to enforce on recursive SQL structures
- **Schema inference attacks:** `SELECT table_name FROM information_schema.tables` — if
  DataFusion registers information_schema, this leaks the prism internal table registry
  to the querying model/user

None of these attacks work against PrismQL today because the Chumsky grammar does not
parse CTEs, recursive structures, information_schema, or cross-join-without-on.

**The re-derivation work is non-trivial:** blocking CTEs, limiting subquery depth,
preventing information_schema access, and gating arbitrary function calls requires either
(a) a pre-plan whitelist/blacklist visitor or (b) a DataFusion `AnalyzerRule` that
inspects and rejects disallowed plan shapes. Either approach is implementable but requires
significant design work and new test coverage.

### E.2 Loss of Pipe Mode Ergonomics

Pipe mode (`FROM crowdstrike_detections | where severity IEQ 'high' | stats count by
device_hostname | sort count desc | head 10`) is concise and familiar to SOC analysts
who know tools like Splunk SPL or Microsoft KQL. The SQL equivalent is more verbose:

```sql
SELECT device_hostname, COUNT(*) AS count
FROM crowdstrike_detections
WHERE lower(severity) = lower('high')
GROUP BY device_hostname
ORDER BY count DESC
LIMIT 10
```

This is not a security argument — it is an operator experience argument. For a product
targeting MSSP analysts who may hand-write queries, pipe mode reduces cognitive overhead.

### E.3 Loss of PrismQL-Encoded Domain Semantics

**CIDR operator:** `device_ip IN CIDR "10.0.0.0/8"` is one token in PrismQL with
parse-time CWE-20 CIDR validation. In SQL it becomes `subnet_contains('10.0.0.0/8',
device_ip)` — a UDF call that moves validation to runtime. If the model hallucinates a
malformed CIDR string, the UDF fails at execution time instead of at parse time. This
is a worse failure mode for agent-facing surfaces.

**Regex operator:** Same issue. `field =~ "pattern"` validates the regex (CWE-1333) at
parse time. `regexp_match(field, 'pattern')` in DataFusion validates at execution time.
An LLM-generated malformed regex pattern reaches the execution layer before being caught.

**Duration literals:** `time > 24h` has no standard SQL equivalent. The SQL form is
`timestamp_col > NOW() - INTERVAL '24' HOUR`. The model must know the column name is
`timestamp_col` (not `time`) and use standard SQL interval syntax. This increases query
failure rates for agents, which loops back to the agent accuracy argument.

### E.4 Re-prove / Re-fuzz Cost

VP-014 and VP-015 are formally proven. Both proofs are anchored to functions in
`security.rs` that would be deleted. The new proof harnesses must target replacement
gate functions. Formal verification work is not free — VP-015 required `--no-unwinding-
checks --default-unwind 2` and four separate harnesses. VP-021's fuzz corpus is not
reusable against a new parser surface.

### E.5 Migration / Regression Risk

1501 parser tests in `prism-query` test PrismQL grammar edge cases. These tests
document the grammar's behavior — including BC-2.11.002 (filter mode), BC-2.11.003
(SQL mode), BC-2.11.004 (pipe mode), BC-2.11.024 (case-insensitive operators),
BC-2.11.021 (temporal grammar). Under pure SQL, many of these tests become:
(a) permanently deleted (pipe mode gone), (b) rewritten (SQL syntax changed), or
(c) kept as regression tests for the new gate layer. Category (c) is the smallest bucket.

### E.6 The Paradox — Net Simplification Is Real but Smaller Than Expected

To make pure DataFusion SQL safe for the prism agent surface, you must:

1. Re-implement the size gate
2. Re-implement the depth gate (on a more complex AST)
3. Re-implement regex validation on SQL LIKE / SIMILAR TO
4. Re-implement CIDR validation (now as UDF runtime, weaker)
5. Block CTEs, recursive queries, information_schema, arbitrary function calls
6. Re-implement the plan-gate chain on DataFusion LogicalPlan
7. Re-anchor VP-014/015 Kani proofs

Items 1-3 are simple. Items 4-7 are not. The honest account is that approximately half
of what PrismQL's gating layer provides must be re-derived on the SQL surface to maintain
equivalent security posture. The other half (pipe mode semantics, custom operators, error
taxonomy) is either lost or requires new engineering.

---

## F. Middle Paths

### F.1 SQL as Additional Frontend (Recommended for Human Consideration)

Accept standard SQL queries through the same PrismQL plan-gate stack by running the
SQL string through DataFusion's SQL parser → LogicalPlan → same E-QUERY gate chain.
PrismQL remains available for pipe mode and custom operators. The `query` tool accepts
both. Detection: if the input starts with `SELECT` (case-insensitive) and passes
DataFusion's SQL parser, route through DataFusion SQL path; otherwise route through
PrismQL.

**Gain captured:** LLM agents can write standard SQL without learning PrismQL. The
prism_describe discovery surface already generates standard SQL example queries
(ADR-041 L1 primer already shows SQL skeletons).

**Cost:** Implement the "DataFusion SQL path" guard layer (size check + a plan-visitor
that blocks CTEs, information_schema, and unbounded CROSSJOINs). Estimated 2-3 sprints.
No existing tests broken. No VPs invalidated.

**Risk:** Two parse paths to maintain, but the SQL path shares the plan-gate chain with
the PrismQL SQL path (both produce a DataFusion LogicalPlan to gate against).

### F.2 Keep Pipe Mode + Drop Filter Mode

Filter mode (`severity >= high AND time > 24h`) is the most ambiguous mode. Dropping it
and keeping SQL + Pipe mode reduces the grammar surface by ~30% while retaining pipe
ergonomics and SQL familiarity.

**Cost:** Moderate — filter mode tests deleted, BC-2.11.002 withdrawn.

### F.3 Pure SQL Core + PrismQL as Thin Sugar (the "deep hybrid")

Redesign the PrismQL parser as a transpiler: PrismQL surface syntax → canonical SQL
AST → existing DataFusion SQL execution path. The security layer operates on the SQL
AST after transpilation, and DataFusion executes the SQL. This is approximately what
the current `pipe_sql_emitter` already does for pipe mode. Formalizing this as "PrismQL
is sugar over SQL" rather than "PrismQL lowers to DataFusion SQL internally" changes the
architecture framing without changing the execution model.

This approach preserves pipe mode, custom operators, and the LLM teaching surface while
making the SQL-core nature explicit. No security perimeter change required.

---

## G. Interaction with the Ratified-but-Held Federation Work

The federated-search-analysis.md (wave-5-e-demo-fidelity cycle, 2026-09-18) documents
that composite sources (`FROM EVENTS`, `FROM ALERTS`, `FROM DEVICES`) are grammar-only
today — they fail at execution time with E-QUERY-036 because the execution routing does
not implement the OCSF-class to sensor-table fan-out.

**SQL does not solve this gap.** `FROM EVENTS` in standard SQL would fail at DataFusion
because no MemTable named "events" is registered. The composite-source routing problem
is an execution-layer problem, not a query-language problem. Whether the frontend is
PrismQL or standard SQL, implementing composite sources requires adding the OCSF-class
routing registry and the multi-table fan-out/unification layer — the same ~6-8 sprint
investment described in the federation gap analysis regardless of frontend language.

The SQL migration and the federation work are largely orthogonal. Neither blocks the
other. But conflating them — "switch to SQL to enable FROM EVENTS" — would be incorrect.

---

## H. Recommendation Framing and Open Questions

This section frames the options for the human decision maker. No decision is made here.

### H.1 Option Summary

| Option | Cost | Security delta | Agent UX delta | BC/VP impact |
|--------|------|---------------|----------------|-------------|
| **Status quo** (PrismQL) | 0 | — | Requires ADR-041 teaching surface | — |
| **SQL as additional frontend** | 2-3 sprints | Neutral (same gate chain) | Large gain | 0 BCs broken |
| **Drop filter mode only** | 1 sprint | Neutral | Small | 1 BC withdrawn |
| **Drop PrismQL, pure SQL** | 12-15 sprints | Requires full re-derivation | Large gain | 25 BCs rewritten, 2 Kani re-proofs, 1501+ tests migrated |
| **PQL as SQL sugar (deep hybrid)** | 3-4 sprints | Neutral | Small gain | Architecture reframing only |

### H.2 Open Questions for the Human

1. **Is the primary pain "agents can't write PrismQL" or "PrismQL adds maintenance burden"?**
   If the former, SQL-as-additional-frontend captures the gain at 1/5 the cost. If the
   latter, the full migration is warranted.

2. **Are pipe mode ergonomics a product differentiator?** Pipe mode (`| where ... | stats
   ... | sort ...`) is familiar to analysts from Splunk SPL and Microsoft KQL. If the
   analyst audience values this, it is a product argument to keep it.

3. **Is the formal verification investment (VP-014/VP-015 Kani proofs) a hard requirement
   on the new surface?** If yes, budget formal-verifier time for re-anchoring the proofs.
   If the re-proofs are not required before ship, the migration can proceed without them
   on a deferred basis — but this leaves VP-014/VP-015 status as "retired" until the new
   proofs land.

4. **What happens to BC-2.11.024 case-insensitive operators (IEQ/IIN/INE)?** These are
   used in the tool description today and in analyst-facing documentation. Under pure SQL
   they become `ILIKE` / `lower(field) = lower('value')`. Is renaming them a user-facing
   breaking change?

5. **What is the ship date pressure?** The 12-15 sprint estimate for full SQL migration
   makes it a Wave-7+ (post-convergence) initiative. The SQL-as-additional-frontend option
   is shippable in Wave-6 alongside existing work.

---

*Assessment-only document. Grounded entirely in code and specs as of 2026-09-18. No
implementation started. Author: architect.*

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.0 | 2026-09-18 | architect | Initial assessment |
