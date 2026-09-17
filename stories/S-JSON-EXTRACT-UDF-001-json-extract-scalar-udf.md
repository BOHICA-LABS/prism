---
document_type: story
story_id: S-JSON-EXTRACT-UDF-001
title: "Minimal json_extract_string ScalarUDF with Literal-Key Plan Gate (E-QUERY-045)"
level: "L4"
version: "1.6"
status: ready
producer: story-writer
timestamp: "2026-09-17T00:00:00Z"
phase: 3
wave: 3
epic_id: E-BETA3-REMEDIATION
cycle: v3-brownfield
priority: P1
points: 5
tdd_mode: strict
target_module: prism-query
subsystems:
  - SS-11
# Subsystem anchor justification:
#   SS-11 (Query Engine) owns crates/prism-query/src/ per ARCH-INDEX Subsystem Registry.
#   UDF registration (engine.rs), plan-gate (plan_gates.rs or engine.rs), pure extraction
#   function (json_extract_udf.rs), and the new prism-core error variants
#   (JsonExtractNonLiteralKey, JsonExtractKeyTooLong) all live within the SS-11 module
#   boundary. SS-11 is the single subsystem affected; no cross-subsystem structural change.
crates_touched:
  - prism-query
  - prism-core
estimated_days: 1.5
inputs:
  - crates/prism-query/src/ast.rs
  - crates/prism-query/src/engine.rs
  - .factory/specs/behavioral-contracts/BC-2.11.025-json-extract-string-scalar-udf.md
  - .factory/specs/architecture/decisions/ADR-066-json-extract-scalar-udf.md
  - .factory/specs/verification-properties/vp-162-json-extract-string-null-safety.md
input-hash: "pending"
traces_to: .factory/stories/S-ADR058-OCSF-ROUTING-001-sensor-spec-ocsf-field-name-routing.md
depends_on:
  - S-ADR058-OCSF-ROUTING-001
# depends_on anchor justification:
#   S-ADR058-OCSF-ROUTING-001 establishes raw_extensions as the Tier-2 Arrow column
#   (crates/prism-query materialization path per BC-2.16.003 §Interpretation A Tier-2).
#   The primary integration use case for this UDF is filtering raw_extensions JSON blobs.
#   The pipe-mode end-to-end test (RG-JEX-011) requires raw_extensions to be populated
#   per the OCSF routing contract, which depends on S-ADR058-OCSF-ROUTING-001 being
#   merged first. The UDF registration itself is independent, but the full integration
#   test surface (SAP-3 reachability obligation) requires it.
blocks:
  - S-JSON-EXTRACT-TYPED-001
  - S-JSON-EXTRACT-NESTED-001
  - S-BETA3-RELEASE-001
# blocks anchor justifications:
#   S-JSON-EXTRACT-TYPED-001: depends_on S-JSON-EXTRACT-UDF-001 per delta-analysis.md
#   §Issue 20 D-2521 fast-follow reconciliation — typed variants (json_extract_int,
#   json_extract_float, json_extract_bool) build on the base UDF registration pattern and
#   plan-gate architecture established here. (Note: story-writer must update
#   S-JSON-EXTRACT-TYPED-001 stub's depends_on to reference S-JSON-EXTRACT-UDF-001.)
#
#   S-JSON-EXTRACT-NESTED-001: depends_on S-JSON-EXTRACT-UDF-001 per delta-analysis.md
#   §Issue 20 D-2521 fast-follow reconciliation — bounded JSONPath nested paths depend
#   on the base UDF infrastructure established here. (Note: story-writer must update
#   S-JSON-EXTRACT-NESTED-001 stub's depends_on to reference S-JSON-EXTRACT-UDF-001.)
#
#   S-BETA3-RELEASE-001: all Wave W3 stories must merge to develop before the beta.3
#   release bundle dispatches. S-JSON-EXTRACT-UDF-001 closes the issue-20 dead-path
#   defect (json_extract_string unregistered in DataFusion context) required for the
#   beta.3 live-validation gate (OQ-002 human decision 2026-08-21).
behavioral_contracts:
  - BC-2.11.025
# BC status: draft v1.9 (frozen per beta3 spec-gate passes; BC-INDEX pin v1.9 confirmed).
#   BC-2.11.025 contains 11 edge cases (EC-11-025-001..011) each anchored to
#   S-JSON-EXTRACT-UDF-001 RG-JEX-001..011 with canonical test names.
#   Spec-First Gate S-7.01 satisfied: behavioral_contracts is non-empty with canonical
#   BC-S.SS.NNN pattern. Every AC below traces to a specific BC-2.11.025 clause.
verification_properties:
  - VP-162
assumption_validations: []
risk_mitigations: []
holdout_scenarios: []
modified: "2026-09-17"
---

# S-JSON-EXTRACT-UDF-001: Minimal json_extract_string ScalarUDF with Literal-Key Plan Gate

## Authority

**ADR-066 v1.6** (`decisions/ADR-066-json-extract-scalar-udf.md`) is the authoritative
design document for this story. Read §A (dead-path defect), §B (decision: synchronous
serde_json + literal-key gate), §C (formal correctness contract), §D1-§D6 (scope
boundaries), §E (DataFusion registration contract), §F (E-QUERY-045 error messages),
§G (mandate anchors — 11 MUST → RG-JEX mappings), and §H (latent defect closure) in full
before implementing.

**BC-2.11.025 v1.9** (`behavioral-contracts/BC-2.11.025-json-extract-string-scalar-udf.md`)
governs the full behavioral contract. The 11 edge cases (EC-11-025-001..011) are the
authoritative acceptance criteria and provide canonical test names for RG-JEX-001..011.

