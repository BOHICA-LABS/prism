# PR Review — Cycle 6 — S-JSON-EXTRACT-UDF-001

**PR:** https://github.com/BOHICA-LABS/prism/pull/296
**Reviewed SHA:** `1cf3638fa57453e4e0773f2788095cdf4c0645eb`
**Reviewer:** pr-reviewer (fresh-eyes, cycle 6)
**Scope:** demo-evidence correctness (cycle-5 B-C5-1 / B-C5-2 / B-C5-3 class-sweep verification) + SAP-1 / SAP-3 probes.
**Posted to GitHub:** https://github.com/BOHICA-LABS/prism/pull/296#issuecomment-5725535747

```
verdict: REQUEST_CHANGES
covered_sha: 1cf3638fa57453e4e0773f2788095cdf4c0645eb
```

## Verdict: REQUEST_CHANGES

The class sweep in `1cf3638fa` fixed `evidence-report.md` but did **not** propagate to the `AC-*.json` sidecars. `B-C5-2` is genuinely closed. `B-C5-1` and `B-C5-3` recur in files the sweep did not touch. Code is clean; all blocking findings are in demo-evidence.

### Posting-mechanism note

`gh pr review --request-changes` and `--approve` both fail on this repository: the PR author and the reviewing account are the same GitHub identity. Verified empirically this cycle, not assumed:

- `gh pr view 296 --json author` → `drbothen`
- `gh api user --jq .login` → `drbothen`
- `gh pr review 296 --request-changes --body-file <this file>` → exit 1 with:
  ```
  failed to create review: GraphQL: Review Can not request changes on your own pull request (addPullRequestReview)
  ```

The `addPullRequestReview` GraphQL mutation rejects the `APPROVE` and `REQUEST_CHANGES` review events outright when author == reviewer.

However, the restriction does **not** extend to the `COMMENT` review event. This review was therefore submitted as a formal review via:

```
gh pr review 296 --comment --body-file <this file>     # exit 0
```

confirmed present in the Reviews timeline:

```
{"author":"drbothen","state":"COMMENTED","submittedAt":"2026-09-18T05:23:47Z"}
```

**Residual gap (unfixable at this account configuration):** the review's machine-readable `state` is `COMMENTED`, not `CHANGES_REQUESTED`, so the verdict does not register as a blocking review in GitHub's merge-gate UI. The REQUEST_CHANGES verdict is authoritative in the body text and in the `verdict:` block above, and must be enforced by the pipeline (pr-manager/orchestrator), **not** inferred from GitHub's review state. Any automation keying off `state == "CHANGES_REQUESTED"` will misread this PR as unblocked.

A verdict comment with identical content was also posted at `#issuecomment-5725535747` before the `--comment` review path was identified; it is redundant with this review and can be ignored.

---

## Cycle-5 finding status

| Cycle-5 finding | Status | Note |
|---|---|---|
| B-C5-1 nextest format / counts | **FIXED** | Format + counts verified exact |
| B-C5-1 "evidence at wrong commit before code fixes" | **RECURS** → B-C6-1 | AC-006/007/012 still pre-fix |
| B-C5-2 wrong counts / phantom row / header mismatch | **FIXED** | 44-row table verified row-by-row against `cargo nextest list` |
| B-C5-3 fabricated assertions | **PARTIAL** → B-C6-2, B-C6-3 | AC-002/003/009 fixed; AC-008 + AC-011 still wrong |

---

## BLOCKING findings

### B-C6-1 — AC-006 / AC-007 / AC-012 wire captures are pre-fix and contradict a passing test at HEAD

| Field | Value |
|---|---|
| Severity | blocking |
| Category | coherence / demo-evidence |
| Files | `docs/demo-evidence/S-JSON-EXTRACT-UDF-001/AC-006-non-literal-key-rejection.json`, `AC-007-key-length-boundary.json`, `AC-012-where-predicate-gate.json` |

`1cf3638fa`'s commit subject states the evidence was re-captured at HEAD `1487d2d9b`. Three of the five JSON sidecars were not re-captured — they still carry `"commit": "7deb3674f"` and show `content[0].text` output that the current code **explicitly forbids**:

- AC-006 — `"...Dynamic key expressions are not supported.. Provide a string literal..."`
- AC-012 — same doubled period
- AC-007 — `"...exceeds the 256-byte maximum (CWE-400).. Reduce the json_extract_string key..."`

Commit `1487d2d9b` added, in `crates/prism-mcp/src/error_mapping.rs`, precisely to remove that doubled period:

```rust
let msg_trimmed = fields.message.trim_end_matches('.');
```

and `test_S_JSON_EXTRACT_UDF_001_e_query_045a_sid2_no_example_duplication_in_content_text` asserts:

