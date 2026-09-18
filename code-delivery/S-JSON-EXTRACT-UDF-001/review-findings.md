# PR #296 Review Convergence Tracking — S-JSON-EXTRACT-UDF-001

**Story:** S-JSON-EXTRACT-UDF-001 — `json_extract_string` ScalarUDF + literal-key plan gate (E-QUERY-045)
**PR:** https://github.com/BOHICA-LABS/prism/pull/296
**Branch:** `feature/S-JSON-EXTRACT-UDF-001` → `develop`
**Max cycles allowed:** 10 (BC-5.39.001)

---

## Convergence Table

| Cycle | HEAD SHA | Findings | Blocking | Fixed | Remaining | Verdict |
|-------|----------|----------|----------|-------|-----------|---------|
| 1 | `4dcc9ee8e` | 8 | 1 (B1) | 8 | 0 | REQUEST_CHANGES |
| 2 | `7deb3674f` | 10 | 1 (B-1) | 10 | 0 | REQUEST_CHANGES |
| 3 | `3c7650fca` | 5 | 3 (B-C3-1/2/3) | 5 | 0 | REQUEST_CHANGES |
| 4 | `3c7650fca` | 23 | 5 (B-1..5) | 5/5 | 0→pending | REQUEST_CHANGES |
| 5 | `1487d2d9b` | 3 BLOCKING + 5 SUGGESTION + 5 NIT | 3 (B-C5-1/2/3) | 0 | 3 | REQUEST_CHANGES — demo-evidence recurrence |
| 6 | `1cf3638fa` | 4 BLOCKING + 4 SUGGESTION | 4 (B-C6-1/2/3/4) | 0→4 | 0 | REQUEST_CHANGES — AC-*.json sidecars not independently verified |
| 7 | `e045ca087` | 2 BLOCKING + 0 other | 2 (B-C7-1/2) | 0→2 | 0 | REQUEST_CHANGES — stale commit pins; two one-line JSON fixes |
| 8 | `99a384c36` | 0 BLOCKING + 2 NIT | 0 | 2 | 0 | **APPROVE** — covered_sha=99a384c3615f66bc90032edb7653fe0fc1a691b8 |

**3-CLEAN streak (BC-5.39.001):** 0/3 on current HEAD (streak resets on push per DRIFT-ORCH-PRLEVEL-PUSH-001)

---

## Cycle 4 Blocking Findings — Triage & Disposition

| Finding | Severity | Description | Route | Status |
|---------|----------|-------------|-------|--------|
| BLOCKING-1 | P0 SECURITY | `fn_call_comparison` in `filter_parser.rs` uses case-sensitive `"json_extract_string"` match; `JSON_EXTRACT_STRING(col, non_literal)` bypasses E-QUERY-045 gate | implementer | ✅ FIXED in working tree (to_ascii_lowercase) |
| BLOCKING-2 | BLOCKING | Doubled period in MCP content_text: `"...not supported.. Provide..."` and `"...CWE-400).. Reduce..."` | implementer | ✅ FIXED in working tree (trim_end_matches('.')) |
| BLOCKING-3 | BLOCKING | AC-001-010 demo evidence fabricated assertion `extracted_col.value(0) == "literal_val"` (0 occurrences in codebase) | demo-recorder | ✅ FIXED in 783c24eef |
| BLOCKING-4 | BLOCKING | evidence-report.md hand-typed nextest output (wrong format, count 33 not 40); false MCP attribution | demo-recorder | ✅ FIXED in 783c24eef |
| BLOCKING-5 | BLOCKING | Nested-call recursion arm in gate (`for arg in args`) has zero coverage | implementer | ✅ FIXED in working tree (RG-JEX-023/024) |

All 5 BLOCKING findings from cycle 4 have verified fixes in commit 1487d2d9b.

## Cycle 5 Blocking Findings — Triage & Disposition

| Finding | Severity | Description | Route | Status |
|---------|----------|-------------|-------|--------|
| B-C5-1 | BLOCKING | evidence-report.md nextest blocks hand-authored; missing elapsed bracket | demo-recorder | FIXED in 1cf3638fa (real nextest output with elapsed bracket) |
| B-C5-2 | BLOCKING | Wrong counts: prism-query 40→44, prism-mcp 6→7, AC-001-010 JSON 12→15; phantom test; wrong total header | demo-recorder | FIXED in 1cf3638fa (44/7, phantom row removed, header updated) |
| B-C5-3 | BLOCKING | 6+ more fabricated assertions in AC-001-010 + other ACs; only 1 instance fixed in 783c24eef; TD-VSDD-097 class sweep not done | demo-recorder | FIXED in 1cf3638fa (class sweep: AC-002/003/005/009 all corrected; TD-VSDD-097 dim-1+2+3 discharged) |

---

## Fix Commits

| Commit | Description | Fixes |
|--------|-------------|-------|
| `cb50f251f` | fix(deps): bump rustls 0.23.40→0.23.45 (RUSTSEC-2026-0285) | Pre-existing CI advisory |
| `4dcc9ee8e` | fix(prism-query,prism-mcp): cycle-1 findings B1+N3+N4+N6+T1+T2 | Cycle 1 blocking |
| `7deb3674f` | fix(prism-query,prism-mcp): cycle-2 findings B-4+N-a+N-c+N-d+N-f+N-g+N-i | Cycle 2 blocking |
| `00167e407` | docs(demo): re-record AC-005/AC-006/AC-007/AC-012 (cycle-2) | Cycle 2 demo evidence |
| `4165fe454` | docs(demo): fix B-C3-3 AC-005-non-literal-key-error.json mislabel | Cycle 3 demo fix |
| `3c7650fca` | fix(prism-query,prism-mcp): cycle-3 findings B-C3-1+B-C3-2+e+inline | Cycle 3 blocking |
| `783c24eef` | docs(demo): fix fabricated assertions + nextest format (cycle-4) | BLOCKING-3, BLOCKING-4 |
| `1487d2d9b` | fix(prism-query,prism-mcp): close case-sensitive bypass + nested-call coverage + composed-text assertion (cycle-4) | BLOCKING-1, BLOCKING-2, BLOCKING-5 |
| `1cf3638fa` | docs(demo): re-capture evidence at HEAD 1487d2d9b (cycle-5 B-C5-1/2/3 class sweep) | B-C5-1, B-C5-2, B-C5-3 |

---

## Triage Comments Posted

| Cycle | Comment URL |
|-------|-------------|
| 4 | https://github.com/BOHICA-LABS/prism/pull/296#issuecomment-5725128966 |
| 5 | https://github.com/BOHICA-LABS/prism/pull/296#issuecomment-5725429332 |

---

## Next Actions

1. ✅ Demo-recorder a28d94f871a5fbb7c pushed 1cf3638fa — all B-C5-1/2/3 fixed
2. ✅ New HEAD on origin: 1cf3638fa57453e4e0773f2788095cdf4c0645eb
3. ✅ Cycle 6 pr-reviewer (a6f250f2018fb5304) dispatched on new HEAD
4. ✅ CI watcher (ab02becb9f0359652) re-launched for new HEAD
5. Cycle 6 = REQUEST_CHANGES; 4 BLOCKING (B-C6-1/2/3/4) — AC-*.json sidecars not independently verified
6. ✅ Cycle 6 triage posted: https://github.com/BOHICA-LABS/prism/pull/296#issuecomment-5725560292
7. Demo-recorder (a33b2fc186c367ce7) dispatched to fix B-C6-1/2/3/4
8. After push → cycle 7 pr-reviewer on new HEAD; if APPROVE + CI green → merge