**VP-162 v1.6** (`verification-properties/vp-162-json-extract-string-null-safety.md`)
defines the Kani proof target (`json_extract_string_impl` pure function). The Kani harness
files live in `crates/prism-query/src/proofs/vp162_json_extract_null_safety.rs` per VP-162
§Kani Proof Harness. The proof is dispatched in Phase 5 (formal-verify), not Phase 3.

> NOTE: ADR-066 v1.6, BC-2.11.025 v1.9, and VP-162 v1.6 are FROZEN per beta3 spec-gate.
> These spec files MUST NOT be amended by the implementer — any spec discrepancy routes
> to product-owner/architect via the orchestrator.

**Security surface (CLAUDE.md §agent-harness):** `json_extract_string` is an agent-facing
query-language surface. The literal-key plan gate (ADR-066 §B3, E-QUERY-045) and the
256-byte key-length cap (ADR-066 §D3, CWE-400) are the primary injection-prevention
mechanisms. A security-reviewer pass is REQUIRED before the PR merges (see Tasks §T-21).

---

## Problem Statement

During the beta.2 Monroe demo live-test against the Claroty xDome tenant (jea-readapi,
2026-09-15, D-2520), an analyst issued a PrismQL query using `json_extract_string`:

```sql
FROM claroty_alerts | SELECT json_extract_string(raw_extensions, 'severity')
```

DataFusion returned an unregistered-function runtime error. The error was not structured
under the prism `E-QUERY-NNN` error taxonomy, making it opaque to the LLM agent.

### Root Cause: Three Dead AST Paths, Zero UDF Registration

Three code sites reference `json_extract_string` but no ScalarUDF is registered:

1. `crates/prism-query/src/ast.rs` — `ScalarFunc::JsonExtractString` variant in the
   `ScalarFunc` enum (grep `JsonExtractString`).
2. `crates/prism-query/src/sql_parser.rs` — `json_extract_string(col, 'key')` SQL syntax
   parsed into `ScalarFunc::JsonExtractString` (one site; grep `json_extract_string` in
   the function-name match arm).
3. `crates/prism-query/src/pipe_sql_emitter.rs` — `ScalarFunc::JsonExtractString` lowered
   to DataFusion SQL `json_extract_string(col, key)` (grep `JsonExtractString` in the
   pipe-SQL emitter match arm).

Without a registered `ScalarUDF`, DataFusion receives an unregistered function call in both
SQL mode and pipe mode. Neither failure path is structured under `E-QUERY-NNN`.

### Secondary Gap: No Literal-Key Plan Gate

No plan-time gate validates that the second argument to `json_extract_string` is a string
literal. A dynamic key (`json_extract_string(col, other_column)`) is accepted at parse time
and would be evaluated with a runtime-resolved key, bypassing:
- The 256-byte key length cap (CWE-400, ADR-066 §D3)
- The literal-key-only injection prevention contract (ADR-066 §B3)

### Fix

1. Implement `json_extract_string_impl` pure function and `JsonExtractStringUdf` struct
   implementing DataFusion `ScalarUDFImpl` in new file
   `crates/prism-query/src/json_extract_udf.rs`.
2. Register the UDF in `engine.rs` at `QueryEngine::new` / `SessionContext` construction.
3. Add `check_json_extract_key_literal` plan gate to reject non-literal key arguments with
   `E-QUERY-045` before DataFusion execution (ADR-066 §B3).
4. Add `PrismError::JsonExtractNonLiteralKey` and `PrismError::JsonExtractKeyTooLong`
   variants to `crates/prism-core/src/error.rs`.

ADR-066 §H explicitly authorizes closing the defect by implementing the UDF and adding the
plan gate — NOT by removing the existing AST/parser/emitter arms.

---

## Narrative

As a PrismQL user querying Claroty (and other sensor) Tier-2 `raw_extensions` JSON data,
I want `json_extract_string(raw_extensions, 'field_name')` to execute correctly in
PrismQL SELECT, WHERE, and HAVING clauses, so that I can filter and project individual
top-level JSON fields from `raw_extensions` without requiring full OCSF schema promotion
to named Arrow columns.

---

## Behavioral Contracts

| BC | Title | Version at Authoring | Scope in This Story |
|----|-------|---------------------|---------------------|
| BC-2.11.025 | `json_extract_string` DataFusion ScalarUDF — Literal-Key-Only JSON String Extraction | v1.9 | All postconditions, invariants, and 11 edge cases (EC-11-025-001..011); 11 MUSTs anchored to S-JSON-EXTRACT-UDF-001 RG-JEX-001..011 with canonical test names |

---

## Acceptance Criteria

### AC-001 — UDF registered: happy path extracts top-level string value

`json_extract_string(column, 'severity')` where `column = '{"severity":"high","count":5}'`
returns `"high"` as a non-null `Utf8` Arrow cell. The UDF is registered in the DataFusion
`SessionContext` at engine construction (before any query executes) under the name
`"json_extract_string"` with input types `(Utf8, Utf8)`, return type `Utf8` (nullable),
and `Volatility::Immutable` (ADR-066 §E).

(traces to BC-2.11.025 postcondition §Happy path + EC-11-025-001; ADR-066 §B1 + §E)

### AC-002 — JSON null value at key returns SQL NULL

`json_extract_string(col, 'status')` where `col = '{"status":null}'` returns SQL NULL.
A JSON null value at the key is treated as absent (null-propagating, not coerced to the
string `"null"` or to an empty string). This is distinct from AC-003 (key absent) and
from AC-004 (Arrow column null).

(traces to BC-2.11.025 postcondition §JSON null at key + EC-11-025-005; ADR-066 §B1 step 5)

