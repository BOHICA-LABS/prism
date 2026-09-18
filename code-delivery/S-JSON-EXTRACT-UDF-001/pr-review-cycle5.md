## PR-LEVEL Review — Cycle 5 (targeted re-verification pass)

**Story:** S-JSON-EXTRACT-UDF-001
**PR:** #296
**HEAD reviewed:** `1487d2d9b8fa44bb6023d7e5fcbe01b87702976c`
**Verdict:** **APPROVE** (0 BLOCKING / 0 SUGGESTION / 1 NIT)

Scope note: this pass re-verified the two cycle-4 BLOCKING fixes and the four standing-probe dimensions by direct source read. No test runner was invoked in this pass — CI remains authoritative for test-green. The pass asserted mechanism presence and test load-bearingness.

---

### Verification table

| # | Item | Result | Evidence |
|---|---|---|---|
| 1 | **BLOCKING-1 (P0 security)** — case-sensitive gate bypass | **CLOSED** | `crates/prism-query/src/filter_parser.rs`, `fn_call_comparison`: `let lowered = func_name.to_ascii_lowercase();` immediately precedes `match lowered.as_str() { "json_extract_string" => ScalarFunc::JsonExtractString, _ => ScalarFunc::Unknown(func_name) }`. Parity with `sql_parser.rs::known_scalar` restored; inline rationale correctly names DataFusion's case-insensitive execution-time resolution as the exploit path. |
| 2 | **BLOCKING-2** — double period in composed `content_text` | **CLOSED** | `crates/prism-mcp/src/error_mapping.rs` compositor: `let msg_trimmed = fields.message.trim_end_matches('.');` precedes `format!("ERROR: [{}] - {}. {}", fields.category, msg_trimmed, fields.suggestion)`. |
| 3 | **SID-2** — full-composed-output assertions | **SATISFIED** | Two tests assert the exact emitted string via `assert_eq!(content_text, expected_content_text, …)`: `test_S_JSON_EXTRACT_UDF_001_e_query_045a_sid2_no_example_duplication_in_content_text` and `test_S_JSON_EXTRACT_UDF_001_e_query_045b_sid2_full_composed_content_text`. Both carry the regression guard `assert!(!content_text.contains(".."), …)`. The (a) test additionally asserts the `json_extract_string(col, 'key_name')` example phrase occurs exactly once across the message+suggestion composition — a genuine no-duplicated-phrase check per SID-2 clause 2, not a component-only assert. |
| 4 | **SAP-3** — arm reachability from the public surface | **SATISFIED** | `crates/prism-query/tests/test_json_extract_udf.rs` contains `test_jex_rg021_uppercase_func_name_where_rejected`, `test_jex_rg022_mixed_case_func_name_key_too_long_where_rejected`, `test_jex_rg023_nested_call_inner_non_literal_rejected`, `test_jex_rg024_nested_call_inner_literal_accepted`. All four drive `engine.execute(<PQL string>, QueryOptions::default())` via `make_gate_engine()` — public surface, not a synthetic AST handed to the internal handler. Each body carries an explicit `// SAP-3:` reachability annotation and a RED-reason string. |
| 5 | **SAP-1** — structured event catalog completeness | **CLEAN** | Zero `event_type =` emissions in either changed file, so no BC-2.16.002 §Postconditions catalog row is owed. Vacuously clean. |
| 6 | **Demo evidence** | **PRESENT** | `docs/demo-evidence/S-JSON-EXTRACT-UDF-001/evidence-report.md` records `Summary 40 tests run: 40 passed, 1758 skipped`, matching the stated `total_tests: 40`. The 33→40 growth is attributed to the cycle-3/cycle-4 fix commits. Per-AC observed output is recorded with `isError` values, and accepted-query paths carry the E-SENSOR-010 no-live-sensor explanation. |

---

### Test load-bearingness assessment (TD-VSDD-059 paper-fix detection)

Neither BLOCKING fix is paper-fixed. Each has a test that fails in the fix's absence:

