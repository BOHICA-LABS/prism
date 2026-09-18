---
document_type: story
story_id: S-MCP-TOOL-GATE-001
title: "Gate 40 operations stubs behind default-off Cargo feature to eliminate -32003 catalog pollution"
level: "L4"
wave: 1
epic_id: E-BETA3-REMEDIATION
version: "1.9"
status: ready
producer: story-writer
timestamp: "2026-09-18T00:00:00Z"
phase: 3
cycle: wave-5-e-demo-fidelity
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
inputs:
  - .factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md
  - .factory/specs/behavioral-contracts/BC-2.10.017-not-yet-available-tools-fast-fail-audit-channel-non-blocking.md
  - .factory/specs/behavioral-contracts/BC-2.10.011-list-capabilities-meta-tool.md
  - crates/prism-mcp/src/server.rs
input-hash: "4ea5f2a"
traces_to: .factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md
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
#   no new public types; NOT_YET_AVAILABLE_TOOLS is compile-time const only. The gating
#   mechanism (two independent `#[tool_router]` blocks + combiner fn per T-C01) is
#   ratified per architect decision D-1110; T-S01 confirms the approach compiles cleanly.
behavioral_contracts:
  - BC-2.10.017
  - BC-2.10.011
# BC status: AMENDMENTS ACTIVE (D-2543).
#   BC-2.10.017 v1.3: operations-feature gate postconditions and invariants now active.
#     - NOT_YET_AVAILABLE_TOOLS = &[] when `operations` feature absent
#     - stub tools NOT registered in tools/list when feature absent
#     - -32003 fast-fail only fires when `operations` feature is enabled
#   BC-2.10.011 v1.7: not_registered_tools empty-slice semantics when operations feature absent now active.
#   Spec-First Gate S-7.01 satisfied; story is unblocked for test-writer dispatch.
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

**BC-2.10.017** (Not-Yet-Available Tools Fast-Fail — Audit Channel Non-Blocking) governs the NOT_YET_AVAILABLE_TOOLS
const and the fast-fail handler behavior. This story implements an amendment to that BC.
Path: `.factory/specs/behavioral-contracts/BC-2.10.017-not-yet-available-tools-fast-fail-audit-channel-non-blocking.md`

**BC-2.10.011** (list_capabilities Meta-Tool) governs the `not_registered_tools` field in
the `list_capabilities` response. This story implements an amendment to that BC.
Path: `.factory/specs/behavioral-contracts/BC-2.10.011-list-capabilities-meta-tool.md`

**BC-2.10.012** (`prism_describe` Schema Discovery Tool (L2)) §Preconditions §1 is a protection
boundary: "`prism_describe` is always registered — it is NOT gated by any feature flag or
capability check." The implementer MUST read this BC to confirm all 14 LIVE_TOOLS remain
ungated after this story lands.
Path: `.factory/specs/behavioral-contracts/BC-2.10.012-prism-describe-schema-discovery-tool.md`

> NOTE: Amendments to BC-2.10.017 and BC-2.10.011 are now active per D-2543.
> Spec-First Gate S-7.01 is satisfied; story is unblocked for test-writer dispatch.

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
| BC-2.10.017 | Not-Yet-Available Tools Fast-Fail — Audit Channel Non-Blocking | v1.3 | §Postconditions + §Invariants: operations-feature gate behavior; NOT_YET_AVAILABLE_TOOLS = &[] when feature absent; no stub tools in catalog when feature absent |
| BC-2.10.011 | list_capabilities Meta-Tool | v1.7 | §Postconditions `not_registered_tools` field: empty slice when operations feature absent; previously-populated slice preserved when feature enabled |

> **Version at Authoring note (POL-39 exemption):** The "Version at Authoring" column records
> the frozen BC versions this story was authored against (point-in-time snapshot); it is NOT
> a current-state pin and is POL-39-exempt under the same rationale as §History/§Changelog and
> the TD-VSDD-091 AC-source-of-truth-table exemption. Current BC versions are tracked in BC-INDEX.

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

### AC-003 — Invoking a previously-stubbed operations tool returns MCP `-32602`, NOT `-32003`, when `operations` feature absent