### AC-003 — Missing key returns SQL NULL

`json_extract_string(col, 'missing_key')` where `col = '{"severity":"high"}'` returns SQL
NULL. The key `"missing_key"` is absent from the JSON object; no warning is emitted.

(traces to BC-2.11.025 postcondition §Missing key + EC-11-025-003; ADR-066 §B1 step 4)

### AC-004 — Null column (Arrow null) returns SQL NULL

`json_extract_string(col, 'severity')` where `col` is an Arrow null cell (SQL NULL) returns
SQL NULL. The function is null-propagating: null input column always produces null output
regardless of key. This is proven by VP-162-B (`vp162_b_none_input_is_none_output`).

(traces to BC-2.11.025 postcondition §Null column + EC-11-025-002; ADR-066 §B1 step 1;
VP-162 invariant 2)

### AC-005 — Non-object JSON column returns SQL NULL

`json_extract_string(col, 'key')` where `col = '["a","b","c"]'` (JSON array) returns SQL
NULL. Applies to any non-object JSON value in the column (array, string primitive, number,
boolean, JSON `null` at root). Only a JSON object (`{}`) qualifies for key extraction.

(traces to BC-2.11.025 postcondition §Non-object JSON + EC-11-025-004; ADR-066 §B1 step 3)

### AC-006 — Non-literal key argument rejected at plan time with E-QUERY-045(a)

`json_extract_string(col, other_col)` where `other_col` is a column reference is rejected
at plan time with `E-QUERY-045(a)` via `PrismError::JsonExtractNonLiteralKey`. The gate
fires AFTER plan-time gates E-QUERY-037/038/039, BEFORE DataFusion execution or any sensor
fan-out. (Temporal validation E-QUERY-041/042 runs in-pipeline per ADR-052 §D4 — inside
`run_materialization_pipeline`, after the plan-time gate sequence — and is not part of the
pre-execution plan-gate ordering; no specific ordering between E-QUERY-045 and in-pipeline
temporal is asserted beyond ADR-066 §B3 "before DataFusion execution".)
The MCP response carries HTTP `-32602` INVALID_PARAMS
with message: `"E-QUERY-045: json_extract_string requires a literal string key (e.g.,
json_extract_string(col, 'key_name')). Dynamic key expressions are not supported."`

**SAP-3 reachability:** This AC MUST be tested via a real PQL query string through the
`prism_query` public API — not only from a synthetic AST injection. RG-JEX-006 exercises
this from the public surface.

(traces to BC-2.11.025 postcondition §Plan-time literal-key gate sub-case (a) + §Error
Cases E-QUERY-045(a) + EC-11-025-006; ADR-066 §B3 + §F)

### AC-007 — Key exceeding 256 UTF-8 bytes rejected at plan time with E-QUERY-045(b)

`json_extract_string(col, '<257-byte-literal>')` is rejected at plan time with
`E-QUERY-045(b)` via `PrismError::JsonExtractKeyTooLong { key_len: 257, max_len: 256 }`.
Fires before DataFusion execution or any sensor fan-out. MCP response carries HTTP `-32602`
INVALID_PARAMS with message: `"E-QUERY-045: json_extract_string key is 257 bytes, which
exceeds the 256-byte maximum (CWE-400)."` (runtime `key_len` and `max_len` substituted).
The 256-byte cap enforces CWE-400; zero per-row runtime overhead because it is plan-time.

(traces to BC-2.11.025 §Error Cases E-QUERY-045(b) + EC-11-025-007; ADR-066 §B3 + §D3
+ §F)

### AC-008 — Non-string JSON value coerced to string representation via to_string()

`json_extract_string(col, 'count')` where `col = '{"count":42}'` returns `"42"` (string).
Non-string JSON values at the key (number, boolean, nested object, array) are coerced via
`serde_json::Value::to_string()` — they are NOT returned as SQL NULL. Examples: integer
`42` → `"42"`, boolean `true` → `"true"`, array `[1,2]` → `"[1,2]"`. This preserves raw
value access; callers can cast downstream.

(traces to BC-2.11.025 postcondition §Non-string value + EC-11-025-008; ADR-066 §B1 step 7)

### AC-009 — Parse failure (invalid JSON column value) returns SQL NULL

`json_extract_string(col, 'key')` where `col = '{not_json}'` (malformed JSON) returns SQL
NULL. `serde_json::from_str` failure is treated as absent value. This does NOT trigger
E-QUERY-045 — that gate fires for plan-time literal-key constraint violations only, not
runtime parse errors. On JSON parse failure of the column value, the UDF returns SQL NULL
silently (no emission), per BC-2.11.025 §Postconditions Parse-failure and ADR-066 §B1
step 2. The `json_extract_string_impl` function is pure (ADR-066 §D1); per-row warn
emissions are not possible without violating purity and would generate per-row log spam.

(traces to BC-2.11.025 postcondition §Parse failure + EC-11-025-010; ADR-066 §B1 step 2)

### AC-010 — Dot-in-key literal treated as top-level key, NOT nested JSONPath

`json_extract_string(col, 'a.b')` where `col = '{"a.b":"literal_val","a":{"b":"nested"}}'`
returns `"literal_val"`. The literal key `"a.b"` matches the exact top-level key named
`"a.b"` — it is NOT interpreted as the nested path `a → b`. Nested JSONPath access is out
of scope in beta.3 per ADR-066 §D4; `S-JSON-EXTRACT-NESTED-001` handles it as a
post-beta.3 fast-follow.

(traces to BC-2.11.025 postcondition §Top-level key only + EC-11-025-009; ADR-066 §D4)

