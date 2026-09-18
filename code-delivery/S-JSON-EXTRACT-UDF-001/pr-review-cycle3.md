# PR-LEVEL Review — PR #296 (cycle 3)

**Story:** S-JSON-EXTRACT-UDF-001 — `json_extract_string` ScalarUDF + literal-key plan gate (E-QUERY-045)
**PR:** https://github.com/BOHICA-LABS/prism/pull/296
**Branch:** `feature/S-JSON-EXTRACT-UDF-001` → `develop`
**Reviewed SHA:** `00167e40725485fe6a11ce2d4d70cd51e2887d04` (verified via `gh pr view 296 --json headRefOid`)
**Merge base:** `561d8baccc670525bccf920615cd6ea305b1410b`
**Diff:** 25 files, +4659 / −16
**Reviewer:** pr-reviewer (fresh-eyes; diff + description + test evidence only)

---

## VERDICT

**REQUEST_CHANGES**

`covered_sha: 00167e40725485fe6a11ce2d4d70cd51e2887d04`

**CLEAN (strict): no**
**CLEAN (PR-merge): no**

4 BLOCKING, 11 NON-BLOCKING.

---

## Summary

The production code in this PR is well-built. The gate design is sound, the pure-core
extraction function is genuinely pure and panic-free, the MCP error mapping is explicit
rather than catch-all, and the story ships the single strongest wire-shape test I have seen
on this surface. Cycle-2 findings N-c, N-g and N-a's arithmetic are **genuinely closed**,
not paper-fixed — I mutation-verified N-g's guard.

What blocks merge is not the implementation; it is **coverage and accuracy of claims about
coverage**. Mutation testing shows that **7 of the 11 Expr-bearing arms of the E-QUERY-045
security gate can be deleted outright with the entire 1,785-test `prism-query` suite still
passing**. The gate's own doc comment asserts a verification that the named test
structurally cannot provide. And the PR description still advertises pre-cycle-2 numbers
plus one wire-shape claim that the cycle-2-corrected evidence report explicitly contradicts.

This is the same defect class as F-JEX-P1-HIGH-001, the HIGH security finding this story
already closed once: a gate position that no test exercised. Seven more such positions
remain.

---

## BLOCKING FINDINGS

### B-C3-1 — 7 of 11 E-QUERY-045 gate arms have ZERO test coverage (mutation-verified)

| Field | Value |
|---|---|
| Severity | **blocking** |
| Category | coverage |
| Probe | SAP-3 rule 2 (spec-arm reachability) |
| Site | `check_jex_in_sql_query`, `check_jex_in_predicate`, `check_jex_in_expr`, `check_json_extract_key_literal` in `prism-query::engine` |

`check_json_extract_key_literal` walks 11 Expr-bearing positions. Coverage:

| # | Arm | Verdict | Covering test |
|---|---|---|---|
| a | SQL SELECT list | COVERED-E2E | `test_jex_rg006_non_literal_key_rejected_e_query_045_a`, `test_jex_rg007_key_exceeds_max_len_rejected_e_query_045_b`, `test_jex_rg007_d_multibyte_key_over_boundary_rejected` |
| b | SQL WHERE | COVERED-E2E | `test_jex_rg012_where_predicate_non_literal_key_rejected_e_query_045`, `test_jex_rg012_b_where_key_too_long_rejected_e_query_045_b` |
| c | SQL HAVING | COVERED-E2E | `test_jex_rg012_c_having_predicate_non_literal_key_rejected_e_query_045` |
| d | SQL GROUP BY | **NOT COVERED** | — |
| e | SQL ORDER BY | **NOT COVERED** | — |
| f | SQL JOIN ON | **NOT COVERED** | — |
| g | `Predicate::InSubquery` | **NOT COVERED** | — |
| h | `Expr::InSubquery` | **NOT COVERED** | — |
| i | `PipeStage::Where` via `Ast::SqlPipe` | COVERED-E2E | `test_jex_rg013_pipe_where_non_literal_key_rejected_e_query_045_a`, `test_jex_rg013_b_pipe_where_key_too_long_rejected_e_query_045_b` |
| j | `Ast::Pipe` stages (bare-source pipe) | **NOT COVERED** | — |
| k | `Ast::Filter` predicate (filter mode) | **NOT COVERED** | — |

