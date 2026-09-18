---
document_type: adversarial-review-pass
pass: 7
story: S-MCP-TOOL-GATE-001
cycle: wave-5-e-demo-fidelity
cascade_level: LOCAL
frozen_head: "0af76be5c"
date: 2026-09-18
clean_strict: false
clean_pr_merge: false
streak_before: 0
streak_after: 0
finding_count: 3
findings_closed_this_burst: 3
---

# S-MCP-TOOL-GATE-001 LOCAL Adversarial Pass 7 — Report

**FROZEN HEAD (pass-7 target):** `0af76be5c` (story v1.8 + feature HEAD frozen from D-2574 code advance; just check 6135 PASS; LOCAL streak 0/3 on entry)

**Story version at review:** v1.8

## Summary

**CLEAN(strict): NO**
**CLEAN(PR-merge): NO**

3 findings (1 MEDIUM, 1 LOW, 1 OBS) — docs-tier only. Production code passes ALL substantive scrutiny dimensions (AC-001/002/003/004/005, BC-2.10.017 v1.5, SAP-1/3, SID-1/2, SAC-1, POL-42; just check 6135). 6th consecutive code-clean pass.

---

## Findings

### F-MED-001 [MEDIUM] — §Tasks Phase-D note falsely claimed a closed gating set

**Severity:** MEDIUM

**Location:** §Tasks, Phase-D (cfg-gate section), task T-D03 note

**Description:** The Phase-D task note stated that T-D03 requires gating "1 inline test" — presenting a closed enumeration. The actual diff (feature HEAD `0af76be5c`) gated 21 ops-invoking inline tests. Following the story note literally produces a broken default build (E0599): code compiles on `features = ["operations"]` but the default build (no feature flag) fails because the note's narrow scope leads a reader to believe only 1 test needs the `#[cfg(feature = "operations")]` guard.

**Impact:** Any engineer following the story's Phase-D task note verbatim would ship a broken default build. MEDIUM severity because the production code itself is correct (all 21 tests are properly gated in the actual diff); the defect is in the story's guidance prose.

**Remediation:** Rewrite the T-D03 note to state the GENERAL RULE: "gate every inline test that invokes any ops-invoking handler," verified against the actual 21 tests in the diff. Remove the closed-set language and the false count.

**Verification:** After fix, T-D03 + Phase-D note must reflect the GENERAL RULE; story must be verifiable against the diff without E0599 in the default build.

---

### F-LOW/OBS-1 [LOW] — POL-7 H1-verbatim restored: BC-2.10.012 backticks + (L2)

**Severity:** LOW

**Location:** §Authority section, BC-2.10.012 row

**Description:** The §Authority BC-2.10.012 row's H1 verbatim title lacked the formatting present in the canonical BC file: backtick notation around the tool name and the "(L2)" layer annotation. The canonical BC-2.10.012 H1 is `` `prism_describe` Schema Discovery Tool (L2) ``; the story row was missing the backticks and the "(L2)" suffix. This is a POL-7 verbatim-H1 violation.

**Impact:** LOW — does not affect compilation, test coverage, or wire behavior. Affects traceability fidelity.

**Remediation:** Restore BC-2.10.012 H1 verbatim in §Authority row: `` `prism_describe` Schema Discovery Tool (L2) ``. Also confirm BC-2.10.017 §Authority H1 verbatim is correct.

---

### OBS-2 [OBS] — POL-39 tension: "Version at Authoring" column pins

**Severity:** OBS

**Location:** §Token Budget table, "Version at Authoring" column

**Description:** The "Version at Authoring" column in §Token Budget carries frozen point-in-time version pins (BC/ADR versions current at the time the story was authored). These are structural authoring-snapshot data, not live narrative prose. However, fresh-context adversary passes may flag them as POL-39 (anti-volatile-pin) violations on re-read because POL-39's current exemption list does not explicitly enumerate "Version at Authoring" frozen-snapshot columns.

**Adjudication question:** Is "Version at Authoring" (and analogous frozen authoring-snapshot columns) an explicit exemption under POL-39, or must these pins be treated as volatile?

