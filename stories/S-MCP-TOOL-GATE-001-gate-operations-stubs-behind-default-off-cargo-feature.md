---
document_type: story
story_id: S-MCP-TOOL-GATE-001
title: "Gate 40 operations stubs behind default-off Cargo feature to eliminate -32003 catalog pollution"
wave: 1
epic_id: E-BETA3-REMEDIATION
version: "1.0"
status: draft
producer: story-writer
phase: 3
priority: P0
points: 3
tdd_mode: strict
target_module: prism-mcp
subsystems: ["SS-10"]
# Subsystem anchor justification:
#   SS-10 (MCP Server) owns all of crates/prism-mcp/src/server.rs, the tool router, and
#   the LIVE_TOOLS / NOT_YET_AVAILABLE_TOOLS consts. Every file touched by this story
#   lives within SS-10's module boundary. No other subsystem is crossed.
crates_touched: [prism-mcp]
estimated_days: 0.5
depends_on: []
# depends_on anchor justification:
#   No product-story hard dependencies — the gate is entirely within prism-mcp.
#   Can enter Wave 1 of the beta.3 batch immediately.
blocks:
  - S-MCP-ENVELOPE-DESCRIBE-001
  - S-MCP-NULL-ENCODING-001
  - S-BETA3-RELEASE-001
# blocks anchor justifications:
#   S-MCP-ENVELOPE-DESCRIBE-001 / S-MCP-NULL-ENCODING-001: same crate; merge this story
#   first (lowest blast-radius, no struct changes) to reduce merge-conflict risk on
#   server.rs for subsequent Wave-2 stories.
#   S-BETA3-RELEASE-001: W1 must merge before the beta.3 release bundle can assemble.
risk: LOW
# Risk justification:
#   LOW — blast radius is prism-mcp only; no cross-crate API changes; no struct changes;
#   no new public types; NOT_YET_AVAILABLE_TOOLS is compile-time const only. The single
#   unknown is rmcp #[tool_router] proc-macro behavior under #[cfg] (Spike Task T-S01
#   addresses this before any tests are written).
behavioral_contracts:
  - BC-2.10.017
  - BC-2.10.011
# BC status: AMENDMENTS PENDING PO authorship.
#   BC-2.10.017 §Postconditions currently states "Tool names in NOT_YET_AVAILABLE_TOOLS
#   are registered in tools/list (visible to clients)" — the OPPOSITE of the new behavior.
#   BC-2.10.017 §Invariants must be amended to add: "When operations feature is absent
#   (default), NOT_YET_AVAILABLE_TOOLS is &[] and no stub tools are registered."
#   BC-2.10.011 §Postconditions `not_registered_tools` description must be amended to
#   clarify the empty-slice state when the operations feature is absent.
#   BOTH amendments must be authored by product-owner and reach active/approved status
#   BEFORE this story is dispatched to test-writer (spec-first gate S-7.01).
#   Amendment scope per beta3-remediation-delta-analysis.md §Issue 1 and §S-MCP-TOOL-GATE-001:
#   - BC-2.10.017: Replace "registered in tools/list (visible to clients) but invoke the
#     fast-fail handler" with "NOT registered when `operations` feature is absent (default);
#     registered and fast-failing ONLY when `operations` feature is enabled."
#   - BC-2.10.011: update `not_registered_tools` semantics: empty slice when feature absent.
verification_properties: []
assumption_validations: []
risk_mitigations: []
---

# S-MCP-TOOL-GATE-001: Gate 40 Operations Stubs Behind Default-off Cargo Feature

## Authority

**beta3-remediation-delta-analysis.md §Issue 1 + §Issue 2 + §S-MCP-TOOL-GATE-001** is the
authoritative design decision for this story. Read that document's Part 1 §Issue 1, Part 1
§Issue 2, and Part 3 §S-MCP-TOOL-GATE-001 in full before implementing.
Path: `.factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md`

