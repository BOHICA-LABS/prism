---
document_type: holdout-scenario
level: L3
id: "HS-MCP-GATE-001-001"
title: "tools/list wire response exposes exactly 14 live tools and zero operations stubs when binary is compiled without operations feature"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "E-BETA3-REMEDIATION"
story_source: "S-MCP-TOOL-GATE-001"
version: "1.0"
status: active
used: false
last_evaluated: null
last_eval_satisfaction: null
single_use: true
producer: product-owner
timestamp: "2026-09-16T00:00:00Z"
modified: "2026-09-16"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.10.017-not-yet-available-tools-fast-fail-audit-channel-non-blocking.md"
input-hash: "TBD"
traces_to: "BC-2.10.017"
behavioral_contracts:
  - BC-2.10.017
verification_properties: []
lifecycle_status: active
introduced: "S-MCP-TOOL-GATE-001"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-MCP-TOOL-GATE-001 (HS-033 group). Validates the operations Cargo feature gate from the tools/list perspective: a default (no --features operations) binary must expose EXACTLY 14 tools in tools/list, all from LIVE_TOOLS, and ZERO previously-stubbed operations tools. Discriminating: pre-patch binary returns 54 tools. Wire-level assertion on the serialized JSON tools array count and spot-checks of LIVE_TOOLS presence and stub tool absence. BC-2.10.017 §Postconditions (absent-feature path). Test-writer and implementer must NOT read this file."
---

