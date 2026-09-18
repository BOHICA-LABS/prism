# Demo Evidence Report — S-JSON-EXTRACT-UDF-001

**Story:** S-JSON-EXTRACT-UDF-001 — Minimal `json_extract_string` ScalarUDF with Literal-Key Plan Gate  
**Branch:** feature/S-JSON-EXTRACT-UDF-001  
**Commit at recording:** 3c7650fca (cycle-4 evidence fix; cycle-2 re-record was at 7deb3674f; original cycle-1 was at 83fa7ff51)  
**Binary:** `target/debug/prism` (built from worktree via `cargo build -p prism-bin`)  
**Date:** 2026-09-17  
**AC coverage:** AC-001 through AC-012 (all 12 acceptance criteria)

### Cycle-2 re-recording notes (2026-09-17)

Three files were re-captured at HEAD `7deb3674f` to close pr-reviewer findings:

- **B-1 → `AC-006-non-literal-key-rejection.json` (re-captured):** Previous capture had stale suggestion text `"Use a literal string key: json_extract_string(col, 'key_name')."`. Current HEAD suggestion is `"Provide a string literal as the second argument, e.g., json_extract_string(raw_extensions, 'severity')."`. AC-006 re-captured at HEAD. Note: an incorrectly-labelled duplicate `AC-005-non-literal-key-error.json` was created during this fix burst (it had `ac: AC-005` / `rg: RG-JEX-005` metadata but captured AC-006 behavior); that file was deleted in the B-C3-3 fix — AC-005 ("Non-object JSON column returns SQL NULL") is covered by `AC-001-010-functional-unit-tests.json` §AC-005_non_object_json.
- **B-2 → `AC-007-key-length-boundary.json` and `AC-012-where-predicate-gate.json`:** Previous files were hand-authored with non-MCP fields (`note`, `content_summary`, `query_note`) and contained no `content[]` array or `_meta` key. Replaced with genuine JSON-RPC stdio captures from the running prism binary.
- **N-e:** Evidence report updated with correct test count (33, was 28), accurate wire-shape labeling (see AC-011 section), and current `captured_at` SHA.

---

## Recording Method

- **MCP wire transcripts** (JSON-RPC 2.0 over stdio) for plan-time gate behaviors — demonstrate the observable MCP tool surface per the story's product type (MCP stdio server).
- **Cargo nextest unit test output** for functional UDF behaviors (null safety, coercion, extraction correctness) — verifies the `json_extract_string_impl` pure function and DataFusion integration at the Arrow/RecordBatch level.
- **VHS terminal recording** (`AC-006-007-012-plan-gate.gif/.webm`) — visual evidence of the plan-gate rejection session.

**Config:** Minimal single-org prism config with `claroty.sensor.toml` (spec dir), dummy credentials (keyring store via `prism credential set`), `CLAROTY_INSTANCE_URL=https://demo.claroty.example.com`. All demonstrated plan-time gates fire before any sensor fan-out — no live Claroty instance required.

---

## AC Coverage Map

### AC-001 — UDF registered: happy path extracts top-level string value

**Evidence file:** `AC-001-010-functional-unit-tests.json` §AC-001_happy_path  
**Test:** `test_jex_rg001_udf_registered_happy_path_executes` — PASS  
**Observed behavior:** `json_extract_string('{"severity":"high","count":5}', 'severity')` → Arrow `StringArray` value `"high"` (non-null Utf8 cell).  
**Key assertion:** `extracted_col.value(0) == "high" && !extracted_col.is_null(0)`  
**Traces to:** BC-2.11.025 EC-11-025-001

---

### AC-002 — JSON null value at key returns SQL NULL

**Evidence file:** `AC-001-010-functional-unit-tests.json` §AC-002_json_null_at_key  
**Test:** `test_jex_rg002_json_null_value_at_key_returns_sql_null` — PASS  
**Observed behavior:** `json_extract_string('{"status":null}', 'status')` → Arrow null cell (SQL NULL). Not coerced to string `"null"` or empty string.  
**Key assertion:** `extracted_col.is_null(0) == true`  
**Traces to:** BC-2.11.025 EC-11-025-005

---

### AC-003 — Missing key returns SQL NULL

**Evidence file:** `AC-001-010-functional-unit-tests.json` §AC-003_missing_key  
**Test:** `test_jex_rg003_missing_key_returns_sql_null` — PASS  
**Observed behavior:** `json_extract_string('{"severity":"high"}', 'missing_key')` → SQL NULL. No warning emitted.  
**Key assertion:** `extracted_col.is_null(0) == true`  
**Traces to:** BC-2.11.025 EC-11-025-003