**Evidence (mutation).** All seven uncovered arms were simultaneously neutered — `Ast::Pipe`
and `Ast::Filter` replaced with `Ok(())`, the `group_by` / `order_by` / `joins` loops in
`check_jex_in_sql_query` removed, `Predicate::InSubquery` and `Expr::InSubquery` replaced
with `Ok(())`. Result: **`1785 tests run: 1785 passed, 6 skipped`.** Zero signal. Worktree
restored to pristine `00167e407` afterwards (`git status --porcelain` empty, suite re-verified
green).

**Evidence (textual, independently re-confirmed).** `rg 'json_extract_string' --type rust crates/`
shows that every `json_extract_string` query string in the workspace places the call in a
SELECT projection, a SQL `WHERE`, a SQL `HAVING`, a `| where` pipe stage, or a DML `DELETE ... WHERE`.
No query string anywhere in the repo places it in GROUP BY, ORDER BY, JOIN ON, an `IN (SELECT ...)`
subquery, a bare-source `Ast::Pipe`, or filter-mode `Ast::Filter` position — and no synthetic-AST
test constructs those nodes either.

**Why this is blocking, not a suggestion.**
1. This is a **security gate** — the dynamic-key-expression injection perimeter plus the
   256-byte CWE-400 cap. The cap is enforced *only* here; `invoke_with_args` does not enforce it
   (the `Expr::TimestampArithmetic` arm comment states this explicitly).
2. CLAUDE.md §SAP-3 rule 2 rates *synthetic-only* coverage as a P2 finding. These seven arms
   are **below synthetic** — zero coverage of any kind.
3. SAP-3 rule 3's defense-in-depth exemption does **not** apply. That exemption requires the
   covering test to carry a reachability rationale; there is no covering test. And the gate's
   own comments assert these arms are load-bearing, e.g. `check_jex_in_sql_query`'s header
   calls the exhaustive destructure an *"injection-perimeter compile-time guard"* against
   *"silently bypassing the E-QUERY-045 security gate"*. That guard protects against **new**
   fields; it does nothing for the seven existing positions no test touches.
4. F-JEX-P1-HIGH-001 — the HIGH finding this story already fixed — was precisely a gate
   position that no test exercised (`ScalarFunc::Unknown` in predicate position). The PR's own
   Security Review section claims *"All four predicate positions … now consistently route … and
   are subject to `check_json_extract_key_literal`"*. Four positions are verified; seven are
   asserted without evidence.
5. `Ast::Pipe` (j) and `Ast::Filter` (k) are **distinct top-level AST entry points**, not
   variations of a covered one. Arm (i) is reached only via `Ast::SqlPipe` because the RG-013
   queries use a `SELECT … FROM …` head. A bare-source pipe takes a different arm with its own
   stage loop.

**Suggestion.** Add 7 end-to-end tests, one PQL query string each through
`QueryEngine::execute`, asserting the specific variant (`Err(PrismError::JsonExtractNonLiteralKey)`
or `Err(PrismError::JsonExtractKeyTooLong { key_len, max_len })`) exactly as the existing
RG-JEX-012/013 tests do. Highest value first:

- `Predicate::InSubquery` / `Expr::InSubquery` — subquery is the classic gate-bypass vector.
- `Ast::Filter` and `Ast::Pipe` — separate top-level entry points; a bypass here is total.
- GROUP BY / ORDER BY / JOIN ON.

Register them as RG-JEX-014..020 (or as sub-variants) and update the BC-5.38.001 density check.

---

### B-C3-2 — False verification claim in the gate's doc comment (TD-VSDD-059 paper-verification)

| Field | Value |
|---|---|
| Severity | **blocking** |
| Category | coverage / documentation accuracy |
| Site | `check_json_extract_key_literal` doc comment, §Gate ordering, in `prism-query::engine` |

The doc comment states the gate fires after E-QUERY-037 / 038 / 039 and that
*"This ordering is enforced by the caller and verified by RG-JEX-006 (AC-006; SAP-3 reachability)."*

**RG-JEX-006 structurally cannot verify that ordering.** It runs on the `make_gate_engine()`
helper, which constructs `QueryEngine::new(...)` with no table registry — the helper's own doc
comment says: *"With `table_registry = None`, the E-QUERY-037 gate is bypassed."* I confirmed
the constructor call independently: `make_gate_engine` passes `AdapterRegistry`,
`NullCredentialStore`, `OcsfNormalizer`, `ClientRegistry`, `QueryEngineConfig` — no registry.