### AC-011 — Pipe-mode end-to-end: json_extract_string executes via MCP query tool

`FROM claroty_alerts | SELECT json_extract_string(raw_extensions, 'severity')` executed
through the MCP `query` tool public API (SAP-3 reachability) returns the extracted severity
string for each row where `raw_extensions` is a valid JSON object containing `"severity"`;
SQL NULL for rows where `raw_extensions` is null, absent, parse-fails, or has no `"severity"`
key. **Wire-level assertion (SID-2):** the serialized JSON output from the MCP `query` tool
MUST include the extracted value in the correct column; null rows carry `null` (not absent
key) per EC-11-079 null-not-absent invariant (BC-2.11.001).

(traces to BC-2.11.025 EC-11-025-011; ADR-066 §G RG-JEX-011 + §B3 SAP-3 cite)

---

## Enumerated Red Gate Test List (SAC-1)

The test-writer MUST author these 11 failing tests BEFORE any implementation begins.
Test names are canonical per BC-2.11.025 §Edge Cases MUST anchors:

| Gate | Failing Test Name | BC Clause |
|------|------------------|-----------|
| RG-JEX-001 | `test_jex_rg001_udf_registered_happy_path_executes` | EC-11-025-001 / postcondition §Happy path |
| RG-JEX-002 | `test_jex_rg002_json_null_value_at_key_returns_sql_null` | EC-11-025-005 / postcondition §JSON null at key |
| RG-JEX-003 | `test_jex_rg003_missing_key_returns_sql_null` | EC-11-025-003 / postcondition §Missing key |
| RG-JEX-004 | `test_jex_rg004_null_column_input_returns_sql_null` | EC-11-025-002 / postcondition §Null column |
| RG-JEX-005 | `test_jex_rg005_non_object_json_returns_sql_null` | EC-11-025-004 / postcondition §Non-object JSON |
| RG-JEX-006 | `test_jex_rg006_non_literal_key_rejected_e_query_045_a` | EC-11-025-006 / §Error Cases E-QUERY-045(a) |
| RG-JEX-007 | `test_jex_rg007_key_exceeds_max_len_rejected_e_query_045_b` | EC-11-025-007 / §Error Cases E-QUERY-045(b) |
| RG-JEX-008 | `test_jex_rg008_non_string_json_value_coerced_to_string` | EC-11-025-008 / postcondition §Non-string value |
| RG-JEX-009 | `test_jex_rg009_parse_failure_non_json_input_returns_sql_null` | EC-11-025-010 / postcondition §Parse failure |
| RG-JEX-010 | `test_jex_rg010_dot_in_key_literal_not_nested_path` | EC-11-025-009 / postcondition §Top-level key only |
| RG-JEX-011 | `test_jex_rg011_pipe_mode_end_to_end_executes` | EC-11-025-011 / SAP-3 pipe-mode reachability |

**BC-5.38.001 density check:** 11 Red Gate tests / 11 ACs = density 1.0 (≥ 0.5 minimum
required). All Red Gate tests must be authored and FAILING before implementation (T-04)
begins. RG-JEX-006 and RG-JEX-011 MUST be reachable from the `prism_query` public API
surface (SAP-3 standing probe), not only from a synthetic AST or direct DataFusion unit
test — a synthetic-AST-only path for either gate is a P2 finding per SAP-3.

---

## Architecture Mapping

| Component | File | Pure/Effectful | Classification |
|-----------|------|---------------|----------------|
| `json_extract_string_impl` pure function | `crates/prism-query/src/json_extract_udf.rs` (new) | Pure | No I/O, no global state, deterministic string → Option\<String\>; VP-162 Kani proof target |
| `JsonExtractStringUdf` struct + `ScalarUDFImpl` | `crates/prism-query/src/json_extract_udf.rs` (new) | Effectful (Arrow column I/O) | DataFusion `invoke_with_args(&self, args: ScalarFunctionArgs)` wrapper; iterates Arrow array per-row; calls `json_extract_string_impl` |
| UDF registration | `crates/prism-query/src/engine.rs` (modify) | Effectful | `ctx.register_udf(json_extract_string_udf())` at `QueryEngine::new` |
| `check_json_extract_key_literal` gate | `crates/prism-query/src/engine.rs` or `plan_gates.rs` (modify/new) | Pure | Pre-planning AST validation; fires after E-QUERY-043 gates, before `ctx.sql()` |
| `PrismError` variants | `crates/prism-core/src/error.rs` (modify) | Pure | `JsonExtractNonLiteralKey` + `JsonExtractKeyTooLong { key_len: usize, max_len: usize }` |
| VP-162 Kani proof harness | `crates/prism-query/src/proofs/vp162_json_extract_null_safety.rs` (new) | Pure | Phase 5 formal-verify target; authored alongside UDF implementation |

---

## Edge Cases