- **BLOCKING-1** → RG-JEX-021 (uppercase `JSON_EXTRACT_STRING` in WHERE with a column-reference key) and RG-JEX-022 (mixed-case `Json_Extract_String` in WHERE with a 257-byte key). Without the `to_ascii_lowercase()` call, both spellings fall to `ScalarFunc::Unknown`, `check_jex_in_expr` never matches, and the E-QUERY-045 gate is bypassed — each test's RED-reason string states exactly this.
- **BLOCKING-2** → the two `assert_eq!` composed-text tests. Without the `trim_end_matches('.')`, E-QUERY-045(a) and (b) both emit `..` between message and suggestion and both `assert_eq!` calls fail on the exact-string comparison.

RG-JEX-024 deserves specific credit: it is the **green counterpart** to RG-JEX-023, proving the new recursive `for arg in args { check_jex_in_expr(arg)?; }` arm does not over-reject a nested call whose keys are all literal. A fix that added recursion without this test could have silently broken valid nested queries.

### Sibling sweep (POL-29 / TD-VSDD-097 dimension 1) — discharged

- **Sibling pair:** `grep -n 'ERROR: \[{' crates/prism-mcp/src/error_mapping.rs` returns exactly one live compositor site (the one fixed); the other three hits are doc comments and root-cause commentary. A sweep of `crates/prism-mcp/src/` outside `error_mapping.rs` returns nothing. BLOCKING-2 has **no unswept twin**. For BLOCKING-1, the named sibling is `sql_parser.rs::known_scalar`, which already lowercased — the fix brings `filter_parser.rs` into parity with it rather than diverging.
- **Downstream copy target:** no section of either changed file is a verbatim copy-source for a downstream artifact in this diff.
- **Mandate anchor:** no new `MUST` was written into a BC or spec by this diff; RG-JEX-021..024 are anchored to BC-2.11.025 §Plan-time literal-key gate and ADR-066 §B3 in their doc comments.

---

### Findings

#### NIT-1 — `trim_end_matches` is a repeated-match trim

| Field | Value |
|-------|-------|
| Severity | **nit** |
| Category | coherence |
| File | `crates/prism-mcp/src/error_mapping.rs` (compositor, `msg_trimmed` binding) |
| Finding | `trim_end_matches('.')` strips *all* trailing periods, not just one. A message ending in an ellipsis (`"…not supported..."`) would be reduced to `"…not supported"` rather than to a single period. |
| Suggestion | `strip_suffix('.')` (with `unwrap_or(fields.message.as_str())`) would express the single-period intent precisely. **No action required for merge** — no message in the current E-QUERY / E-SENSOR taxonomy ends in an ellipsis, so the branch is unreachable today, and the resulting behavior is arguably the friendlier of the two. Recorded for the record in case this line is touched again. |

No BLOCKING and no SUGGESTION findings were minted in this pass.

---

### Checklist coverage (8-item)

| Item | Assessment |
|---|---|
| 1. Diff coherence | All changes trace to the two cycle-4 BLOCKING findings plus their covering tests and evidence refresh. No unrelated changes observed in the reviewed surface. |
| 2. Description accuracy | Commit subject (`fix(prism-query,prism-mcp): close case-sensitive gate bypass + nested-call coverage + composed-text assertion`) matches the actual changes. |
| 3. Test coverage | Both changed lines are covered by load-bearing tests (see load-bearingness section). |
| 4. Demo evidence | `evidence-report.md` present with per-AC observed output; test count consistent at 40. |
| 5. Commit quality | Conventional format with scope and story ID present. |
| 6. Diff size | Cycle-5 delta is small and targeted (two production lines plus tests/evidence). |
| 7. Missing changes | No gap found between the cycle-4 finding set and the delivered fixes. |
| 8. Dependency status | No upstream PR dependency surfaced in the reviewed surface. |

---

### Verdict

**APPROVE.** Both cycle-4 BLOCKING findings are closed at the mechanism level rather than paper-fixed, each backed by a test that fails without the fix. SAP-3 reachability is satisfied through the public `QueryEngine::execute` surface rather than synthetic AST, and includes a green counterpart test guarding against over-rejection by the new recursion arm. SAP-1 is vacuously clean. The sibling sweep found no unswept compositor twin. Demo evidence is internally consistent with the claimed test count. The single NIT is unreachable in the current error taxonomy and is not a merge condition.

CLEAN (strict): no — 1 NIT finding present
CLEAN (PR-merge): yes — 0 CRIT / 0 HIGH / 0 MED

covered_sha: 1487d2d9b8fa44bb6023d7e5fcbe01b87702976c
