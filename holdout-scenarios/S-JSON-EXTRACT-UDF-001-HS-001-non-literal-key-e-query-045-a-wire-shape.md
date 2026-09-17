---
document_type: holdout-scenario
level: L3
id: "HS-JEX-001-001"
title: "json_extract_string with non-literal key argument returns E-QUERY-045(a) structured error at wire level"
category: "security-gate"
must_pass: true
priority: P1
epic_id: "E-BETA3-REMEDIATION"
story_source: "S-JSON-EXTRACT-UDF-001"
version: "1.0"
status: active
used: false
last_evaluated: null
last_eval_satisfaction: null
single_use: true
producer: product-owner
timestamp: "2026-09-17T00:00:00Z"
modified: "2026-09-17"
phase: 3
inputs:
  - ".factory/stories/S-JSON-EXTRACT-UDF-001-json-extract-scalar-udf.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.025-json-extract-string-scalar-udf.md"
  - ".factory/specs/architecture/decisions/ADR-066-json-extract-scalar-udf.md"
input-hash: "TBD"
traces_to: "BC-2.11.025"
behavioral_contracts:
  - BC-2.11.025
verification_properties:
  - VP-162
lifecycle_status: active
introduced: "S-JSON-EXTRACT-UDF-001"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-JSON-EXTRACT-UDF-001 (HS-040 group). Combined UDF-registration and plan-gate security test: non-literal key column reference rejected with E-QUERY-045(a) at plan time. Pre-patch: DataFusion 'unknown function json_extract_string' error (UDF not registered). Post-fix: structured -32602 JSON-RPC error with E-QUERY-045 in message. NO DTU required — plan gate fires before any sensor fetch. Wire-level assertion on error code and message string. Security surface: literal-key plan gate is the injection prevention mechanism (ADR-066 §B3). Test-writer and implementer must NOT read this file."
---

# HS-JEX-001-001: json_extract_string with non-literal key argument returns E-QUERY-045(a) structured error at wire level

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-JSON-EXTRACT-UDF-001 (HS-040 group)
**Must Pass:** YES (P1 — important gate; not P0 but required for merge)
**BC Traced:** BC-2.11.025 §Error Cases E-QUERY-045(a) + EC-11-025-006
  "A non-literal key argument `json_extract_string(col, other_col)` where `other_col` is a column reference is rejected at plan time with `E-QUERY-045(a)` via `PrismError::JsonExtractNonLiteralKey`. The gate fires BEFORE DataFusion execution or any sensor fan-out."
**Gate:** Story-level holdout gate (HS-040) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario serves a dual purpose: it simultaneously verifies that the UDF is REGISTERED and that the literal-key PLAN GATE is active.

**Why this is both a UDF-registration probe and a security gate:**

The `json_extract_string` UDF must be registered in the DataFusion `SessionContext` at engine construction (ADR-066 §E). If it is NOT registered, DataFusion returns a "unknown function" plan-time error — not `E-QUERY-045`. If the UDF IS registered but the plan gate is absent, the non-literal key would be passed to the UDF at runtime (which is a security gap — CWE-400, ADR-066 §B3).

The compound behavior — UDF registered AND plan gate active — is discriminated by the SPECIFIC error code and message:
- **Pre-patch state 1 (UDF not registered):** DataFusion error, typically a JSON-RPC error with code -32600 or -32603 and message containing "unknown function" or "unregistered" and "json_extract_string".
- **Pre-patch state 2 (UDF registered, no plan gate):** Either runtime DataFusion error or unexpected result (no structured E-QUERY-045).
- **Post-fix:** JSON-RPC error code exactly `-32602` (INVALID_PARAMS) with message containing `"E-QUERY-045"`.

**Three wire-level assertions on the serialized MCP response:**

**Part A — JSON-RPC error code is -32602 (INVALID_PARAMS):**
- Assert: response has `error` key (not `result`).
- Assert: `error.code == -32602`.
  If `error.code != -32602`: distinguish — is it a DataFusion unknown-function code? Record the actual code.