**BC-2.10.017** (Not-Yet-Available Tools Fast-Fail) governs the NOT_YET_AVAILABLE_TOOLS
const and the fast-fail handler behavior. This story implements an amendment to that BC.
Path: `.factory/specs/behavioral-contracts/BC-2.10.017-not-yet-available-tools-fast-fail-audit-channel-non-blocking.md`

**BC-2.10.011** (list_capabilities Meta-Tool) governs the `not_registered_tools` field in
the `list_capabilities` response. This story implements an amendment to that BC.
Path: `.factory/specs/behavioral-contracts/BC-2.10.011-list-capabilities-meta-tool.md`

**BC-2.10.012** (prism_describe Schema Discovery Tool) §Preconditions §1 is a protection
boundary: "`prism_describe` is always registered — it is NOT gated by any feature flag or
capability check." The implementer MUST read this BC to confirm all 14 LIVE_TOOLS remain
ungated after this story lands.
Path: `.factory/specs/behavioral-contracts/BC-2.10.012-prism-describe-schema-discovery-tool.md`

> NOTE: The amendments to BC-2.10.017 and BC-2.10.011 described above must be authored
> by the product-owner and reach `status: active` BEFORE test-writer dispatch. See
> §Spec-First Gate note in frontmatter `behavioral_contracts` comment block.

---

## Problem Statement

All ~40 `prism-operations` tool handler methods are unconditionally compiled and registered in
the MCP router. At runtime every invocation returns `-32003 not_yet_available_msg`. This
causes two defects observed in the beta.2 Monroe demo live-test log:

**Issue 1:** 40 of 54 tools visibly listed in `tools/list` never do anything. An LLM agent
reading the tool catalog sees 54 tools and must filter noise from the 40 permanently-broken
stubs. This degrades agent reasoning quality.

**Issue 2:** `list_capabilities` reports 40 stubs in `not_registered_tools`, signaling to agents
that these tools exist but are unavailable — prompting unnecessary tool invocation attempts.

The correct fix is to gate the entire operations block behind a default-off `operations` Cargo
feature. When the feature is absent (the default), the stub methods do not exist, cannot be
registered, and cannot return `-32003`. The MCP catalog shows only 14 live tools.

---

## Narrative

As an LLM agent using the Prism MCP server, I want `tools/list` to show only the tools that
are actually callable, so that I can reason efficiently over the tool catalog without being
confused by 40 permanently-failing stubs that return `-32003`.

---

## Behavioral Contracts

| BC | Title | Version at Authoring | Scope in This Story |
|----|-------|---------------------|---------------------|
| BC-2.10.017 | Not-Yet-Available Tools Fast-Fail — Audit Channel Non-Blocking | v1.1 (AMENDMENT PENDING) | §Postconditions + §Invariants: operations-feature gate behavior; NOT_YET_AVAILABLE_TOOLS = &[] when feature absent; no stub tools in catalog when feature absent |
| BC-2.10.011 | list_capabilities Meta-Tool | v1.6 (AMENDMENT PENDING) | §Postconditions `not_registered_tools` field: empty slice when operations feature absent; previously-populated slice preserved when feature enabled |

BC-2.10.012 is a PROTECTION BOUNDARY (not an implemented contract): the story must not gate
any tool in LIVE_TOOLS, and specifically must not gate `prism_describe`, `list_capabilities`,
or `prism_query`. All 14 LIVE_TOOLS remain unconditionally registered per BC-2.10.012 §Preconditions.

---

## Acceptance Criteria

### AC-001 — `tools/list` returns exactly 14 tools when `operations` feature absent

When prism-mcp is compiled WITHOUT the `operations` feature (the default, as in production
binaries and all existing CI builds), a `tools/list` MCP request returns a response whose
`tools` array has exactly 14 entries. The 14 tools are those enumerated in the `LIVE_TOOLS`
const: `query`, `explain_query`, `create_alias`, `list_aliases`, `delete_alias`,
`explain_alias`, `confirm_action`, `reload_config`, `add_sensor_spec`, `list_sensor_specs`,
`validate_config`, `list_capabilities`, `prism_describe`, `check_sensor_health`.