| ID | Scenario | Expected Behavior | AC / RG |
|----|----------|------------------|---------|
| EC-11-025-001 | Happy path: `'{"severity":"high","count":5}'` + key `'severity'` | Returns `"high"` (non-null Utf8 cell) | AC-001 / RG-JEX-001 |
| EC-11-025-002 | Null column: Arrow null cell + any key | Returns SQL NULL (null-propagating) | AC-004 / RG-JEX-004 |
| EC-11-025-003 | Missing key: `'{"severity":"high"}'` + key `'missing_key'` | Returns SQL NULL | AC-003 / RG-JEX-003 |
| EC-11-025-004 | Non-object: `'["a","b","c"]'` + any key | Returns SQL NULL (not an object) | AC-005 / RG-JEX-005 |
| EC-11-025-005 | JSON null at key: `'{"status":null}'` + key `'status'` | Returns SQL NULL | AC-002 / RG-JEX-002 |
| EC-11-025-006 | Non-literal key: `json_extract_string(col, other_col)` at plan time | E-QUERY-045(a) before fan-out | AC-006 / RG-JEX-006 |
| EC-11-025-007 | Key > 256 bytes: 257-byte literal key at plan time | E-QUERY-045(b) before fan-out | AC-007 / RG-JEX-007 |
| EC-11-025-008 | Non-string value: `'{"count":42}'` + key `'count'` | Returns `"42"` (coerced, NOT NULL) | AC-008 / RG-JEX-008 |
| EC-11-025-009 | Dot-in-key: `'{"a.b":"lit","a":{"b":"nested"}}'` + key `'a.b'` | Returns `"lit"` (literal top-level key, NOT nested path) | AC-010 / RG-JEX-010 |
| EC-11-025-010 | Parse failure: `'{not_json}'` + any key | Returns SQL NULL silently (no emission); silent null-propagation per BC-2.11.025 §Postconditions Parse-failure + ADR-066 §B1 step 2 | AC-009 / RG-JEX-009 |
| EC-11-025-011 | Pipe mode: MCP `query` tool + `json_extract_string(raw_extensions, 'severity')` | Correct wire-level extraction per SID-2 | AC-011 / RG-JEX-011 |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `json_extract_string_impl` | pure-core | Deterministic Option\<&str\> × &str → Option\<String\>; no I/O, no global state, no async. Uses `serde_json` + Option combinators only. VP-162 Kani proof target (ADR-066 §D1). |
| `JsonExtractStringUdf::invoke_with_args` | effectful-IO | Iterates Arrow `StringArray` (column I/O); calls pure `json_extract_string_impl` per row; builds output `StringArray`. |
| `check_json_extract_key_literal` gate | pure-core | Inspects AST `ScalarFunc::JsonExtractString` nodes; returns `Result<()>` with no side effects. |

---

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~6,000 |
| BC-2.11.025 v1.9 (full contract) | ~8,000 |
| ADR-066 v1.6 (§A–§H) | ~9,000 |
| VP-162 v1.6 (Kani harness) | ~3,500 |
| `crates/prism-query/src/engine.rs` (registration + plan-gate sections) | ~4,000 |
| `crates/prism-core/src/error.rs` (existing error variants for context) | ~2,000 |
| `crates/prism-query/src/ast.rs` (ScalarFunc enum — relevant section) | ~1,000 |
| Test files (11 Red Gate tests) | ~4,500 |
| **Total** | **~38,000** |
| Agent context window | ~200K (Sonnet) |
| **Budget usage** | **~19% (within 20-30% threshold)** |

---

## Tasks

Tasks are ordered RED-THEN-GREEN per SAC-1: test authoring precedes all implementation.

### Phase 1 — Stubs (compile-only; Red Gate prerequisite)

- [ ] **T-01:** Create `crates/prism-query/src/json_extract_udf.rs` with `todo!()` body stubs:
  `pub fn json_extract_string_udf() -> ScalarUDF { todo!() }` and
  `pub(crate) fn json_extract_string_impl(...) -> Option<String> { todo!() }`.
  Add `pub struct JsonExtractStringUdf` with `ScalarUDFImpl` trait impl returning
  `todo!()`. Wire `mod json_extract_udf;` in `engine.rs` or `lib.rs`. DO NOT register the
  UDF or implement the gate yet. Goal: workspace compiles, all existing tests pass.

- [ ] **T-02:** Add `PrismError::JsonExtractNonLiteralKey` and
  `PrismError::JsonExtractKeyTooLong { key_len: usize, max_len: usize }` to
  `crates/prism-core/src/error.rs`. Add MCP `-32602` mapping for both variants in the
  error → MCP code conversion path.

- [ ] **T-03:** Verify `just iter prism-query` GREEN with stubs (all pre-existing tests pass).

### Phase 2 — Red Gate test authoring (all tests must FAIL at end of this phase)

- [ ] **T-04 (test-writer dispatch):** Author all 11 Red Gate tests in the canonical location
  (`crates/prism-query/tests/` or `crates/prism-query/src/json_extract_udf.rs` `#[cfg(test)]`
  block). Test names must exactly match the RG-JEX-001..011 list above. Tests for RG-JEX-006
  and RG-JEX-011 MUST invoke from the `prism_query` public API surface (SAP-3; not only
  synthetic AST). Tests for AC-011 / RG-JEX-011 MUST include a SID-2 wire-shape assertion
  on the serialized JSON output (not only pre-serialization Rust structs).
  
  **Density gate:** 11 failing tests ≥ 0.5 × 11 ACs = density 1.0. All 11 must be FAILING
  (Red Gate, per BC-5.38.001) before Phase 3 begins. Run `just iter prism-query --no-fail-fast`
  and confirm 11 failures.

### Phase 3 — Implementation (one RG at a time)

- [ ] **T-05:** Implement `json_extract_string_impl` pure function (ADR-066 §B1 steps 1–7):
  - Step 1: `None` input → `None` (AC-004)
  - Step 2: `serde_json::from_str` failure → `None` (AC-009)
  - Step 3: non-`Value::Object` → `None` (AC-005)
  - Step 4: key absent from object → `None` (AC-003)
  - Step 5: `Value::Null` at key → `None` (AC-002)
  - Step 6: `Value::String(s)` at key → `Some(s.clone())` (AC-001)
  - Step 7: non-string at key → `Some(value.to_string())` (AC-008)
  Run `just iter prism-query -E 'test(test_jex_rg001|rg002|rg003|rg004|rg005|rg008|rg009)'`
  — target: 7 GREEN.