```rust
assert!(!content_text.contains(".."), "[SID-2/BLOCKING-2] VIOLATION: content_text contains '..' ...");
```

That test **passes at HEAD**. The shipped evidence therefore demonstrates behavior the test suite guarantees cannot occur — the exact "evidence recorded at wrong commit before code fixes" defect B-C5-1 raised.

**Suggestion:** re-capture AC-006, AC-007, AC-012 against a binary built from `1487d2d9b` or later, and update each `_demo_meta.commit`. The composed `text` must read `...not supported. Provide...` and `...(CWE-400). Reduce...` with a single period. This requires an actual rebuild + MCP stdio re-run, not a text edit.

### B-C6-2 — AC-008 coercion inputs/outputs fabricated in both files, and the two files contradict each other

| Field | Value |
|---|---|
| Severity | blocking |
| Category | coherence / demo-evidence |
| Files | `evidence-report.md` §AC-008, `AC-001-010-functional-unit-tests.json` §`AC-008_non_string_coercion` |

Actual test source (`crates/prism-query/tests/test_json_extract_udf.rs`):

| Test | Input JSON | Key | Asserted output |
|---|---|---|---|
| `test_jex_rg008_non_string_json_value_coerced_to_string` | `{"count":42}` | `count` | `"42"` |
| `test_jex_rg008_bool_true_coerced_to_string` | `{"flag":true}` | `flag` | `"true"` |
| `test_jex_rg008_array_coerced_to_string` | `{"items":[1,2]}` | `items` | `"[1,2]"` |
| `test_jex_rg008_object_coerced_to_string` | `{"meta":{"k":"v"}}` | `meta` | `{"k":"v"}` |

`evidence-report.md` §AC-008 claims:
- `'{"active":true}' + 'active'` — wrong input and key (actual `{"flag":true}` / `flag`)
- `'{"tags":["a","b"]}' + 'tags'` → `"[\"a\",\"b\"]"` — **wrong input, key, and output** (actual `{"items":[1,2]}` / `items` → `"[1,2]"`)
- `'{"nested":{"k":"v"}}' + 'nested'` — wrong input and key (actual `{"meta":{"k":"v"}}` / `meta`)

`AC-001-010-functional-unit-tests.json` claims:
- `"object_nested": "{\"nested\":\"value\"} -> \"{\\\"nested\\\":\\\"value\\\"}\""` — **wrong input and output** (actual asserts `{"k":"v"}`)

The array case also disagrees between the two files (`"[\"a\",\"b\"]"` vs `"[1,2]"`), so at most one can be right — and the report's is the wrong one.

**Suggestion:** replace the AC-008 block in both files with the four rows above, transcribed from the `assert_eq!` expectations.

### B-C6-3 — AC-011 JSON mislabels row 1 and row 2

| Field | Value |
|---|---|
| Severity | blocking |
| Category | coherence / demo-evidence |
| File | `AC-011-pipe-mode-udf-registration.json` → `unit_test_evidence.key_assertions` |

The JSON claims:

```
"extracted_col.is_null(1) == true (row 1: {} missing key → SQL NULL)",
"extracted_col.is_null(2) == true (row 2: null column → SQL NULL)"
```

`test_jex_rg011_pipe_mode_end_to_end_executes` actually asserts:
- row 1 — **JSON null at `'severity'`** → SQL NULL
- row 2 — **`'severity'` key absent** → SQL NULL

No row in the mock adapter has a null column at all. Both labels are wrong, and they attribute the wrong EC arms (claiming null-column EC-11-025-002 instead of EC-11-025-005 and EC-11-025-003).

`evidence-report.md` §AC-011 states this correctly — the sweep corrected the narrative and left the sidecar stale. Same split-fix pattern as B-C6-1.

**Suggestion:** change to `row 1: {"severity":null} JSON null → SQL NULL` and `row 2: {"host":"server03"} key absent → SQL NULL`.

### B-C6-4 — AC-001-010 JSON reports 12 tests; actual is 15

| Field | Value |
|---|---|
| Severity | blocking |
| Category | coverage / demo-evidence |
| File | `AC-001-010-functional-unit-tests.json` → `test_results_summary` |

```json
"test_results_summary": { "total_tests": 12, "passed": 12,
  "run_command": "cargo nextest run -p prism-query -E 'test(test_jex_rg00)' --no-fail-fast" }
```

Running that exact command at HEAD:

```
Summary [   0.065s] 15 tests run: 15 passed, 1787 skipped
```

The filter matches 15 tests (rg001–rg005, rg006, rg007 + `_b`/`_c`/`_d`, four rg008 variants, rg009). Recurrence of the B-C5-2 wrong-count class in the file the sweep edited most recently.

**Suggestion:** set `total_tests`/`passed` to 15, or narrow `run_command` to the 12 tests actually summarized here — the eight AC sections cover 12 test functions, so if that was the intent the `run_command` filter is what needs correcting.