When compiled without the `operations` feature, calling `tools/call` with a previously-stubbed
operations tool name (e.g., `get_diagnostics`, `create_schedule`, or any of the 40 names that
were in `NOT_YET_AVAILABLE_TOOLS`) returns MCP error code `-32602` (InvalidParams, message
`tool not found`). It MUST NOT return `-32003` (which signals "tool exists but is not yet
available").

The distinction is semantically significant: `-32003` tells the agent "this tool will work
eventually"; `-32602` (InvalidParams, `tool not found`) tells the agent "this tool does not
exist". The correct signal when the `operations` feature is absent is `-32602` (the tool was
never registered; rmcp 1.7.0 returns `-32602` with message `"tool not found"` for unregistered-tool
invocations — confirmed in rmcp source and `error_mapping.rs`).

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
`get_diagnostics` returns error code `-32003`. After the fix, the test must be split across
two `#[cfg]` arms so it reflects the correct behavior in both compilation contexts:

- `#[cfg(not(feature = "operations"))]`: In the operations-ABSENT build, `get_diagnostics` is
  NOT a registered tool. The inline arm asserts its ABSENCE from `production_tool_catalog()`
  — i.e., the tool name does NOT appear in the catalog slice. Calling `get_diagnostics` as a
  method is NOT a valid assertion in this arm: in the operations-absent build the ops `impl`
  block is gated out and the method does not exist on `PrismServer` (would fail to compile
  with E0599). The `-32602` (`InvalidParams`, message `"tool not found"`) WIRE behavior — the
  MCP protocol error an LLM agent receives when invoking this tool name against an
  operations-absent server — is verified end-to-end by RG-GATE-003
  (`test_BC_2_10_017_ops_tool_invocation_returns_invalid_params_without_operations_feature`),
  not by this inline arm.
- `#[cfg(feature = "operations")]`: Calling `get_diagnostics` still returns `-32003`
  (fast-fail preserved; unchanged from current behavior).

This prevents the test from becoming a false negative (paper-fix) after the feature gate lands.

(traces to BC-2.10.017 amended §Postconditions: -32003 only applies when `operations` feature
is ENABLED; operations-absent inline arm asserts catalog-ABSENCE of get_diagnostics; wire
-32602 behavior discharged by RG-GATE-003)

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

## Purity Classification

Derived from the Architecture Mapping table above.

| Module | Classification | Justification |
|--------|---------------|---------------|
| `[features]` declaration in `Cargo.toml` | pure-core | Cargo build metadata; no runtime code; no I/O, no global state. |
| `NOT_YET_AVAILABLE_TOOLS` const (feature-gated) | pure-core | Compile-time `&[&str]` constant; value resolved at compile time via `#[cfg(feature = "operations")]`; no runtime state, no side effects. |
| `not_yet_available_msg` helper | pure-core | Error constructor returning `CallToolError` from a `&str` key; deterministic, no I/O, no global state. Gated under `#[cfg(feature = "operations")]`. |
| 40 operations stub `#[tool]` handler methods | effectful-IO | MCP tool handlers gated under `#[cfg(feature = "operations")] #[tool_router(router = operations_tool_router)]`; return async `CallToolResult` via rmcp tool router (network I/O path). |
| `operations.rs` module | effectful-IO | Stub module gated under `#[cfg(feature = "operations")]`; async handler stubs returning -32003 fast-fail responses traverse the rmcp network dispatch path. |
| `list_capabilities` handler `not_registered_tools` binding | effectful-IO | MCP tool handler; reads `NOT_YET_AVAILABLE_TOOLS` const and serializes result to JSON wire response (BC-2.10.011 §Postconditions). |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `operations` feature absent + all 14 LIVE_TOOLS called in sequence | All 14 return non-error responses; none return -32003; tool catalog unchanged |
| EC-002 | `operations` feature enabled (explicit opt-in) | NOT_YET_AVAILABLE_TOOLS = full 40-name slice; all 40 stubs return -32003; behavior identical to current pre-fix state |
| EC-003 | `list_capabilities(null)` (cross-client summary mode) with `operations` feature absent | Response carries `not_registered_tools: []` in both single-client and cross-client modes |
| EC-004 | `test_MCP_01_capability_classification_partitions_tool_catalog` after feature gate lands | Test continues to pass because `production_tool_catalog()` drops gated methods and NOT_YET_AVAILABLE_TOOLS = &[]; catalog = LIVE_TOOLS; union = LIVE_TOOLS; no phantom, no unclassified |
| EC-005 | rmcp two-router-block + combiner under `#[cfg(feature = "operations")]` | Gated ops block absent from `production_tool_catalog()` when feature absent; confirmed by T-S01 compile-check using the ratified two-router-block + combiner approach (T-C01(a)-(d)) |

