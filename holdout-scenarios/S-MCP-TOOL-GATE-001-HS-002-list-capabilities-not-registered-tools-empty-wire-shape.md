---
document_type: holdout-scenario
level: L3
id: "HS-MCP-GATE-001-002"
title: "list_capabilities not_registered_tools field is empty array [] on wire when operations feature absent (not null, not 40-element array)"
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
  - ".factory/specs/behavioral-contracts/BC-2.10.011-list-capabilities-meta-tool.md"
  - ".factory/specs/behavioral-contracts/BC-2.10.017-not-yet-available-tools-fast-fail-audit-channel-non-blocking.md"
input-hash: "TBD"
traces_to: "BC-2.10.011"
behavioral_contracts:
  - BC-2.10.011
  - BC-2.10.017
verification_properties: []
lifecycle_status: active
introduced: "S-MCP-TOOL-GATE-001"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-MCP-TOOL-GATE-001 (HS-033 group). Validates that list_capabilities returns not_registered_tools: [] (empty array, NOT null, NOT a 40-element array) at wire level when operations feature absent. Discriminating: pre-patch binary returns not_registered_tools with 40 stub names. Wire-shape assertion on the serialized JSON response body. Exercises both cross-client (null) and single-client modes per EC-10-023. BC-2.10.011 §Postconditions + BC-2.10.017 §Postconditions absent-feature path. Test-writer and implementer must NOT read this file."
---