---

## Suggestions (non-blocking)

| ID | Severity | Finding |
|---|---|---|
| S-C6-1 | suggestion | Stale provenance: `AC-001-010-functional-unit-tests.json` and `AC-011-pipe-mode-udf-registration.json` both still declare `"commit": "83fa7ff51"` (cycle-1) despite being edited in the cycle-6 sweep. Bump to the recording HEAD. |
| S-C6-2 | nit | `evidence-report.md` §AC-005 renders the call as `json_extract_string('["a","b","c"]', 'key')`; the test uses key `'severity'`. The JSON sidecar is correct. |
| S-C6-3 | suggestion | `evidence-report.md` §AC-009 renders the input as `'"not valid json at all !!"'` with inner double quotes. A double-quoted string is *valid* JSON, so as written it exercises the non-object arm (EC-11-025-004), not the parse-failure arm this AC claims. The test registers the unquoted `not valid json at all !!`. Drop the inner quotes — this misstates the very arm being demonstrated. |
| S-C6-4 | nit | `evidence-report.md` §AC-011 says envelope fields are asserted in "`prism-mcp` error-mapping tests (6 tests)", but the table directly below lists 7 and the run reports 7. |

---

## Verified clean

- `cargo nextest run -p prism-query -E 'test(test_jex)' --no-fail-fast` → `Summary [ 0.119s] 44 tests run: 44 passed, 1758 skipped`. Report claims 44/44 with 1758 skipped and a well-formed elapsed bracket. **Matches.**
- `cargo nextest run -p prism-mcp -E 'test(JSON_EXTRACT_UDF)' --no-fail-fast` → `7 tests run: 7 passed, 502 skipped`. **Exact match.** (The `test(json_extract)` filter returns 1 — it matches only the wire-null test; the report documents the `JSON_EXTRACT_UDF` filter and that claim is accurate.)
- 44-row table diffed programmatically against `cargo nextest list`: **0 phantom rows, 0 missing rows.** Cycle-5's phantom `test_jex_dml_ast_safe_skip_with_jex_variant_integration` is gone; the real `test_jex_dml_ast_safe_skip_with_jex_variant` is present. Header "44 total" matches 44 rows.
- AC-002 (`{"severity":null}` / `severity`), AC-003 (`{"other_field":"value"}` / `severity`), AC-004, AC-005 (JSON), AC-009 (JSON), AC-010 all match test source.
- AC-007 boundary keys verified by byte count: rejection = **257** bytes, acceptance = **256** bytes. Correct inclusive boundary.
- **SAP-1 PASS** — zero added `event_type` emissions in the diff; no BC-2.16.002 catalog row required.
- **SAP-3 PASS** — `test_jex_rg021`–`rg024` all construct `make_gate_engine()` and drive `engine.execute(...)`, the `QueryEngine::execute` public surface, not a synthetic AST.

---

## 8-item checklist

| # | Item | Result |
|---|---|---|
| 1 | Diff coherence | PASS — all changes scoped to the UDF, its gate, tests, and evidence |
| 2 | Description accuracy | **FAIL** — `1cf3638fa` claims re-capture at `1487d2d9b`; AC-006/007/012 are at `7deb3674f` (B-C6-1) |
| 3 | Test coverage | PASS — 44 prism-query + 7 prism-mcp + 1 wire-null test, all green |
| 4 | Demo evidence | **FAIL** — B-C6-1..B-C6-4. `.gif`/`.webm` present (not `.txt`); both success and error paths recorded |
| 5 | Commit quality | PASS — conventional format, story ID present |
| 6 | Diff size | ~5,117 insertions, dominated by a 2,266-line test file. Acceptable for a strict-TDD story |
| 7 | Missing changes | PASS — AC-001..AC-012 all have evidence and a named passing test |
| 8 | Dependency status | PASS — no unmerged upstream PRs identified |

---

## Root cause & routing

**Root cause across B-C6-1 / B-C6-2 / B-C6-3:** the cycle-5 sweep treated `evidence-report.md` as the artifact and the `AC-*.json` sidecars as derived from it. They are independent evidence files consumed on their own. A fix applied to the narrative but not the sidecar leaves the sidecar asserting pre-fix behavior — and in B-C6-1's case, behavior a currently-passing test forbids.

**Routing:** all four blocking findings are demo-evidence artifacts → `vsdd-factory:demo-recorder`. The fix pass must verify **each JSON file independently** against test source *and* against a binary built at the recording HEAD — not by diffing against the report. B-C6-1 requires a genuine re-capture, not a text edit.

Per the frozen-HEAD streak rule (DRIFT-ORCH-PRLEVEL-PUSH-001), pushing the fix resets the PR-LEVEL 3-CLEAN streak to 0/3.
