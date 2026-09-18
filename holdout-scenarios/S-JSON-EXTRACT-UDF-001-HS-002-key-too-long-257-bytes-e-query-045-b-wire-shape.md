---
document_type: holdout-scenario
level: L3
id: "HS-JEX-001-002"
title: "json_extract_string with 257-byte literal key returns E-QUERY-045(b) with key_len and max_len in error message"
category: "security-gate"
must_pass: true
priority: P1
epic_id: "E-BETA3-REMEDIATION"
story_source: "S-JSON-EXTRACT-UDF-001"
version: "1.1"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-JSON-EXTRACT-UDF-001 (HS-040 group). CWE-400 key-length cap: json_extract_string with a 257-byte literal key string is rejected at plan time with E-QUERY-045(b). Prism MCP query-validation errors use the ratified isError:true tool-result convention (NOT top-level error.code:-32602). Error message in result.content[0].text must contain both '257' and '256'. Uses SQL-projection form (SELECT ... FROM ...) — NOT pipe+SELECT which is malformed. NO DTU required. Test-writer and implementer must NOT read this file."
---

# HS-JEX-001-002: json_extract_string with 257-byte literal key returns E-QUERY-045(b) with key_len and max_len in error message

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-JSON-EXTRACT-UDF-001 (HS-040 group)
**Must Pass:** YES (P1 — important gate; not P0 but required for merge)
**BC Traced:** BC-2.11.025 §Error Cases E-QUERY-045(b) + EC-11-025-007
  "json_extract_string key is 257 bytes, which exceeds the 256-byte maximum (CWE-400). Message format: `E-QUERY-045: json_extract_string key is 257 bytes, which exceeds the 256-byte maximum (CWE-400).`"
  ADR-066 §B3 + §D3 + §F: 256-byte cap; plan-time rejection; zero per-row overhead.
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
    "content": [{"type": "text", "text": "ERROR: [validation] - E-QUERY-045: json_extract_string key is 257 bytes, which exceeds the 256-byte maximum (CWE-400). Reduce the json_extract_string key to 256 UTF-8 bytes or fewer."}],
    "structuredContent": {
      "error": {
        "code": "E-QUERY-045",
        "message": "E-QUERY-045: json_extract_string key is 257 bytes, which exceeds the 256-byte maximum (CWE-400).",
        "category": "validation",
        ...
      }
    }
  }
}
```

**The outer JSON-RPC wrapper uses `result`, NOT `error`.** The `-32602 INVALID_PARAMS` classification in the error taxonomy is an internal `map_prism_error` category; it does NOT appear at the JSON-RPC protocol level for tool-call responses. A query with a valid MCP parameter structure but an invalid PrismQL plan (including a too-long key) always returns `result.isError=true`.

Infrastructure failures (injection rejection) use top-level `error.code:-32602`. Query-validation errors (E-QUERY-NNN) use tool-result `result.isError=true`.

---

## Scenario

This scenario tests the CWE-400 (Uncontrolled Resource Consumption) key-length cap: a literal key string of exactly 257 UTF-8 bytes must be rejected at plan time before any DataFusion execution or sensor fan-out.

**Why 257 bytes (boundary test):**

The cap is 256 bytes (inclusive). A 256-byte key must be ACCEPTED; a 257-byte key must be REJECTED. The boundary assertion is more discriminating than a very long key: it confirms the exact threshold (256, not 255 or 512). This scenario uses 257 bytes (one byte over the limit) to test the boundary precisely.

**Exact query to issue (SQL projection form — correct syntax):**

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query","arguments":{"query":"SELECT json_extract_string(raw_extensions, 'aaaaaaa...a') FROM claroty_alerts","client_id":"<test_client>"}}}
```

Where the single-quoted string `'aaa...a'` contains exactly 257 lowercase 'a' characters. Do NOT use pipe+SELECT form (`FROM t | SELECT ...`) which is malformed PrismQL syntax.

**Pre-patch behavior (what this scenario catches):**

- **Pre-patch state 1 (UDF not registered):** `result.isError=true` with DataFusion "unknown function json_extract_string" message. No length check.
- **Pre-patch state 2 (UDF registered, no length gate):** Query proceeds to execution and either succeeds (returning null for JSON key lookup of a 257-char key) or fails at DataFusion runtime — but NOT with E-QUERY-045(b).

**Post-fix behavior:**

`result.isError=true` with `result.content[0].text` containing:
- `"E-QUERY-045"` (structured error taxonomy)
- `"257"` (the actual key length)
- `"256"` (the maximum allowed length)

**Four wire-level assertions:**

**Part A — response is a tool-result with isError:true (ratified transport convention):**
- Assert: JSON-RPC `result` key is present (not `error`).
- Assert: `result.isError == true` (query rejected).

**Part B — isError:true and result key (primary transport discriminator):**
- Assert: outer JSON-RPC wrapper has `result` (not `error`) AND `result.isError == true`.

**Part C — message contains E-QUERY-045:**
- Assert: `result.content[0].text` (or `result.structuredContent.error.message`) contains the string `"E-QUERY-045"`.

