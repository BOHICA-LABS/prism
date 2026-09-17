---
document_type: holdout-scenario
level: L3
id: "HS-MCP-GATE-001-003"
title: "Invoking a previously-stubbed operations tool returns JSON-RPC error -32602 with message 'tool not found', not -32003 or -32601"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-MCP-TOOL-GATE-001 (HS-033 group). Validates that invoking a previously-stubbed operations tool returns JSON-RPC error -32602 (InvalidParams, message 'tool not found') from rmcp 1.7.0's unregistered-tool path, NOT -32003 (prism fast-fail handler). Discriminating: pre-patch binary returns -32003. The code/message pair is semantically critical: -32003 tells agents the tool will eventually work; -32602 tells agents the tool does not exist. Two-tool anti-special-case check. BC-2.10.017 §Postconditions absent-feature path + INV-OPERATIONS-FEATURE-GATE. Test-writer and implementer must NOT read this file."
---

# HS-MCP-GATE-001-003: Invoking a previously-stubbed operations tool returns JSON-RPC error -32602 with message "tool not found", not -32003 or -32601

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-MCP-TOOL-GATE-001 (HS-033 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.10.017 §Postconditions (absent-feature path): when `operations` is absent and
a previously-stubbed tool is invoked, rmcp 1.7.0 returns the standard unregistered-tool error
(`-32602` InvalidParams, message `"tool not found"`) because the tool was never registered —
NOT the prism-specific `-32003` fast-fail which only fires for registered-but-not-yet-available
tools. INV-OPERATIONS-FEATURE-GATE: `-32003` applies ONLY when `operations` is ENABLED.
See `error_mapping.rs:86-91` for the prism mapping that deliberately chooses -32602 over -32601.
**Gate:** Story-level holdout gate (HS-033) — runs after LOCAL 3-CLEAN convergence, before demo
recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the **error code semantics for unregistered tool invocations**
(BC-2.10.017 §Postconditions absent-feature path; S-MCP-TOOL-GATE-001 AC-003).

When prism is compiled WITHOUT the `operations` feature and a previously-stubbed tool is called:

1. The tool handler was never compiled into the binary and is not registered with rmcp's router.
2. rmcp 1.7.0 handles unknown-tool invocations at the router level by returning
   `ErrorData::invalid_params("tool not found")` → JSON-RPC error code `-32602`.
3. The prism-specific `-32003` fast-fail path is NEVER reached because there is no handler to
   enter. The router rejects the call before any handler body executes.
4. The error code distinction is semantically critical for LLM agent reasoning:
   - `-32003`: "This tool exists but is not yet available — it may work in a future release."
   - `-32602` (InvalidParams, `"tool not found"`): "This tool does not exist — stop trying."

**The defect this scenario catches:** An unpatched binary registers the stub handler, which calls
`Err(not_yet_available_msg(...))` and returns `-32003`. A partially-patched binary that gates at
runtime (via an `if` guard inside the handler rather than compile-time feature gating) might still
return `-32003`. Only a binary where the handler is NEVER COMPILED IN produces `-32602` from
rmcp's own router rejection path.

**Two parts in this scenario:**

**Part A — First previously-stubbed tool:**
- Invoke `tools/call` with a previously-stubbed operations tool name.
- Expected: JSON-RPC error `{"code": -32602, "message": "tool not found"}`.
- Assert `error.code == -32602` (NOT -32003, NOT -32601, NOT -32600).
- Assert `error.message == "tool not found"` (exact string from rmcp 1.7.0).

**Part B — Second previously-stubbed tool (anti-special-case check):**
- Invoke `tools/call` with a DIFFERENT previously-stubbed tool name.
- Expected: same `-32602` / `"tool not found"` response.
- This rules out the possibility that Part A passed because one specific name was handled
  differently while the general gate was not applied.

**BDD supplement (Part A):**

**Given** prism is built from the S-MCP-TOOL-GATE-001 story branch without `--features operations`
**And** prism MCP stdio is started with a minimal valid configuration
**When** `tools/call` is issued with a previously-stubbed operations tool name and empty arguments
**Then** the response is a JSON-RPC error (not a result)
**And** `error.code` equals -32602
**And** `error.message` equals "tool not found"
**And** `error.code` does NOT equal -32003
**And** `error.code` does NOT equal -32601

---

## Setup Instructions

1. Use the same prism binary as HS-001 and HS-002 (built from the story branch without
   `--features operations`).

2. Start prism in MCP stdio mode with a minimal valid configuration. No sensor configuration
   is needed — the error response is generated by rmcp's router before any sensor lookup occurs.

3. Complete the MCP `initialize` handshake. Optionally issue `tools/list` to confirm the binary
   is the correctly-gated build (catalog should be 14 tools, not 54) before proceeding.

4. Issue the Part A `tools/call` request (first previously-stubbed tool):
   ```json
   {"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_diagnostics","arguments":{}}}
   ```
   Capture the full raw wire-level JSON response bytes from stdout.

5. Issue the Part B `tools/call` request (second previously-stubbed tool, different from Part A):
   ```json
   {"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"create_schedule","arguments":{}}}
   ```
   Capture the full raw wire-level JSON response bytes from stdout.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.10.017 | §Postconditions (absent-feature): stub tools not registered → rmcp returns `-32602` for unknown tool invocation, NOT `-32003` | Part A + B: error.code == -32602 |
| BC-2.10.017 | §Postconditions (absent-feature): `emit_tool_audit` never reached for unregistered tools — rmcp's router rejects before any handler executes | Mechanism: no prism handler invoked; rmcp router error path fires |
| BC-2.10.017 | INV-OPERATIONS-FEATURE-GATE: `-32003` applies ONLY when `operations` feature ENABLED; when absent, previously-stubbed tool names return the standard rmcp unregistered-tool error | Negative assertion: error.code != -32003 |

---

## Verification Approach

**Part A verification:**

1. Parse the raw wire-level JSON response from Part A.

2. Check that the outer JSON-RPC structure has an `error` key and no meaningful `result` key.
   If only `result` is present (non-error response): record FAIL on "Part A is error response"
   dimension. (This would mean the tool was somehow registered and returned a non-error result —
   likely an empty result from a handler that wasn't fully gated.)

3. Extract `error.code`. Assert it equals exactly -32602.
   - If -32003: record FAIL on "-32602 not -32003" dimension. Pre-patch behavior confirmed.
   - If -32601: record FAIL on "-32602 not -32601" dimension. Incorrect rmcp mapping.
   - If any other code: record FAIL with the actual code noted.

4. Extract `error.message`. Assert it equals exactly `"tool not found"`.
   - If message contains "not yet available": record FAIL (this indicates the -32003 fast-fail
     path ran — handler was reached despite the gate).
   - If message is present but differs from "tool not found": record partial credit (code correct,
     message differs; possible rmcp version discrepancy; acceptable if code assertion passes).
   - If message is null or absent: record partial credit.

**Part B verification:**

5. Parse the raw wire-level JSON response from Part B.

6. Assert `error.code == -32602` for the second tool name.
   If Part A returned -32602 but Part B returned -32003: record FAIL on "Part B consistent"
   dimension. This indicates a partial gate — one specific name was left without a handler while
   others retained handlers. The anti-special-case check caught an incomplete implementation.

7. Assert `error.message == "tool not found"` for Part B.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Part A: response is a JSON-RPC error (not result)** (weight: 0.10):
  Full credit (1.0): `error` key present; `result` key absent or null.
  Zero credit (0.0): non-error response (tool was registered and returned a result).

- **Part A: error.code == -32602** (weight: 0.45): Primary discriminating assertion.
  Full credit (1.0): code is exactly -32602.
  Zero credit (0.0): code is -32003 (pre-patch), -32601 (wrong rmcp mapping), or other.

- **Part A: error.message == "tool not found"** (weight: 0.20):
  Full credit (1.0): message is exactly `"tool not found"` (rmcp 1.7.0 standard).
  Partial credit (0.5): message is non-null but differs (acceptable if -32602 code correct;
  may indicate rmcp version variation).
  Zero credit (0.0): message contains "not yet available" (prism fast-fail ran; contradicts
  code check), OR message is null/absent.

- **Part B: error.code == -32602 (anti-special-case)** (weight: 0.25):
  Full credit (1.0): Part B also returns code -32602.
  Partial credit (0.3): Part B returns a different code than Part A (inconsistent gate; partial
  implementation where some stubs removed and others remain).
  Zero credit (0.0): Part B returns -32003 while Part A returned -32602 (one name left in handler;
  general gate not applied).

---

## Edge Conditions

- **Both parts return -32003:** Pre-patch behavior. Both stub handlers are still registered and
  the fast-fail path runs. Record FAIL on both error.code dimensions.

- **Part A returns -32602, Part B returns -32003:** Partial gate — the stub for Part A's name
  was removed but the stub for Part B's name was not. The anti-special-case check caught an
  incomplete implementation. Record FAIL on Part B dimension.

- **Part A returns -32601 (MethodNotFound):** Incorrect rmcp error mapping. `-32601` is the
  JSON-RPC "method not found" code (used when the JSON-RPC method name is unknown, not the
  tool name). `-32602` is the rmcp 1.7.0 standard for an unknown tool name within a known
  method (`tools/call`). See `error_mapping.rs:86-91` for the intentional -32602 choice.
  Record FAIL on "-32602 not -32601" dimension.

- **Part A returns a successful non-error result:** The tool stub returned an empty or default
  result instead of an error. This could indicate the handler compiled in but no logic executes.
  Record FAIL on "Part A is error response" dimension.

- **prism stdout closes after the unknown tool call:** Some rmcp transport configurations may
  terminate on protocol errors. If the connection closes, record SETUP-FAILURE and note the
  error mode for the implementer to investigate.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-MCP-GATE-001-003 (satisfaction: X.XX) — unregistered operations tool returned wrong JSON-RPC error code; check rmcp unregistered-tool error path (BC-2.10.017 §Postconditions absent-feature path: -32602 InvalidParams 'tool not found' from rmcp router, NOT -32003 from prism fast-fail handler, NOT -32601; INV-OPERATIONS-FEATURE-GATE: -32003 only fires when operations feature enabled; verify error_mapping.rs:86-91 and that the handler is never compiled in)"`

Do NOT disclose: the specific tool names used for Part A and Part B, the exact code/message pair
asserted, or which part triggered the failure.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary built from S-MCP-TOOL-GATE-001 story branch HEAD (default build, no --features operations) |
| corpus_size | 2 tool invocations (previously-stubbed names; unregistered in default build) |
| known_edge_cases | -32601 vs -32602 rmcp mapping distinction; partial gate (one name unregistered, another not); empty-result response (handler compiled in but no logic) |
| false_positive_threshold | Zero: error.code == -32602 is an unambiguous exact-integer assertion |
| false_negative_threshold | Zero: pre-patch binary returns -32003 consistently for all 40 stub names |

**Known-good corpus:** prism binary compiled WITHOUT `--features operations` (story branch HEAD).
Expected: `{"error": {"code": -32602, "message": "tool not found"}}` for both Part A and Part B
tool names.

**Known-problematic corpus:** prism binary compiled WITH `--features operations` (or pre-patch
binary before this story). Expected: `{"error": {"code": -32003, "message": "Tool '...' is not
yet available in this release..."}}` — the prism-specific fast-fail fires because the stub
handler is registered and the `not_yet_available_msg` path is reached.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-s-mcp-tool-gate-001-holdout-authoring | 2026-09-16 | product-owner | Initial authoring. HS-033 group for S-MCP-TOOL-GATE-001. Error code semantics: previously-stubbed operations tools return JSON-RPC -32602 (rmcp unregistered-tool error) not -32003 (prism fast-fail) when operations feature absent. Two-tool anti-special-case check (Part A + Part B with different tool names). Wire-level error.code and error.message assertions. BC-2.10.017 §Postconditions absent-feature path + INV-OPERATIONS-FEATURE-GATE. SINGLE-USE. |