**Part B — error message contains "E-QUERY-045":**
- Assert: `error.message` (or the content within the error envelope) contains the string `"E-QUERY-045"`.
- Assert: `error.message` does NOT contain `"unknown function"` paired with `"json_extract_string"` (which would indicate the UDF is not registered — the pre-patch state).

**Part C — error message does NOT reference "json_extract_string" as unregistered:**
- Assert: `error.message` does not contain the pattern `"unknown function json_extract_string"` or `"unregistered" ... "json_extract_string"`.
- This confirms the UDF is registered (DataFusion knows about it) even though the query is rejected for a different reason (non-literal key).

**BDD supplement:**

**Given** prism is built from the S-JSON-EXTRACT-UDF-001 story branch
**And** prism is running in MCP stdio mode with any Claroty-capable client configuration
**When** `tools/call query {"query": "FROM claroty_alerts | SELECT json_extract_string(raw_extensions, severity_id)", "client_id": "<test_client>"}` is issued via MCP stdio
  (note: `severity_id` here is a column reference, not a string literal — no quotes)
**Then** the response has `error` (not `result`)
**And** `error.code == -32602`
**And** `error.message` contains `"E-QUERY-045"`
**And** `error.message` does NOT contain `"unknown function json_extract_string"`

---

## Setup Instructions

1. Build prism from the S-JSON-EXTRACT-UDF-001 story branch.

2. NO DTU is required. The plan gate fires BEFORE any sensor fetch — the query is rejected
   at plan time without contacting the sensor at all.

3. Configure `prism.toml` with ANY test client that has the Claroty sensor in scope
   (needed so `claroty_alerts` and `raw_extensions` are recognized as valid table/column names
   for the parse step to accept the query syntax). Alternatively, any table with a column
   named such that the parser accepts the query.

4. Start prism in MCP stdio mode. Complete `initialize` handshake.
   SETUP-FAILURE condition: prism fails to start or MCP handshake fails.

5. Issue the non-literal-key query:
   ```
   {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query","arguments":{"query":"FROM claroty_alerts | SELECT json_extract_string(raw_extensions, severity_id)","client_id":"<test_client>"}}}
   ```
   Note: `severity_id` in this query has NO quotes — it is a column reference, not a string literal.
   Capture the full wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.025 | EC-11-025-006 / §Error Cases E-QUERY-045(a): non-literal key rejected at plan time with PrismError::JsonExtractNonLiteralKey | Parts A, B, C |
| BC-2.11.025 | §Postconditions: UDF registered under name "json_extract_string" with (Utf8, Utf8) input types at engine construction | Part C: UDF registered (no unknown-function error) |
| ADR-066 | §B3: literal-key plan gate; §F: E-QUERY-045(a) message format | Part B: E-QUERY-045 in message |

---

## Verification Approach

1. Parse wire-level JSON-RPC response.

2. Assert: `error` key is present (query rejected as expected). If `result` is present instead: record FAIL (plan gate not active; UDF may have accepted the non-literal key).

3. **Part A — error code:**
   Extract `error.code`. Assert `error.code == -32602`.
   If `error.code != -32602`: record actual code. If code is -32600 or -32603 with DataFusion message: likely pre-patch "unknown function" state.

4. **Part B — E-QUERY-045 in message:**
   Extract `error.message`. Search for substring `"E-QUERY-045"`.
   If absent: record FAIL (plan gate not returning structured E-QUERY-045; check whether the error is from DataFusion unknown-function path instead).

5. **Part C — not an unknown-function error:**
   Assert `error.message` does NOT contain the pattern `"unknown function"` paired with `"json_extract_string"` within the same message string.
   A message like "unknown function json_extract_string" is the pre-patch behavior (UDF not registered).
   Post-fix: the message must be about E-QUERY-045 (plan gate), not about UDF registration.