**Impact:** OBS — does not affect code, tests, or wire behavior. Affects adversary pass oscillation: each fresh-context pass may independently raise or accept this pattern, causing unnecessary fix-burst churn.

**Disposition:** ORCHESTRATOR-ADJUDICATED EXEMPT (D-2576): "Version at Authoring" columns are frozen point-in-time authoring snapshots analogous to TD-VSDD-091 AC-source-of-truth-table exemption. POL-39-exempt per §History/§Changelog frozen-record rationale. Documented via footnote in story. Frozen pins retained.

**Process-gap spawned:** PROCESS-GAP D-2576 — POL-39 exemption list does not explicitly cover "Version at Authoring" frozen-snapshot columns; oscillation risk across all E-BETA3-REMEDIATION batch stories carrying this column. Codification candidate: amend POL-39 to enumerate this column class as exempt.

---

## Positive Scrutiny — Code Quality (6th Consecutive Code-Clean Pass)

All code-path scrutiny dimensions pass:

- **AC-001 (LIVE_TOOLS unconditional registration):** verified — 14 tools registered unconditionally regardless of `operations` feature state.
- **AC-002 (wire round-trip):** verified — `list_capabilities_not_registered_tools_empty` test asserts wire-level MCP response; feature-gated tools absent from capabilities list.
- **AC-003 (E-QUERY error code):** verified — absent `operations` feature returns `−32602` (unregistered tool) not `−32003` (catalog error).
- **AC-004 (NOT_YET_AVAILABLE_TOOLS empty slice):** verified — ops-absent path returns empty slice, not stubs.
- **AC-005 (no E0599 in default build):** verified — all 21 ops-invoking inline tests correctly gated; default build (no `operations` feature) is clean.
- **BC-2.10.017 v1.5:** all postcondition arms covered by tests; SAP-3 reachability verified end-to-end from MCP stdio surface.
- **SAP-1 (tracing emission catalog):** no new `event_type =` sites added; no catalog gap.
- **SAP-3 (spec-arm reachability):** all BC arms reachable from the MCP tool-call surface.
- **SID-1 (no ignored-test rationalization):** no `#[ignore]` tests bypassed.
- **SID-2 (composed-output assertions):** wire-level assertions cover full composed MCP response envelope.
- **SAC-1 (enumerated Red Gate list):** all 4 RG-GATE-001..004 tests present and named.
- **POL-42:** no new unsafe code blocks.
- **just check 6135:** all 6135 workspace tests pass on frozen HEAD `0af76be5c`.

---

## Fix Burst Summary (D-2576)

**D-2576 docs fix-burst closure:** story-writer v1.8→v1.9 at `946fe7b16`.

- **F-MED-001 CLOSED:** T-D03 + Phase-D note rewritten to GENERAL RULE: gate every inline test invoking a gated ops handler; 21 ops-invoking inline tests verified against the diff; false closed-set/count language removed.
- **OBS-1 (F-LOW/OBS-1) CLOSED:** POL-7 H1-verbatim restored: BC-2.10.012 `` `prism_describe` Schema Discovery Tool (L2) ``; BC-2.10.017 full H1 confirmed correct.
- **OBS-2 ORCHESTRATOR-ADJUDICATED EXEMPT (POL-39):** "Version at Authoring" column is a frozen point-in-time authoring snapshot, POL-39-exempt per §History/§Changelog + TD-VSDD-091 AC-source-of-truth-table rationale; documented via footnote; frozen pins retained. PROCESS-GAP opened for codification.

**NO code change.** Feature HEAD `0af76be5c` FROZEN. LOCAL 3-CLEAN streak remains 0/3.

**TD-VSDD-097 3-dim verdict:**
- Dim-1 (sibling pair): CLEAR — docs-tier only; no sibling story pair.
- Dim-2 (downstream copy target): CLEAR — no downstream verbatim copy.
- Dim-3 (mandate anchor): CLEAR — no new MUST added.

**NEXT:** adversary LOCAL pass-8 re-gate on frozen HEAD `0af76be5c` + story v1.9 — CLEAN(strict) candidate → streak 1/3.
