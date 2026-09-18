# PR Review — Cycle 8 — S-JSON-EXTRACT-UDF-001

**PR:** https://github.com/BOHICA-LABS/prism/pull/296
**Reviewed SHA:** `99a384c3615f66bc90032edb7653fe0fc1a691b8`
**Reviewer:** pr-reviewer (fresh-eyes, cycle 8)
**Scope:** cycle-7 finding verification (B-C7-1 / B-C7-2) + full `AC-*.json` class sweep in both dimensions (commit pin + double period) + SAP-1 probe + independent reproduction of every numeric evidence claim.
**Posted to GitHub:** https://github.com/BOHICA-LABS/prism/pull/296#issuecomment-5725753053

```
verdict: APPROVE
covered_sha: 99a384c3615f66bc90032edb7653fe0fc1a691b8
```

**Posting mechanism note:** verdict delivered via `gh pr comment`, carrying the full machine-readable `verdict:` / `covered_sha:` block. `gh pr review --approve` is blocked by GitHub's self-review restriction on this repo (author == reviewer identity), per the orchestrator's standing instruction for this PR and consistent with the cycle-3..cycle-7 precedent recorded in the archived reviews in this directory.

---

## Summary

Both cycle-7 findings are closed. The fix is exactly as narrow as the findings it closes — two `"commit"` string values, 2 files / 2 insertions / 2 deletions, nothing else touched. The full class sweep is clean in both dimensions across all five `AC-*.json` files, SAP-1 is vacuously satisfied, and every numeric claim in the evidence set reproduces byte-identically at HEAD.

**Zero blocking findings. Zero suggestions. Two NITs, explicitly non-gating.**

| Severity | Category | ID | Finding |
|---|---|---|---|
| nit | demo-evidence presentation | N-C8-1 | Mixed literal-quoting styles within the AC-008 `coercions` map |
| nit | demo-evidence provenance | N-C8-2 | `evidence-report.md` file-inventory cycle labels lag their content |

No code findings. Implementation has been clean since cycle 5; this cycle's delta is demo-evidence provenance only.

---

## Cycle-7 finding verification — both CLOSED

| ID | Finding | Status | Evidence |
|----|---------|--------|----------|
| B-C7-1 | `AC-001-010-functional-unit-tests.json` asserted `total_tests: 15` at pin `83fa7ff51`, where only 12 such tests existed | **CLOSED** | `_demo_meta.commit` = `1487d2d9b`; `total_tests: 15` / `passed: 15` independently reproduced at HEAD |
| B-C7-2 | `AC-011-pipe-mode-udf-registration.json` retained stale pin `83fa7ff51` despite cycle-6 content edit | **CLOSED** | `_demo_meta.commit` = `1487d2d9b`, matching the B-C6-3 row-label edit |

Both closures are structural, not paper-fixes (TD-VSDD-059): the pin now names the commit at which the recorded values are actually true, and that truth was re-derived by execution rather than accepted from the file.

---

## Class sweep — all five `AC-*.json` files read in full

### Dimension A — commit pin

| File | `_demo_meta.commit` | Verdict |
|------|---------------------|---------|
| `AC-001-010-functional-unit-tests.json` | `1487d2d9b` | PASS |
| `AC-006-non-literal-key-rejection.json` | `1487d2d9b` | PASS |
| `AC-007-key-length-boundary.json` | `1487d2d9b` | PASS |
| `AC-011-pipe-mode-udf-registration.json` | `1487d2d9b` | PASS |
| `AC-012-where-predicate-gate.json` | `1487d2d9b` | PASS |

Negative sweep for all seven prohibited SHAs (`83fa7ff51`, `7deb3674f`, `4dcc9ee8e`, `3c7650fca`, `783c24eef`, `1cf3638fa`, `e045ca087`) across the entire evidence directory: **zero hits in any `AC-*.json`.**

The only matches are three provenance-history sentences in `evidence-report.md` (§header and §re-record narrative) that explicitly label those SHAs as *prior* cycles. Per the cycle-7 review's "Explicitly NOT findings" section, these are legitimate historical narration and were correctly **left as-is** by the cycle-8 fix burst — no regression was introduced there.

**Pin currency is structurally guaranteed, not merely textually matched.** `git diff --name-only 1487d2d9b..HEAD` returns six paths, all under `docs/demo-evidence/S-JSON-EXTRACT-UDF-001/`:

```
AC-001-010-functional-unit-tests.json
AC-006-non-literal-key-rejection.json
AC-007-key-length-boundary.json
AC-011-pipe-mode-udf-registration.json
AC-012-where-predicate-gate.json
evidence-report.md
```

No file outside demo-evidence has changed since `1487d2d9b`, so `1487d2d9b` remains the correct behavioral pin for every transcript. The evidence *cannot* have drifted from the binary it describes — this closes the recurring stale-pin class at the root rather than per-file.

### Dimension B — double period

