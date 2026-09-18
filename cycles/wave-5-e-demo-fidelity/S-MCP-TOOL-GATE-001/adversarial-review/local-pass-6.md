---
document_type: adversarial-review-pass
pass: 6
story: S-MCP-TOOL-GATE-001
cycle: wave-5-e-demo-fidelity
cascade_level: LOCAL
frozen_head: "0af76be5c"
date: 2026-09-18
clean_strict: false
clean_pr_merge: false
streak_before: 0
streak_after: 0
finding_count: 2
findings_closed_this_burst: 2
---

# S-MCP-TOOL-GATE-001 LOCAL Adversarial Pass 6 — Report

**FROZEN HEAD (pass-6 target):** `0af76be5c` (story v1.7 + feature HEAD advanced from 09658db3a via OBS-1 AC-002 wire round-trip test; just check 6135 PASS; LOCAL streak 0/3 on entry)

**Story version at review:** v1.7

## Summary

**CLEAN(strict): NO**
**CLEAN(PR-merge): NO**

2 findings (1 MEDIUM, 1 LOW) — docs-tier only. Production code passes ALL substantive scrutiny dimensions.

---

## Findings

### F-MED-001 [MEDIUM] — §File Structure Self-Contradictory + Omits Justfile Modification

**Category:** Documentation — story §File Structure section contradicts the actual diff boundary

**Detail:**

The story §File Structure section lists files the story modifies/creates and files explicitly excluded ("Files NOT to touch"). Two sub-defects identified:

1. **Self-contradiction (§Files Modified vs §Files NOT to touch):** The §Files Modified list does not include `Justfile` (repository root), yet the actual implementation diff `git diff --name-only f38604da4..0af76be5c` shows `Justfile` as one of the 7 touched files. The `Justfile` modification carries the operations-off gate leg that is load-bearing for AC-001 (CI runs without `operations` feature, so `just check` must not enable it). The §File Structure section as written implies the Justfile was NOT touched, which contradicts the feature HEAD contents.

   **7 files in diff:** `CHANGELOG.md`, `Justfile` [ADDED — not listed], `Cargo.toml`, `crates/prism-mcp/src/server.rs`, `crates/prism-mcp/src/tools/mod.rs` [listed as `operations.rs` in story — WRONG filename], `crates/prism-mcp/tests/bc_2_10_017_operations_feature_gate.rs`, `crates/prism-mcp/src/mcp_infrastructure.rs`

2. **Incorrect filename:** §Files Modified lists `crates/prism-mcp/src/tools/operations.rs` but the actual touched file is `crates/prism-mcp/src/tools/mod.rs`. This is a stale reference from a prior naming iteration.

3. **§Files NOT to touch missing carve-out:** The "Files NOT to touch" exclusion list should explicitly carve out the repository-root `Justfile` and `CHANGELOG.md` as repo-root files (not prism-mcp crate files) so that the scope is unambiguous.

**Severity rationale:** MEDIUM — the §File Structure section is the authoritative spec boundary for the implementer; an omitted load-bearing file (`Justfile`) + a wrong filename (`operations.rs` vs `mod.rs`) can cause a future re-delivery or regression attempt to miss the gate leg entirely.

**TD-VSDD-097 dimension:** Dim-1 (sibling pair) N/A; Dim-2 (downstream copy target) — §File Structure is consulted by test-writer and implementer; Dim-3 (mandate anchor) — AC-001 gate leg (`just check --no-ops-feature`) is anchored in story via T-D03/T-D04, which is correct; the defect is in the §File Structure inventory only.

---

### F-LOW-001 [LOW] — §Token Budget Test Inventory Documents Only OBS-2 E2E + Stale Frozen-HEAD Reference

**Category:** Documentation — §Token Budget section understates the e2e test inventory and carries a stale HEAD reference

**Detail:**

1. **Incomplete e2e test inventory:** The story §Token Budget section documents the e2e test as `tools_list_14` (OBS-2 / AC-001). The feature file `crates/prism-mcp/tests/bc_2_10_017_operations_feature_gate.rs` ships **2 e2e tests**: (a) `tools_list_14` (OBS-2 / AC-001, LIVE_TOOLS 14-tool wire-roundtrip) and (b) `list_capabilities_not_registered_tools_empty` (OBS-1 / AC-002, absent-ops slice wire-roundtrip closed by D-2574 fix-burst). Only the first is named in §Token Budget; the second is absent.