- [ ] **T-06:** Author VP-162 Kani proof harness file
  `crates/prism-query/src/proofs/vp162_json_extract_null_safety.rs` per VP-162 §Kani Proof
  Harness (two harnesses: `vp162_json_extract_string_null_safety` + `vp162_b_none_input_is_none_output`).
  The harness is authored now alongside the implementation so Phase 5 dispatch is ready.

- [ ] **T-07:** Wire `json_extract_string_udf()` into `engine.rs` at `SessionContext`
  construction: `ctx.register_udf(json_extract_string_udf())`. Register ONCE at engine
  construction per BC-2.11.025 postcondition §Registration (ADR-066 §E). Run
  `just iter prism-query -E 'test(test_jex_rg001)'` — RG-JEX-001 GREEN.

- [ ] **T-08:** Implement dot-in-key literal behavior (top-level key `.get(key)` without
  path interpretation per ADR-066 §D4). Run `just iter prism-query -E 'test(test_jex_rg010)'`
  — RG-JEX-010 GREEN.

- [ ] **T-09:** Implement `check_json_extract_key_literal` plan gate in `engine.rs` or
  new `plan_gates.rs`. Gate must:
  - Walk AST for `ScalarFunc::JsonExtractString` nodes
  - Reject non-`Expr::Literal(Literal::String(_))` second argument with E-QUERY-045(a)
  - Reject literal key > 256 UTF-8 bytes with E-QUERY-045(b)
  - Fire AFTER plan-time gates E-QUERY-037/038/039, BEFORE `ctx.sql()` / DataFusion
    execution; temporal E-QUERY-041/042 runs in-pipeline per ADR-052 §D4 (inside
    `run_materialization_pipeline`) — not part of the pre-execution gate sequence; no
    specific ordering between E-QUERY-045 and in-pipeline temporal asserted beyond
    ADR-066 §B3 "before DataFusion execution"
  - Use verbatim error messages from ADR-066 §F / BC-2.11.025 §Error Cases (including
    `{key_len}` and `{max_len}` substitutions)
  Run `just iter prism-query -E 'test(test_jex_rg006|rg007)'` — RG-JEX-006 + RG-JEX-007 GREEN.

- [ ] **T-10:** Implement pipe-mode end-to-end test path (SAP-3). The `prism_query` public
  API test for RG-JEX-011 must route through the full pipe-mode path: PQL pipe expression →
  `pipe_sql_emitter` → SQL → plan gate → DataFusion execution → extracted column in result.
  Wire-shape assertion: assert on serialized JSON output bytes (SID-2). Run
  `just iter prism-query -E 'test(test_jex_rg011)'` — RG-JEX-011 GREEN.

- [ ] **T-11:** Run `just iter prism-query` — all 11 Red Gate tests GREEN; all pre-existing
  tests GREEN (no regression).

### Phase 4 — Observability and compliance

- [ ] **T-12 (purity gate):** Confirm that `json_extract_string_impl` and
  `JsonExtractStringUdf::invoke_with_args` contain no `tracing::*!` calls. The
  implementation function is pure (ADR-066 §D1); per-row warn emissions are forbidden.
  `serde_json::from_str` failure → `None` is the complete handling per AC-009 and
  BC-2.11.025 §Postconditions Parse-failure. No event-catalog row is required or
  permitted for parse-failure behavior in this story.

- [ ] **T-13:** Run `just check` — full workspace gate GREEN (`just iter` was inner loop;
  `just check` is the pre-push canonical gate).

### Phase 5 — Security and documentation

- [ ] **T-14 (security review — BLOCKING):** Dispatch security-reviewer for this story
  before PR creation. The `json_extract_string` plan gate is an agent-facing injection
  prevention surface (CLAUDE.md §agent-harness). Security reviewer must verify:
  (a) literal-key gate cannot be bypassed at plan time (ADR-066 §B3);
  (b) 256-byte cap enforced before any UDF execution (ADR-066 §D3);
  (c) `json_extract_string_impl` has no `unsafe` blocks and no panic paths (VP-162);
  (d) E-QUERY-045 error messages do not echo back raw user key content in a way that
  could enable reflection or timing attacks.
  No merge without a clean security-reviewer pass.

- [ ] **T-15 (CHANGELOG):** Add CHANGELOG entry under [Unreleased] > Added BEFORE creating
  the PR:
  ```
  - json_extract_string(column, 'key') ScalarUDF registered in DataFusion query engine
    (prism-query). Literal-key plan gate (E-QUERY-045) enforces injection prevention;
    256-byte key-length cap enforces CWE-400. Closes beta.2 dead-path defect (Issue 20).
  ```

- [ ] **T-16:** Create PR targeting `develop`. PR description must include:
  - Link to ADR-066 v1.6 and BC-2.11.025 v1.9
  - Summary of 11 Red Gate tests (RG-JEX-001..011) all GREEN
  - Security review confirmation (T-14 PASS)
  - SAP-1 / SAP-3 compliance confirmation

---

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| S-ADR058-OCSF-ROUTING-001 | `raw_extensions` is the Tier-2 Arrow column carrying non-OCSF-mapped fields as a JSON blob per BC-2.16.003 §Interpretation A Tier-2 | Tier-2 accumulation loop in `pipeline_result_to_record_batch`; the JSON blob is an Arrow `Utf8` column | Tier-2 filtering requires a ScalarUDF — DataFusion does not support `$.path` syntax natively; this story closes that gap |