---

### AC-004 — Null column (Arrow null) returns SQL NULL

**Evidence file:** `AC-001-010-functional-unit-tests.json` §AC-004_null_column_input  
**Test:** `test_jex_rg004_null_column_input_returns_sql_null` — PASS  
**Observed behavior:** Arrow null cell as column input → SQL NULL output. Null-propagating.  
**Key assertion:** `extracted_col.is_null(0) == true`  
**Kani proof:** VP-162-B `vp162_b_none_input_is_none_output` (authored per T-06; Phase 5 formal-verify target)  
**Traces to:** BC-2.11.025 EC-11-025-002

---

### AC-005 — Non-object JSON column returns SQL NULL

**Evidence file:** `AC-001-010-functional-unit-tests.json` §AC-005_non_object_json  
**Test:** `test_jex_rg005_non_object_json_returns_sql_null` — PASS  
**Observed behavior:** `json_extract_string('["a","b","c"]', 'key')` → SQL NULL. JSON array is not an object; extraction returns NULL.  
**Key assertion:** `extracted_col.is_null(0) == true`  
**Traces to:** BC-2.11.025 EC-11-025-004

---

### AC-006 — Non-literal key argument rejected at plan time with E-QUERY-045(a)

**Evidence file:** `AC-006-non-literal-key-rejection.json`  
**VHS recording:** `AC-006-007-012-plan-gate.gif` / `.webm`  
**Query:** `SELECT json_extract_string(raw_extensions, severity_id) FROM claroty_alerts`  
**Observed MCP wire response:**
```json
{
  "result": {
    "isError": true,
    "structuredContent": {
      "error": {
        "code": "E-QUERY-045",
        "category": "validation",
        "message": "E-QUERY-045: json_extract_string requires a literal string key (e.g., json_extract_string(col, 'key_name')). Dynamic key expressions are not supported.",
        "retryable": false
      }
    }
  }
}
```
**Assertions:** `isError: true`, `code: E-QUERY-045`, fires before sensor fan-out (no E-SENSOR-010 in response), verbatim message from ADR-066 §F.  
**SAP-3 reachability:** Test `test_jex_rg006_non_literal_key_rejected_e_query_045_a` — PASS (exercises `QueryEngine::execute` public surface).  
**Traces to:** BC-2.11.025 EC-11-025-006

---

### AC-007 — Key exceeding 256 UTF-8 bytes rejected at plan time with E-QUERY-045(b)

**Evidence file:** `AC-007-key-length-boundary.json`  
**VHS recording:** `AC-006-007-012-plan-gate.gif` / `.webm`

**257-byte key (rejected):**  
Query: `SELECT json_extract_string(raw_extensions, '<257 a-chars>') FROM claroty_alerts`  
**Observed MCP wire response:**
```json
{
  "result": {
    "isError": true,
    "structuredContent": {
      "error": {
        "code": "E-QUERY-045",
        "message": "E-QUERY-045: json_extract_string key is 257 bytes, which exceeds the 256-byte maximum (CWE-400).",
        "retryable": false
      }
    }
  }
}
```

**256-byte key (accepted — boundary inclusive):**  
Query: `SELECT json_extract_string(raw_extensions, '<256 a-chars>') FROM claroty_alerts`  
**Observed:** `isError: false` — query accepted, passed plan gate, proceeded to sensor fan-out (E-SENSOR-010 due to no live sensor).

**Assertions:** 257-byte key: `isError: true`, message contains "257 bytes" and "256-byte maximum (CWE-400)"; 256-byte key: `isError: false` (no E-QUERY-045).  
**Traces to:** BC-2.11.025 EC-11-025-007

---

### AC-008 — Non-string JSON value coerced to string representation via to_string()

**Evidence file:** `AC-001-010-functional-unit-tests.json` §AC-008_non_string_coercion  
**Tests (4 variants):** `test_jex_rg008_non_string_json_value_coerced_to_string`, `test_jex_rg008_bool_true_coerced_to_string`, `test_jex_rg008_array_coerced_to_string`, `test_jex_rg008_object_coerced_to_string` — all PASS  
**Observed behaviors:**
- `'{"count":42}' + 'count'` → `"42"` (NOT SQL NULL)
- `'{"active":true}' + 'active'` → `"true"`
- `'{"tags":["a","b"]}' + 'tags'` → `"[\"a\",\"b\"]"`
- `'{"nested":{"k":"v"}}' + 'nested'` → `"{\"k\":\"v\"}"`  
**Traces to:** BC-2.11.025 EC-11-025-008