**Part D — message contains both 257 and 256 (boundary diagnostic):**
- Assert: the content text contains `"257"` (the actual key length reported).
- Assert: the content text contains `"256"` (the maximum key length cap reported).
  This confirms the message is from the E-QUERY-045(b) length-specific branch, not a generic E-QUERY-045(a) message.

**BDD supplement:**

**Given** prism is built from the S-JSON-EXTRACT-UDF-001 story branch
**And** prism is running in MCP stdio mode
**When** `tools/call query` is issued with SQL query `SELECT json_extract_string(raw_extensions, '<257-char-literal>') FROM claroty_alerts` via MCP stdio
  (where `<257-char-literal>` is a single-quoted string of exactly 257 ASCII characters; SQL projection form)
**Then** the response has `result` (not `error`)
**And** `result.isError == true`
**And** `result.content[0].text` contains `"E-QUERY-045"`
**And** `result.content[0].text` contains `"257"` and `"256"`

---

## Setup Instructions

1. Build prism from the S-JSON-EXTRACT-UDF-001 story branch.

2. NO DTU required. Plan gate fires at plan time before sensor contact.

3. Configure `prism.toml` with any test client that includes the Claroty sensor
   (so `claroty_alerts` and `raw_extensions` are recognized as table/column names).

4. Start prism in MCP stdio mode. Complete `initialize` handshake.
   SETUP-FAILURE condition: prism fails to start or MCP handshake fails.

5. Construct the 257-byte key literal. The key must be exactly 257 UTF-8 bytes.
   Use 257 ASCII characters (e.g., 257 lowercase 'a' characters). Verify the byte count
   before issuing the query — the assertion is sensitive to the exact byte count.

   Example query string (Python-style pseudocode for construction):
   ```
   key_257 = "a" * 257
   query = f"SELECT json_extract_string(raw_extensions, '{key_257}') FROM claroty_alerts"
   ```

6. Issue the query via MCP using SQL projection form:
   ```json
   {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query","arguments":{"query":"SELECT json_extract_string(raw_extensions, 'aaaaa...a') FROM claroty_alerts", "client_id":"<test_client>"}}}
   ```
   (The literal must be exactly 257 'a' characters; SQL projection form `SELECT ... FROM ...`.)
   Capture the full wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.025 | EC-11-025-007 / §Error Cases E-QUERY-045(b): 257-byte key exceeds 256-byte cap; rejected at plan time; message includes key_len=257 and max_len=256 | Parts A, B, C, D |
| ADR-066 | §B3: literal-key plan gate fires before execution; §D3: 256-byte cap (CWE-400); §F: message format `"key is {key_len} bytes, which exceeds the {max_len}-byte maximum (CWE-400)"` | Part D: 257 and 256 in message |
| BC-2.10.007 | §Postconditions: query-validation domain errors surface as isError:true tool-result (not top-level JSON-RPC error) | Parts A/B: transport envelope assertion |

---

## Verification Approach

1. Parse wire-level JSON-RPC response.

2. **Part A:** Assert `result` key present (not `error`). Assert `result.isError == true`.
   If `error.code` is present: note this is an infrastructure-level routing failure (unexpected for a query-validation error path). Record the code.
   If `result.isError == false` or absent: FAIL (256-byte gate not implemented; UDF executed the 257-char key lookup).

3. **Part B:** Confirm outer JSON-RPC uses `result` AND `result.isError == true`. This is the primary transport discriminator.

4. **Part C:** Extract `result.content[0].text`. Search for substring `"E-QUERY-045"`.
   Also check `result.structuredContent.error.message` if content text is not available.
   If absent: FAIL (plan gate not generating structured E-QUERY-045 code in message).

5. **Part D:** In `result.content[0].text` (or structuredContent.error.message), search for the substring `"257"` (key length) AND the substring `"256"` (max length).
   - If `"257"` absent: the message may be a generic E-QUERY-045 without the specific length diagnostic.
   - If `"256"` absent: the cap value is not being reported in the message.
   Both `"257"` and `"256"` must be present for full credit.

6. All assertions on serialized wire bytes.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response is isError:true tool-result (prerequisite — transport envelope)** (weight: 0.10):
  Full credit (1.0): `result` key present AND `result.isError == true` (query correctly rejected via ratified tool-result convention).
  Zero credit (0.0): query returned a successful result (length gate entirely absent). SETUP-FAILURE if prism startup failed.

- **isError:true and result key present (Part B — primary transport discriminator)** (weight: 0.35):
  Full credit (1.0): outer JSON-RPC wrapper has `result` (not `error`) AND `result.isError == true`.
  Half credit (0.5): `result.isError == true` confirmed but evaluator could not determine outer key structure with certainty.
  Zero credit (0.0): `result` absent, `error` present instead (wrong transport convention), or `result.isError == false`/absent.

- **E-QUERY-045 in content text (Part C)** (weight: 0.30):
  Full credit (1.0): `"E-QUERY-045"` present in `result.content[0].text` (or structuredContent.error.message).
  Zero credit (0.0): absent (gate active but not generating E-QUERY-045 code, or wrong error path).