**Note on fast-follow stubs:** S-JSON-EXTRACT-TYPED-001 and S-JSON-EXTRACT-NESTED-001
currently have stale `depends_on:` frontmatter. Per delta-analysis.md §Issue 20 D-2521
reconciliation, story-writer must update those stubs' `depends_on:` fields to reference
`S-JSON-EXTRACT-UDF-001` explicitly before they are dispatched. This is a separate burst
from implementing this story.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `json_extract_string` UDF registered at `QueryEngine::new` under exact name `"json_extract_string"` with types `(Utf8, Utf8)` → `Utf8` nullable and `Volatility::Immutable` | ADR-066 §E + BC-2.11.025 postcondition §Registration | RG-JEX-001; adversary §E check |
| `check_json_extract_key_literal` gate fires AFTER plan-time gates E-QUERY-037/038/039, BEFORE `ctx.sql()` / DataFusion execution — never skipped for any query mode; temporal E-QUERY-041/042 runs in-pipeline per ADR-052 §D4 (not part of pre-execution gate ordering); no specific ordering between E-QUERY-045 and in-pipeline temporal asserted beyond ADR-066 §B3 | BC-2.11.025 v1.9 postcondition §Plan-time literal-key gate; ADR-066 §B3 | RG-JEX-006 (SAP-3 public surface); adversary gate-ordering check |
| 256-byte key cap enforced at plan time (byte length of UTF-8 encoded literal key ≤ 256), NOT at runtime | ADR-066 §D3 + BC-2.11.025 §Error Cases E-QUERY-045(b) | RG-JEX-007; zero per-row overhead confirmed |
| `json_extract_string_impl` MUST be a `pub(crate)` pure function in `json_extract_udf.rs` — no DataFusion or Arrow types in its signature | ADR-066 §B1 + VP-162 §Proof Target | VP-162 Kani harness provability; security review T-14 |
| `prism-query` MUST NOT gain a dependency on `prism-bin` | dependency-graph.md §Dependency Rules Rule 2 (Level 6 / Level 7 ordering) | `cargo tree -p prism-query` must show no `prism-bin` edge post-merge |
| No `unsafe` blocks in `json_extract_udf.rs` or `plan_gates.rs` additions | CLAUDE.md §Conventions (error taxonomy + no-unwrap rule) | Security review T-14 + `just check` clippy |
| VP-162 Kani harness file created alongside implementation (Phase 3 T-06) | VP-162 v1.6 §Kani Proof Harness | Phase 5 formal-verify dispatch; harness must compile clean under `cargo kani -p prism-query` before merge |

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `serde_json` | `"1"` (workspace pin per `Cargo.toml`) | JSON parsing and key extraction in `json_extract_string_impl`; `Value::as_object().get(key)` API; already in dependency tree |
| `datafusion` | `53.1` (workspace pin per `Cargo.toml`) | `ScalarUDF`, `ScalarUDFImpl`, `Volatility`, `Signature`, `ColumnarValue` API for UDF registration and invocation |
| `arrow` | `58.x` (transitive via datafusion 53.1) | `StringArray`, `ArrayRef` for `invoke_with_args` Arrow column iteration |
| `kani-verifier` | per `rust-toolchain.toml` / `Cargo.toml` dev-dep | VP-162 Kani proof harness (authored in T-06; dispatched Phase 5) |
| Rust stable | per `rust-toolchain.toml` | Build toolchain |

**D-1110 remove-uncertainty resolution (confirmed):** `ScalarUDFImpl` in DataFusion 53.1
uses `fn invoke_with_args(&self, args: ScalarFunctionArgs) -> DataFusionResult<ColumnarValue>`.
`invoke_batch` was deprecated at DataFusion 46.0 and does not appear anywhere in this
workspace. Confirmed via `crates/prism-query/src/infusion_udf.rs` (`impl ScalarUDFImpl for
InfusionAsyncUdf`) and Context7 DataFusion official docs. The factory function pattern is
`ScalarUDF::from(JsonExtractStringUdf::new())` (uses `From<impl ScalarUDFImpl>` — confirmed
from Context7 DataFusion registration example). If ADR-066 §E uses `invoke_batch`, that is a
stale illustration; the implementer MUST use `invoke_with_args` with `ScalarFunctionArgs`.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-query/src/json_extract_udf.rs` | Create (new) | `json_extract_string_udf()` factory fn + `JsonExtractStringUdf` struct + `ScalarUDFImpl` impl + `pub(crate) json_extract_string_impl()` pure function |
| `crates/prism-query/src/engine.rs` | Modify | Register UDF at `QueryEngine::new`: `ctx.register_udf(json_extract_string_udf())`. Add or call `check_json_extract_key_literal` gate in plan-validation step (after E-QUERY-043, before `ctx.sql()`). Wire `mod json_extract_udf;` |
| `crates/prism-query/src/engine.rs` or `crates/prism-query/src/plan_gates.rs` | Modify or Create | `check_json_extract_key_literal(ast: &PrismQuery) -> Result<(), PrismError>` — AST walk rejecting non-literal key arguments |
| `crates/prism-core/src/error.rs` | Modify | Add `PrismError::JsonExtractNonLiteralKey` and `PrismError::JsonExtractKeyTooLong { key_len: usize, max_len: usize }` variants; wire into MCP `-32602` mapping |
| `crates/prism-query/src/proofs/vp162_json_extract_null_safety.rs` | Create (new) | VP-162 Kani harnesses (two harnesses per VP-162 §Kani Proof Harness); `#[cfg(kani)]` gated |
| `crates/prism-query/tests/test_json_extract_udf.rs` or inline `#[cfg(test)]` block | Create or Modify | 11 Red Gate tests (RG-JEX-001..011); RG-JEX-006 + RG-JEX-011 MUST use `prism_query` public API surface (SAP-3) |
| `CHANGELOG.md` | Modify | [Unreleased] > Added: `json_extract_string` UDF entry (T-15) |

