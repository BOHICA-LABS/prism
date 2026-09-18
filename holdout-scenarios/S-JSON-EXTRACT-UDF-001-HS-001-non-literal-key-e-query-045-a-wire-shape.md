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
version: "1.1"
status: active
used: true
last_evaluated: 2026-09-17
last_eval_satisfaction: 1.00
single_use: true
producer: product-owner
timestamp: "2026-09-17T00:00:00Z"
modified: "2026-09-17"
phase: 3
inputs:
  - ".factory/stories/S-JSON-EXTRACT-UDF-001-json-extract-scalar-udf.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.025-json-extract-string-scalar-udf.md"
  - ".factory/specs/architecture/decisions/ADR-066-json-extract-scalar-udf.md"
input-hash: "ada57c4"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-JSON-EXTRACT-UDF-001 (HS-040 group). Combined UDF-registration and plan-gate security test: non-literal key column reference rejected with E-QUERY-045(a) at plan time. Prism MCP tool-level query-validation errors use the ratified isError:true tool-result convention (NOT top-level JSON-RPC error.code:-32602). The -32602 classification is internal to map_prism_error; the wire shape is result.isError=true with E-QUERY-045 in result.content[0].text. Uses SQL-projection form (SELECT ... FROM ...) — NOT pipe+SELECT which is malformed syntax. NO DTU required. Test-writer and implementer must NOT read this file."
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

## Transport Envelope Convention (RATIFIED — read before evaluating)

Prism's `query` tool uses the **MCP tool-result error convention** for all query-validation domain errors. When a query fails plan-time validation (E-QUERY-NNN family), the wire-level response is:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "isError": true,
    "content": [{"type": "text", "text": "ERROR: [validation] - E-QUERY-045: ..."}],
    "structuredContent": {
      "error": {
        "code": "E-QUERY-045",
        "message": "E-QUERY-045: ...",
        "category": "validation",
        ...
      }
    }
  }
}
```

**The outer JSON-RPC wrapper uses `result`, NOT `error`.** The `-32602 INVALID_PARAMS` classification in the error taxonomy is an internal `map_prism_error` category; it does NOT appear at the JSON-RPC protocol level for tool-call responses. Infrastructure failures (injection rejection, server crashes) return `error.code:-32602`, but those are not query-validation errors.

The canonical -32602 wire shape (`error.code=-32602` at top level) only appears for:
- Malformed MCP request parameters (e.g., wrong field name/type in the JSON-RPC params)
- Injection rejection from the safety scanner

A query with a valid MCP parameter structure but an invalid PrismQL plan always returns `result.isError=true`.

---

## Scenario

This scenario serves a dual purpose: it simultaneously verifies that the UDF is REGISTERED and that the literal-key PLAN GATE is active.

**Why this is both a UDF-registration probe and a security gate:**

The `json_extract_string` UDF must be registered in the DataFusion `SessionContext` per ephemeral context (ADR-066 §E). If it is NOT registered, DataFusion returns a "unknown function" plan-time error — not `E-QUERY-045`. If the UDF IS registered but the plan gate is absent, the non-literal key would be passed to the UDF at runtime (which is a security gap — CWE-400, ADR-066 §B3).

The compound behavior — UDF registered AND plan gate active — is discriminated by the SPECIFIC content in the tool-result:
- **Pre-patch state 1 (UDF not registered):** `result.isError=true` with message containing "unknown function" and "json_extract_string" (DataFusion's own error, surfaced through the structured error envelope).
- **Pre-patch state 2 (UDF registered, no plan gate):** Either runtime DataFusion error or unexpected successful result (no structured E-QUERY-045).
- **Post-fix:** `result.isError=true` with `result.content[0].text` containing `"E-QUERY-045"` and NOT containing `"unknown function json_extract_string"`.

**Query to issue (SQL projection form — correct syntax):**

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query","arguments":{"query":"SELECT json_extract_string(raw_extensions, severity_id) FROM claroty_alerts","client_id":"<test_client>"}}}
```

Note: `severity_id` has NO quotes — it is a column reference, not a string literal. SQL projection form (`SELECT ... FROM ...`) is used. Do NOT use pipe+SELECT form (`FROM t | SELECT ...`) which is malformed syntax.

**Three wire-level assertions on the serialized MCP response:**

**Part A — response is a tool-result with isError:true (ratified transport convention):**
- Assert: response has `result` key (not `error`).
- Assert: `result.isError == true`.
  If `result.isError` is false or absent: FAIL (plan gate not active — query succeeded when it should have been rejected).
  If `error` key is present instead of `result`: record FAIL (infrastructure-level error; unexpected for a query-validation path).

