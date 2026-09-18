---
document_type: adversarial-review-pass
story: S-MCP-TOOL-GATE-001
pass: 1
scope: LOCAL
frozen_head: b61c25dad
reviewed_by: adversary
date: 2026-09-18
clean_strict: false
clean_pr_merge: true
finding_count: 3
finding_severity_breakdown: "OBS: 3"
streak_effect: "RESET — feature HEAD advanced b61c25dad→09658db3a via fix-burst; new frozen HEAD 09658db3a; LOCAL streak 0/3"
---

# S-MCP-TOOL-GATE-001 LOCAL Adversarial Review — Pass 1

**Frozen HEAD reviewed:** b61c25dad
**CLEAN(strict):** NO
**CLEAN(PR-merge):** YES (zero CRIT/HIGH/MED findings)
**Finding count:** 3 OBS

---

## Findings

### OBS-1 [process-gap] — check/check-ci coverage gap: --all-features only

**Severity:** OBS (process-gap)
**Category:** Test coverage completeness

`just check` and `just check-ci` were invoked with `--all-features` only. The `RG-GATE` tests (T-D01/T-D02 cfg-gate compile-fail gates: operations-absent build compiles without ops stubs; operations-absent build emits exactly 14 LIVE_TOOLS) are compiled OUT under `--all-features` because the `cfg(not(feature="operations"))` blocks are inactive. CI covers the `test-no-default-features` leg, but local verification did not explicitly run the no-operations build to confirm RG-GATE test compilation.

**Resolution:** Add a `just check` / `just check-ci` leg for the `prism-mcp` crate without the `operations` feature, so RG-GATE tests compile and run locally. Implemented via Justfile amendment @09658db3a (operations-off `prism-mcp` leg added to `just check` and `just check-ci`).

**Status after fix-burst:** CLOSED — @09658db3a

---

### OBS-2 [wire-roundtrip] — AC-001/AC-002 verified via internal catalog, not true client round-trip

**Severity:** OBS
**Category:** Test coverage completeness — wire-level assertion discipline (CLAUDE.md §Conventions)

AC-001 (tools/list returns exactly the 14 LIVE_TOOLS when `operations` feature absent) and AC-002 (all 14 tools are ungated in default build) were verified by reading the `production_tool_catalog()` internal function directly, rather than via a true MCP client `tools/list` round-trip that exercises the full JSON-RPC 2.0 serialization path. Per CLAUDE.md wire-shape assertion discipline (D-1715), any test covering an MCP-visible surface must include at least one assertion on the serialized JSON output.

**Resolution:** Add an end-to-end `McpClient`-based test that calls `client.list_tools()` and asserts the count is exactly 14, exercising the full `tools/list` JSON-RPC wire path. Implemented as `test_BC_2_10_017_tools_list_14_via_end_to_end_client_roundtrip` @f5fa2e2a1.

**Status after fix-burst:** CLOSED — @f5fa2e2a1 (default build 487 pass incl e2e; operations build 506 pass; just check exit 0)

---

### OBS-3 [spec-completeness] — BC-2.10.017 §Error Cases + §Canonical Test Vectors omit operations-absent -32602 path

**Severity:** OBS
**Category:** Spec completeness — error-table coverage

BC-2.10.017 §Error Cases table and §Canonical Test Vectors did not include the scenario where an operations-absent build receives a call to an operations-scoped tool (e.g., `prism_list_infusions`). The contract for the `operations` feature absent state defines a -32602 response (unregistered tool) but the error-table rows covering this path were missing, creating an incomplete contract that could mislead future implementers.

**Resolution:** BC-2.10.017 v1.3→v1.4 @9c90ec8db: operations-absent -32602 rows added to §Error Cases and §Canonical Test Vectors. TD-VSDD-097 three-dimension sweep: Dim-1 CLEAR (no sibling twin), Dim-2 CLEAR (no verbatim copy-source section in downstream artifacts), Dim-3 CLEAR (no new unanchored MUSTs introduced; existing -32602 MUST already anchored to RG-GATE). No story-body propagation required (story v1.4 already cites BC-2.10.017 v1.4 pin from D-2568).

**Status after fix-burst:** CLOSED — BC-2.10.017 v1.4 @9c90ec8db

---

## Positive Verifications

- **AC-004:** All 14 LIVE_TOOLS ungated — confirmed via `production_tool_catalog()` enumeration and AC-004 test; `operations` feature absent build emits exactly 14.
- **AC-003:** -32602 wire-correct for unregistered tools — confirmed via `test_bc_2_10_017_unregistered_tool_returns_minus_32602` test harness.
- **Sibling-sweep (TD-VSDD-060):** Complete — including a 3rd `mcp_infrastructure.rs` test beyond T-D04 sweep scope. Zero sibling callsite drift.
- **SAP-1 (tracing emission catalog):** No new `event_type =` emissions introduced; no catalog gap.
- **SID-1 (no-ignored-test rationalization prohibition):** No `#[ignore]`'d tests used as defer-pattern.
- **SID-2 (composed-output assertions):** Full composed error string asserted, not only components.
- **SAC-1 (enumerated Red Gate list):** RG-GATE, RG-001..RG-NNN present; density check passes.
- **POL-12 (no hardcoded paths):** Clean.
- **POL-14 (BC lifecycle auto-promotion):** N/A at pass-1 (story not yet merged).
- **POL-36 (no unwrap in production code):** Clean.

---

## Fix-Burst Summary

**Feature HEAD:** b61c25dad → 09658db3a (OBS-1+OBS-2 code fixes)
**Spec commit:** @9c90ec8db (BC-2.10.017 v1.3→v1.4; OBS-3 fix)

| Finding | Fix Location | Commit |
|---------|-------------|--------|
| OBS-1 (check-ci coverage) | Justfile: operations-off prism-mcp leg | @09658db3a |
| OBS-2 (wire roundtrip) | crates/prism-mcp/tests/mcp_infrastructure.rs: test_BC_2_10_017_tools_list_14_via_end_to_end_client_roundtrip | @f5fa2e2a1 |
| OBS-3 (spec error-table) | .factory/specs/behavioral-contracts/BC-2.10.017-not-yet-available-tools-fast-fail-audit-channel-non-blocking.md | @9c90ec8db |

**Streak ruling (BC-5.39.001 frozen-HEAD rule):** RESET — feature HEAD advanced b61c25dad→09658db3a via OBS-1/OBS-2 fix-burst. NEW frozen HEAD for re-gate = 09658db3a. LOCAL 3-CLEAN streak = 0/3.

**NEXT:** adversary LOCAL re-gate pass on frozen HEAD 09658db3a.