Every gate test uses that helper (`rg006`, `rg007`, `rg007_b/c/d`, `rg012`, `rg012_b`, `rg012_c`,
`rg013`, `rg013_b`), so **E-QUERY-037 is disabled in all of them**, and the table `test_events`
does not exist. No test anywhere presents a nonexistent table *and* a non-literal key together
to assert which error wins. The relative ordering of E-QUERY-045 against 037/038/039 is entirely
unverified.

This is the pattern CLAUDE.md names directly: *"Doc comment claiming 'this requires capability X'
with no capability check → either implement the gate or remove the docs"* (Standing Rule 3 §3),
and TD-VSDD-059 paper-fix detection — a claimed closure whose named test provides no load-bearing
assertion.

**Suggestion.** Either (a) add an ordering test that constructs an engine **with** a real
`table_registry`, queries a nonexistent table with a non-literal key, and asserts
`E-QUERY-037` wins; or (b) delete the "verified by RG-JEX-006" clause and state plainly that
ordering is enforced by call-site placement only. (a) is preferred — the ordering is an
ADR-066 §B3 contract.

---

### B-C3-3 — Demo evidence artifact falsely labels itself AC-005

| Field | Value |
|---|---|
| Severity | **blocking** |
| Category | demo evidence / description accuracy |
| Site | `docs/demo-evidence/S-JSON-EXTRACT-UDF-001/AC-005-non-literal-key-error.json` (new in this HEAD) |

The file's `_demo_meta` declares `"ac": "AC-005"`, `"rg": "RG-JEX-005"`. Both are wrong:

- The story spec defines **AC-005 as "Non-object JSON column returns SQL NULL"** (RG-JEX-005 /
  EC-11-025-004). The file actually captures a **non-literal-key rejection**, which is
  AC-006 / RG-JEX-006 / EC-11-025-006.
- The file is **internally self-contradictory**: its own `bc_clause` field says
  `"EC-11-025-006 / E-QUERY-045(a)"`, which cannot coexist with `rg: RG-JEX-005`.
- It is a **near-duplicate** of `AC-006-non-literal-key-rejection.json` — identical MCP
  response; only the non-literal column name differs (`severity_col` vs `severity_id`).
- `evidence-report.md` §Artifact Index **already contradicts the file**, listing its
  "ACs Covered" as `AC-006 (plan-gate E-QUERY-045(a))`. The report's cycle-2 note compounds the
  error in prose: *"Both AC-005 and AC-006 re-captured at HEAD."*

AC-005 is not left without evidence — it is genuinely covered by
`AC-001-010-functional-unit-tests.json §AC-005_non_object_json` plus a passing
`test_jex_rg005_non_object_json_returns_sql_null`. The defect is that a committed evidence
artifact asserts a false AC/RG mapping, so the evidence package no longer reads as a reliable
AC→artifact index. This was introduced **new** by the cycle-2 B-1 fix (the cycle-2 triage
comment itself routes "Re-record AC-005", propagating the mislabel).

**Suggestion.** Rename the file to reflect AC-006 (or delete it as redundant with
`AC-006-non-literal-key-rejection.json`), correct `_demo_meta.ac` / `.rg` / `.scenario`, fix the
Artifact Index row, and correct the cycle-2 note prose.

---

### B-C3-4 — PR description advertises pre-cycle-2 evidence, including one claim the corrected evidence report contradicts

| Field | Value |
|---|---|
| Severity | **blocking** |
| Category | description accuracy |
| Probe | TD-VSDD-097 dimension 2 (downstream copy target) |

Four stale or false claims in the PR body at this HEAD:

**(a) False wire-shape claim.** §Test Evidence states: *"RG-JEX-011 includes SID-2 wire-shape
assertions on **serialized** RecordBatch / StringArray output."* The cycle-2 N-e fix corrected
exactly this claim in `evidence-report.md`, which now reads: *"these are Arrow RecordBatch /
StringArray assertions — **pre-serialization**."* I verified the test: all its null assertions
are Arrow-level (`extracted_col.is_null(..)`). The report was swept; the PR body was not. This
is the TD-VSDD-097 dim-2 failure mode — a corrected artifact whose downstream copy retains the
pre-correction text.