**Part B — E-QUERY-045 in content:**
- Assert: `result.content[0].text` (or `result.structuredContent.error.message`) contains the string `"E-QUERY-045"`.
- Assert: the error text does NOT contain `"unknown function"` paired with `"json_extract_string"` within the same string (which would indicate the UDF is not registered — the pre-patch state).

**Part C — error is about plan gate, not UDF registration:**
- Assert: the error text does not contain the pattern `"unknown function json_extract_string"` or `"unregistered" ... "json_extract_string"`.
- This confirms the UDF is registered (DataFusion knows about it) even though the query is rejected for a different reason (non-literal key).

**BDD supplement:**

**Given** prism is built from the S-JSON-EXTRACT-UDF-001 story branch
**And** prism is running in MCP stdio mode with any Claroty-capable client configuration
**When** `tools/call query {"query": "SELECT json_extract_string(raw_extensions, severity_id) FROM claroty_alerts", "client_id": "<test_client>"}` is issued via MCP stdio
  (note: `severity_id` here is a column reference, not a string literal — no quotes; SQL projection form used)
**Then** the response has `result` (not `error`)
**And** `result.isError == true`
**And** `result.content[0].text` contains `"E-QUERY-045"`
**And** `result.content[0].text` does NOT contain `"unknown function json_extract_string"`

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

5. Issue the non-literal-key query using SQL projection form:
   ```
   {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query","arguments":{"query":"SELECT json_extract_string(raw_extensions, severity_id) FROM claroty_alerts","client_id":"<test_client>"}}}
   ```
   Note: `severity_id` has NO quotes — it is a column reference, not a string literal.
   Capture the full wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.025 | EC-11-025-006 / §Error Cases E-QUERY-045(a): non-literal key rejected at plan time with PrismError::JsonExtractNonLiteralKey | Parts A, B, C |
| BC-2.11.025 | §Postconditions: UDF registered under name "json_extract_string" with (Utf8, Utf8) input types per ephemeral SessionContext | Part C: UDF registered (no unknown-function error) |
| ADR-066 | §B3: literal-key plan gate; §F: E-QUERY-045(a) message format | Part B: E-QUERY-045 in message |
| BC-2.10.007 | §Postconditions: query-validation domain errors surface as isError:true tool-result (not top-level JSON-RPC error) | Part A: transport envelope assertion |

---

## Verification Approach

1. Parse wire-level JSON-RPC response.

2. **Part A — transport envelope:**
   Assert `result` key is present (not `error`). Assert `result.isError == true`.
   If `error.code` is present: record the code and note this is an infrastructure-level routing failure (unexpected for a query-validation error path).
   If `result.isError == false` or `result` has no `isError`: FAIL (plan gate not active; query succeeded when it should have been rejected).

3. **Part B — E-QUERY-045 in content:**
   Extract `result.content[0].text`. Search for substring `"E-QUERY-045"`.
   Also check `result.structuredContent.error.message` if content text is not available.
   If absent: record FAIL (plan gate not returning structured E-QUERY-045; check whether the error is from DataFusion unknown-function path instead).

4. **Part C — not an unknown-function error:**
   In `result.content[0].text` (or the structuredContent message), assert the text does NOT contain the pattern `"unknown function"` paired with `"json_extract_string"`.
   A message like "unknown function json_extract_string" is the pre-patch behavior (UDF not registered).
   Post-fix: the message must be about E-QUERY-045 (plan gate), not about UDF registration.

5. All assertions on the serialized wire bytes.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response is isError:true tool-result (prerequisite — transport envelope)** (weight: 0.10):
  Full credit (1.0): `result` key present, `result.isError == true` (query correctly rejected via ratified tool-result convention).
  Zero credit (0.0): query returned a successful result (plan gate entirely absent — severe defect). SETUP-FAILURE if prism didn't start.