---

### AC-009 — Parse failure (invalid JSON column value) returns SQL NULL

**Evidence file:** `AC-001-010-functional-unit-tests.json` §AC-009_parse_failure  
**Test:** `test_jex_rg009_parse_failure_non_json_input_returns_sql_null` — PASS  
**Observed behavior:** `json_extract_string('{not_json}', 'key')` → SQL NULL silently. No E-QUERY-045 (that gate is plan-time only). Pure function: no per-row tracing emission per ADR-066 §D1.  
**Key assertion:** `extracted_col.is_null(0) == true`  
**Traces to:** BC-2.11.025 EC-11-025-010

---

### AC-010 — Dot-in-key literal treated as top-level key, NOT nested JSONPath

**Evidence file:** `AC-001-010-functional-unit-tests.json` §AC-010_dot_in_key  
**Test:** `test_jex_rg010_dot_in_key_literal_not_nested_path` — PASS  
**Observed behavior:** `json_extract_string('{"a.b":"dotted","a":{"b":"nested"}}', 'a.b')` → `"dotted"` (exact top-level key `"a.b"`, NOT nested path `a → b`).  
**Key assertion:** `extracted_col.value(0) == "dotted"`  
**Traces to:** BC-2.11.025 EC-11-025-009

---

### AC-011 — Pipe-mode end-to-end: json_extract_string executes via MCP query tool

**Evidence file:** `AC-011-pipe-mode-udf-registration.json`  
**VHS recording:** `AC-006-007-012-plan-gate.gif` / `.webm`

**MCP wire demo:**  
Query: `SELECT json_extract_string(raw_extensions, 'severity') AS jex_severity FROM claroty_alerts | limit 5`  
**Observed:** `isError: false` — UDF registered and resolved, query passed plan gate and executed. No "unknown function" error. Sensor returned E-SENSOR-010 (dummy credential, no live sensor) — confirms the failure point is the sensor, NOT the UDF.

**Unit test (RG-JEX-011) Arrow-level assertions (SID-2):**  
- `test_jex_rg011_pipe_mode_end_to_end_executes` — PASS
- Exercises `QueryEngine::execute()` with `JexMockAdapter` (SAP-3 public surface)
- Arrow-level assertions (pre-serialization Rust structs): `extracted_col.value(0) == "critical"`, `extracted_col.is_null(1) == true` (JSON null → SQL NULL), `extracted_col.is_null(2) == true` (absent key → SQL NULL)
- Note: these are Arrow RecordBatch / StringArray assertions — pre-serialization. Serialized wire-level null-not-absent coverage (BC-2.11.001 EC-11-079, asserting on the serialized JSON envelope with `explicit_nulls`) is in `prism-mcp` test `test_BC_2_11_025_json_extract_string_null_row_wire_null_not_absent` (`crates/prism-mcp/tests/bc_2_11_025_jex_wire_null_test.rs`). The `prism-query` crate intentionally has no `arrow-json` or `prism-mcp` dependency (ADR-066 §B2 §Forbidden Dependencies); MCP envelope fields (`structuredContent.error`, `content[].text` bytes) are asserted only in `prism-mcp` error-mapping tests (6 tests; see prism-mcp Full Test Suite section below). The plan-gate unit tests (RG-JEX-006, RG-JEX-007, RG-JEX-012, RG-JEX-013) assert on `PrismError` variants and their `Display` format strings — not on MCP JSON envelopes.

**Traces to:** BC-2.11.025 EC-11-025-011

---

### AC-012 — Predicate-position gate: WHERE/HAVING/pipe-where with non-literal or oversized key rejected; valid literal key executes correctly

**Evidence file:** `AC-012-where-predicate-gate.json`  
**VHS recording:** `AC-006-007-012-plan-gate.gif` / `.webm`

**WHERE non-literal key (rejected):**  
Query: `SELECT id FROM claroty_alerts WHERE json_extract_string(raw_extensions, severity_id) = 'x'`  
**Observed MCP wire response:**
```json
{
  "result": {
    "isError": true,
    "structuredContent": {
      "error": {
        "code": "E-QUERY-045",
        "message": "E-QUERY-045: json_extract_string requires a literal string key (e.g., json_extract_string(col, 'key_name')). Dynamic key expressions are not supported.",
        "retryable": false
      }
    }
  }
}
```