**(b) Stale test count.** The collapsed section says *"28 total `test_jex_*` tests pass (28/28)"*.
Actual is **33** (28 in `test_json_extract_udf.rs` + 5 in `engine::jex_gate_walk_completeness_tests`),
which is what the cycle-2-corrected evidence report states. I re-ran it:
`33 tests run: 33 passed, 1758 skipped`.

**(c) Red Gate inventory omits cycle-2 additions.** §Coverage Summary says
*"Red Gate tests (RG-JEX-001..012) | 12/12 PASS"* and the Red Gate table has 12 rows. It omits
**RG-JEX-013 / RG-JEX-013-b** — the pipe-where tests added by the cycle-2 B-4/SAP-3 fix, which
mutation testing shows are the *sole* coverage of arm (i) — and omits RG-JEX-007-b/c/d and the
four `test_jex_f4_*` tests. The BC-5.38.001 density statement is therefore also stale.

**(d) Stale workspace count.** The badge and §Coverage Summary claim `6106/6106`. That predates
roughly nine tests added across `4dcc9ee8e` and `7deb3674f`.

**Suggestion.** Sweep the PR body against `evidence-report.md` at this HEAD: correct (a) to
"pre-serialization Arrow-level, with serialized-wire coverage in prism-mcp
`test_BC_2_11_025_json_extract_string_null_row_wire_null_not_absent`", set the counts to 33 and
the real workspace total, and extend the Red Gate table through RG-JEX-013-b.

---

## NON-BLOCKING FINDINGS

### N-C3-a — `test_jex_rg007_c`'s stated rationale is false (suggestion)

Its doc comment and failure message both claim it *"prevents a regression from `len()` →
`chars().count()`"*. It cannot. Under a `chars().count()` gate, 128 `é` yields 128, which is
`< 256`, so the gate does not fire — and `rg007_c` asserts non-firing, so it **still passes**.
Only `test_jex_rg007_d_multibyte_key_over_boundary_rejected` provides that guard (129 → 129 < 256
→ no fire → its positive assertion fails). N-a's arithmetic is sound (`é` = 2 bytes; 128×2 = 256,
129×2 = 258; `key_len: 258` matches `key.len()`), but the fix is effective only via `rg007_d`.
**Suggestion:** fix the comment, or strengthen the test with a mixed 1-byte/2-byte 257-byte key
(e.g. `"a".to_string() + &"é".repeat(128)`), which would also close the gap that 257 bytes is
currently only ever exercised with ASCII.

### N-C3-b — `test_jex_rg007_c` carries zero positive behavioral assertions (suggestion)

Two assertions: one tautology (`assert_eq!(key_256_bytes_unicode.len(), 256)` — exercises
`str::repeat`, not prism) and one negative-only (`!matches!(.., JsonExtractKeyTooLong{..})`). It
is also weaker than `rg007_b`, which additionally guards against `JsonExtractNonLiteralKey`.
Mutation-verified to pass with all seven gate arms deleted. Its one real value is catching the
`>`/`>=` off-by-one — already covered by `rg007_b` in ASCII.

### N-C3-c — N-d's wire-level error-envelope test is ceremonial (suggestion)

`test_S_JSON_EXTRACT_UDF_001_e_query_045a_wire_level_serialized_json` calls
`serde_json::to_string(sc)` then `from_str` back into a `serde_json::Value`. Since `sc` **is**
`structured_content` — already a `serde_json` value — the round-trip is an identity transform and
cannot surface field omissions, key renames, or enum-representation changes. Its assertions are
functionally identical to the sibling `..._structured_path_validation_category`. The one
representation-sensitive field, the MCP `isError` envelope key, is asserted **pre-serialization**,
as the test's own comment concedes: *"assert is_error == true at the struct level (no
serialization needed)"*. There is also a filler `assert!(!wire_json.is_empty())`.
**Suggestion:** serialize the full `CallToolResult` and assert `isError` / `structuredContent` /
`content` as they appear on the wire. N-d is satisfied in form, not substance, for the error
envelope. (The *row-shape* wire surface is genuinely covered — see Verified below.)

### N-C3-d — Residual disjunctive `.contains()` on a user-visible error message (suggestion)