2. **Stale frozen-HEAD reference:** §Token Budget (or §Status) carries a reference to frozen HEAD `09658db3a`, which was the HEAD at the time of pass-5 review. After the D-2574 fix-burst advanced the feature HEAD to `0af76be5c`, any prose that cites the prior frozen HEAD as "current" is stale. The correct reference for the re-gate epoch is `0af76be5c`.

**Severity rationale:** LOW — documentation completeness; no behavioral impact. Both tests ship in the worktree; the inventory omission does not affect runtime correctness.

---

## Substantive Scrutiny — ALL PASS

The following dimensions were examined and pass at production-grade quality:

| Dimension | Result | Notes |
|-----------|--------|-------|
| AC-004 P0 — 14 LIVE_TOOLS unconditionally registered | PASS | `LIVE_TOOLS` const registered unconditionally; gated only behind `#[cfg(not(feature = "operations"))]` arm removes stub tools, not real tools |
| AC-003 — −32602 wire error for unregistered tool | PASS | rmcp 1.7.0 `error_mapping.rs` `MethodNotFound` maps to −32602; `list_capabilities_not_registered_tools_empty` + RG-GATE-003 confirm wire level |
| AC-002 — absent-ops slice wire round-trip | PASS | `list_capabilities_not_registered_tools_empty` e2e test added D-2574; passes `just check` 6135; OBS-1 CLOSED |
| BC-2.10.017 v1.5 — tables + §Story Anchor coherent | PASS | v1.5 carries dual-anchor (S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 + S-MCP-TOOL-GATE-001); §Story Anchor present; gate-absent −32602 rows added |
| BC-2.10.016 v1.3 Dim-1 sibling-sweep | PASS | §Story Anchor populated D-2574; TD-VSDD-097 Dim-1 discharged |
| SAP-1 — tracing emission catalog completeness | PASS | No new `event_type =` emissions in feature diff; BC-2.16.002 catalog sweep CLEAN |
| SAP-3 — spec-arm reachability | PASS | All BC-2.10.017 postcondition arms covered by real e2e tests (feature-present + feature-absent paths) |
| SID-1 — no-ignored-test rationalization prohibition | PASS | No `#[ignore]`'d tests without blocking-dep citation; all gate tests run in CI |
| SID-2 — composed-output assertions | PASS | Wire-level JSON assertions in e2e tests cover tool-list array shape |
| SAC-1 — enumerated Red Gate list on strict story | PASS | RG-GATE-001..004 enumerated in story; density check present |
| POL-42 / TD-VSDD-060 sibling-site sweep | PASS | No function signature changes in diff; sibling sweep N/A |
| just check 6135 | PASS | Exit 0; all 6135 tests pass on HEAD `0af76be5c` |
| Justfile CI gate leg (operations-off) | PASS | Justfile `check` target confirmed to NOT enable `operations` feature; CI gate arm load-bearing for AC-001 |

---

## Closure Actions Required

| Finding | Owner | Action |
|---------|-------|--------|
| F-MED-001 | story-writer | Reconcile §File Structure 1:1 against `git diff --name-only f38604da4..0af76be5c`: add `Justfile` to §Files Modified; correct `operations.rs` → `mod.rs`; add carve-out note in §Files NOT to touch for repo-root `Justfile` + `CHANGELOG.md` |
| F-LOW-001 | story-writer | §Token Budget: add `list_capabilities_not_registered_tools_empty` (OBS-1 / AC-002) to e2e test inventory; update stale frozen-HEAD reference from `09658db3a` → `0af76be5c` |

NO CODE change required. Feature HEAD `0af76be5c` FROZEN.

---

## Convergence Trajectory Update

Pass 6 result: **2 findings** (1 MED + 1 LOW). Trajectory update pending docs fix-burst.

LOCAL streak status: 0/3 on `0af76be5c` (findings present; streak cannot advance this pass).

**NEXT:** story-writer docs fix-burst (F-MED-001 + F-LOW-001) → state-manager D-2575 burst → adversary LOCAL pass-7 re-gate on frozen HEAD `0af76be5c` + story v1.8 — CLEAN(strict) candidate → 1/3.