`grep -rnoE '\.\. |\. \.|\.\.$'` over the full evidence directory: **CLEAN, zero defects.**

The composed `text` fields read with exactly one sentence separator at the message/suggestion seam, satisfying SID-2 composed-output discipline:

- AC-006 / AC-012 — `…Dynamic key expressions are not supported. Provide a string literal…`
- AC-007 — `…exceeds the 256-byte maximum (CWE-400). Reduce the json_extract_string key…`

Two near-matches were evaluated and dismissed as out-of-class: the `,...}` sequences in `AC-011` `unit_test_evidence.key_assertions` are intentional JSON-fragment ellipsis, and the `1..0` form in the `evidence-report.md` file-inventory table is a range (`AC-001..005`). Neither is the `trim_end_matches` artifact.

---

## Independent reproduction of every numeric claim

Rather than accept the recorded summaries, all three were re-executed at HEAD:

| Claim source | Asserted | Reproduced at HEAD | Verdict |
|---|---|---|---|
| `AC-001-010` `test_results_summary` | 15 run / 15 passed | `15 tests run: 15 passed, 1787 skipped` | MATCH |
| `evidence-report.md` §prism-query | `44 tests run: 44 passed, 1758 skipped` | byte-identical | MATCH |
| `evidence-report.md` §prism-mcp | `7 tests run: 7 passed, 502 skipped` | byte-identical | MATCH |

The **skipped-count** halves match as well (1787 / 1758 / 502), which is the component a hand-written or fabricated summary gets wrong. This is strong evidence of genuine captured runs.

Commands used:

```
cargo nextest run -p prism-query -E 'test(test_jex_rg00)' --no-fail-fast
cargo nextest run -p prism-query -E 'test(test_jex)'      --no-fail-fast
cargo nextest run -p prism-mcp   -E 'test(JSON_EXTRACT_UDF)' --no-fail-fast
```

---

## Standing probes

### SAP-1 — tracing emission catalog completeness. PASS

`git diff origin/develop...HEAD -- 'crates/**/*.rs' | grep event_type` → **no matches.** The diff introduces no `tracing::*!(event_type=…)` emission site, so no BC-2.16.002 Canonical Structured Event Catalog row is required, and no catalog obligation is triggered by this PR.

This is consistent with ADR-066 §B1 step 2 (the per-row purity gate forbids per-row tracing in the UDF hot path) and is corroborated by `AC-009`'s recorded silent-null-propagation behavior on parse failure.

### SAP-3 — spec-arm reachability. PASS (carried, re-confirmed)

Code is unchanged since `1487d2d9b`, at which cycle 7 verified SAP-3 directly. Re-confirmed at the evidence layer this cycle: `AC-011`'s `sap3_compliance` field documents the end-to-end path `QueryEngine::execute() → pipe_sql_emitter → DataFusion → json_extract_string UDF → result batches` — public-surface reachability, not synthetic-AST coverage.

---

## Verification beyond the assigned scope

- **AC-007 boundary is really the boundary.** Parsed both `mcp_request` payloads and measured the quoted key directly: rejection path = **257** bytes, acceptance path = **256** bytes. The inclusive-boundary claim is backed by the actual wire bytes rather than by its label.
- **AC-008 coercion table diffed against source of truth.** All four recorded coercions match the `assert_eq!` literals in `crates/prism-query/tests/test_json_extract_udf.rs` (`test_jex_rg008_*`): integer→`42`, bool→`true`, array→`[1,2]`, object→`{"k":"v"}` (compact, no spaces). The cycle-6 fabrication class has not recurred.
- **AC-011 registration argument is logically sound.** `isError: false` with the sole failure being `E-SENSOR-010` in `results.sensor_errors` correctly isolates the failure to credential/connectivity, which proves DataFusion planned the query and therefore resolved `json_extract_string`. Absence of both `E-QUERY-045` and any unknown-function error is the right discriminator for a registration claim.
- **Code freeze confirmed.** `git diff --name-only 1487d2d9b..HEAD | grep -v '^docs/demo-evidence/'` → empty.

---

## Checklist

| # | Item | Verdict |
|---|------|---------|
| 1 | Diff coherence | PASS — every path traces to the JEX UDF surface (`json_extract_udf.rs`, `engine.rs` gate, `error_mapping.rs`, `filter_parser.rs` predicate parity, VP-162 proof, tests, evidence) |
| 2 | Description accuracy | PASS — last commit claims exactly two pin updates; diff is exactly 2 files / 2 insertions / 2 deletions |
| 3 | Test coverage | PASS — 44 prism-query + 7 prism-mcp JEX tests, plus Kani `vp162_json_extract_null_safety` |
| 4 | Demo evidence | PASS — `evidence-report.md` present; VHS `.gif` + `.webm` + `.tape` for the plan-gate ACs; MCP wire transcripts for all remaining ACs; both success and error paths recorded (AC-006/007/012 rejections vs. AC-007/011/012 acceptances). No `.txt` stand-ins |
| 5 | Commit quality | PASS — conventional format, story ID and finding IDs in subject, no AI attribution |
| 6 | Diff size | NOTED — ~5,117 insertions, but 2,266 are the JEX test file and ~800 are evidence; production delta is moderate and single-concern |
| 7 | Missing changes | PASS — AC-001..AC-012 each have a named RG test and a recorded transcript |
| 8 | Dependency status | PASS — no upstream story PR gating this one |