---

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story file | ~4,500 | |
| BC-2.10.017 (active) | ~5,000 | Primary AC source |
| BC-2.10.011 (active) | ~15,000 | not_registered_tools semantics |
| BC-2.10.012 (protection boundary check) | ~20,000 | Read §Preconditions only; large file |
| `server.rs` (LIVE_TOOLS/NOT_YET_AVAILABLE_TOOLS + all 40 stub handlers) | ~40,000 | Large file; implementer reads entire file |
| `Cargo.toml` (prism-mcp) | ~1,000 | Small; add [features] section |
| `tools/operations.rs` (stub module) | ~3,000 | Gate with #[cfg(feature = "operations")] |
| New test file `bc_2_10_017_operations_feature_gate.rs` | ~3,000 | RG-GATE-001..004 (unit/catalog assertions) + OBS-1 e2e `test_BC_2_10_017_list_capabilities_not_registered_tools_empty_via_end_to_end_client_roundtrip` (AC-002) + OBS-2 e2e `test_BC_2_10_017_tools_list_14_via_end_to_end_client_roundtrip` (AC-001); 6 tests total |
| beta3-remediation-delta-analysis.md §Issue 1 + §S-MCP-TOOL-GATE-001 | ~3,000 | Reference |
| **Total estimated** | **~94,500** | Well within one context window |

---

## Tasks

### Spike task (BEFORE writing any Red Gate tests)

- [ ] **T-S01** — rmcp two-router-block compile-confirmation: verify that the ratified
  gating mechanism (T-C01: two separate `#[tool_router]` impl blocks + combiner `fn
  tool_router()`) compiles cleanly for BOTH the default (no-operations) and
  `--features operations` configurations. This is a **confirmation step, not an open
  question** — the architect decision (D-1110) has already ratified the two-router-block +
  combiner approach as the correct rmcp 1.7.0 pattern (per-method `#[cfg]` inside a single
  `#[tool_router]` block does NOT compile; rmcp-macros 1.7.0 emits unguarded `.with_route(...)`
  → E0599).
  1. After T-C01 implementation: run `cargo build -p prism-mcp` (no features — default)
  2. Run `cargo build -p prism-mcp --features operations`
  3. Run `cargo nextest run -p prism-mcp -E 'test(test_BC_2_10_017_tools_list_returns_14_tools_without_operations_feature)'`
     (asserts `PrismServer::production_tool_catalog().len() == 14`; this is RG-GATE-001 — the spike-only placeholder name was superseded by the shipped RG-GATE function name)

  If either build fails, stop and report to orchestrator. Do NOT proceed to the Red Gate
  tests until both builds pass cleanly (zero E0599 errors).

### Red Gate tests (to be written by test-writer BEFORE implementation)

All tests live in `crates/prism-mcp/tests/bc_2_10_017_operations_feature_gate.rs` (new file)
unless noted otherwise. All non-trivial function bodies use `todo!()` stubs until the
Green Gate phase.

- [ ] **RG-GATE-001**: `test_BC_2_10_017_tools_list_returns_14_tools_without_operations_feature`
  Assert: `PrismServer::production_tool_catalog().len() == 14` when compiled without
  `operations` feature. Wire-shape assertion: serialize `tools/list` response and assert
  `result.tools.len() == 14` in the JSON. Currently FAILS (returns 54).
  AC-001.

- [ ] **RG-GATE-002**: `test_BC_2_10_017_list_capabilities_not_registered_tools_empty_without_operations_feature`
  Assert: Calling `list_capabilities(client_id: "test-client")` on a PrismServer returns a
  response JSON where `not_registered_tools == []` (empty array). Wire-shape (SID-2): assert
  on serialized JSON bytes. Currently FAILS (returns 40-element array).
  AC-002.

- [ ] **RG-GATE-003**: `test_BC_2_10_017_ops_tool_invocation_returns_invalid_params_without_operations_feature`
  Assert: Calling `tools/call` with `method = "get_diagnostics"` returns an MCP error response
  where `err.code == -32602` (InvalidParams, message `"tool not found"`). MUST NOT return
  `-32003` (fast-fail) and MUST NOT return `-32601`. Wire-shape: assert the exact code and
  message on the serialized response (SID-2). Currently FAILS (returns -32003 from not_yet_available_msg).
  AC-003.

