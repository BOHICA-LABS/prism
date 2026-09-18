# PR Review — Cycle 7 — S-JSON-EXTRACT-UDF-001

**PR:** https://github.com/BOHICA-LABS/prism/pull/296
**Reviewed SHA:** `e045ca087ad6f329c6cb44b0c81cc6450ff038ff`
**Reviewer:** pr-reviewer (fresh-eyes, cycle 7)
**Scope:** cycle-6 finding verification (B-C6-1 / B-C6-2 / B-C6-3 / B-C6-4) + full `AC-*.json` class sweep (TD-VSDD-097 dim-1 + dim-2) + SAP-1 / SAP-3 probes.
**Posted to GitHub:** https://github.com/BOHICA-LABS/prism/pull/296#issuecomment-5725659587

```
verdict: REQUEST_CHANGES
covered_sha: e045ca087ad6f329c6cb44b0c81cc6450ff038ff
```

**Posting mechanism note:** verdict delivered via `gh pr comment`. `gh pr review --approve` / `--request-changes` is blocked by GitHub's self-review restriction on this repo (author == reviewer identity), per the orchestrator's standing instruction for this PR. The comment carries the full machine-readable `verdict:` / `covered_sha:` block so downstream triage is unaffected.

---

## Summary

All four cycle-6 findings are **genuinely fixed and independently verified against source** — not paper-fixed. Both standing probes pass. However, the accompanying class sweep required by task 5 swept the `commit` provenance pin in only **3 of the 5** `AC-*.json` evidence files. Two residuals remain, and one of them carries a **provably false** claim that is strictly worse than the pre-fix state. Cannot approve.

| Severity | Category | ID | Finding |
|---|---|---|---|
| blocking | coverage / demo | B-C7-1 | `AC-001-010-functional-unit-tests.json` asserts `total_tests: 15` at pinned commit `83fa7ff51`, where only 12 such tests existed |
| blocking | coherence / demo | B-C7-2 | `AC-011-pipe-mode-udf-registration.json` retains stale pin `83fa7ff51` despite being content-edited in cycle 6 |

No code findings. Implementation has been clean since cycle 5; the entire remaining delta is demo-evidence provenance.

---

## Verified FIXED — cycle-6 findings

### B-C6-1 — AC-006 / AC-007 / AC-012 re-captured, double-period eliminated. PASS

All three files now pin `commit: "1487d2d9b"`, and all three `content[0].text` values carry exactly one period at the message/suggestion seam.

Verified against the authoritative compositor in `crates/prism-mcp/src/error_mapping.rs`:

```rust
let msg_trimmed = fields.message.trim_end_matches('.');
let content_text = format!(
    "ERROR: [{}] - {}. {}",
    fields.category, msg_trimmed, fields.suggestion
);
```

Hand-composing E-QUERY-045(a): category `validation`, message trimmed to `…Dynamic key expressions are not supported`, then the literal `". "`, then the suggestion — reproduces the AC-006 and AC-012 strings byte-for-byte. Same for E-QUERY-045(b) in AC-007 (`…maximum (CWE-400). Reduce the …`). Directory-wide grep for the `.. ` artifact across all JSON returns clean.

### B-C6-2 — AC-008 coercions. PASS

Each of the four entries now matches the actual test fixture in `crates/prism-query/tests/test_json_extract_udf.rs`:

| Evidence entry | Test | Input JSON / key | Asserted value |
|---|---|---|---|
| `integer_42` → `"42"` | `test_jex_rg008_non_string_json_value_coerced_to_string` | `{"count":42}` / `count` | `"42"` |
| `bool_true` → `"true"` | `test_jex_rg008_bool_true_coerced_to_string` | `{"flag":true}` / `flag` | `"true"` |
| `array_1_2` → `"[1,2]"` | `test_jex_rg008_array_coerced_to_string` | `{"items":[1,2]}` / `items` | `"[1,2]"` |
| `object_nested` → `{"k":"v"}` | `test_jex_rg008_object_coerced_to_string` | `{"meta":{"k":"v"}}` / `meta` | `{"k":"v"}` |