- **257 and 256 both in content text (Part D — length diagnostic)** (weight: 0.25):
  Full credit (1.0): both `"257"` and `"256"` present in the content text.
  Half credit (0.5): one of the two present (partial message implementation).
  Zero credit (0.0): neither present (generic E-QUERY-045 without length info, or wrong branch).

---

## Edge Conditions

- **Query syntax:** Use SQL projection form `SELECT json_extract_string(raw_extensions, '<257-char-literal>') FROM claroty_alerts`. Do NOT use `FROM claroty_alerts | SELECT json_extract_string(...)` (pipe+SELECT is malformed PrismQL — mixes pipe syntax with SQL SELECT keywords and produces a parse error rather than E-QUERY-045).

- **256-byte key must NOT be rejected (boundary correctness):** The evaluator should optionally
  issue a second query with exactly 256 'a' characters to confirm the boundary is inclusive (256 = max_len = accepted, 257 > max_len = rejected). A 256-byte key query should NOT return E-QUERY-045(b) — it should succeed or return sensor-layer results. This is an optional verification; the primary assertion uses 257 bytes.

- **UTF-8 multi-byte characters:** The 256-byte cap is on UTF-8 BYTES, not characters. The scenario
  uses ASCII characters (1 byte each) to avoid ambiguity. If the implementation counts characters
  instead of bytes, a multi-byte character test would expose the gap — but that is out of scope here.

- **Query string escaping in JSON:** The 257 'a' characters must be delivered as a JSON string value
  in the `query` field. Standard JSON encoding: surround with double quotes, no special escaping needed
  for repeated 'a' characters.

- **Message format may vary slightly:** The exact message format per ADR-066 §F is:
  `"E-QUERY-045: json_extract_string key is 257 bytes, which exceeds the 256-byte maximum (CWE-400)."`
  The assertion looks for the numeric substrings "257" and "256" within the content text, not the exact
  full string, to tolerate minor formatting variations.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-JEX-001-002 (satisfaction: X.XX) — 257-byte literal key did not return E-QUERY-045(b) with length diagnostic; check check_json_extract_key_literal plan gate 256-byte cap (CWE-400) implementation and message format with key_len/max_len substitution (BC-2.11.025 EC-11-025-007 + ADR-066 §B3 + §D3 + §F)"`

Do NOT disclose: the specific error message text, the transport envelope shape observed, or whether
the failure was in the gate itself vs the message formatting.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-JSON-EXTRACT-UDF-001 branch. No DTU. Synthetic query with exactly 257-byte literal key. SQL projection form. |
| corpus_size | Single MCP query call; plan-time rejection; zero sensor contact. Boundary test at the 256/257 byte threshold. |
| known_edge_cases | Key byte count off-by-one: must be exactly 257 bytes (not 256, not 258). 256-byte key should be accepted (boundary inclusive). UTF-8 encoding: use ASCII-only to ensure 1 byte per character. Transport: isError:true tool-result, NOT top-level error.code:-32602. SQL form only: SELECT ... FROM ... not FROM ... | SELECT ... |
| false_positive_threshold | Near-zero: the specific E-QUERY-045 message with both "257" and "256" is only emitted by the implemented plan gate |
| false_negative_threshold | Near-zero: pre-patch (no UDF) gives "unknown function" not E-QUERY-045(b); no plan gate gives success result not an error |

**Known-good corpus:** post-fix binary with 256-byte cap gate implemented. Expected: isError:true with E-QUERY-045 and both "257"/"256" in content text.

**Known-problematic corpus:** pre-patch develop HEAD. Expected: isError:true with DataFusion "unknown function json_extract_string" (UDF not registered; no length check attempted).

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | beta3-w3-holdout-correction | 2026-09-17 | product-owner | **Scenario-authoring correction (two defects found by holdout-evaluator run on 83fa7ff51).** (1) Transport-envelope defect: rubric Part B asserted `error.code == -32602` at the top-level JSON-RPC level. Prism's ratified product-wide convention for ALL query-validation domain errors (E-QUERY-NNN) is `result.isError=true` tool-result (server.rs `prism_error_to_structured_call_result` path), NOT a top-level JSON-RPC `error.code`. Fix: Part B now asserts `result` key present + `result.isError == true`; E-QUERY-045 and length values assert in `result.content[0].text`. (2) Query syntax defect: scenario used `FROM claroty_alerts | SELECT json_extract_string(raw_extensions, '<257-char-literal>')` (pipe+SELECT — malformed PrismQL). Fix: corrected to SQL projection form `SELECT json_extract_string(raw_extensions, '<257-char-literal>') FROM claroty_alerts`. Transport Convention section added. Rubric rewritten to reflect isError:true assertion. used: false (re-authored for re-evaluation). |
| 1.0 | beta3-w3-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-040 group for S-JSON-EXTRACT-UDF-001. CWE-400 key-length cap boundary test: 257-byte literal key rejected with E-QUERY-045(b) carrying key_len=257 and max_len=256 in message. Wire-level -32602 + E-QUERY-045 + "257" + "256" assertions. No DTU required. BC-2.11.025 EC-11-025-007 + ADR-066 §D3 + §F. SINGLE-USE HIDDEN. |