- [ ] **RG-GATE-004**: `test_BC_2_10_017_live_tools_all_present_without_operations_feature`
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

- [ ] **T-C01**: Ratified gating mechanism — two-router-block + combiner. All changes are
  within `crates/prism-mcp/src/server.rs`; no new files required:
  - **(a)** Rename the existing `#[tool_router]` on the `impl PrismServer` block that contains
    all 14 LIVE_TOOLS `#[tool]` handlers to `#[tool_router(router = live_tool_router)]`.
    Keep all 14 LIVE_TOOLS `#[tool]` methods in this block unchanged. This generates
    `PrismServer::live_tool_router()`.
  - **(b)** Create a new `impl PrismServer` block containing all 40 ops `#[tool]` methods,
    annotated: `#[cfg(feature = "operations")] #[tool_router(router = operations_tool_router)]`.
    Move all 40 ops `#[tool]` methods into it. This generates `operations_tool_router()` only
    when the `operations` feature is enabled.
  - **(c)** Add a plain (non-macro) combiner in a separate `impl PrismServer` block:
    ```rust
    fn tool_router() -> ToolRouter<Self> {
        let base = Self::live_tool_router();
        #[cfg(feature = "operations")]
        let base = base + Self::operations_tool_router();
        base
    }
    ```
    This combiner is used by `#[tool_handler]` and `production_tool_catalog()`.
    `ToolRouter: Add` merge is the rmcp 1.7 composition API.
  - **(d)** Relocate the `not_yet_available_msg` helper into the ops `impl` block OR add
    `#[cfg(feature = "operations")]` to it to avoid a dead-code warning when the feature is
    absent.
  Verify: `cargo build -p prism-mcp` (default, no features) AND
  `cargo build -p prism-mcp --features operations` both compile with zero E0599 errors.

- [ ] **T-C02**: In `crates/prism-mcp/src/tools/operations.rs`, add `#[cfg(feature = "operations")]`
  at the top of the file (or to the module declaration in `mod.rs` / `lib.rs` where it is
  declared). Verify the crate still compiles cleanly after gating.

#### Phase D — Update existing tests that assert on old behavior

- [ ] **T-D01**: Update `test_MCP_01_partition_positive_coverage` (inline `#[cfg(test)] mod tests`
  in `server.rs`):
  - Wrap the `get_diagnostics` → `-32003` assertion in `#[cfg(feature = "operations")]` so
    the fast-fail assertion is preserved when the feature is enabled (no behavior change for
    the operations-enabled build).
  - Add a `#[cfg(not(feature = "operations"))]` arm that asserts `get_diagnostics` is ABSENT
    from `production_tool_catalog()` — i.e., the tool name does NOT appear in the catalog
    slice (e.g., `assert!(!production_tool_catalog().iter().any(|t| t.name == "get_diagnostics"))`).
    Do NOT attempt to call `get_diagnostics` as a method in this arm: in the operations-absent
    build the ops `impl` block is compiled out and the method does not exist on `PrismServer`;
    calling it would fail to compile with E0599. The `-32602` wire assertion (the MCP
    `InvalidParams` / `"tool not found"` error returned to an LLM agent that invokes this tool
    name) is discharged end-to-end by RG-GATE-003
    (`test_BC_2_10_017_ops_tool_invocation_returns_invalid_params_without_operations_feature`).
  - This test update enforces AC-005.

- [ ] **T-D02**: Verify `test_MCP_01_capability_classification_partitions_tool_catalog` still
  passes without changes. It should pass because `production_tool_catalog()` drops gated
  methods and `NOT_YET_AVAILABLE_TOOLS = &[]`, so union(LIVE_TOOLS, &[]) == catalog.
  If it fails, investigate and fix.

