---
document_type: adversarial-review-pass
story: S-MCP-TOOL-GATE-001
pass: 4
scope: LOCAL
frozen_head: 09658db3a
reviewed_by: adversary
date: 2026-09-18
clean_strict: false
clean_pr_merge: true
finding_count: 2
finding_severity_breakdown: "LOW: 2"
streak_effect: "NOT CLEAN(strict) — story-writer records-only de-pin sweep v1.6→v1.7 dispatched (TD-VSDD-096); feature HEAD 09658db3a FROZEN/UNCHANGED; LOCAL streak 0/3 UNCHANGED. NEXT: pass-5 re-gate on 09658db3a + story v1.7 — CLEAN(strict) candidate → 1/3."
---

# S-MCP-TOOL-GATE-001 LOCAL Adversarial Review — Pass 4

**Frozen HEAD reviewed:** 09658db3a
**Story version under review:** v1.6
**CLEAN(strict):** NO
**CLEAN(PR-merge):** YES (zero CRIT/HIGH/MED findings)
**Finding count:** 2 LOW

---

## CLEAN Report (all targeted scrutiny)

- **AC-004 P0 (14 LIVE_TOOLS ungated + partition test):** CLEAN — 14 tools registered unconditionally in `LIVE_TOOLS`; no `operations` feature gate on the live-tool slice; wire-confirmed. Both `#[cfg(feature="operations")]` and `#[cfg(not(feature="operations"))]` partition-test paths compile and assert correctly; AC-004 fully satisfied.
- **AC-003 −32602 wire behavior:** CLEAN — `get_diagnostics` and all operations stubs return `MethodNotFound (−32602)` under `operations` feature absent; `RG-GATE-003` exercises this path; wire-level assertion present.
- **BC-2.10.017 v1.4 both compilation states:** CLEAN — v1.4 rows (operations-absent −32602 §Error Cases + §Canonical Test Vectors) fully reflected in both cfg arms; no gap between spec and code under either feature state.
- **Story v1.6 traceability — all 10 test names verbatim-present:** CLEAN — all 10 story-referenced test names grep-verified present in worktree (T-D04 3rd gated test `test_bc_2_10_017_sibling_handlers_guard_precedes_audit` confirmed); §Red Gate planning list RG-GATE-001/002/004 infix-mismatch OBS-1 fully resolved in v1.6.
- **CI + Justfile coverage (both-leg):** CLEAN — CI workflow and Justfile correctly exercise both feature configurations; no gap between the two paths.
- **SAP-1 (tracing emission catalog completeness):** CLEAN — no new `event_type =` emission sites introduced on this feature branch; BC-2.16.002 catalog not affected.
- **SAP-3 (spec-arm reachability):** CLEAN — every BC-2.10.017 v1.4 EC arm has at least one end-to-end test from the MCP tool-call surface (not only synthetic-AST); no defense-in-depth-only arm.
- **SID-1 (no-ignored-test rationalization):** CLEAN — no `#[ignore]`'d tests used to defer required behavior; unit tests in production modules cover the cfg-gated paths without external-service dependency.
- **SID-2 (composed-output assertions):** CLEAN — composed error envelope (`error.code` + `error.message`) asserted on full wire shape, not only component fields.
- **SAC-1 (enumerated Red Gate list on strict stories):** CLEAN — v1.6 carries RG-GATE-001..004 enumerated list + BC-5.38.001 density check + red-then-green task ordering; structure complete.
- **TD-VSDD-060 (sibling-site sweep):** CLEAN — no function signature or canonical identifier changed on this branch; sweep scope N/A.

---

## Findings

### LOW-1 (POL-39) — Stale BC-version pins in §Authority / §Token Budget narrative

**Location:** Story v1.6 §Authority NOTE block + §Token Budget BC list  
**Observed:** §Authority NOTE cites `BC-2.10.017 v1.3` and `BC-2.10.012 v1.9` in prose; §Token Budget narrative references `BC-2.10.017 v1.3` and `BC-2.10.011` without version pin and `BC-2.10.012 v1.9`. Actual current versions: BC-2.10.017 is v1.4 (bumped in D-2570 OBS-3 fix, operations-absent −32602 rows added); BC-2.10.012 is v1.11 (D-2547 additive post-freeze).  
**Impact:** Records drift only; zero behavioral impact; no test/code/BC contract change required.  
**Fix route:** TD-VSDD-096 records-only micro-burst — story-writer de-pins (removes) stale version-pin prose from §Authority NOTE and §Token Budget BC narrative. Per POL-39/TD-VSDD-091 these pins are volatile and should be removed entirely (not updated to current version), matching the convention applied to all other Wave-5 stories.  
**Classification:** LOW — records-tier drift (POL-39 volatile-pin).

---

### LOW-2 (TD-VSDD-091) — Volatile server.rs line-number cite in §Tasks Phase-D note

**Location:** Story v1.6 §Tasks Phase-D implementation note  
**Observed:** A note references `server.rs:NNN` with a specific line number (volatile positional cite) to locate the operations-tool dispatch arm.  
**Impact:** Records drift only; will self-invalidate on the next diff touching server.rs. Zero behavioral or test impact.  
**Fix route:** TD-VSDD-096 records-only micro-burst — story-writer removes/replaces the `server.rs:NNN` line-number cite with a section/symbol anchor (e.g., `server.rs §handle_call operations arm` or equivalent non-positional reference).  
**Classification:** LOW — volatile line-number cite (TD-VSDD-091).

---

## Summary

Code is production-grade. Two records-tier LOW findings — stale BC-version pins (LOW-1, POL-39) and a volatile server.rs line-cite (LOW-2, TD-VSDD-091) — are both story-text-only and have zero behavioral impact. No code/spec/BC changes required.

**Adversary assessment:** Feature HEAD 09658db3a is converged on the code axis. All targeted scrutiny GREEN (AC-003/AC-004 both compilation states; BC-2.10.017 v1.4; story v1.6 traceability all 10 test names; both-leg CI+Justfile; SAP-1/3; SID-1/2; SAC-1; TD-VSDD-060). Only records-tier drift remained.

**CLEAN(strict):** NO (2 LOW records-tier findings)  
**CLEAN(PR-merge):** YES (zero CRIT/HIGH/MED)  
**Feature HEAD:** 09658db3a — FROZEN (no code change required)  
**LOCAL streak:** 0/3 UNCHANGED  
**Next:** story-writer exhaustive records-only de-pin sweep v1.6→v1.7 (TD-VSDD-096), then adversary LOCAL pass-5 re-gate on 09658db3a + story v1.7 — CLEAN(strict) candidate → 1/3.