N-c is fully closed in `test_json_extract_udf.rs` and the wire test (zero `.contains(` in both;
five tests assert the complete `Display` string via `assert_eq!`). One weak instance survives on
the same E-QUERY-045 surface, in `error_mapping`'s
`test_S_JSON_EXTRACT_UDF_001_e_query_045a_sid2_no_example_duplication_in_content_text`:
`suggestion.contains("raw_extensions") || suggestion.contains("severity")`. A disjunctive
substring check is the weakest form — `"severity"` alone satisfies it, and that word saturates
this error surface, so the assertion is near-vacuous for its stated purpose (proving the
suggestion uses a *different* example than the message). The full `suggestion` string is never
asserted verbatim anywhere in the workspace. That test's *primary* assertion (occurrence count
of the shared phrase == 1 on the composed `content_text`) is strong and correctly SID-2-compliant.
**Suggestion:** `assert_eq!` on the complete suggestion text.

### N-C3-e — `SelectItem` walk is the only non-exhaustive Expr site in the gate (suggestion; 3rd cycle)

`check_jex_in_sql_query` uses `if let SelectItem::Expr { expr, .. } = item` where every sibling
walk in the same function uses an exhaustive destructure or `match` and documents that choice as
*"a deliberate injection-perimeter compile-time guard."* No live bypass — `SelectItem`'s only
Expr-bearing variant is `Expr` (the others are `Star` and `TableStar(String)`). But `SelectItem`
is `#[non_exhaustive]`, and because the gate lives in the same crate an exhaustive `match` **would**
compile-error on an in-crate variant addition, delivering the same guard the file claims
everywhere else. Raised as N1 in cycle 1 and deferred as "acceptable risk"; this is its third
cycle. Per CLAUDE.md Canonical Principle Rule 3, a 3-line edit that could have been inline is a
defer-pattern smell. **Suggestion:** replace with
`match item { SelectItem::Expr { expr, .. } => check_jex_in_expr(expr)?, SelectItem::Star | SelectItem::TableStar(_) => {} }`.

### N-C3-f — `Ast::Pipe` arm's write-safety rests on an undocumented call-site invariant (suggestion)

The `Ast::Pipe(pq)` arm walks `pq.stages` and never inspects `pq.write: Option<WriteNode>`. This
is safe **today**, and I verified the structural chain: plain `PrismQlParser::parse` takes no
`&WriteVerbRegistry` and routes pipe input to `parse_pipe_internal` → `parse_pipe_with_limits` →
`build_pipe_parser`, which hardcodes `write: None` in all three grammar variants. The only
`write: Some(...)` construction site in the crate is inside `parse_pipe_with_write`, whose sole
non-test caller is `parse_with_registry`. Both gate call sites (in `execute_inner` and
`execute_scheduled_inner`) use plain `parse`. So the un-walked field is unreachable from the
gate's inputs.

But that is a **call-site** property, not a type-level one, and there is no compile guard, test,
or comment recording it. If a future change points either gate call site at
`parse_with_registry`, or adds a third call site fed by it, a `json_extract_string(col, non_literal)`
in a write pipeline's terminal stage bypasses E-QUERY-045 silently. The arm's three siblings
(`Ast::Sql(_)`, `Ast::SqlPipe`, `Ast::Filter`) all carry explicit rationale comments; this one
carries none. **Suggestion:** destructure `PipeQuery { source: _, stages, write: _ }` with a
comment stating the invariant, so a field addition or a registry-parse migration surfaces here.

### N-C3-g — VP-162 Kani unwind bounds are almost certainly too low to verify the stated property (suggestion)

`vp162_json_extract_string_null_safety` uses `#[kani::unwind(8)]` against a symbolic key of up to
256 bytes and a symbolic column of up to 1024 bytes fed to `serde_json::from_str`;
`vp162_b_none_input_is_none_output` uses `#[kani::unwind(4)]`. An unwind bound of 8 cannot cover a
parse loop over a 1024-byte input — the harness will most likely report unwinding-assertion
failures, or verify vacuously. Nothing in this PR demonstrates the harness even compiles: it is
`#[cfg(kani)]`-gated, so `just check` and CI never touch it, and there is no `cargo kani` output in
the evidence package. Cycle 1 accepted this as "N2 Kani gap"; the specific unwind-bound defect
should be recorded against the Phase-6 formal-hardening anchor so it is not rediscovered as a new
finding there. The `json_extract_string_impl` purity constraint the harness depends on
(no DataFusion/Arrow types, ADR-066 §D1) **is** correctly satisfied.