- **isError:true and result key present (Part A — primary plan-gate transport discriminator)** (weight: 0.40):
  Full credit (1.0): outer JSON-RPC wrapper has `result` (not `error`) AND `result.isError == true`.
  Half credit (0.5): `result.isError == true` is confirmed but the evaluator could not determine the outer key structure with certainty.
  Zero credit (0.0): `result` absent, `error` present instead (wrong transport convention), or `result.isError == false`/absent (gate didn't fire).

- **E-QUERY-045 in content text (Part B — structured error taxonomy)** (weight: 0.35):
  Full credit (1.0): `result.content[0].text` (or structuredContent.error.message) contains `"E-QUERY-045"`.
  Zero credit (0.0): E-QUERY-045 absent from all content (plan gate may be returning unstructured DataFusion error).

- **Not an unknown-function error (Part C — UDF registration confirmed)** (weight: 0.15):
  Full credit (1.0): content text does NOT contain "unknown function json_extract_string" (UDF is registered).
  Zero credit (0.0): content text contains "unknown function json_extract_string" (pre-patch: UDF not registered at all).

---

## Edge Conditions

- **Query syntax:** Use SQL projection form `SELECT json_extract_string(raw_extensions, severity_id) FROM claroty_alerts`. Do NOT use `FROM claroty_alerts | SELECT json_extract_string(...)` (pipe+SELECT is malformed PrismQL — this mixes pipe syntax with SQL SELECT keywords and produces a parse error rather than a plan-gate rejection).

- **severity_id might be quoted in some MCP clients:** The evaluator must ensure `severity_id` in the query string is unquoted (column reference, not string literal). Double-check the raw query bytes before execution.

- **Structuredness of content:** The E-QUERY-045 text appears in both `result.content[0].text` (plain-text form: "ERROR: [validation] - E-QUERY-045: ...") and `result.structuredContent.error.message` (the structured form). Assert on `content[0].text` as the primary check; `structuredContent.error.message` as the secondary.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-JEX-001-001 (satisfaction: X.XX) — json_extract_string non-literal key did not return E-QUERY-045(a) structured error; check (a) UDF registration at engine construction (BC-2.11.025 §Postconditions + ADR-066 §E) and (b) check_json_extract_key_literal plan gate before DataFusion execution (BC-2.11.025 EC-11-025-006 + ADR-066 §B3)"`

Do NOT disclose: the actual error message text, the transport envelope shape observed, or whether
the failure was in UDF registration vs plan gate.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-JSON-EXTRACT-UDF-001 branch. No DTU. Synthetic query with non-literal column reference as second argument. SQL projection form. |
| corpus_size | Single MCP query call; plan-time rejection path; zero sensor contact |
| known_edge_cases | PrismQL SQL form only: `SELECT ... FROM ...` not `FROM ... \| SELECT ...`. severity_id column reference: must be unquoted in the query string. Transport convention: isError:true tool-result, NOT top-level error.code:-32602 |
| false_positive_threshold | Near-zero: E-QUERY-045 in the content text is the post-fix discriminating signal; pre-patch gives DataFusion unknown-function or no UDF registration error |
| false_negative_threshold | Near-zero: plan gate either fires (E-QUERY-045 in isError:true content) or doesn't; there is no ambiguous middle state |

**Known-good corpus:** post-fix binary with UDF registered and plan gate active. Expected: isError:true with E-QUERY-045 message; no "unknown function" reference.

**Known-problematic corpus:** pre-patch develop HEAD. Expected: isError:true with error referencing "unknown function json_extract_string" (UDF not registered); no E-QUERY-045.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | beta3-w3-holdout-correction | 2026-09-17 | product-owner | **Scenario-authoring correction (two defects found by holdout-evaluator run on 83fa7ff51).** (1) Transport-envelope defect: rubric Part A asserted `error.code == -32602` at the top-level JSON-RPC level. Prism's ratified product-wide convention for ALL query-validation domain errors (E-QUERY-NNN) is `result.isError=true` tool-result (server.rs `prism_error_to_structured_call_result` path), NOT a top-level JSON-RPC `error.code`. The `-32602` classification is an internal `map_prism_error` code, not a wire-level protocol code. Fix: Part A now asserts `result` key present + `result.isError == true`; `result.content[0].text` contains E-QUERY-045. (2) Query syntax defect: scenario used `FROM claroty_alerts \| SELECT json_extract_string(raw_extensions, severity_id)` (pipe+SELECT — malformed PrismQL). Fix: corrected to SQL projection form `SELECT json_extract_string(raw_extensions, severity_id) FROM claroty_alerts`. Transport Convention section added to scenario body. Rubric rewritten to reflect isError:true assertion instead of error.code:-32602 assertion. used: false (re-authored for re-evaluation). |
| 1.0 | beta3-w3-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-040 group for S-JSON-EXTRACT-UDF-001. Combined UDF-registration probe + literal-key plan gate security gate. Non-literal key → E-QUERY-045(a). Wire-level -32602 + E-QUERY-045 message assertions. No DTU required. BC-2.11.025 EC-11-025-006 + ADR-066 §B3. SINGLE-USE HIDDEN. |