---

## Class-sweep assessment (TD-VSDD-097)

**Dimension 1 — sibling pair: DISCHARGED.** The sibling set is the five `AC-*.json` files produced by the same demo-recorder burst. All five were read in full and verified individually, not grepped for the changed string — which was precisely the cycle-7 failure mode (absence of `7deb3674f` in a file reported clean while an *older* stale SHA remained). This cycle's sweep enumerated the pin value in every sibling and additionally proved currency structurally via the `1487d2d9b..HEAD` path list, so no per-file grep blind spot remains.

**Dimension 2 — downstream copy target: DISCHARGED.** `evidence-report.md` is the downstream copy target for per-AC JSON content. Its §header `Commit at recording: 1487d2d9b` is correct, and its two embedded nextest summaries were re-executed and matched byte-for-byte including skipped counts. The residual lag is the file-inventory cycle labels — recorded as N-C8-2, non-gating.

**Dimension 3 — mandate anchor: N/A.** This review introduces no `MUST` into a BC or spec.

---

## Non-blocking observations

### N-C8-1 — Mixed literal-quoting styles in the AC-008 coercion map

| Field | Value |
|---|---|
| Severity | **nit** |
| Category | demo-evidence presentation |
| File | `docs/demo-evidence/S-JSON-EXTRACT-UDF-001/AC-001-010-functional-unit-tests.json` |
| Location | `AC-008_non_string_coercion.coercions` |

Three entries wrap the value in escaped quotes (`"\"42\""`, `"\"true\""`, `"\"[1,2]\""`, mirroring the Rust `"42"` string literal in the assertion), while the fourth omits them (`"{\"k\":\"v\"}"`, mirroring the content of the `r#"…"#` raw literal). Both forms faithfully reproduce their respective `assert_eq!` literal, so **no claim is wrong** — but a reader comparing entries side by side could misread the first three as containing literal quote characters in the Arrow cell.

**Suggestion** (future evidence refresh only; not a merge gate):

```json
"coercions": { "integer_42": "42", "bool_true": "true", "array_1_2": "[1,2]", "object_nested": "{\"k\":\"v\"}" }
```

### N-C8-2 — Inventory-table cycle labels lag their content

| Field | Value |
|---|---|
| Severity | **nit** |
| Category | demo-evidence provenance bookkeeping |
| File | `docs/demo-evidence/S-JSON-EXTRACT-UDF-001/evidence-report.md` |
| Location | §file-inventory table |

The table tags `AC-011-pipe-mode-udf-registration.json` and `AC-001-010-functional-unit-tests.json` as `cycle-1`, but AC-011's assertion content was corrected in cycle-6 (B-C6-3) and both files' commit pins in cycle-8. The load-bearing claim (§header `Commit at recording: 1487d2d9b`) is correct, so this is bookkeeping only with no behavioral or evidentiary impact.

**Suggestion:** a "last touched" column rather than a single cycle tag, if the table is revised. Deliberately **not** escalated to blocking: unlike B-C7-2, no pin contradicts an edit here — the pins are correct and only the descriptive label is stale.

---

## Assessment

This cycle's two findings were narrow and mechanical, and the fix is correspondingly narrow — two `"commit"` string updates with nothing else touched. Rather than confirm only the two named fixes, I swept the full class across all five `AC-*.json` files in both dimensions and then went past the assigned scope to re-derive every numeric and byte-level claim in the evidence set from scratch: three nextest summaries reproduced byte-identically (including the skipped counts a fabrication would miss), both AC-007 key lengths measured off the recorded wire payloads, and all four AC-008 coercions diffed against the test assertions.

The decisive structural finding is that code has been frozen since `1487d2d9b` — every changed path since then lives under `docs/demo-evidence/` — so no evidence pin can be silently stale and the recurring stale-pin class (cycles 4, 5, 6, 7) is closed at the root rather than file-by-file. SAP-1 is vacuously satisfied since the diff adds no emission site.

Nothing blocking remains. The two NITs are cosmetic, explicitly not merge gates, and do not require a fix burst.

---

## Routing

No routing required — zero blocking findings, zero suggestions. PR is ready to merge at `99a384c3615f66bc90032edb7653fe0fc1a691b8`.

The two NITs need no dispatch. If a future evidence refresh touches this directory for any other reason, `vsdd-factory:demo-recorder` may fold N-C8-1 and N-C8-2 in opportunistically; neither justifies a burst on its own, and neither should trigger a re-gate.