### N-C3-h — `evidence-report.md` under-claims the story's strongest evidence (suggestion)

The genuinely-failable serialized-row test — prism-mcp's
`test_BC_2_11_025_json_extract_string_null_row_wire_null_not_absent` — is referenced **nowhere** in
`evidence-report.md` or the PR description. The report's §AC-011 instead says wire-level assertions
are *"covered by the plan-gate tests (RG-JEX-006, RG-JEX-007, RG-JEX-012, RG-JEX-013)"*, which are
error-**envelope** assertions, not row-shape. A reviewer relying on the report would conclude the
row-shape wire assertion is missing when it exists and is excellent. It is also outside the
`test_jex` filter (it is named `test_BC_2_11_025_*` and lives in `prism-mcp`), so it is not in the
report's "33 tests" total. **Suggestion:** cite it explicitly under AC-011.

### N-C3-i — `invoke_with_args` LargeUtf8 handling is asymmetric (nit)

The key argument accepts both `ScalarValue::Utf8` and `ScalarValue::LargeUtf8`; the JSON-column
argument accepts only `Utf8` / `Null` and errors on `LargeUtf8`. Unreachable under the
`TypeSignature::Exact(vec![Utf8, Utf8])` signature, so cosmetic — but the asymmetry invites a
future reader to assume `LargeUtf8` is supported on both.

### N-C3-j — `test_jex_dml_ast_safe_skip_with_jex_variant_integration` is misnamed (nit)

It asserts only the **parser** mapping (`func == &ScalarFunc::JsonExtractString`); it never calls
`check_json_extract_key_literal`. The name says "safe_skip" but the safe-skip property is proven
elsewhere, by the inline `test_jex_dml_ast_returns_ok_no_scope`. The doc comment delegates honestly
and the named test does exist, so this is correctly scoped — only the name overclaims.

### N-C3-k — Redundant `#[cfg(test)] mod tests` wrapper inside an integration-test file (nit)

`bc_2_11_025_jex_wire_null_test.rs` wraps its single test in `#[cfg(test)] mod tests { ... }`.
Verified functional (`cargo nextest run -p prism-mcp --test bc_2_11_025_jex_wire_null_test` →
`Starting 1 test` / `1 passed`), but this is the construct that silently zeroes a suite if harness
config ever changes. Integration-test files are already compiled under `cfg(test)`.

---

## MERGE-GATING OBSERVATIONS (not findings)

- **CI is not green at this SHA.** Still `pending`: `Test (aarch64-apple-darwin)`,
  `Test (x86_64-unknown-linux-gnu)`, `Test (x86_64-unknown-linux-musl)`,
  `Test (x86_64-pc-windows-msvc)`, `Test (no-default-features)`,
  `Perimeter compile-fail check`, `Non-exhaustive violation compile-fail check`,
  `ADR-023 No-Hardcoded-Sensors compile-fail gate`, `E2E smoke`, `Fuzz smoke`.
  All other 19 checks pass, including `Cargo audit`, `Cargo deny`, `Clippy`, `Semver compatibility`.
- **BC-5.39.001 frozen-HEAD:** the PR body pins LOCAL 3-CLEAN(strict) to `9645d0726`. Four commits
  have landed since, and one (`4dcc9ee8e`) changed **production MCP-visible text** — the E-QUERY-045
  `suggestion` strings in `error_mapping`. Holdout HS-040 (3/3, mean 1.00) also predates that change.
  The body discloses the frozen SHA honestly, but the header line "CONVERGED after 3 LOCAL adversarial
  passes" reads as current. This PR-LEVEL cascade is the governing gate; noted for bookkeeping.
- **`Cargo.lock` blast radius:** the RUSTSEC-2026-0285 rustls bump (0.23.40 → 0.23.45) also pulls
  `aws-lc-rs` 1.16.3 → 1.18.1, `aws-lc-sys` 0.40.0 → 0.45.0 (a build-from-source crypto crate,
  gaining a `pkg-config` build dependency), and `rustls-webpki` 0.103.13 → 0.103.15. `cargo audit`,
  `cargo deny` and `Semver compatibility` all pass. N-b's dep table in the description is accurate.

