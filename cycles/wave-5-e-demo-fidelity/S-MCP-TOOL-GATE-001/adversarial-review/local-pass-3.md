---
document_type: adversarial-review-pass
story: S-MCP-TOOL-GATE-001
pass: 3
scope: LOCAL
frozen_head: 09658db3a
reviewed_by: adversary
date: 2026-09-18
clean_strict: false
clean_pr_merge: true
finding_count: 1
finding_severity_breakdown: "LOW: 1"
streak_effect: "NOT CLEAN(strict) — story-text-only fix (v1.5→v1.6 @8d87c1801); feature HEAD 09658db3a FROZEN/UNCHANGED; LOCAL streak 0/3 UNCHANGED. NEXT: pass-4 re-gate on 09658db3a + story v1.6 — first candidate for CLEAN(strict) → 1/3."
---

# S-MCP-TOOL-GATE-001 LOCAL Adversarial Review — Pass 3

**Frozen HEAD reviewed:** 09658db3a
**Story version under review:** v1.5
**CLEAN(strict):** NO
**CLEAN(PR-merge):** YES (zero CRIT/HIGH/MED findings)
**Finding count:** 1 LOW

---

## CLEAN Report (all targeted scrutiny)

- **AC-004 P0 (14 LIVE_TOOLS ungated):** CLEAN — 14 tools registered unconditionally in `LIVE_TOOLS`; no operations gate on the live-tool slice; wire-confirmed.
- **AC-003 −32602 wire behavior:** CLEAN — `get_diagnostics` and all operations stubs return `MethodNotFound (−32602)` under `operations` feature absent; `RG-GATE-003` exercises this path; wire-level assertion present.
- **AC-005/T-D01 pass-2 OBS-A fix coherence:** CLEAN — v1.5 story text correctly describes the `RG-GATE-003` wire behavior approach (invoke MCP tool by name; assert `−32602` response) rather than the unsatisfiable direct Rust method call; E0599 trigger eliminated; fix is coherent.
- **NOT_YET_AVAILABLE_TOOLS gating:** CLEAN — `NOT_YET_AVAILABLE_TOOLS` returns an empty slice when `operations` feature is absent; no `−32003` catalog pollution.
- **Sibling-sweep complete (incl. 3rd mcp_infra test):** CLEAN — all three ops-gated `mcp_infrastructure` tests swept (server.rs inline test + two `mcp_infrastructure.rs` tests including newly added `test_bc_2_10_017_sibling_handlers_guard_precedes_audit`); no ungated residuals; T-D04 task correctly reflects the 3-test obligation.
- **OBS-2 e2e load-bearing:** CLEAN — e2e assertion in `mcp_infrastructure.rs` tests serialized JSON wire output; load-bearing (not a no-op).
- **Justfile operations-off leg runs RG tests:** CLEAN — `just check` operations-off leg is non-redundant (distinct `--no-default-features` invocation targeting `RG-GATE-003` path); exercises the gated-out configuration correctly.
- **BC-INDEX pins match v1.4/v1.7 — no drift:** CLEAN — BC-2.10.017 at v1.4 and BC-2.10.011 at v1.7 in both artifacts and BC-INDEX rows; no POL-40/L10 drift.
- **POL-32/40 + SAP-1:** CLEAN — no new `event_type =` sites added; governance policies satisfied.
- **SID-1 (no-ignored-test rationalization):** CLEAN.
- **SID-2 (composed-output assertions):** CLEAN.

---

## Findings

### OBS-1 [LOW] — Story §Red Gate planning list: test-name infix mismatch on RG-GATE-001/002/004

**Severity:** LOW
**Category:** Story text / specification prose defect
**Affects:** Story file only; feature code at HEAD 09658db3a is CORRECT; behavioral contracts are CORRECT; no runtime or test-execution impact

**Description:**

The story's §Red Gate planning list (the enumerated RG-GATE-001..RG-GATE-004 test-name entries per SAC-1) names three test functions as `test_BC_2_10_017_<suffix>` without the `test_BC_2_10_017_` infix that the actual shipped test functions carry. Specifically:

- RG-GATE-001: story prose names the test without the `test_BC_2_10_017_` infix; shipped function name is `test_BC_2_10_017_<suffix_001>`
- RG-GATE-002: same pattern; shipped function name is `test_BC_2_10_017_<suffix_002>`
- RG-GATE-004: same pattern; shipped function name is `test_BC_2_10_017_<suffix_004>`

RG-GATE-003 is already correct — its story reference matches the shipped function name exactly.

The actual test functions all carry the `test_BC_2_10_017_` infix in the codebase. The story's planning list uses abbreviated names that drop the BC anchor prefix. This creates a traceability gap: a reader of the story cannot grep for the listed names to find the tests, undermining the SAC-1 enumerated-list discipline.

**Code behavior (CORRECT — no code change required):**
All four RG-GATE-001..RG-GATE-004 tests exist in the worktree under their full `test_BC_2_10_017_` names. The coverage is complete. This is a docs-only discrepancy.

**Fix:**
Story-writer reconciliation sweep: update RG-GATE-001, RG-GATE-002, and RG-GATE-004 planning list entries in the story to use the full `test_BC_2_10_017_` infix, matching the shipped test function names exactly. Verify all 10 story-referenced test names grep-present in worktree as part of the sweep.

**TD-VSDD-097 (3-dim sweep):**
- Dim-1 (sibling pair): No sibling story twin for S-MCP-TOOL-GATE-001. CLEAR.
- Dim-2 (downstream copy target): §Red Gate planning list entries are not copy-sources for downstream artifacts. CLEAR.
- Dim-3 (mandate anchor): No new MUSTs introduced by the name-reconciliation fix. CLEAR.

---

## Adversary Assessment

The code at frozen HEAD 09658db3a is **effectively converged on the code axis.** All behavioral contracts (BC-2.10.017 v1.4 + BC-2.10.011 v1.7), wire behavior, test coverage (14 LIVE_TOOLS ungated, −32602 ops-absent, e2e 14-tool wire-roundtrip), and the T-D04 3-test sibling-sweep obligation are all correct and complete.

The single finding is a story-text precision defect: the §Red Gate planning list uses abbreviated test names for three of four entries, making the test-name traceability grep-unfriendly. The intended tests exist and pass. After story-text name reconciliation (v1.5→v1.6), re-gate pass-4 on frozen HEAD 09658db3a is the first candidate for CLEAN(strict) → streak 1/3.
