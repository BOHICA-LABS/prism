---
document_type: adversarial-review-pass
pass: 5
story: S-MCP-TOOL-GATE-001
cycle: wave-5-e-demo-fidelity
cascade_level: LOCAL
frozen_head: "09658db3a"
feature_head_after_fixes: "0af76be5c"
date: 2026-09-18
clean_strict: false
clean_pr_merge: true
streak_before: 0
streak_after: 0
finding_count: 2
findings_closed_this_burst: 2
---

# S-MCP-TOOL-GATE-001 LOCAL Adversarial Pass 5 — Report

**FROZEN HEAD (pass-5 target):** `09658db3a` (story v1.7 + 6 de-pins applied TD-VSDD-096; LOCAL streak 0/3 on entry)

**CLEAN(strict):** NO
**CLEAN(PR-merge):** YES

**Streak result:** BC-5.39.001 frozen-HEAD rule — feature HEAD ADVANCED to `0af76be5c` (OBS-1 code fix added wire round-trip test); LOCAL 3-CLEAN streak RESETS to **0/3** on new frozen HEAD `0af76be5c`.

---

## Findings

### F-001 [LOW] BC-2.10.017 §Story Anchor TBD — POL-4/POL-5

**Severity:** LOW
**Dimension:** Records — spec traceability field
**Location:** `BC-2.10.017` §Story Anchor (was literal "TBD")
**Defect:** §Story Anchor field contained literal "TBD" — no implementing story cited; POL-4 (every BC MUST trace to an anchor story at implementation) and POL-5 (story backref completeness) unsatisfied.

**Finding detail:** BC-2.10.017 §Story Anchor read "TBD" at pass-5 target `09658db3a` (story v1.7). The contract was originally authored at demo-readiness-2026-06-24 and amended six times; none of the amendments populated §Story Anchor. The implementing story S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 (PR #203 2026-06-26, AC-017/AC-018) merged before this contract was finalized; the anchor was never back-populated.

**TD-VSDD-097 Dim-1 (sibling sweep):** BC-2.10.016 is the sibling BC in the same SS-10/CAP-034 pair (MCP prompts vs MCP tools fast-return). BC-2.10.016 §Story Anchor also read "TBD" — swept in same burst.

**Status:** CLOSED — product-owner populated §Story Anchor in BC-2.10.017 v1.5 (dual-anchor: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 + S-MCP-TOOL-GATE-001) and BC-2.10.016 v1.3 (S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001). BC-INDEX v10.30→v10.31 pins synced.

---

### OBS-1 [OBS] AC-002 — direct-handler-only coverage (SAP-3 symmetry)

**Severity:** OBS (observation — SAP-3 symmetry deficiency)
**Dimension:** Test coverage — spec-arm reachability from public surface
**Location:** `AC-002` (NOT_YET_AVAILABLE_TOOLS fast-fail via `operations` feature enabled path)
**Defect:** AC-002 coverage consisted of direct-handler unit tests only (calling the handler stub directly with a synthesized request). Per SAP-3 §2, this counts as defense-in-depth ONLY; an arm with ONLY synthetic coverage = P2 finding if the arm is reachable from the product surface. The `operations`-enabled path IS reachable from the real MCP stdio surface.

**Finding detail:** The `test_BC_2_10_017_*` unit tests exercised the handler via synthetic `CallToolRequest` structs. No end-to-end wire round-trip test (real MCP client → server → response) covered AC-002. Wire-round-trip tests existed for AC-003 (the `operations`-ABSENT path, verifying −32602 on `get_diagnostics`). The asymmetry — absent-feature path had wire coverage, enabled-feature path did not — left a testability gap.

**Status:** CLOSED — implementer added `test_BC_2_10_017_list_capabilities_not_registered_tools_empty_via_end_to_end_client_roundtrip` (end-to-end MCP client round-trip for AC-002 enabled-feature path); feature HEAD advanced `09658db3a→0af76be5c` (488 default tests incl new test; 506 total with operations; `just check` 6135 PASS).

---

## Targeted Scrutiny — All GREEN at pass-5

| Probe | Verdict | Notes |
|-------|---------|-------|
| AC-004 P0: 14 LIVE_TOOLS unconditionally registered + partition test | PASS | ops-absent + ops-enabled feature states both verified at `09658db3a` |
| AC-003 −32602 wire test | PASS | `test_BC_2_10_017_get_diagnostics_returns_not_found_when_operations_absent` present |
| BC-2.10.017 v1.4 both compilation states | PASS | Error tables enumerate both states |
| Story v1.7 all 10 test names verbatim | PASS | All RG-GATE-001..005 + supporting tests grep-verified |
| Both-leg coverage (CI + Justfile) | PASS | `just check-ci` and `just check` both include operations feature path |
| SAP-1 (tracing emission catalog completeness) | PASS | No new `event_type=` sites in diff |
| SAP-3 (spec-arm reachability) | OBS-1 CLOSED | Wire round-trip added for AC-002 enabled path |
| SID-1 (no-ignored-test rationalization) | PASS | No `#[ignore]` on gate-related tests |
| SID-2 (composed-output assertions) | PASS | Error message content asserted at wire level |
| SAC-1 (enumerated Red Gate list) | PASS | Story v1.7 carries RG-GATE-001..005 + SAC-1 density check |
| TD-VSDD-060 (sibling-site sweep) | N/A | No function signature / constant changes in this burst |
| TD-VSDD-097 Dim-1 (sibling pair BC-2.10.016) | DISCHARGED | BC-2.10.016 swept in same burst (§Story Anchor populated) |
| TD-VSDD-097 Dim-2 (downstream copy target) | CLEAR | §Story Anchor is not a verbatim copy-source section |
| TD-VSDD-097 Dim-3 (mandate anchor) | CLEAR | No new MUSTs introduced |
| Code production-grade assessment | PASS | Strong 3-CLEAN candidate once F-001 + OBS-1 closed |

---

## Post-Fix State

**Feature HEAD after fixes:** `0af76be5c` (OBS-1 wire round-trip test added; `just check` 6135 PASS)
**LOCAL 3-CLEAN streak:** RESETS to **0/3** on `0af76be5c` per BC-5.39.001 frozen-HEAD rule (feature HEAD advanced via code change)
**STORY-INDEX pin:** UNCHANGED at v1.7 (story body not touched this burst — only BCs + code)
**BC-2.10.016:** v1.2→v1.3 (§Story Anchor populated)
**BC-2.10.017:** v1.4→v1.5 (§Story Anchor populated, dual-anchor)
**BC-INDEX:** v10.30→v10.31
**STATE:** v10.079→v10.080

**NEXT:** adversary LOCAL pass-6 re-gate on frozen HEAD `0af76be5c`.