---

## WHAT I VERIFIED CLEAN (no rubber-stamping)

**Gate walk completeness against the current AST — verified independently, arm by arm.** No
bypass path exists in today's grammar:
- `SqlQuery` is destructured exhaustively. `from: _` is genuinely safe — `SourceRef` is a raw
  string plus a `SourceRefKind` enum (`Composite` / `External` / `Internal` / `Custom`) with **no
  subquery variant**, so there is no derived-table Expr position. `limit: _` is `Option<u64>`.
- `PipeStage`'s non-`Where` variants are genuinely Expr-free, which I confirmed by reading each
  payload: `JoinCondition` is `SameField(FieldPath)` / `Pair(FieldPath, FieldPath)`;
  `EnrichStage` is `String` + `FieldPath`; `FieldsStage` is `bool` + `Vec<FieldPath>`;
  `StatsStage.aggregates: Vec<StatFunction>` → `AggFunc`, whose every variant carries only
  `FieldPath`. The arm comment's claim holds.
- `Predicate` and `Expr` are both matched exhaustively, with `InSubquery` recursion on both sides
  and `TimestampArithmetic.base` recursion as documented defense-in-depth.

**No parser-divergence bypass.** This was my highest-priority hypothesis and it does not hold.
The gate's best-effort `if let Ok(ast) = PrismQlParser::parse(...)` uses the **same** function on
the **same** `effective_query` / `query_str` that the execution path parses. There is no second
grammar that accepts a query the gate's parse rejects, so a parse failure means the pipeline
surfaces a parse error rather than executing ungated.

**`json_extract_string_impl` is genuinely pure and panic-free.** Zero `unwrap()` / `expect()` in
`json_extract_udf.rs`; `?`-propagation throughout; no Arrow or DataFusion types in the signature,
satisfying ADR-066 §D1.

**Error taxonomy and MCP mapping are correct and explicitly tested.** Both variants are
categorized in `error_category_coverage`, and mapped **explicitly** (not via catch-all) in both
`map_prism_error` → `-32602 INVALID_PARAMS` and `prism_error_to_structured_call_result` →
`category: "validation"`, `original_params_valid: false`, with full-string `assert_eq!` message
assertions and explicit `assert_ne!(code, INTERNAL_ERROR)` negative guards. 7/7 prism-mcp tests pass.

**Wire-shape discipline (CLAUDE.md) is satisfied on the row-shape surface.** prism-mcp's
`test_BC_2_11_025_json_extract_string_null_row_wire_null_not_absent` drives the real MCP `query`
tool through the production `arrow_json` serializer and asserts NULL-vs-absent correctly
decomposed: `rows[1].get("extracted").is_some()` catches `explicit_nulls=false` key omission, and
`rows[1]["extracted"].is_null()` catches the string-`"null"` regression, repeated for the
key-absent arm plus a positive sanity check. This is genuinely failable and directly targets the
DEFECT-MCP-ROWSHAPE-NULLS-001 / [C3]/[H20] class. Strongest asset in the story.

**SID-2 composed-output discipline correctly applied** via the occurrence-count assertion
(`content_text.matches(phrase).count() == 1`) on the composed message+suggestion string.

**No `is_err()`-accepts-anything anti-pattern anywhere.** Every error-path test binds the specific
variant, and the too-long tests bind exact field values
(`JsonExtractKeyTooLong { key_len: 257, max_len: 256 }` / `{ key_len: 258, max_len: 256 }`). A
parse error or table-not-found would satisfy none of them. Genuine strength.

**No zero-assertion tests.** All 28 + 1 + 5 have real `assert*!` macros.

**Cycle-2 closures confirmed genuine, not paper-fixed:**
- **N-g** — mutation-verified: flipping the gate's `key_len > MAX` to `>=` makes
  `test_jex_rg007_b_key_at_exactly_max_len_accepted` and `rg007_c` fail. Properly closed.
- **N-c** — zero `.contains(` in both audited test files; five tests assert the full composed
  `Display` string verbatim.