- [ ] **T-D03**: Gate with `#[cfg(feature = "operations")]` EVERY inline `#[cfg(test)]` test
  in `crates/prism-mcp/src/server.rs` that invokes an ops `#[tool]` handler method moved
  into the `#[cfg(feature = "operations")]` impl block by T-C01. The complete set is defined
  by the general rule: **all inline tests whose bodies call a gated ops handler** — this is
  approximately 20 such tests in the feature HEAD (the exact verified count in the shipped
  worktree is 21 ops-handler-invoking inline test functions). This includes
  `test_operations_tools_return_not_implemented_error_code`,
  `test_not_yet_available_msg_uses_not_implemented_code`, all
  delete_rule/get_case/update_case/create_pack/create_action/fire_action length-bound tests
  (`test_F_PR163_PASS2_IMP_2_*` and `test_F_PR163_PASS3_MED_1_*` for ops handlers), and any
  other inline test whose body invokes a method on an ops handler that no longer exists in the
  operations-absent build. **Do NOT treat this as a fixed short list** — enumerate by
  inspection: for each inline test function, if its body calls a method that lives in the
  `#[cfg(feature = "operations")]` impl block, that test must be gated. Rationale: without the
  gate on each such test, the default (no-`operations`) build fails with E0599 — the handler
  methods it calls do not exist in that compilation context. The assertions each test makes
  (ops tools return -32003 or enforce length bounds) are only meaningful when the `operations`
  feature is enabled; there is no corresponding behavior to test when the feature is absent.

- [ ] **T-D04**: Gate the three integration tests in `crates/prism-mcp/tests/mcp_infrastructure.rs`
  — `test_bc_2_10_017_not_yet_available_fast_fail_under_1s`,
  `test_bc_2_10_017_not_yet_available_guard_precedes_audit`, and
  `test_bc_2_10_017_sibling_handlers_guard_precedes_audit` — by adding
  `#[cfg(feature = "operations")]` as a function-level attribute on each `#[tokio::test]`
  function. Rationale: all three tests drive the -32003 fast-fail path or guard ordering
  through gated ops handlers (they call ops handler methods moved into the
  `#[cfg(feature = "operations")]` impl block by T-C01). All three will fail to compile in
  the default (no-`operations`) build after T-C01 gates those handlers. The behaviors they
  assert — fast-fail under 1s; guard precedes audit; sibling-handler guard ordering —
  exist only when the `operations` feature is enabled; the tests have no valid target when
  the feature is absent.

> **Phase D compile-safety constraint:** Do NOT add `#[cfg(feature = "operations")]` to the
> parameter structs `ListInfusionsParams`, `InfusionStatusParams`, or `PluginStatusParams`
> (defined in `server.rs`). These are `pub struct` definitions outside any impl block; gating
> them would break other references (e.g., uses in non-ops handler code, derive macros, or
> downstream crates). The general gating rule: gate `#[cfg(feature = "operations")]` onto each
> of the following — (a) the ops `impl` block containing `#[tool]` handler methods (T-C01);
> (b) the `NOT_YET_AVAILABLE_TOOLS` const (T-B01); (c) the `not_yet_available_msg` helper
> (T-C01(d)); (d) the `operations.rs` module declaration (T-C02); (e) every inline
> `#[cfg(test)]` test in server.rs that invokes a gated ops handler method (~20 inline tests;
> enumerate by inspection, not by a fixed short list — T-D03); (f) the 3 tests in
> `mcp_infrastructure.rs` that drive the -32003 fast-fail path through gated ops handlers
> (T-D04). All `pub struct` definitions, LIVE_TOOLS handler methods, and all non-ops-handler
> code remain ungated.

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

2. **Compile-confirm before Red Gate.** The gating mechanism is ratified (T-C01: two
   independent `#[tool_router]` blocks + combiner `fn tool_router()`; D-1110 architect
   decision). T-S01 is a compile-confirmation step: run both the default build and the
   `--features operations` build before writing RG tests. If either build fails with E0599
   or other errors, stop and report to orchestrator. The per-method `#[cfg]`-inside-one-
   `#[tool_router]`-block approach is NOT the ratified design and MUST NOT be used.

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
| `rmcp` | workspace pin | Uses two-router-block + combiner pattern (T-C01); per-method `#[cfg]` inside a single `#[tool_router]` block is NOT supported (emits E0599). T-S01 confirms both builds compile cleanly. |
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
| `crates/prism-mcp/src/server.rs` | Gate `NOT_YET_AVAILABLE_TOOLS` const (two `#[cfg]` variants); rename existing `#[tool_router]` → `#[tool_router(router = live_tool_router)]`; create new `#[cfg(feature = "operations")] #[tool_router(router = operations_tool_router)]` impl block with all 40 ops `#[tool]` methods; add plain combiner `fn tool_router()`; relocate/gate `not_yet_available_msg`; update `test_MCP_01_partition_positive_coverage` (T-D01); gate `test_operations_tools_return_not_implemented_error_code` (T-D03) |
| `crates/prism-mcp/src/tools/mod.rs` | Add `#[cfg(feature = "operations")]` to the `pub mod operations;` module declaration (T-C02; feature gate applied at the module declaration, not inside `operations.rs` directly) |
| `crates/prism-mcp/tests/mcp_infrastructure.rs` | Gate `test_bc_2_10_017_not_yet_available_fast_fail_under_1s`, `test_bc_2_10_017_not_yet_available_guard_precedes_audit`, and `test_bc_2_10_017_sibling_handlers_guard_precedes_audit` with `#[cfg(feature = "operations")]` (T-D04) |
| `CHANGELOG.md` | Add [Unreleased] > Fixed entry (T-F01) |
| `Justfile` | Add operations-OFF `cargo nextest run -p prism-mcp` leg to `check` and `check-ci` so the default-feature (operations-absent) RG-GATE tests execute in the canonical local gate (the existing `--all-features` legs compile OUT the `#![cfg(not(feature="operations"))]` RG-GATE test file, so these default-feature legs are the ONLY way those tests run) |

