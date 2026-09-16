---
document_type: behavioral-contract
level: L3
version: "1.8"
status: draft
producer: product-owner
timestamp: 2026-09-16T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: draft
introduced: 2026-09-16
modified: "2026-09-16"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-066-json-extract-scalar-udf.md"
input-hash: "TBD"
traces_to: ["CAP-015"]
related_bcs: ["BC-2.11.001", "BC-2.11.016", "BC-2.11.019"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.11.025: `json_extract_string` DataFusion ScalarUDF — Literal-Key-Only JSON String Extraction

## Description

`json_extract_string(column, key)` is a synchronous DataFusion ScalarUDF registered in the query engine at construction time. It extracts a single top-level string value from a JSON-encoded column using a literal string key that must be supplied at plan time. The UDF is null-safe and pure: null column, parse failure, non-object JSON, missing key, and JSON null all produce SQL NULL. Non-string JSON values are coerced to string via `to_string()`. The literal-key restriction is enforced as a plan-time gate (ADR-066 §B3) that fires E-QUERY-045 before any execution or fan-out; this restriction is permanent per ADR-066 §D2 and is not relaxed by S-JSON-EXTRACT-NESTED-001.

## Preconditions

- The `json_extract_string` ScalarUDF is registered in the DataFusion SessionContext during engine construction (at `QueryEngine::new` time, before any queries execute) using the name `"json_extract_string"`, input types `(Utf8, Utf8)`, return type `Utf8` (nullable), and `Volatility::Immutable` (ADR-066 §E)
- The UDF implementation function is `json_extract_string_impl(column_value: Option<&str>, key: &str) -> Option<String>` in `crates/prism-query/src/json_extract_udf.rs` (ADR-066 §E)
- At plan time, the second argument to `json_extract_string` MUST be a string literal (`Expr::Literal(Literal::String(_))`); a non-literal second argument causes plan-time rejection with E-QUERY-045(a) before any execution or fan-out
- At plan time, the literal key MUST be ≤256 bytes (UTF-8 byte length); a key exceeding 256 bytes causes plan-time rejection with E-QUERY-045(b) before any execution (ADR-066 §D3, CWE-400)
- The first argument MUST be a column reference resolving to a `Utf8` (String) column; column-type errors are governed by E-QUERY-002, not E-QUERY-045

## Postconditions

- **Happy path:** when `column` is non-null, the column value parses as a JSON object, and `key` names a top-level key whose value is a JSON string, `json_extract_string` returns that string value as a non-null `Utf8` Arrow cell
- **Null column:** when `column` is SQL NULL (Arrow null), the return value is SQL NULL — the function is null-propagating (ADR-066 §B1)
- **Missing key:** when the JSON object does not contain a top-level key matching `key`, the return value is SQL NULL (ADR-066 §B1)
- **Non-object JSON:** when the column value parses as a JSON value other than an object (array, string, number, boolean, or JSON literal null `null`), the return value is SQL NULL (ADR-066 §B1)
- **JSON null at key:** when the JSON object contains `key` but its value is JSON null (`Value::Null`), the return value is SQL NULL (ADR-066 §B1)
- **Non-string value at key:** when the JSON object contains `key` and its value is a non-string JSON value (number, boolean, nested object, or array), the return value is `Some(value.to_string())` — coerced to a string representation (e.g., integer `42` → `"42"`, boolean `true` → `"true"`) (ADR-066 §B1)
- **Parse failure:** when the column value is not valid JSON, the return value is SQL NULL (ADR-066 §B1)
- **Plan-time literal-key gate (ADR-066 §B3):** the `check_json_extract_key_literal` gate fires at plan time (after E-QUERY-037/038/039/041/042/043 gates, before DataFusion execution). Sub-case (a): second argument is not `Expr::Literal(Literal::String(_))` → E-QUERY-045(a). Sub-case (b): literal key exceeds 256 UTF-8 bytes → E-QUERY-045(b). No sensor fan-out occurs for a rejected query.
- **Top-level key only (ADR-066 §D4):** `json_extract_string` extracts a single top-level JSON object key. Nested path expressions (e.g., `"a.b.c"`) are treated as the literal key string `"a.b.c"` — they are NOT interpreted as JSONPath. Nested extraction is out of scope in beta.3.
- **No sensor push-down (ADR-066 §D6):** `json_extract_string` is NOT push-down eligible; it is evaluated in the DataFusion execution layer after sensor fan-out and materialization.
- **Volatility (ADR-066 §E):** `Volatility::Immutable` — the same inputs always produce the same output; DataFusion may cache results for repeated identical calls within a query plan.
- **Registration (ADR-066 §E):** UDF is registered ONCE at engine construction; it is available in every DataFusion SessionContext created by the engine without re-registration.

## Invariants

- DI-019 ("Query Security Limits"): the `json_extract_string` UDF is subject to the same 30s timeout and 10K record cap as all query execution; memory budget is governed by NFR-015, not DI-019
- The literal-key plan-time gate is PERMANENT: dynamic key expressions (column references, subqueries, function calls as the key argument) will never be supported without a new ADR authorizing a distinct UDF variant (ADR-066 §D2)
- The key length cap is 256 UTF-8 bytes (CWE-400 mitigation per ADR-066 §D3); the cap applies to the byte length of the literal key, not its Unicode character count
- The `json_extract_string_impl` function is a pure function (ADR-066 §B1): it has no side effects, no I/O, and no global mutable state; `Volatility::Immutable` registration is its correct DataFusion classification (ADR-066 §E)
- The three previously dead AST paths (`ast.rs ScalarFunc::JsonExtractString`, `sql_parser.rs`, `pipe_sql_emitter.rs`) are wired by S-JSON-EXTRACT-UDF-001; no new dead-path debt is introduced (ADR-066 §A)

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-045(a)` | Second argument to `json_extract_string` is not a string literal at plan time (e.g., a column reference, expression, or computed value: `json_extract_string(col, other_col)`) | Plan-time rejection before execution or fan-out. `PrismError::JsonExtractNonLiteralKey`. MCP `-32602` INVALID_PARAMS. Message: `"E-QUERY-045: json_extract_string requires a literal string key (e.g., json_extract_string(col, 'key_name')). Dynamic key expressions are not supported."` |
| `E-QUERY-045(b)` | Literal key exceeds 256 UTF-8 bytes (CWE-400 guard, ADR-066 §D3) | Plan-time rejection before execution or fan-out. `PrismError::JsonExtractKeyTooLong { key_len: usize, max_len: usize }`. MCP `-32602` INVALID_PARAMS. Message: `"E-QUERY-045: json_extract_string key is {key_len} bytes, which exceeds the {max_len}-byte maximum (CWE-400)."` |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-025-001 | Happy path: `json_extract_string(col, 'severity')` where `col = '{"severity":"high","count":5}'` | Returns `"high"` (UTF-8 string, non-null Arrow cell). MUST: S-JSON-EXTRACT-UDF-001 RG-JEX-001 (`test_jex_rg001_udf_registered_happy_path_executes`). |
| EC-11-025-002 | Null column: `json_extract_string(col, 'severity')` where `col` is Arrow null | Returns SQL NULL (Arrow null cell). Null-propagating per ADR-066 §B1. MUST: S-JSON-EXTRACT-UDF-001 RG-JEX-004 (`test_jex_rg004_null_column_input_returns_sql_null`). |
| EC-11-025-003 | Missing key: `json_extract_string(col, 'missing_key')` where `col = '{"severity":"high"}'` | Returns SQL NULL — key `"missing_key"` absent from object. MUST: S-JSON-EXTRACT-UDF-001 RG-JEX-003 (`test_jex_rg003_missing_key_returns_sql_null`). |
| EC-11-025-004 | Non-object JSON: `json_extract_string(col, 'key')` where `col = '["a","b","c"]'` (JSON array) | Returns SQL NULL — input is not a JSON object. Also applies to JSON string/number/boolean column values. MUST: S-JSON-EXTRACT-UDF-001 RG-JEX-005 (`test_jex_rg005_non_object_json_returns_sql_null`). |
| EC-11-025-005 | JSON null at key: `json_extract_string(col, 'status')` where `col = '{"status":null}'` | Returns SQL NULL — JSON null value at `"status"` treated as absent string value per ADR-066 §B1. MUST: S-JSON-EXTRACT-UDF-001 RG-JEX-002 (`test_jex_rg002_json_null_value_at_key_returns_sql_null`). |
| EC-11-025-006 | Non-literal key at plan time: `json_extract_string(col, other_col)` where `other_col` is a column reference | Plan-time rejection with E-QUERY-045(a). Gate fires AFTER E-QUERY-037/038/039/041/042/043, BEFORE DataFusion execution. No fan-out occurs. MUST: S-JSON-EXTRACT-UDF-001 RG-JEX-006 (`test_jex_rg006_non_literal_key_rejected_e_query_045_a`). |
| EC-11-025-007 | Key exceeds 256 bytes at plan time: `json_extract_string(col, '<257-byte-literal>')` | Plan-time rejection with E-QUERY-045(b): `key_len = 257`, `max_len = 256`. No fan-out occurs. CWE-400 guard per ADR-066 §D3. MUST: S-JSON-EXTRACT-UDF-001 RG-JEX-007 (`test_jex_rg007_key_exceeds_max_len_rejected_e_query_045_b`). |
| EC-11-025-008 | Non-string value at key (integer coercion): `json_extract_string(col, 'count')` where `col = '{"count":42}'` | Returns `"42"` — integer coerced via `to_string()`. Booleans (`true` → `"true"`), objects, and arrays are coerced similarly to their JSON text representation. MUST: S-JSON-EXTRACT-UDF-001 RG-JEX-008 (`test_jex_rg008_non_string_json_value_coerced_to_string`). |
| EC-11-025-009 | Dot in literal key: `json_extract_string(col, 'a.b')` where `col = '{"a.b":"literal_val","a":{"b":"nested"}}'` | Returns `"literal_val"` — the literal key `"a.b"` matches the top-level key named exactly `"a.b"`, NOT the nested path `a → b`. Top-level-key-only per ADR-066 §D4; nested JSONPath is out of scope in beta.3. MUST: dot-in-key treated as literal top-level key, NOT JSONPath nested traversal (ADR-066 §D4) — S-JSON-EXTRACT-UDF-001 RG-JEX-010 (`test_jex_rg010_dot_in_key_literal_not_nested_path`). |
| EC-11-025-010 | Parse failure: `json_extract_string(col, 'key')` where `col = 'not valid json'` (column value is malformed JSON — e.g., `{not_json}`, raw text, or a truncated JSON fragment) | Returns SQL NULL — column value does not parse as valid JSON; `serde_json::from_str` error treated as absent value (null-propagating per ADR-066 §B1). This does NOT produce E-QUERY-045 — that gate fires for literal-key constraint violations at plan time, not runtime parse failures. Same NULL-return semantics as non-object input (EC-11-025-004). MUST: parse failure returns SQL NULL without E-QUERY-045 — S-JSON-EXTRACT-UDF-001 RG-JEX-009 (`test_jex_rg009_parse_failure_non_json_input_returns_sql_null`). |
| EC-11-025-011 | Pipe-mode end-to-end reachability (SAP-3): `FROM claroty_alerts \| SELECT json_extract_string(raw_payload, 'severity')` — `json_extract_string` called via a pipe-mode PrismQL query through the MCP `query` tool public API surface | Returns the extracted string value for each row where `raw_payload` is a valid JSON object containing the `"severity"` key; SQL NULL for rows where `raw_payload` is null, absent, parse-fails, or has no `"severity"` key. **SAP-3 reachability obligation (CLAUDE.md §SAP-3):** this probe verifies that `json_extract_string` is reachable end-to-end from the pipe-mode MCP surface — not only from a direct DataFusion unit test or synthetic AST. **Wire-level assertion:** the serialized JSON output from the MCP `query` tool MUST include the extracted value in the correct column per EC-11-079 null-not-absent invariant (key present as `null` for NULL rows, non-null string for extracted rows). MUST: pipe-mode MCP `query` call using `json_extract_string` returns correct wire-level output — S-JSON-EXTRACT-UDF-001 RG-JEX-011 (`test_jex_rg011_pipe_mode_end_to_end_executes`). |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `json_extract_string('{"severity":"high","count":5}', 'severity')` | `"high"` | happy-path (EC-11-025-001) |
| `json_extract_string(NULL, 'key')` | SQL NULL | null-propagation (EC-11-025-002) |
| `json_extract_string('{"severity":"high"}', 'missing')` | SQL NULL | missing-key (EC-11-025-003) |
| `json_extract_string('["a","b"]', 'key')` | SQL NULL | non-object (EC-11-025-004) |
| `json_extract_string('{"status":null}', 'status')` | SQL NULL | json-null-at-key (EC-11-025-005) |
| `json_extract_string(col, other_col)` (non-literal second arg) | E-QUERY-045(a) at plan time | plan-gate non-literal (EC-11-025-006) |
| `json_extract_string(col, '<257-byte-string>')` | E-QUERY-045(b) at plan time | plan-gate key-too-long (EC-11-025-007) |
| `json_extract_string('{"count":42}', 'count')` | `"42"` | non-string-coercion (EC-11-025-008) |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-162 | **Kani-proven:** (1) null-safety and panic-freedom — `json_extract_string_impl` never panics on any input; (2) None-in→None-out — when `column_value` is `None` (the Arrow/SQL column cell is SQL NULL), `json_extract_string_impl` always returns `None`, regardless of `key` (VP-162-B harness); (3) structural key-length bound — any key ≤ 256 bytes accepted without overflow (ADR-066 §D1; VP-162 §Kani Proof Harness). **NOT Kani-proven — covered by RG-JEX Red Gate tests (ADR-066 §G):** behavioral routing (missing/parse-failure/non-object/JSON-null → `None`; string → `Some(s)`; non-string → `Some(value.to_string())`). | kani |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| Capability Anchor Justification | CAP-015 ("Ephemeral OCSF Query Engine") per capabilities.md §CAP-015 — this BC specifies a DataFusion ScalarUDF registered in the ephemeral query engine at construction time, governed by a plan-time validation gate (E-QUERY-045) that fires within the standard PrismQL gate ordering. Registration at engine construction and plan-time gate enforcement are both core CAP-015 behaviors per the ephemeral federated query engine design (ADR-066 §E / §B3). |
| L2 Invariants | DI-019 |
| L2 Edge Cases | DEC-023, DEC-026 |
| Priority | P0 |
| §Authority | ADR-066 §G |

## Related BCs

- BC-2.11.001 — composes with: the `query` MCP tool is the entry point for all PrismQL queries; `json_extract_string` is available in any query governed by BC-2.11.001; E-QUERY-045 slots into the gate ordering after E-QUERY-043
- BC-2.11.016 — composes with: E-QUERY-038 column-not-found gate fires before E-QUERY-045 if the first argument references an unregistered column
- BC-2.11.019 — composes with: E-QUERY-039 enrich-UDF-not-found gate fires before E-QUERY-045 in the gate ordering

## Architecture Anchors

- `crates/prism-query/src/json_extract_udf.rs` — `json_extract_string_impl` pure function + `JsonExtractStringUdf` struct implementing DataFusion `ScalarUDFImpl`
- `crates/prism-query/src/engine.rs` — UDF registration at `QueryEngine::new` (before any query executes)
- `crates/prism-query/src/plan_gates.rs` (or equivalent plan-time validation module) — `check_json_extract_key_literal` gate
- `crates/prism-core/src/error.rs` — `PrismError::JsonExtractNonLiteralKey` and `PrismError::JsonExtractKeyTooLong` variants
- `crates/prism-query/src/proofs/` — VP-162 Kani proof for `json_extract_string_impl`

## Story Anchor

S-JSON-EXTRACT-UDF-001 (beta.3 remediation cycle F2, issue 20)

## VP Anchors

VP-162 — `json_extract_string_impl` pure function bounds proof (Kani; ADR-066 §D1)

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.8 | beta3-f13-regate11-spec | 2026-09-16 | product-owner | **F-1 (MED — mis-anchor) + Consistency F-1 (LOW — residual NOT-Kani overlap).** **F-1:** §Invariants purity bullet cited `(ADR-066 §B2)` but §B2 is "Why serde_json (Not arrow-rs JSON Reader)" — no purity or volatility content. Purity (`pure extraction function`) is in §B1; `Volatility::Immutable` is defined in §E. **Fix:** `(ADR-066 §B2)` → `(ADR-066 §B1)` for the purity claim; `(ADR-066 §E)` added after the `Volatility::Immutable` clause. Final invariant: "The `json_extract_string_impl` function is a pure function (ADR-066 §B1): it has no side effects, no I/O, and no global mutable state; `Volatility::Immutable` registration is its correct DataFusion classification (ADR-066 §E)". No §B2 cite remains for purity/volatility in that bullet. **Consistency F-1:** §Verification Properties VP-162 NOT-Kani-proven list read "null/missing/parse-failure/non-object/JSON-null → None". The leading "null" denotes the Arrow/SQL-NULL (`column_value = None`) case — the same case already claimed in Kani-proven item (2) (VP-162-B harness). Listing it in NOT-Kani contradicts VP-162-B. **Fix:** removed leading "null" token from the NOT-Kani list; it now reads "missing/parse-failure/non-object/JSON-null → None". "JSON-null" (the distinct `Value::Null`-at-key case) retained in NOT-Kani as correct. Post-fix: Arrow/SQL-NULL input case appears ONLY in Kani-proven item (2); NOT-Kani list no longer starts with bare "null". **TD-VSDD-097 three-dimension discharge:** (1) Dim-1 sibling — no named split-event twin; CLEAR. (2) Dim-2 downstream copy target — §Invariants purity bullet not verbatim-copied in downstream artifacts; VP-162 §Property Statement is the upstream source for the Kani scope; this BC is the downstream copy; this burst IS the Dim-2 discharge for both findings. (3) Dim-3 mandate anchor — no new MUSTs; CLEAR. BC-INDEX updated same burst: pin bumped v1.7→v1.8. |
| 1.7 | beta3-f1-regate10-spec | 2026-09-16 | product-owner | **F-1 (MED — internal contradiction in VP-162 scope).** Re-gate pass-10 found that §Verification Properties VP-162 row item (2) read "(2) None-in→None-out — `Value::Null` and `Value::Object` with a missing key each produce `None`" — contradicting itself: `Value::Null` (JSON null at a key) and missing-key-in-object are behavioral routing cases already listed in the "NOT Kani-proven — covered by RG-JEX" section; item (2) must refer to the Arrow/SQL NULL case (`column_value = None`) that VP-162-B actually proves (ADR-066 §C item 2). The overlap caused both sections to claim the same concept in incompatible terms. **Fix:** item (2) replaced verbatim per adversary directive: "(2) None-in→None-out — when `column_value` is `None` (the Arrow/SQL column cell is SQL NULL), `json_extract_string_impl` always returns `None`, regardless of `key` (VP-162-B harness)". `Value::Null`, `Value::Object`, and "missing key" phrases removed from Kani-proven section; those concepts remain only in the NOT-Kani-proven RG-JEX routing list where they correctly belong. Post-fix: Kani-proven lists exactly {null-safety/panic-freedom; None-in(SQL NULL)→None-out; structural any_vec ≤256 key bound}; no term appears in both sections. **TD-VSDD-097 three-dimension discharge:** (1) Dim-1 sibling — no named split-event twin; CLEAR. (2) Dim-2 downstream copy target — VP-162 §Property Statement is the upstream authoritative source; this BC is the downstream copy; this burst IS the Dim-2 discharge (VP-162-B harness language flows from VP-162). (3) Dim-3 mandate anchor — no new MUSTs; CLEAR. BC-INDEX updated same burst: pin bumped v1.6→v1.7. |
| 1.6 | beta3-f12-regate9-spec | 2026-09-16 | product-owner | **F-1 (MED — POL-20 violation) + F-2 (MED — VP-scope overstatement).** **F-1:** `introduced: beta3-remediation-f2` violated POL-20 (`bc_introduced_field_canonical_format`); the opaque burst-ID notation is prohibited; corrected to `introduced: 2026-09-16`. `origin: greenfield` retained — json_extract_string is a new feature (S-JSON-EXTRACT-UDF-001), not retroactively extracted from existing code; `brownfield` is reserved for pre-existing behavioral extraction; `greenfield` is correct. **F-2:** §Verification Properties VP-162 row's Property column described the full behavioral routing ("null/missing/parse-failure/non-object/JSON-null → None; string → Some(s); non-string → Some(value.to_string())") as though Kani-proven. VP-162 (authoritative) and ADR-066 §C prove only: (1) null-safety and panic-freedom; (2) None-in→None-out; (3) structural key-length bound ≤ 256 bytes. Behavioral routing is covered by RG-JEX Red Gate tests (ADR-066 §G), NOT by VP-162. **Fix:** VP-162 row rewritten with explicit "Kani-proven:" and "NOT Kani-proven — covered by RG-JEX Red Gate tests (ADR-066 §G):" sections; a reader can no longer misread the full routing as Kani-proven. **TD-VSDD-097 three-dimension discharge:** (1) Dim-1 sibling — no named split-event twin; CLEAR. (2) Dim-2 downstream copy target — VP-162 §Property Statement is the upstream source for the Kani-proven scope; this BC is the downstream copy; this burst IS the Dim-2 discharge for F-2. (3) Dim-3 mandate anchor — no new MUSTs; CLEAR. BC-INDEX updated same burst: pin bumped v1.5→v1.6. |
| 1.5 | beta3-fc-regate5-spec | 2026-09-16 | product-owner | **F-C (LOW) — VP-162 section anchor corrected: `§Harnesses` → `§Kani Proof Harness`.** Re-gate pass-5 found that the §Verification Properties VP-162 row cross-reference cited `VP-162 §Harnesses`, but VP-162's actual section heading is `## Kani Proof Harness` (no `§Harnesses` heading exists). The architect had already made the identical correction in ADR-066 §D1. **Fix:** `(ADR-066 §D1; VP-162 §Harnesses)` → `(ADR-066 §D1; VP-162 §Kani Proof Harness)`. Full grep of non-changelog body confirms no other `§Harnesses` references to VP-162 remain. The v1.4 changelog row's mention of `VP-162 §Harnesses` is inside a changelog entry (exempt per POL-39) and not updated. **TD-VSDD-097 three-dimension discharge:** (1) Dim-1 sibling — no named split-event twin; CLEAR. (2) Dim-2 downstream copy target — this is the downstream copy of ADR-066 §D1; ADR-066 already corrected its own cite; this burst closes the copy-target leg. (3) Dim-3 mandate anchor — no new MUSTs; CLEAR. BC-INDEX updated same burst: pin bumped v1.4→v1.5. |
| 1.4 | beta3-f1-regate4-spec | 2026-09-16 | product-owner | **F1 (MED — downstream copy miss) — VP-162 Kani mechanism corrected: `kani::assume` → `any_vec` structural bound.** Re-gate pass-4 adversary found that §Verification Properties VP-162 row still read "formal bounds under `kani::assume(key.len() <= 256)` (ADR-066 §D1)". ADR-066 v1.4 + VP-162 removed the `kani::assume` mechanism; the proof now uses a structural `any_vec::<u8,256>()` key bound (per ADR-066 §D1 and VP-162 §Harnesses). **Fix:** VP-162 row updated to "formal bounds via structural `any_vec::<u8,256>()` key bound (ADR-066 §D1; VP-162 §Harnesses)". No other `kani::assume(key.len)` occurrences found in this file. **TD-VSDD-097 three-dimension discharge:** (1) Dim-1 sibling — no named split-event twin; CLEAR. (2) Dim-2 downstream copy target — VP-162 §Harnesses is the upstream source; this BC is the downstream copy — this burst IS the Dim-2 discharge. (3) Dim-3 mandate anchor — no new MUSTs; CLEAR. BC-INDEX updated same burst: pin bumped v1.3→v1.4. |
| 1.3 | beta3-f2-regate3-spec | 2026-09-16 | product-owner | **OBS — §Invariants DI-019 gloss corrected: memory budget is NFR-015, not DI-019.** The DI-019 invariant gloss read "subject to the same 30s timeout, 10K record cap, and **memory limits** as all query execution". Memory budget is governed by NFR-015, not DI-019. DI-019 label = "Query Security Limits" covers the 30s timeout and 10K record cap only. Fixed to: `DI-019 ("Query Security Limits"): subject to the same 30s timeout and 10K record cap; memory budget governed by NFR-015, not DI-019`. **TD-VSDD-097 three-dimension discharge:** (1) Dim-1 sibling — no named split-event twin; CLEAR. (2) Dim-2 downstream copy target — §Invariants DI-019 gloss is not verbatim-copied into any downstream artifact; CLEAR. (3) Dim-3 mandate anchor — no new MUSTs; CLEAR. BC-INDEX updated same burst: pin bumped v1.2→v1.3. |
| 1.2 | beta3-f2-batch2-spec | 2026-09-16 | product-owner | **RG-JEX-001..011 canonical transcription — scenario rotation fix + test_jex_ prefix + §Authority depin.** (1) **EC-025-002/004/005 rotation corrected:** BC sequence has Null-Column (EC-025-002), Non-Object (EC-025-004), JSON-null-at-key (EC-025-005); canonical RG table has RG-JEX-002=JSON-null-AT-KEY, RG-JEX-004=Arrow-NULL-COLUMN, RG-JEX-005=NON-OBJECT. Fixed by re-assigning MUST cites per scenario: EC-025-002 → RG-JEX-004 (`test_jex_rg004_null_column_input_returns_sql_null`); EC-025-004 → RG-JEX-005 (`test_jex_rg005_non_object_json_returns_sql_null`); EC-025-005 → RG-JEX-002 (`test_jex_rg002_json_null_value_at_key_returns_sql_null`). (2) **test_qtt_ → test_jex_ prefix rename** on all 11 EC MUSTs per canonical architect table. (3) **Test names added** to EC-025-001/003/006/007/008 (previously bare RG-JEX-NNN with no test name). (4) **§Authority depin (POL-39 / TD-VSDD-091):** `ADR-066 v1.0 (2026-09-16)` → `ADR-066 §G` (volatile version pin removed; section anchor substituted). (5) **error-taxonomy.md E-QUERY-045 Description depin:** `BC-2.11.025 v1.0` → `BC-2.11.025 §Error Cases` (same burst, v2.85). **TD-VSDD-097 three-dimension discharge:** (1) Dim-1 sibling — no named split-event twin; CLEAR. (2) Dim-2 downstream copy target — error-taxonomy.md E-QUERY-045 BC anchor depinned same burst; CLEAR. (3) Dim-3 mandate anchor — all 11 MUST test names anchored to S-JSON-EXTRACT-UDF-001 RG-JEX-001..011; no unanchored MUSTs. BC-INDEX updated same burst: pin bumped v1.1→v1.2. |
| 1.1 | beta3-f2-batch1-spec | 2026-09-16 | product-owner | **F7+F9 — RG-JEX canonicalization + E-QUERY-045(b) message standardization.** **(F7) EC-11-025-009 updated:** added explicit MUST anchor for RG-JEX-010 (dot-in-key treated as literal top-level key, NOT JSONPath; test `test_qtt_rg010_dot_in_key_treated_as_literal_top_level_key`). **(F7) EC-11-025-010 ADDED:** parse failure → SQL NULL (RG-JEX-009; column value is non-JSON; same null-propagating semantics as EC-11-025-004 non-object input; does NOT trigger E-QUERY-045 at runtime; test `test_qtt_rg009_parse_failure_returns_null`). **(F7) EC-11-025-011 ADDED:** pipe-mode end-to-end reachability (SAP-3 obligation per CLAUDE.md §SAP-3; RG-JEX-011; wire-level assertion on serialized JSON output; test `test_qtt_rg011_pipe_mode_json_extract_string_e2e`). **(F9) §Error Cases E-QUERY-045(b) message format standardized:** "the 256-byte maximum" → "the `{max_len}`-byte maximum" — `PrismError::JsonExtractKeyTooLong { key_len, max_len }` variant carries `max_len` as a field; the message MUST use `{max_len}` not a hardcoded literal (consistent with error-taxonomy.md v2.84 updated in same burst). **High-water EC update:** EC-11-025-009 → EC-11-025-011 (2 new ECs). **TD-VSDD-097 three-dimension discharge:** (1) Dim-1 sibling — no named split-event twin for BC-2.11.025; CLEAR. (2) Dim-2 downstream copy target — error-taxonomy.md E-QUERY-045(b) is the co-modified downstream copy-source for the message format; updated in same burst to v2.84 with `{max_len}`; CLEAR. (3) Dim-3 mandate anchor — all 11 MUSTs in EC-11-025-001..011 anchored to S-JSON-EXTRACT-UDF-001 RG-JEX-001..011; no unanchored MUSTs. BC-INDEX updated same burst: pin bumped v1.0→v1.1. |
| 1.0 | beta3-f2-batch0-spec | 2026-09-16 | product-owner | Initial contract for `json_extract_string` DataFusion ScalarUDF. Sourced from ADR-066 v1.0 §B1/§B3/§D1..D6/§E/§F/§G. Nine MUSTs anchored to S-JSON-EXTRACT-UDF-001 RG-JEX-001..008 + VP-162 per ADR-066 §G mandate table. E-QUERY-045 reserved in error-taxonomy.md same burst (two sub-cases: non-literal key; key >256 bytes). EC-11-025-001..009 allocated; high-water mark EC-11-025-009. TD-VSDD-097: (1) Sibling pair — no named split-event twin for BC-2.11.025; CLEAR. (2) Downstream copy target — ADR-066 §G is the upstream mandate source; this BC IS the VSDD downstream copy target; written from scratch with no stale copy propagation needed; CLEAR. (3) Mandate anchor — all 9 MUSTs anchored to S-JSON-EXTRACT-UDF-001 RG-JEX-001..008 and VP-162; no unanchored MUSTs. BC-INDEX updated same burst: new draft row added (draft_contracts 3→4, total_contracts 277→278). error-taxonomy.md updated same burst: E-QUERY-045 row added (v2.82→v2.83). |