The previously fabricated `{"nested":"value"}` and `{"tags":["a","b"]}` shapes are gone from both the JSON and `evidence-report.md` (report bullets reconciled independently).

### B-C6-3 — AC-011 row labels. PASS

Row 1 is now labelled JSON-null-at-key and row 2 key-absent, matching the `JexMockAdapter` fixture vector exactly:

- row 0 — `{"severity":"critical","host":"server01"}` → `"critical"`
- row 1 — `{"severity":null,"host":"server02"}` → SQL NULL (JSON null at key)
- row 2 — `{"host":"server03"}` → SQL NULL (key absent)

The prior labels had rows 1 and 2 mislabelled (`{}` missing key / `null` column) — i.e. transposed relative to the fixture.

### B-C6-4 — `total_tests`. PASS on the number

Executed at HEAD:

```
cargo nextest run -p prism-query -E 'test(test_jex_rg00)' --no-fail-fast
Summary [0.051s] 15 tests run: 15 passed, 1787 skipped
```

15 confirmed; evidence reads `"total_tests": 15, "passed": 15`. The *number* is correct at HEAD. The *pin* attached to it is not — see B-C7-1.

---

## Standing probes

### SAP-1 — tracing emission catalog completeness. PASS

No added or removed `event_type` emission site anywhere in the story's `crates/**/*.rs` diff against `develop` (`561d8bacc..HEAD`). No BC-2.16.002 Canonical Structured Event Catalog obligation is triggered by this PR.

### SAP-3 — spec-arm reachability. PASS

`test_jex_rg021` through `test_jex_rg024` all drive the E-QUERY-045 gate through `QueryEngine::execute` — the prism_query public surface — rather than handing a synthetic AST to the internal handler. The nested-call arms are reachable end-to-end in both directions (rg023 inner-non-literal reject, rg024 inner-literal accept). The case-insensitivity arms (rg021 uppercase, rg022 mixed-case) likewise route through the public surface.

---

## BLOCKING findings

### B-C7-1 — `AC-001-010-functional-unit-tests.json` asserts a test count impossible at its own pinned commit

| Field | Value |
|---|---|
| Severity | **blocking** |
| Category | coverage / demo-evidence provenance |
| File | `docs/demo-evidence/S-JSON-EXTRACT-UDF-001/AC-001-010-functional-unit-tests.json` |

The file pins:

```json
"commit": "83fa7ff51",
...
"total_tests": 15,
"passed": 15,
```

`83fa7ff51` is an **ancestor of** `7deb3674f` — older than the commit cycle 5 already flagged as stale, and 12 commits behind HEAD. At `83fa7ff51` the test file contained exactly **12** `test_jex_rg00*` tests. The three that bring the count to 15 were all introduced in `1487d2d9b`:

- `test_jex_rg007_b_key_at_exactly_max_len_accepted`
- `test_jex_rg007_c_multibyte_key_at_boundary_accepted`
- `test_jex_rg007_d_multibyte_key_over_boundary_rejected`

Verified by enumerating `fn test_jex_rg00*` at both revisions (12 at `83fa7ff51`, 15 at `1487d2d9b` and at HEAD).

**Why this is blocking rather than cosmetic:** the B-C6-4 fix corrected the count to the HEAD-true value while leaving the pin at a commit where that value is arithmetically impossible. The file now makes a *stronger* false claim than before the fix — `12 @ 83fa7ff51` was at least self-consistent and independently reproducible; `15 @ 83fa7ff51` is falsifiable by checking out the pin and counting. This is the same fabricated-provenance class as cycle 4 and cycle 5, third recurrence.

**Suggestion:** set `"commit": "1487d2d9b"` and add a `re_recorded` note in the style of the AC-006/007/012 siblings, e.g.:

```json
"re_recorded": "cycle-7: pin corrected to 1487d2d9b — the 15-test count reflects rg007_b/c/d added in that commit"
```