- **N-a arithmetic** — byte counts correct and byte-based (see N-C3-a for the rationale caveat).
- **N-i** — no duplicate test names anywhere; the near-pair is deliberately disambiguated.
- **B-2** — `AC-007-key-length-boundary.json` and `AC-012-where-predicate-gate.json` are now
  genuine JSON-RPC stdio captures with proper `mcp_request` / `mcp_response` / `content[]` /
  `_meta` structure.
- **B-3** — the TD-DECOMP-EPIC-001 decomposition rationale for `engine.rs` +540 lines is present
  and correctly notes the additions are inline test modules, not new production responsibility.

**Standing probes:**
- **SAP-1 (tracing catalog): CLEAN, no trigger.** 0 added and 0 removed `event_type` lines across
  the whole diff; no new `tracing::*!` or `println!` anywhere. No BC-2.16.002 catalog row required.
- **SAP-2 (DTU↔TOML parity): NOT APPLICABLE.** Zero `*.sensor.toml` and zero
  `.prism/specs/sensors/` paths in the diff.
- **SID-1 (no-ignored-test): CLEAN, no trigger.** Zero `#[ignore]` added anywhere in the diff; the
  single `ignore` hit is a rustdoc ```` ```ignore ```` fence.
- **SAP-3:** see B-C3-1 — this is the probe that fails.

**Toolchain gates re-run locally at this SHA:**
- `cargo nextest run -p prism-query -E 'test(jex) or test(json_extract)'` → **33 passed, 0 failed**
- `cargo nextest run -p prism-mcp -E 'test(json_extract) or test(jex) or test(e_query_045)'` → **7 passed, 0 failed**
- Canonical clippy (`cargo clippy --all-features -- -D warnings`, per the Justfile `check` recipe) → **exit 0, zero warnings**
- `scripts/check-non-exhaustive.sh` → **PASS**, 99/99 violations, 98 unique symbols. `JsonExtractStringUdf`
  was appended to `EXPECTED_SYMBOLS` (the single source of truth) with a matching `v99` compile-fail
  entry, and the equality check confirms both moved in lockstep.

*(Note: `cargo clippy --all-targets` reports 221 errors across 12 files from
`unwrap_used`/`expect_used = "deny"` in test code. This is pre-existing and corpus-wide — 219 of
221 are outside this PR's diff — and the project's canonical gate deliberately omits
`--all-targets`. The 2 hits on added lines are descriptive `expect()` calls in the new inline test
module, matching 40 pre-existing hits in the same file. Not a violation of any enforced gate.)*

---

## Checklist disposition

| # | Item | Verdict |
|---|---|---|
| 1 | Diff coherence | PASS — all 25 files trace to the story or the RUSTSEC rustls bump |
| 2 | Description accuracy | **FAIL** — B-C3-4 |
| 3 | Test coverage | **FAIL** — B-C3-1 (7 of 11 security-gate arms untested), B-C3-2 |
| 4 | Demo evidence | **FAIL** — B-C3-3; all 12 ACs otherwise have evidence, with VHS `.gif`/`.webm` for AC-006/007/011/012 and MCP stdio captures elsewhere |
| 5 | Commit quality | PASS — conventional format, story ID present, clear messages, no AI attribution |
| 6 | Diff size | NOTED — +4659 (>500). Rationale present and accurate (majority is tests + demo evidence); TD-DECOMP-EPIC-001 anchor cited per B-3 |
| 7 | Missing changes | PASS — AC-001..AC-012 all implemented; T-09a `filter_parser` parity fix present |
| 8 | Dependency status | PASS — S-ADR058-OCSF-ROUTING-001 (PR #241) merged |

---

## Route

| Finding | Severity | Suggested route |
|---|---|---|
| B-C3-1 (7 untested gate arms) | blocking | `test-writer` → `implementer` (7 E2E tests); `story-writer` for RG-JEX-014..020 + density check |
| B-C3-2 (false verification claim) | blocking | `implementer` (add ordering test **or** delete the claim) |
| B-C3-3 (AC-005 mislabel) | blocking | `demo-recorder` |
| B-C3-4 (stale PR description) | blocking | `pr-manager` |
| N-C3-a..k | suggestion / nit | `implementer` in the same burst where cheap (a, b, d, e, f, i, j, k); `demo-recorder` for h; record g against the Phase-6 anchor |

Re-gate required on the new HEAD after the fix burst — pushing any commit resets the PR-LEVEL
streak to 0/3 per BC-5.39.001 frozen-HEAD rule.