### Forbidden Dependencies

The following MUST NOT appear in `crates/prism-query`'s dependency graph after this story.
If any of these appear, the build MUST fail (checked by `cargo tree -p prism-query` in CI):

| Forbidden Dep | Reason |
|--------------|--------|
| `prism-bin` | Level 6/7 ordering constraint per dependency-graph.md §Dependency Rules Rule 2 |
| `arrow-json` (the arrow JSON reader crate) | Rejected in ADR-066 §B2 — bulk NDJSON deserializer; unsuitable for per-row scalar extraction; carries `explicit_nulls` defect risk (DEFECT-MCP-ROWSHAPE-NULLS-001) |
| Any `async` executor or `tokio::spawn` in `json_extract_udf.rs` | UDF is synchronous per ADR-066 §B1; async usage in `invoke_with_args` would violate DataFusion's `Immutable` volatility contract |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.6 | 2026-09-17 | state-manager | D-2556 LOCAL pass CLEAN(PR-merge) errata pin sync. VP-162 v1.5→v1.6 authority pins updated in §Authority, FROZEN NOTE, Token Budget table, and Architecture Compliance Rules (TD-VSDD-060 sibling-site sweep; F-JEX-P1-003: §Kani Proof Harness inner module renamed vp162_proofs→kani_proofs; sibling convention + code rename @9ce8642e). Historical changelog rows preserved verbatim. |
| 1.5 | 2026-09-17 | story-writer | LOW-1 (LOCAL pass): AC-006 / T-09 / Architecture Compliance Rules gate-ordering reconciled to BC-2.11.025 v1.9 (stale E-QUERY-041/042/043 pre-execution enumeration corrected; temporal in-pipeline per ADR-052 §D4). BC-2.11.025 body-pin updated v1.8 → v1.9 in §Authority, FROZEN NOTE, Behavioral Contracts table, Token Budget, and T-16 PR description (POL-8 bc_array_changes_propagate_to_body_and_acs; frontmatter pin left for state-manager). RG-JEX-006 gate mapping unchanged — gate still asserts fire-before-execution. |
| 1.4 | 2026-09-17 | state-manager | D-2553 LOCAL re-gate CLEAN(PR-merge) errata pin sync. VP-162 v1.4→v1.5 authority pins updated in §Authority, FROZEN NOTE, Token Budget table, and Architecture Compliance Rules (TD-VSDD-060 sibling-site sweep; v1.3 cite in Architecture Compliance Rules also corrected — stale from D-2551 sweep miss). Historical changelog rows preserved verbatim. |
| 1.3 | 2026-09-17 | story-writer | F-2 (LOCAL pass-1): AC-009 + EC-11-025-010 aligned to ratified BC-2.11.025/ADR-066 silent-null-propagation contract; type_mismatch warn/catalog obligation struck (precedence rule 1: BC supersedes on contract semantics; ADR-066 §D1 purity). T-12 rewritten as purity-gate verification. SAP-1 Architecture Compliance row removed — no emissions exist in this story. |
| 1.2 | 2026-09-17 | state-manager | D-2551 pre-TDD errata pin sync. ADR-066 v1.5→v1.6 and VP-162 v1.3→v1.4 authority pins updated in §Authority, FROZEN NOTE, Token Budget table, and T-16 PR bullet (TD-VSDD-060 sibling-site sweep). Historical changelog rows preserved verbatim. Additive/errata post-freeze; spec-gate NOT reopened. |
| 1.1 | 2026-09-17 | story-writer | D-1110 remove-uncertainty pass. Three corrections applied in-scope: (1) Architecture Mapping `invoke_batch` → `invoke_with_args(&self, args: ScalarFunctionArgs)` — confirmed DataFusion 53.1 method via `infusion_udf.rs` `impl ScalarUDFImpl` and Context7 docs; `invoke_batch` deprecated at DataFusion 46.0, absent from workspace. (2) Library & Framework Requirements implementer obligation rewritten from uncertain "resolve X vs Y vs Z" to definitive: `invoke_with_args` + `ScalarUDF::from(impl)` factory pattern confirmed. (3) Problem Statement dead-code location annotations converted from volatile line numbers to symbol/grep anchors per TD-VSDD-091; factual error corrected (sql_parser.rs had one site at the function-name match arm, not two — the erroneous second reference was verified absent by grep). |
| 1.0 | 2026-09-17 | story-writer | Stub → full materialization. Full AC layer (AC-001..011) traced to BC-2.11.025 v1.8 EC-11-025-001..011. Enumerated RG-JEX-001..011 list with canonical test names per ADR-066 §G + BC-2.11.025 §Edge Cases MUST anchors. Red-then-green task ordering (T-01..T-16) per SAC-1. BC-5.38.001 density check: 11/11 = 1.0. Behavioral contracts: BC-2.11.025. Verification properties: VP-162. Frozen anchor pins: ADR-066 v1.5, BC-2.11.025 v1.8, VP-162 v1.3 (confirmed from ARCH-INDEX/BC-INDEX/VP-INDEX). Security reviewer pass (T-14) mandated — agent-facing injection surface. Fast-follow depends_on reconciliation note added (S-JSON-EXTRACT-TYPED-001, S-JSON-EXTRACT-NESTED-001 stubs need updating). Forbidden dependencies section added. |
| 0.1 | 2026-08-21 | story-writer | Initial draft stub — scope capture per OQ-002 human decision 2026-08-21; full BC/AC/RG deferred to materialization. v1-chain obligation documented. |