6. All assertions on the serialized wire bytes.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response has error (prerequisite)** (weight: 0.10):
  Full credit (1.0): JSON-RPC error response (query correctly rejected).
  Zero credit (0.0): query returned a result (plan gate entirely absent — severe defect). SETUP-FAILURE if prism didn't start.

- **error.code == -32602 (Part A — primary plan-gate discriminator)** (weight: 0.40):
  Full credit (1.0): `error.code` is exactly -32602.
  Zero credit (0.0): wrong error code (e.g., -32600/-32603 from DataFusion pre-pass, or -32603 internal error).

- **E-QUERY-045 in message (Part B — structured error taxonomy)** (weight: 0.35):
  Full credit (1.0): `error.message` contains `"E-QUERY-045"`.
  Zero credit (0.0): E-QUERY-045 absent from message (plan gate may be returning unstructured DataFusion error).

- **Not an unknown-function error (Part C — UDF registration confirmed)** (weight: 0.15):
  Full credit (1.0): message does NOT contain "unknown function json_extract_string" (UDF is registered).
  Zero credit (0.0): message contains "unknown function json_extract_string" (pre-patch: UDF not registered at all).

---

## Edge Conditions

- **Query parsing fails before plan gate:** If the PrismQL parser rejects `json_extract_string(col, severity_id)` at parse time (before the plan gate), the error may be a parse error, not E-QUERY-045. This is acceptable only if the parse error correctly identifies the non-literal key argument as invalid. However, BC-2.11.025 §Error Cases explicitly specifies the plan-gate path; a parse-time rejection with a generic syntax error is not sufficient.

- **severity_id might be quoted in some MCP clients:** The evaluator must ensure `severity_id` in the query string is unquoted (column reference, not string literal). Double-check the raw query bytes before execution.

- **MCP error envelope nesting:** Some MCP implementations nest the prism error inside a JSON-encoded text field. The evaluator must parse `result.content[0].text` if the outer JSON-RPC shows `result`, then look for the error in the inner envelope.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-JEX-001-001 (satisfaction: X.XX) — json_extract_string non-literal key did not return E-QUERY-045(a) structured error; check (a) UDF registration at engine construction (BC-2.11.025 §Postconditions + ADR-066 §E) and (b) check_json_extract_key_literal plan gate before DataFusion execution (BC-2.11.025 EC-11-025-006 + ADR-066 §B3)"`

Do NOT disclose: the actual error code observed, the actual error message text, or whether
the failure was in UDF registration vs plan gate.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-JSON-EXTRACT-UDF-001 branch. No DTU. Synthetic query with non-literal column reference as second argument. |
| corpus_size | Single MCP query call; plan-time rejection path; zero sensor contact |
| known_edge_cases | PrismQL parser rejects non-literal key at parse time (before plan gate): acceptable only if the error is about non-literal key specifically, not generic syntax. severity_id column reference: must be unquoted in the query string |
| false_positive_threshold | Near-zero: E-QUERY-045 in the error message is the post-fix discriminating signal; pre-patch gives DataFusion unknown-function or no UDF registration error |
| false_negative_threshold | Near-zero: plan gate either fires (E-QUERY-045) or doesn't; there is no ambiguous middle state |

**Known-good corpus:** post-fix binary with UDF registered and plan gate active. Expected: -32602 with E-QUERY-045(a) message; no "unknown function" reference.

**Known-problematic corpus:** pre-patch develop HEAD. Expected: error referencing "unknown function json_extract_string" (UDF not registered); no E-QUERY-045.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w3-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-040 group for S-JSON-EXTRACT-UDF-001. Combined UDF-registration probe + literal-key plan gate security gate. Non-literal key → E-QUERY-045(a). Wire-level -32602 + E-QUERY-045 message assertions. No DTU required. BC-2.11.025 EC-11-025-006 + ADR-066 §B3. SINGLE-USE HIDDEN. |