# HS-MCP-GATE-001-001: tools/list wire response exposes exactly 14 live tools and zero operations stubs when binary is compiled without operations feature

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-MCP-TOOL-GATE-001 (HS-033 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.10.017 §Postconditions (absent-feature path): when the `operations` Cargo
feature is absent (the default), `NOT_YET_AVAILABLE_TOOLS = &[]` → no stub tools are registered
→ `tools/list` shows only the 14 `LIVE_TOOLS`. Stub tools are not compiled in and not visible.
INV-OPERATIONS-FEATURE-GATE: 14 LIVE_TOOLS unconditionally registered in both feature states.
**Gate:** Story-level holdout gate (HS-033) — runs after LOCAL 3-CLEAN convergence, before demo
recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the **operations Cargo feature gate from the tool catalog perspective**
(BC-2.10.017 §Postconditions absent-feature path; S-MCP-TOOL-GATE-001 AC-001).

When prism is compiled WITHOUT the `operations` feature (the default production configuration):

1. `NOT_YET_AVAILABLE_TOOLS` is the compile-time constant `&[]` (empty slice).
2. The operations router block does not exist at compile time — no stub tool handlers are compiled
   in and none are registered with the MCP router.
3. `tools/list` returns exactly 14 tools: the complete `LIVE_TOOLS` set.
4. No previously-stubbed operations tool names appear in the catalog.

**The defect this scenario catches:** An unpatched prism binary compiles and registers all 40
operations stub handlers unconditionally, polluting the MCP catalog with tools that permanently
return -32003. A regression or partial implementation might produce an intermediate catalog size
(e.g., some stubs remain registered) or accidentally gate one of the 14 LIVE_TOOLS (a P0
regression of BC-2.10.012).

**Two assertions in this scenario:**

**Part A — Exact catalog count:**
- Issue `tools/list` to the MCP server.
- Expected: `result.tools` array length equals exactly 14 on the wire.

**Part B — Catalog membership spot-checks:**
- All 14 LIVE_TOOLS names are present: `query`, `explain_query`, `create_alias`, `list_aliases`,
  `delete_alias`, `explain_alias`, `confirm_action`, `reload_config`, `add_sensor_spec`,
  `list_sensor_specs`, `validate_config`, `list_capabilities`, `prism_describe`,
  `check_sensor_health`.
- Previously-stubbed operations tool names are absent: `get_diagnostics` and `create_schedule`
  must NOT appear in `result.tools[*].name`.

**BDD supplement (Part A):**

**Given** prism is built from the S-MCP-TOOL-GATE-001 story branch without `--features operations`
**And** prism MCP stdio is started with a minimal valid configuration
**When** a `tools/list` JSON-RPC request is issued and the raw wire response is captured
**Then** the response is not a JSON-RPC error
**And** the `result.tools` array has exactly 14 entries
**And** `prism_describe` appears in `result.tools[*].name`
**And** `query` appears in `result.tools[*].name`
**And** `list_capabilities` appears in `result.tools[*].name`
**And** `check_sensor_health` appears in `result.tools[*].name`
**And** `get_diagnostics` does NOT appear in `result.tools[*].name`
**And** `create_schedule` does NOT appear in `result.tools[*].name`

---

## Setup Instructions

1. Confirm prism is built from the S-MCP-TOOL-GATE-001 story branch HEAD commit WITHOUT
   specifying `--features operations`. The default build (no feature flags) is the correct
   configuration for this scenario. The binary may be built with:
   `cargo build -p prism-mcp` (or the full workspace build `cargo build`).
   The `operations` feature must NOT be active — verify there is no `--features operations`
   in the build invocation.

2. Start prism in MCP stdio mode with a minimal valid configuration. Tool catalog behavior
   is compile-time-fixed and independent of runtime sensor configuration — no sensor specs
   or client registrations are required for this test. If prism requires a `prism.toml`
   to start, an empty or minimal one (no `[[sensors]]` entries) is sufficient.

3. Complete the MCP `initialize` handshake:
   Send: `{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"holdout-evaluator","version":"1.0"}}}`
   Await a non-error initialize response before proceeding.

4. Issue the `tools/list` request:
   `{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}`
   Capture the full raw wire-level JSON response bytes from stdout.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.10.017 | §Postconditions (absent-feature path): `NOT_YET_AVAILABLE_TOOLS = &[]` → only 14 LIVE_TOOLS in catalog | Part A: catalog count == 14 |
| BC-2.10.017 | §Postconditions (absent-feature path): stub tools NOT registered in `tools/list` when feature absent | Part B: get_diagnostics and create_schedule absent from catalog |
| BC-2.10.017 | INV-OPERATIONS-FEATURE-GATE: 14 LIVE_TOOLS unconditionally registered in both feature states — gate applies ONLY to operations block | Part B: all 14 LIVE_TOOLS present |

---

## Verification Approach

1. Parse the raw wire-level JSON response from the `tools/list` request.

2. Verify the response is NOT a JSON-RPC error: the `error` key must be absent (or null) and the
   `result` key must be present.

3. **Part A — Count assertion:**
   Extract the `result.tools` array. Count the entries.
   Assert `result.tools.length == 14`.
   If count != 14: record FAIL on "catalog count" dimension. Note the actual count.

4. **Part B — LIVE_TOOLS presence:**
   Extract the set of tool names: `result.tools[*].name`.
   Assert that ALL 14 of the following names appear:
   `query`, `explain_query`, `create_alias`, `list_aliases`, `delete_alias`, `explain_alias`,
   `confirm_action`, `reload_config`, `add_sensor_spec`, `list_sensor_specs`, `validate_config`,
   `list_capabilities`, `prism_describe`, `check_sensor_health`.
   If any name is missing: record FAIL on "LIVE_TOOLS completeness" dimension with the missing name.

5. **Part B — Stub tool absence:**
   Assert that `get_diagnostics` is NOT in `result.tools[*].name`.
   Assert that `create_schedule` is NOT in `result.tools[*].name`.
   If either name is present: record FAIL on "stub tool absent" dimension.

6. The wire-shape assertion is on the serialized JSON response bytes from MCP stdout. Do NOT
   accept passing results from pre-serialization Rust struct inspection — assert on the raw
   wire output.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response is non-error (prerequisite)** (weight: 0.10):
  Full credit (1.0): `result` key present; `error` key absent.
  Zero credit (0.0): error response or no response (SETUP-FAILURE).

- **Catalog count == 14** (weight: 0.40): Does `result.tools.length` equal exactly 14?
  Full credit (1.0): exactly 14 tools.
  Partial credit (0.3): count > 0 but != 14 (partial gate or regression in either direction).
  Zero credit (0.0): count == 0 or JSON parse failure.

- **All 14 LIVE_TOOLS present** (weight: 0.30): Are all 14 expected names in the catalog?
  Full credit (1.0): all 14 names present.
  Partial credit (0.5): 10–13 names present (gate accidentally removed a LIVE_TOOLS entry;
  P0 regression of BC-2.10.012 — prism_describe and list_capabilities must always be registered).
  Zero credit (0.0): fewer than 10 LIVE_TOOLS present.

- **Stub tools absent** (weight: 0.20): Are `get_diagnostics` and `create_schedule` absent?
  Full credit (1.0): both names absent from catalog.
  Partial credit (0.5): one name absent, the other present (partial gate application).
  Zero credit (0.0): both names present (operations stub gate not applied; pre-patch state).

---

## Edge Conditions

- **Catalog count == 0:** MCP server started but tool registration failed. Record SETUP-FAILURE
  if prism failed to start; record FAIL on all dimensions if prism started but catalog is empty.

- **Count == 54:** The operations gate is not applied at all — pre-patch behavior. FAIL on count
  dimension and stub-absent dimension.

- **Count between 15 and 53:** Partial gate — some stubs gated, others not. FAIL on count
  dimension; partial credit on stub-absent dimension.

- **One of the 14 LIVE_TOOLS is missing from a 14-count catalog:** Gate inadvertently removed a
  LIVE_TOOLS tool while gating the stubs. This is a P0 regression of BC-2.10.012. Report as FAIL
  with the specific missing tool name.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-MCP-GATE-001-001 (satisfaction: X.XX) — tool catalog size or membership gap; check NOT_YET_AVAILABLE_TOOLS const and operations feature gate compilation in server.rs (BC-2.10.017 §Postconditions absent-feature path: only LIVE_TOOLS registered when operations feature absent; INV-OPERATIONS-FEATURE-GATE: 14 LIVE_TOOLS unconditionally registered)"`

Do NOT disclose: the expected catalog count, the specific tool names checked for presence or
absence, or the specific discriminating assertions used.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary built from S-MCP-TOOL-GATE-001 story branch HEAD (default build, no --features operations) |
| corpus_size | 14 tools (MCP tool catalog — fixed at compile time; independent of sensor data) |
| known_edge_cases | LIVE_TOOLS regression (gate removes one of 14); partial gate (some stubs remain); count 0 (server not started) |
| false_positive_threshold | Zero: catalog count 14 is an unambiguous assertion on a compile-time constant |
| false_negative_threshold | Zero: pre-patch binary returns 54 consistently |

**Known-good corpus:** prism binary compiled WITHOUT `--features operations` (story branch HEAD).
Expected: `result.tools.length == 14`, all 14 LIVE_TOOLS present, no stub tool names in catalog.

**Known-problematic corpus:** prism binary compiled WITH `--features operations` (or pre-patch
binary from before this story). Expected: `result.tools.length == 54`, operations stubs visible
in catalog (`get_diagnostics`, `create_schedule`, etc. present).

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-s-mcp-tool-gate-001-holdout-authoring | 2026-09-16 | product-owner | Initial authoring. HS-033 group for S-MCP-TOOL-GATE-001. Tool catalog count gate: tools/list must return exactly 14 tools when operations feature absent; all 14 LIVE_TOOLS must be present; stub tool names must be absent. Discriminates against pre-patch 54-tool catalog. BC-2.10.017 §Postconditions absent-feature path + INV-OPERATIONS-FEATURE-GATE. SINGLE-USE. |