**WHERE valid literal key (accepted):**  
Query: `SELECT id FROM claroty_alerts WHERE json_extract_string(raw_extensions, 'severity') = 'high'`  
**Observed:** `isError: false` — query accepted, passed plan gate.

**Predicate-position parity:** T-09a fix — `filter_parser.rs` `fn_call_comparison` maps `"json_extract_string"` → `ScalarFunc::JsonExtractString` (was `ScalarFunc::Unknown` prior to fix, bypassing the gate).  
**Unit tests:** `test_jex_rg012_where_predicate_non_literal_key_rejected_e_query_045` (PASS), `test_jex_rg012_b_where_key_too_long_rejected_e_query_045_b` (PASS), `test_jex_rg012_c_having_predicate_non_literal_key_rejected_e_query_045` (PASS), `test_jex_rg012_d_where_valid_literal_key_executes_correctly` (PASS)  
**Traces to:** BC-2.11.025 EC-11-025-012

---

## Full Test Suite Results

### prism-query (captured at HEAD 3c7650fca)

```
cargo nextest run -p prism-query -E 'test(test_jex)' --no-fail-fast
    Starting 40 tests across 22 binaries (1758 tests skipped)
     Summary 40 tests run: 40 passed, 1758 skipped
```

(Cycle-4 note: test count grew from 33 to 40 as cycle-3/cycle-4 fix commits added
`test_jex_ast_parser_maps_jex_to_scalar_func_json_extract_string` and
gate-walk tests for ORDER BY (RG-015), JOIN ON (RG-016), IN subquery predicate
(RG-017), IN subquery expr (RG-018), AST pipe-where (RG-019), and AST filter (RG-020).)

40 total test_jex tests all PASS:

| Test | Result |
|------|--------|
| test_jex_ast_parser_maps_jex_to_scalar_func_json_extract_string | PASS |
| test_jex_rg001_udf_registered_happy_path_executes | PASS |
| test_jex_rg002_json_null_value_at_key_returns_sql_null | PASS |
| test_jex_rg003_missing_key_returns_sql_null | PASS |
| test_jex_rg004_null_column_input_returns_sql_null | PASS |
| test_jex_rg005_non_object_json_returns_sql_null | PASS |
| test_jex_rg006_non_literal_key_rejected_e_query_045_a | PASS |
| test_jex_rg007_key_exceeds_max_len_rejected_e_query_045_b | PASS |
| test_jex_rg007_b_key_at_exactly_max_len_accepted | PASS |
| test_jex_rg007_c_multibyte_key_at_boundary_accepted | PASS |
| test_jex_rg007_d_multibyte_key_over_boundary_rejected | PASS |
| test_jex_rg008_non_string_json_value_coerced_to_string | PASS |
| test_jex_rg008_bool_true_coerced_to_string | PASS |
| test_jex_rg008_array_coerced_to_string | PASS |
| test_jex_rg008_object_coerced_to_string | PASS |
| test_jex_rg009_parse_failure_non_json_input_returns_sql_null | PASS |
| test_jex_rg010_dot_in_key_literal_not_nested_path | PASS |
| test_jex_rg011_pipe_mode_end_to_end_executes | PASS |
| test_jex_rg012_where_predicate_non_literal_key_rejected_e_query_045 | PASS |
| test_jex_rg012_b_where_key_too_long_rejected_e_query_045_b | PASS |
| test_jex_rg012_c_having_predicate_non_literal_key_rejected_e_query_045 | PASS |
| test_jex_rg012_d_where_valid_literal_key_executes_correctly | PASS |
| test_jex_rg013_pipe_where_non_literal_key_rejected_e_query_045_a | PASS |
| test_jex_rg013_b_pipe_where_key_too_long_rejected_e_query_045_b | PASS |
| test_jex_rg014_group_by_non_literal_key_rejected_e_query_045_a | PASS |
| test_jex_rg015_order_by_non_literal_key_rejected_e_query_045_a | PASS |
| test_jex_rg016_join_on_non_literal_key_rejected_e_query_045_a | PASS |
| test_jex_rg017_in_subquery_predicate_non_literal_key_rejected_e_query_045_a | PASS |
| test_jex_rg018_in_subquery_expr_non_literal_key_rejected_e_query_045_a | PASS |
| test_jex_rg019_ast_pipe_where_non_literal_key_rejected_e_query_045_a | PASS |
| test_jex_rg020_ast_filter_non_literal_key_rejected_e_query_045_a | PASS |
| test_jex_f4_coerce_arm_engine_execute | PASS |
| test_jex_f4_dot_in_key_arm_engine_execute | PASS |
| test_jex_f4_non_object_arm_engine_execute | PASS |
| test_jex_f4_parse_fail_arm_engine_execute | PASS |
| test_jex_dml_ast_safe_skip_with_jex_variant_integration | PASS |
| jex_gate_walk_completeness_tests::test_jex_dml_ast_returns_ok_no_scope | PASS |
| jex_gate_walk_completeness_tests::test_jex_dml_ast_safe_skip_with_jex_variant | PASS |
| jex_gate_walk_completeness_tests::test_jex_timestamp_arithmetic_base_key_too_long_defense_in_depth | PASS |
| jex_gate_walk_completeness_tests::test_jex_timestamp_arithmetic_base_rejected_defense_in_depth | PASS |
| jex_gate_walk_completeness_tests::test_jex_window_variant_is_fieldless_compile_guard | PASS |