# HS-MCP-GATE-001-002: list_capabilities not_registered_tools field is empty array [] on wire when operations feature absent

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-MCP-TOOL-GATE-001 (HS-033 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.10.011 §Postconditions `not_registered_tools` field: when the `operations`
Cargo feature is absent (the default), `NOT_YET_AVAILABLE_TOOLS = &[]` → `list_capabilities`
binds `not_registered_tools` to the empty slice → wire response carries `"not_registered_tools":[]`.
EC-10-023: both single-client and cross-client summary modes return `not_registered_tools: []`
when the feature is absent. Verified by S-MCP-TOOL-GATE-001 AC-002 / RG-GATE-002.
**Gate:** Story-level holdout gate (HS-033) — runs after LOCAL 3-CLEAN convergence, before demo
recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the **`not_registered_tools` wire shape** in the `list_capabilities`
response when the `operations` Cargo feature is absent
(BC-2.10.011 §Postconditions EC-10-023; S-MCP-TOOL-GATE-001 AC-002).

When prism is compiled WITHOUT the `operations` feature:

1. `NOT_YET_AVAILABLE_TOOLS = &[]` (empty slice at compile time).
2. The `list_capabilities` handler binds `not_registered_tools` to `NOT_YET_AVAILABLE_TOOLS`.
3. The serialized response includes `"not_registered_tools":[]` — an empty JSON array.
4. No previously-stubbed operations tool names appear in the array.

**The defect this scenario catches:** An unpatched binary returns `"not_registered_tools":
["get_diagnostics","create_schedule",...]` (40 entries). This signals to LLM agents that 40 tools
exist but are unavailable, prompting unnecessary tool invocation attempts. A correctly-gated
binary signals no unavailable tools when the operations feature is off.

**NULL vs ABSENT vs EMPTY discrimination (wire-shape assertion discipline):**
- `null`: a compliance violation — the field must be an array when present
- ABSENT: a compliance violation — BC-2.10.011 §Postconditions requires the field
- `[]`: CORRECT (operations feature absent; empty compile-time constant)
- `["get_diagnostics",...]`: WRONG (pre-patch behavior; 40 entries)

**Two parts in this scenario:**

**Part A — Cross-client summary mode (`client_id: null`):**
- Issue `list_capabilities` with `client_id: null`.
- Parse the response body from `result.content[0].text`.
- Assert `not_registered_tools` key IS PRESENT in the parsed JSON.
- Assert `not_registered_tools` value is an empty array `[]`.
- Assert `not_registered_tools` is NOT `null`.
- Assert `get_diagnostics` and `create_schedule` are NOT present in the array.

**Part B — Single-client mode (`client_id: "holdout-test"`):**
- Issue `list_capabilities` with `client_id: "holdout-test"` (unknown-but-well-formed client_id;
  BC-2.10.011 §Preconditions: never errors for an unknown-but-well-formed client_id).
- Parse the response body.
- Assert `not_registered_tools` key IS PRESENT.
- Assert `not_registered_tools` value is an empty array `[]`.

**BDD supplement (Part A):**

**Given** prism is built from the S-MCP-TOOL-GATE-001 story branch without `--features operations`
**And** prism MCP stdio is started with a minimal valid configuration
**When** `list_capabilities` is called with `client_id: null`
**Then** the response is not a JSON-RPC error
**And** the parsed response body JSON contains the `not_registered_tools` key
**And** `not_registered_tools` is an empty array `[]` (not null, not a non-empty array)
**And** `get_diagnostics` is NOT present in `not_registered_tools`
**And** `create_schedule` is NOT present in `not_registered_tools`

---

## Setup Instructions

1. Use the same prism binary as HS-001 (built from the story branch without `--features operations`).

2. Start prism in MCP stdio mode with a minimal valid configuration. `list_capabilities` does not
   require specific sensor configuration — it reads the capability registry seeded from compiled-in
   defaults. No registered clients are required (cross-client mode works without any clients;
   single-client mode with an unknown client_id returns `client_registered: false` without error).

3. Complete the MCP `initialize` handshake.

4. Issue the Part A request (`client_id: null` — cross-client summary mode):
   ```json
   {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_capabilities","arguments":{"client_id":null}}}
   ```
   Capture the full raw wire-level JSON response bytes from stdout.

5. Issue the Part B request (`client_id: "holdout-test"` — single-client mode):
   ```json
   {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_capabilities","arguments":{"client_id":"holdout-test"}}}
   ```
   Capture the full raw wire-level JSON response bytes from stdout.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.10.011 | §Postconditions `not_registered_tools`: empty slice `[]` when `operations` feature absent; field is required (not optional) | Part A + B: field present as `[]` |
| BC-2.10.011 | EC-10-023: both single-client and cross-client modes return `not_registered_tools: []` when feature absent | Part A (null) and Part B (client_id: "holdout-test") |
| BC-2.10.011 | §Postconditions: `not_registered_tools` is an array (never null) | NULL vs absent vs empty discrimination |
| BC-2.10.017 | §Postconditions (absent-feature): `NOT_YET_AVAILABLE_TOOLS = &[]` is the compile-time source for `not_registered_tools` binding | Mechanism: empty const → empty field |

---

## Verification Approach

**Part A verification:**

1. Parse the raw wire-level JSON response from Part A.

2. If the outer JSON-RPC envelope is an error: record FAIL on "Part A non-error" dimension.

3. Extract `result.content[0].text`. Parse it as JSON.
   If `result.content` is absent or `result.content[0].type != "text"`: record SETUP-FAILURE.

4. Check that `not_registered_tools` key is PRESENT in the parsed JSON object.
   If absent: record FAIL on "field present" dimension.

5. Check the type and value of `not_registered_tools`:
   - If `null`: record FAIL on "field is array" dimension.
   - If non-empty array: record FAIL on "field is empty" dimension. Note the item count.
   - If empty array `[]`: PASS on "field is empty" dimension.

6. Check that `get_diagnostics` and `create_schedule` are NOT in the array.
   (If the array is already empty, this assertion passes automatically.)

**Part B verification:**

7. Parse the raw wire-level JSON response from Part B.

8. If the outer JSON-RPC envelope is an error: record FAIL on "Part B non-error" dimension.
   An unknown-but-well-formed `client_id` must NOT produce an error (BC-2.10.011 §Preconditions).

9. Extract and parse the response body JSON from `result.content[0].text`.

10. Assert `not_registered_tools` key is PRESENT and its value is `[]`.
    If absent or non-empty: record FAIL on "Part B not_registered_tools empty" dimension.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Part A: response is non-error** (weight: 0.10):
  Full credit (1.0): non-error JSON-RPC response.
  Zero credit (0.0): error response or no response.

- **`not_registered_tools` key present** (weight: 0.20): Is the field present in the response?
  Full credit (1.0): key present in the parsed response body.
  Zero credit (0.0): key absent (incomplete implementation of BC-2.10.011 §Postconditions;
  field was never added or was removed by a rename to `not_implemented`).

- **`not_registered_tools` value is `[]`** (weight: 0.40): Is the value an empty array?
  Full credit (1.0): value is `[]` (empty JSON array).
  Partial credit (0.1): value is non-null but non-empty (const not updated; partial gate).
  Zero credit (0.0): value is `null` or field is absent.

- **No stub names in array** (weight: 0.10): Absence of `get_diagnostics` and `create_schedule`.
  Full credit (1.0): both names absent (implied by empty array; pass automatically if empty).
  Partial credit (0.3): one name absent, the other present (partial gate or selective removal).
  Zero credit (0.0): both names present (pre-patch state).

- **Part B: not_registered_tools empty in single-client mode** (weight: 0.20):
  Full credit (1.0): Part B also has `not_registered_tools: []`.
  Partial credit (0.5): Part B returns error on unknown `client_id` (BC-2.10.011 §Preconditions
  violation — error on unknown client is incorrect behavior; but the primary assertion
  in Part A may still pass).
  Zero credit (0.0): Part B `not_registered_tools` is non-empty.

---

## Edge Conditions

- **`not_registered_tools` key absent:** Implementation did not include the field — possibly
  it was left as the renamed `not_implemented` field from the merged code. BC-2.10.011 requires
  `not_registered_tools` (not `not_implemented`). Record FAIL on "field present" dimension.

- **`not_registered_tools: null`:** Implementation serialized the empty slice as null instead of
  `[]`. This is a distinct failure mode from "absent". Record FAIL on "field is array" dimension.

- **Part B errors on `"holdout-test"` client_id:** BC-2.10.011 §Preconditions requires that
  unknown-but-well-formed client_ids do NOT error. Record partial credit on Part B dimension.

- **Response body not JSON-parseable:** MCP tool results are returned as text in
  `result.content[0].text`. If the text field is not valid JSON, record SETUP-FAILURE.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-MCP-GATE-001-002 (satisfaction: X.XX) — list_capabilities not_registered_tools wire shape incorrect; check NOT_YET_AVAILABLE_TOOLS const binding in list_capabilities handler (BC-2.10.011 §Postconditions EC-10-023: not_registered_tools must be empty array [] when operations feature absent; not null, not absent, not a non-empty array; BC-2.10.017 §Postconditions absent-feature path)"`

Do NOT disclose: the `client_id` values used, whether the failure was null vs absent vs non-empty,
or the specific stub tool names checked for absence.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary built from S-MCP-TOOL-GATE-001 story branch HEAD (default build, no --features operations) |
| corpus_size | 1 not_registered_tools entry (empty array — fixed at compile time by NOT_YET_AVAILABLE_TOOLS = &[]) |
| known_edge_cases | null vs absent vs empty array distinction; Part B unknown client_id non-error; Part A cross-client summary mode |
| false_positive_threshold | Zero: not_registered_tools: [] is an unambiguous empty-array assertion on a compile-time constant |
| false_negative_threshold | Zero: pre-patch binary returns 40 stub names consistently |

**Known-good corpus:** prism binary compiled WITHOUT `--features operations` (story branch HEAD).
Expected: `not_registered_tools: []` in both cross-client (Part A) and single-client (Part B) modes.

**Known-problematic corpus:** prism binary compiled WITH `--features operations` (or pre-patch
binary). Expected: `not_registered_tools: ["get_diagnostics", "create_schedule", ...]` (40 entries)
— this signals to LLM agents that 40 tools exist but are unavailable.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-s-mcp-tool-gate-001-holdout-authoring | 2026-09-16 | product-owner | Initial authoring. HS-033 group for S-MCP-TOOL-GATE-001. list_capabilities not_registered_tools wire shape: empty array [] when operations feature absent. Tests cross-client mode (client_id: null, Part A) and single-client mode (unknown client_id, Part B). NULL vs absent vs empty array discrimination per wire-shape assertion discipline. BC-2.10.011 §Postconditions EC-10-023 + BC-2.10.017 §Postconditions absent-feature path. SINGLE-USE. |