### B-C7-2 — `AC-011-pipe-mode-udf-registration.json` retains the same stale pin

| Field | Value |
|---|---|
| Severity | **blocking** |
| Category | coherence / demo-evidence provenance |
| File | `docs/demo-evidence/S-JSON-EXTRACT-UDF-001/AC-011-pipe-mode-udf-registration.json` |

```json
"commit": "83fa7ff51"
```

Unlike B-C7-1, this file's *assertions* are accurate. I verified the `JexMockAdapter` payload fixtures and the `QueryEngine::execute` path are byte-identical at `83fa7ff51` and at HEAD, so nothing stated here is false, and the `sap3_compliance` claim holds at both revisions.

It is still blocking because it is a stale provenance reference in the same class, and — decisively — this is the file whose row labels were **edited in cycle 6** under B-C6-3. An evidence file cannot be corrected at HEAD and simultaneously claim capture 12 commits earlier. The pin contradicts the edit that produced its current content.

**Suggestion:** set `"commit": "1487d2d9b"`, matching the B-C6-3 edit.

---

## Class-sweep assessment (TD-VSDD-097)

**Dimension 1 — sibling pair: NOT DISCHARGED.** The sibling set here is the five `AC-*.json` files produced by the same demo-recorder burst. Commit `1cf3638fa` carries the subject *"re-capture S-JSON-EXTRACT-UDF-001 evidence at HEAD 1487d2d9b (cycle-5 B-C5-1/2/3 class sweep)"*, and `e045ca087` re-pinned three more files. Neither burst touched the two pins above, so the cycle-5 commit subject is falsified by its own content. Absence of the changed string in a sibling is the failure mode, not evidence of cleanliness — a grep for `7deb3674f` would have reported clean on both residual files while leaving an older stale SHA in place.

**Dimension 2 — downstream copy target: DISCHARGED.** `evidence-report.md` is the downstream copy target for the per-AC JSON content. Its AC-008 observed-behavior bullets and its file-inventory cycle attributions for AC-006/007/012 were both updated consistently in `e045ca087`.

**Dimension 3 — mandate anchor: N/A.** This review introduces no `MUST` into a BC or spec.

---

## What was verified beyond the named findings

- Every `AC-*.json` file in `docs/demo-evidence/S-JSON-EXTRACT-UDF-001/` read in full; no fabricated assertion found beyond B-C7-1.
- Directory-wide grep for the `.. ` double-period artifact across all JSON: clean.
- Directory-wide grep for stale SHAs (`7deb3674`, `83fa7ff5`, `4dcc9ee8`, `3c7650fc`, `4165fe45`, `00167e40`): only the two pins above, plus legitimate history lines.
- `evidence-report.md` AC-008 observed-behavior bullets reconciled against the four test sources.
- File-inventory table cycle attributions for AC-006/007/012 confirmed updated to cycle-6.
- Full cycle-6 diff (`1cf3638fa..e045ca087`) reviewed line by line — 6 files, 20 insertions, 20 deletions, all demo-evidence, no code.

### Explicitly NOT findings

`evidence-report.md` lines 5 and 12 mention `83fa7ff51` and `7deb3674f` inside an explicit cycle-by-cycle provenance history (*"cycle-2 re-record was at 7deb3674f; original cycle-1 was at 83fa7ff51"*). That is legitimate historical narration of where each cycle recorded, not a stale pin, and must be **left as-is**. Flagged here so a cycle-8 fix burst does not "correct" it into a regression.

---

## Routing

Both findings are demo-evidence artifacts → route to `vsdd-factory:demo-recorder`. Two one-line JSON edits. No code change required — implementation clean since cycle 5, both SAP probes pass at `e045ca087`.

Per the frozen-HEAD streak rule (BC-5.39.001, DRIFT-ORCH-PRLEVEL-PUSH-001), the fix push resets the PR-LEVEL streak; cycle 8 must re-gate on the newly pushed HEAD.