Wire-shape assertion (SID-2): serialize the `tools/list` response to JSON and assert that
`result.tools.len() == 14` on the wire, not only on the pre-serialization Rust struct.

(traces to BC-2.10.017 amended §Postconditions: NOT_YET_AVAILABLE_TOOLS = &[] when
`operations` feature absent → catalog has only LIVE_TOOLS count 14)

### AC-002 — `list_capabilities.not_registered_tools` is empty when `operations` feature absent

When compiled without the `operations` feature, a `list_capabilities` call (with any valid
`client_id`) returns a JSON response where the `not_registered_tools` array is empty: `[]`.
No previously-stubbed operations tool names appear in `not_registered_tools`.

Wire-shape assertion (SID-2): assert on the serialized JSON `not_registered_tools: []`
field value, not only on the Rust slice length.

(traces to BC-2.10.011 amended §Postconditions: `not_registered_tools` binding
`NOT_YET_AVAILABLE_TOOLS` becomes &[] when `operations` feature absent)

### AC-003 — Invoking a previously-stubbed operations tool returns MCP `-32601`, NOT `-32003`, when `operations` feature absent

When compiled without the `operations` feature, calling `tools/call` with a previously-stubbed
operations tool name (e.g., `get_diagnostics`, `create_schedule`, or any of the 40 names that
were in `NOT_YET_AVAILABLE_TOOLS`) returns MCP error code `-32601` (MethodNotFound —
standard MCP "unknown tool" response). It MUST NOT return `-32003` (which signals "tool
exists but is not yet available").

The distinction is semantically significant: `-32003` tells the agent "this tool will work
eventually"; `-32601` tells the agent "this tool does not exist". The correct signal when the
`operations` feature is absent is `-32601` (the tool was never registered).

(traces to BC-2.10.017 amended §Invariants: when operations feature absent, stub tools are not
registered; an unregistered tool invocation returns the MCP protocol-level unknown-tool error,
not the prism-specific -32003 fast-fail)

### AC-004 — All 14 LIVE_TOOLS remain registered and non-gated regardless of `operations` feature state

The 14 tools in `LIVE_TOOLS` — including `prism_describe`, `list_capabilities`, `query`,
`check_sensor_health`, and the other 10 — are registered unconditionally. Adding
`#[cfg(feature = "operations")]` to stub methods MUST NOT inadvertently gate any LIVE_TOOLS
method. After the fix, `test_MCP_01_capability_classification_partitions_tool_catalog` must
continue to pass (all catalog tools classified; no phantom entries).

The existing inline `#[cfg(test)]` test `test_MCP_01_capability_classification_partitions_tool_catalog`
in `server.rs` serves as the regression gate for this invariant.

(traces to BC-2.10.017 amended §Invariants: `operations` gate applies ONLY to the operations
block; BC-2.10.012 §Preconditions: prism_describe always registered; BC-2.10.011 §Preconditions:
list_capabilities always registered)

### AC-005 — Existing `test_MCP_01_partition_positive_coverage` assertion for `get_diagnostics` is updated for new behavior

The existing inline test `test_MCP_01_partition_positive_coverage` currently asserts that
`get_diagnostics` returns error code `-32003`. After the fix, with the `operations` feature
absent, `get_diagnostics` is NOT registered and will return a different error. The test must
be updated (gated with `#[cfg(feature = "operations")]` or its assertions updated) so it
reflects the correct behavior in both worlds:
- `#[cfg(not(feature = "operations"))]`: calling `get_diagnostics` returns a non-`-32003` error (unknown tool)
- `#[cfg(feature = "operations")]`: calling `get_diagnostics` still returns `-32003` (fast-fail preserved)

This prevents the test from becoming a false negative (paper-fix) after the feature gate lands.

(traces to BC-2.10.017 amended §Postconditions: -32003 only applies when `operations` feature
is ENABLED; the test must verify the correct code in each compilation context)

---

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| `[features]` section addition | `crates/prism-mcp/Cargo.toml` | Pure (build metadata) |
| `NOT_YET_AVAILABLE_TOOLS` const — feature-gated | `crates/prism-mcp/src/server.rs` | Pure (compile-time const) |
| `LIVE_TOOLS` const — unchanged | `crates/prism-mcp/src/server.rs` | Pure (compile-time const — no modification) |
| All 40 operations stub handler methods | `crates/prism-mcp/src/server.rs` | Effectful (MCP tool handlers) |
| `not_yet_available_msg` helper | `crates/prism-mcp/src/server.rs` | Pure (error constructor — gated when feature absent) |
| `operations.rs` module | `crates/prism-mcp/src/tools/operations.rs` | Effectful (stub module — gated when feature absent) |
| `list_capabilities` handler `not_registered_tools` binding | `crates/prism-mcp/src/server.rs` | Effectful (reads NOT_YET_AVAILABLE_TOOLS const) |
| Test file (new) | `crates/prism-mcp/tests/bc_2_10_017_operations_feature_gate.rs` | Pure (test assertions) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `operations` feature absent + all 14 LIVE_TOOLS called in sequence | All 14 return non-error responses; none return -32003; tool catalog unchanged |
| EC-002 | `operations` feature enabled (explicit opt-in) | NOT_YET_AVAILABLE_TOOLS = full 40-name slice; all 40 stubs return -32003; behavior identical to current pre-fix state |
| EC-003 | `list_capabilities(null)` (cross-client summary mode) with `operations` feature absent | Response carries `not_registered_tools: []` in both single-client and cross-client modes |
| EC-004 | `test_MCP_01_capability_classification_partitions_tool_catalog` after feature gate lands | Test continues to pass because `production_tool_catalog()` drops gated methods and NOT_YET_AVAILABLE_TOOLS = &[]; catalog = LIVE_TOOLS; union = LIVE_TOOLS; no phantom, no unclassified |
| EC-005 | rmcp `#[tool_router]` proc-macro + `#[cfg(feature = "operations")]` on a method | Gated method is absent from `production_tool_catalog()` when feature absent (verified by Spike T-S01 before RG tests are written) |

---

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story file | ~4,500 | |
| BC-2.10.017 v1.1 (amended — full text) | ~5,000 | Primary AC source after amendment |
| BC-2.10.011 v1.6 (amended — full text) | ~15,000 | not_registered_tools semantics |
| BC-2.10.012 v1.9 (protection boundary check) | ~20,000 | Read §Preconditions only; large file |
| `server.rs` (LIVE_TOOLS/NOT_YET_AVAILABLE_TOOLS + all 40 stub handlers) | ~40,000 | Large file; implementer reads entire file |
| `Cargo.toml` (prism-mcp) | ~1,000 | Small; add [features] section |
| `tools/operations.rs` (stub module) | ~3,000 | Gate with #[cfg(feature = "operations")] |
| New test file `bc_2_10_017_operations_feature_gate.rs` | ~3,000 | RG-GATE-001..003 |
| beta3-remediation-delta-analysis.md §Issue 1 + §S-MCP-TOOL-GATE-001 | ~3,000 | Reference |
| **Total estimated** | **~94,500** | Well within one context window |

---

## Tasks

### Spike task (BEFORE writing any Red Gate tests)

- [ ] **T-S01** — rmcp `#[cfg]` compile-spike: verify that `rmcp`'s `#[tool_router]`
  proc-macro and method-dispatch machinery correctly DROPS methods when
  `#[cfg(feature = "operations")]` is absent. Write a minimal test binary in
  `crates/prism-mcp/tests/` (or a small inline test in server.rs `#[cfg(test)] mod tests`)
  that:
  1. Builds the test WITHOUT the `operations` feature (the default)
  2. Calls `PrismServer::production_tool_catalog()` and asserts `len() == 14`
  3. Run: `cargo test -p prism-mcp test_spike_tool_catalog_count_without_operations_feature`

  If the proc-macro does NOT correctly drop gated methods, stop and escalate to architect —
  the implementation design may need an alternative gating mechanism. Do NOT proceed to
  the Red Gate tests until the spike passes.

### Red Gate tests (to be written by test-writer BEFORE implementation)

All tests live in `crates/prism-mcp/tests/bc_2_10_017_operations_feature_gate.rs` (new file)
unless noted otherwise. All non-trivial function bodies use `todo!()` stubs until the
Green Gate phase.

- [ ] **RG-GATE-001**: `test_tools_list_returns_14_tools_without_operations_feature`
  Assert: `PrismServer::production_tool_catalog().len() == 14` when compiled without
  `operations` feature. Wire-shape assertion: serialize `tools/list` response and assert
  `result.tools.len() == 14` in the JSON. Currently FAILS (returns 54).
  AC-001.

- [ ] **RG-GATE-002**: `test_list_capabilities_not_registered_tools_empty_without_operations_feature`
  Assert: Calling `list_capabilities(client_id: "test-client")` on a PrismServer returns a
  response JSON where `not_registered_tools == []` (empty array). Wire-shape (SID-2): assert
  on serialized JSON bytes. Currently FAILS (returns 40-element array).
  AC-002.

- [ ] **RG-GATE-003**: `test_ops_tool_invocation_returns_method_not_found_without_operations_feature`
  Assert: Calling `tools/call` with `method = "get_diagnostics"` returns an MCP error response
  with code != -32003. The specific expected behavior is "unknown tool" (rmcp returns -32601 or
  equivalent for an unregistered tool). Currently FAILS (returns -32003 from not_yet_available_msg).
  AC-003.

- [ ] **RG-GATE-004**: `test_live_tools_all_present_without_operations_feature`
  Assert: `NOT_YET_AVAILABLE_TOOLS.len() == 0` (the const is the empty slice) AND all 14 names
  in `LIVE_TOOLS` appear in `PrismServer::production_tool_catalog()`. This is a compile-time
  safety check as a test. Currently FAILS (NOT_YET_AVAILABLE_TOOLS.len() == 40).
  AC-004 regression guard.

**Red Gate density check** (BC-5.38.001): **4 failing tests** (RG-GATE-001..004) before
implementation begins. The story has 5 ACs; AC-005 is an existing-test update task, not a
new failing test. Density: 4 RG tests / 5 ACs = 0.80 — satisfies the ≥ 0.5 threshold.
All non-trivial function bodies use `todo!()` stubs; `production_tool_catalog()` returns 54
entries; tests fail on count assertions.

### Implementation tasks (to be executed by implementer after Red Gate)

#### Phase A — Cargo feature gate

- [ ] **T-A01**: Add `[features]` section to `crates/prism-mcp/Cargo.toml`:
  ```toml
  [features]
  # Gate all prism-operations stub tool handlers.
  # Default-off: production binaries do not expose stub tools in tools/list.
  # Enable with: cargo build -p prism-mcp --features operations
  operations = []
  ```
  Verify no `default = ["operations"]` entry. The feature is intentionally default-off.

#### Phase B — Gate NOT_YET_AVAILABLE_TOOLS const

- [ ] **T-B01**: In `crates/prism-mcp/src/server.rs`, replace the single
  `NOT_YET_AVAILABLE_TOOLS` const with two `#[cfg]`-gated variants:
  ```rust
  /// When the `operations` feature is enabled, stub tool names are registered in the
  /// MCP catalog and return -32003. When the feature is absent (default), this slice
  /// is empty — stub tools are not registered and not visible in tools/list.
  /// BC-2.10.017 §Postconditions (amended: beta.3 remediation).
  #[cfg(feature = "operations")]
  const NOT_YET_AVAILABLE_TOOLS: &[&str] = &[
      "get_diagnostics",
      "create_schedule",
      // ... all 40 names ...
  ];
  #[cfg(not(feature = "operations"))]
  const NOT_YET_AVAILABLE_TOOLS: &[&str] = &[];
  ```
  Verify the existing comment "must appear in exactly one of LIVE_TOOLS / NOT_YET_AVAILABLE_TOOLS"
  above the consts is updated to reflect that NOT_YET_AVAILABLE_TOOLS may be empty.

#### Phase C — Gate stub handler methods and operations module

- [ ] **T-C01**: Add `#[cfg(feature = "operations")]` to the `impl PrismServer` block
  containing all 40 operations stub handlers (the block beginning at the `// ─── Operations
  tools (NotImplemented — prism-operations not merged) ───` comment in `server.rs`).
  Verify the `not_yet_available_msg` helper function is also gated or becomes dead-code-free.
  If `not_yet_available_msg` is used by other non-operations handlers, gate ONLY the
  operations handler block; leave the helper ungated.

- [ ] **T-C02**: In `crates/prism-mcp/src/tools/operations.rs`, add `#[cfg(feature = "operations")]`
  at the top of the file (or to the module declaration in `mod.rs` / `lib.rs` where it is
  declared). Verify the crate still compiles cleanly after gating.

#### Phase D — Update existing tests that assert on old behavior

- [ ] **T-D01**: Update `test_MCP_01_partition_positive_coverage` (inline `#[cfg(test)] mod tests`
  in `server.rs`):
  - Wrap the `get_diagnostics` → -32003 assertion in `#[cfg(feature = "operations")]`
  - Add a `#[cfg(not(feature = "operations"))]` arm that asserts calling `get_diagnostics`
    does NOT return -32003 (asserts the error code is not -32003 or the call returns
    "unknown tool")
  - This test update enforces AC-005.

- [ ] **T-D02**: Verify `test_MCP_01_capability_classification_partitions_tool_catalog` still
  passes without changes. It should pass because `production_tool_catalog()` drops gated
  methods and `NOT_YET_AVAILABLE_TOOLS = &[]`, so union(LIVE_TOOLS, &[]) == catalog.
  If it fails, investigate and fix.

#### Phase E — Final verification

- [ ] **T-E01**: Run `cargo test -p prism-mcp --no-fail-fast` (without `operations` feature,
  the default). All tests must pass. RG-GATE-001..004 must be GREEN.
- [ ] **T-E02**: Run `cargo test -p prism-mcp --features operations --no-fail-fast`. The 40
  stub handlers exist again; `test_MCP_01_partition_positive_coverage`'s -32003 assertion fires.
  All tests must pass.
- [ ] **T-E03**: Run `just check` (full workspace). Must exit 0.

#### Phase F — CHANGELOG

- [ ] **T-F01** (BEFORE creating the PR): Add a CHANGELOG entry under `[Unreleased] > Fixed`:
  ```markdown
  - Gate 40 prism-operations stub tool handlers behind default-off `operations` Cargo
    feature; `tools/list` now returns 14 live tools in production (was 54). Eliminates
    `-32003` catalog noise that confused LLM agents into attempting unimplemented tool calls.
    `list_capabilities.not_registered_tools` is empty by default (was 40 entries).
    Resolves beta.2 live-test issues 1 and 2.
  ```

---

## Previous Story Intelligence

N/A — first story in beta.3 Wave 1. No predecessor stories in this epic have shipped.

**Context from the beta.2 live-test log (D-2520):**
- Issue 1 was identified at the Monroe demo against a real Claroty xDome tenant (jea-readapi).
- 40 of 54 registered tools returned -32003 at runtime.
- The LLM agent was confused by the large stub-polluted tool catalog.
- This is the first (and lowest-risk) story in the nine-story beta.3 remediation batch.

---

## Architecture Compliance Rules

1. **Never gate LIVE_TOOLS methods.** The 14 tools in `LIVE_TOOLS` — `prism_describe`,
   `list_capabilities`, `query`, `check_sensor_health`, and the other 10 — MUST remain
   unconditionally registered. `#[cfg(feature = "operations")]` applies ONLY to the
   operations impl block and the `NOT_YET_AVAILABLE_TOOLS` const. Gating any LIVE_TOOLS
   method is a P0 defect. Verified by AC-004 / RG-GATE-004 and the existing
   `test_MCP_01_capability_classification_partitions_tool_catalog` partition test.

2. **Spike before Red Gate.** Per delta-analysis §Issue 1 rmcp spike requirement and
   Spike Task T-S01: do NOT write RG tests until T-S01 confirms that `#[cfg]` method
   gating is correctly handled by the `#[tool_router]` proc-macro. If the proc-macro
   requires a different mechanism (e.g., conditional impl block, conditional trait impl),
   the story design must be updated before testing begins.

3. **Three sync points after any router change (BC-2.10.017 invariant).**
   Whenever `build_tool_router` registration is changed, all three must stay in sync:
   (a) `build_tool_router` registration list
   (b) `LIVE_TOOLS` / `NOT_YET_AVAILABLE_TOOLS` consts
   (c) `list_capabilities.not_registered_tools` binding
   The `#[cfg]` gate on `NOT_YET_AVAILABLE_TOOLS` satisfies (b) + (c) automatically;
   the gated method block satisfies (a) automatically via the `#[tool_router]` proc-macro.

4. **No `--no-verify` hook bypass.** Pre-commit hooks must pass on every commit.
   If a hook fails, fix the root cause; do not bypass.

5. **No inline line-number cites in tests.** Per TD-VSDD-091: test comments must
   reference function names and BC section anchors, not `server.rs:NNN` line numbers.

---

## Library & Framework Requirements

All versions pinned in workspace `Cargo.toml` — use workspace pins, not standalone versions.

| Dependency | Version | Note |
|-----------|---------|------|
| `rmcp` | workspace pin | `#[tool_router]` proc-macro must support `#[cfg]`-gated methods (verify via T-S01 spike) |
| `schemars` | workspace pin | No change required |
| Rust toolchain | per `rust-toolchain.toml` | Stable channel; edition 2024 |

No new dependencies are introduced by this story.

---

## File Structure Requirements

### Files to CREATE

| File | Purpose |
|------|---------|
| `crates/prism-mcp/tests/bc_2_10_017_operations_feature_gate.rs` | Red Gate tests RG-GATE-001..004 (new) |

### Files to MODIFY

| File | Change |
|------|--------|
| `crates/prism-mcp/Cargo.toml` | Add `[features]` section with `operations = []` |
| `crates/prism-mcp/src/server.rs` | Gate `NOT_YET_AVAILABLE_TOOLS` const (two `#[cfg]` variants); add `#[cfg(feature = "operations")]` to the operations impl block (~40 stub handler methods); update `test_MCP_01_partition_positive_coverage` (T-D01); update `not_yet_available_msg` placement if needed |
| `crates/prism-mcp/src/tools/operations.rs` | Add `#[cfg(feature = "operations")]` to module |
| `CHANGELOG.md` | Add [Unreleased] > Fixed entry (T-F01) |

### Files NOT to touch

- `crates/prism-mcp/src/tools/prism_describe.rs` — out of scope (Wave 2 story)
- `crates/prism-mcp/src/safety_envelope.rs` — out of scope (Wave 2 story)
- Any file outside `crates/prism-mcp/` — no cross-crate changes for this story

---

## Holdout Authoring Note

`behavioral_contracts: [BC-2.10.017, BC-2.10.011]` is non-empty. Per the story-level holdout
gate protocol (D-1715/D-1716, human-approved 2026-07-13), the product-owner must author 2–4
HIDDEN, SINGLE-USE holdout scenarios for this story at story-materialization time (the same
touchpoint as the remove-uncertainty pass). Holdout scenarios should exercise:
- The `tools/list` count gate (expected: 14, not 54)
- The `list_capabilities.not_registered_tools` empty-slice assertion at the wire level
- A previously-stubbed tool invocation returning the correct error code

Holdout scenarios are stored in the holdout directory that test-writer/implementer never read.