### Files NOT to touch

- `crates/prism-mcp/src/tools/prism_describe.rs` — out of scope (Wave 2 story)
- `crates/prism-mcp/src/safety_envelope.rs` — out of scope (Wave 2 story)
- Any file outside `crates/prism-mcp/` **except** the following repo-root files which are EXPECTED modifications: `Justfile` (operations-OFF default-feature gate legs in `check` + `check-ci`) and `CHANGELOG.md` (release notes entry T-F01). All other crates' `src/` directories, `prism-query`, `prism-core`, `prism-sensors`, etc. are out of scope (no cross-crate src changes).

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

---

## History

| Version | Date | Change |
|---------|------|--------|
| 1.9 | 2026-09-18 | LOCAL pass-7 docs-only fix-burst (no code/behavior change; feature HEAD 0af76be5c frozen). F-MED-001 (MEDIUM): T-D03 rewritten to state the GENERAL RULE — gate ALL ~20 ops-handler-invoking inline tests, not only the one named test; Phase-D constraint note corrected to remove false "three tests" miscount and false "Everything else remains ungated" closed-set claim; correct form is "enumerate by inspection, not a fixed short list." OBS-1 (POL-7 H1-verbatim): BC-2.10.012 §Authority citation restored to verbatim H1 `` `prism_describe` Schema Discovery Tool (L2) `` (was missing backticks + "(L2)"); BC-2.10.017 §Authority citation restored to full H1 "Not-Yet-Available Tools Fast-Fail — Audit Channel Non-Blocking" (was truncated). OBS-2 (POL-39 orchestrator adjudication): footnote added under §Behavioral Contracts table stating the "Version at Authoring" column is a frozen point-in-time snapshot, POL-39-exempt per same rationale as §History/§Changelog and TD-VSDD-091 AC-source-of-truth-table exemption. TD-VSDD-097 3-dim sweep: (1) sibling pair — §Authority BC citations and §Behavioral Contracts table BC title rows both swept for H1-verbatim compliance; (2) downstream copy — none; (3) mandate anchor — no new MUSTs added. |
| 1.8 | 2026-09-18 | LOCAL pass-6 F-MED-001/F-LOW-001 docs reconciliation: §File Structure reconciled against actual f38604da4..0af76be5c diff (added Justfile [load-bearing operations-off gate legs] + confirmed CHANGELOG.md/mcp_infrastructure.rs); tools/operations.rs corrected to tools/mod.rs (T-C02 gated module declaration in mod.rs, not operations.rs directly); 'Files NOT to touch' corrected to carve out repo-root Justfile+CHANGELOG.md as EXPECTED modifications; test inventory documents both e2e round-trip tests (OBS-1 list_capabilities AC-002 + OBS-2 tools_list_14 AC-001; 6 tests total); frozen-HEAD ref updated to 0af76be5c. No code/behavior change; feature HEAD 0af76be5c frozen. TD-VSDD-097 3-dim sweep: (1) sibling pair — §File-Structure MODIFY table + 'Files NOT to touch' list + §Token Budget rows all swept together; (2) downstream copy — none; (3) mandate anchor — no new MUST. |
| 1.7 | 2026-09-18 | LOCAL pass-4 LOW-1/LOW-2 records-only sweep (TD-VSDD-096): exhaustive de-pin of volatile BC-version pins in §Authority/§Token Budget narrative (POL-39) + removed server.rs line-number cite from §Tasks Phase-D note (TD-VSDD-091); cite ID+§anchor form only. No code/behavior change; feature HEAD 09658db3a frozen. TD-VSDD-097 3-dim sweep: (1) sibling pair — §Authority NOTE and §Token Budget rows both carried BC pins, swept together; (2) downstream copy — none; (3) mandate anchor — no new MUST. |
| 1.6 | 2026-09-18 | OBS-1 (LOCAL pass-3): complete test-name reconciliation sweep (docs-only; no code or behavior change; feature HEAD 09658db3a frozen). All 4 RG-GATE test names corrected to include `test_BC_2_10_017_` infix (RG-GATE-001..004). T-S01 step-3 spike test name `test_spike_tool_catalog_count_without_operations_feature` updated to shipped name `test_BC_2_10_017_tools_list_returns_14_tools_without_operations_feature` (spike placeholder superseded by RG-GATE-001 in shipped code). T-D04 and §File Structure MODIFY updated to include third gated test `test_bc_2_10_017_sibling_handlers_guard_precedes_audit` (present in worktree; was omitted from T-D04 which previously cited only two tests). Token Budget file-row note updated from RG-GATE-001..003 to RG-GATE-001..004 + OBS-2 e2e. TD-VSDD-097 3-dim sweep: (1) sibling pair — §Red Gate list and §Tasks T-D01/T-D04 references to the same tests swept together; (2) downstream copy — none (test names not copied into BC/ADR/VP artifacts); (3) mandate anchor — no new MUST added. |
| 1.5 | 2026-09-18 | OBS-A (LOCAL pass-2): AC-005 + T-D01 prose corrected — operations-absent inline arm asserts get_diagnostics catalog-ABSENCE (calling it cannot compile when gated out, E0599); -32602 wire behavior discharged by RG-GATE-003. No code or behavior change; implementation already correct. input-hash updated to reflect current inputs state (17d6c8f). |
| 1.4 | 2026-09-18 | Template conformance (D-2567 resume; Canonical Principle Rule 4 fix-in-scope): added missing frontmatter keys (level/cycle/inputs/input-hash/timestamp/traces_to) + Purity Classification section, values derived from sibling E-BETA3-REMEDIATION stories. No AC/RG/task/BC/ADR content change. |
| 1.3 | 2026-09-18 | Phase D compile-safety cfg-gate tasks added (D-2567 resume, SAC-1 completeness): T-D03 gates inline server.rs `test_operations_tools_return_not_implemented_error_code`; T-D04 gates `tests/mcp_infrastructure.rs` `test_bc_2_10_017_not_yet_available_fast_fail_under_1s` + `test_bc_2_10_017_not_yet_available_guard_precedes_audit` — all three call gated ops methods and would fail no-operations-build compilation after T-C01. `mcp_infrastructure.rs` added to Files-to-MODIFY. Explicit no-gate constraint on param structs (`ListInfusionsParams`/`InfusionStatusParams`/`PluginStatusParams`). No BC/ADR/AC change; no new Red Gate test (RG density unchanged 4/5). |
| 1.2 | 2026-09-16 | F3 BC/ADR pin propagation (D-2543/D-2544): BC-2.10.017 v1.1→v1.3; BC-2.10.011 v1.6→v1.7. AMENDMENT PENDING annotations removed from frontmatter comment, §Authority NOTE, §Behavioral Contracts table, and §Token Budget. |
| 1.1 | 2026-09-16 | U-1: Corrected error code from `-32601` (MethodNotFound) to `-32602` (InvalidParams, message `tool not found`) per rmcp 1.7.0 source + `error_mapping.rs:86-91` confirmation (D-1110 uncertainty scan). Applied to AC-003, RG-GATE-003, T-D01, AC-005. U-2/U-3: Replaced T-C01 per-method-`#[cfg]`-inside-one-`#[tool_router]`-block approach (fails E0599 in rmcp-macros 1.7.0) with ratified two-router-block + combiner pattern per architect design decision D-1110. T-S01 updated from open-question spike to compile-confirmation step. Architecture Compliance Rule 2 and risk comment updated to reflect ratified mechanism. |
| 1.0 | 2026-09-15 | Initial story decomposition |