### prism-mcp (captured at HEAD 3c7650fca)

MCP error-mapping tests verify the wire-level JSON serialization of E-QUERY-045 errors
(`error.code`, `error.category`, `content[0].text` bytes). These live in `prism-mcp`
because `prism-query` has no `arrow-json` or `prism-mcp` dependency (ADR-066 §B2).

```
cargo nextest run -p prism-mcp -E 'test(JSON_EXTRACT)' --no-fail-fast
    Starting 6 tests across 28 binaries (502 tests skipped)
     Summary 6 tests run: 6 passed, 502 skipped
```

| Test | Asserts | Result |
|------|---------|--------|
| error_mapping::tests::test_S_JSON_EXTRACT_UDF_001_e_query_045a_map_prism_error_non_literal_key | `PrismError::JsonExtractNonLiteralKey` → MCP error code E-QUERY-045 | PASS |
| error_mapping::tests::test_S_JSON_EXTRACT_UDF_001_e_query_045b_map_prism_error_key_too_long | `PrismError::JsonExtractKeyTooLong` → MCP error code E-QUERY-045 | PASS |
| error_mapping::tests::test_S_JSON_EXTRACT_UDF_001_e_query_045b_structured_path_validation_category | `error.category == "validation"` for E-QUERY-045(b) | PASS |
| error_mapping::tests::test_S_JSON_EXTRACT_UDF_001_e_query_045a_structured_path_validation_category | `error.category == "validation"` for E-QUERY-045(a) | PASS |
| error_mapping::tests::test_S_JSON_EXTRACT_UDF_001_e_query_045a_wire_level_serialized_json | Serialized JSON bytes: `error.code`, `error.message` exact match | PASS |
| error_mapping::tests::test_S_JSON_EXTRACT_UDF_001_e_query_045a_sid2_no_example_duplication_in_content_text | No duplicate phrase in composed `content[0].text` (SID-2) | PASS |

---

## Artifact Index

| File | Type | ACs Covered | Cycle |
|------|------|-------------|-------|
| `AC-006-007-012-plan-gate.gif` | VHS GIF recording | AC-006, AC-007, AC-011, AC-012 | cycle-1 |
| `AC-006-007-012-plan-gate.webm` | VHS WEBM recording | AC-006, AC-007, AC-011, AC-012 | cycle-1 |
| `AC-006-007-012-plan-gate.tape` | VHS tape source | AC-006, AC-007, AC-011, AC-012 | cycle-1 |
| `AC-006-non-literal-key-rejection.json` | MCP wire transcript (cycle-2, HEAD 7deb3674f) | AC-006 | cycle-2 |
| `AC-007-key-length-boundary.json` | MCP wire transcript (cycle-2, HEAD 7deb3674f) | AC-007 | cycle-2 |
| `AC-011-pipe-mode-udf-registration.json` | MCP wire transcript + unit test summary | AC-011 | cycle-1 |
| `AC-012-where-predicate-gate.json` | MCP wire transcript (cycle-2, HEAD 7deb3674f) | AC-012 | cycle-2 |
| `AC-001-010-functional-unit-tests.json` | Unit test assertion summary | AC-001..005, AC-008..010 | cycle-1 |
| `prism_plan_gate_demo.py` | Demo driver script (VHS source) | AC-006, AC-007, AC-011, AC-012 | cycle-1 |
