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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-JSON-EXTRACT-UDF-001 (HS-040 group). CWE-400 key-length cap: json_extract_string with a 257-byte literal key string is rejected at plan time with E-QUERY-045(b). Error message must contain both the actual key length ('257') and the maximum allowed length ('256') as per ADR-066 §F message format. Pre-patch: DataFusion 'unknown function' error (UDF not registered; no E-QUERY-045). Post-fix: -32602 with E-QUERY-045 message carrying both lengths. NO DTU required. Wire-level assertion on error code and message substrings. Test-writer and implementer must NOT read this file."
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

## Scenario

This scenario tests the CWE-400 (Uncontrolled Resource Consumption) key-length cap: a literal key string of exactly 257 UTF-8 bytes must be rejected at plan time before any DataFusion execution or sensor fan-out.

**Why 257 bytes (boundary test):**

The cap is 256 bytes (inclusive). A 256-byte key must be ACCEPTED; a 257-byte key must be REJECTED. The boundary assertion is more discriminating than a very long key: it confirms the exact threshold (256, not 255 or 512). This scenario uses 257 bytes (one byte over the limit) to test the boundary precisely.

**Exact query to issue:**

The second argument is a single-quoted string literal of exactly 257 ASCII characters (each ASCII character = 1 UTF-8 byte). The evaluator constructs this as 257 lowercase 'a' characters:

```
FROM claroty_alerts | SELECT json_extract_string(raw_extensions, 'aaaaaaa...a')
```

Where the quoted string `'aaa...a'` contains exactly 257 'a' characters.

**Pre-patch behavior (what this scenario catches):**

- **Pre-patch state 1 (UDF not registered):** DataFusion "unknown function json_extract_string" error. No length check.
- **Pre-patch state 2 (UDF registered, no length gate):** Query proceeds to execution and either succeeds (returning null for JSON key lookup of a 257-char key) or fails at DataFusion runtime — but NOT with E-QUERY-045(b).

**Post-fix behavior:**

JSON-RPC error with:
- `error.code == -32602`
- `error.message` contains `"E-QUERY-045"` 
- `error.message` contains `"257"` (the actual key length)
- `error.message` contains `"256"` (the maximum allowed length)

**Four wire-level assertions:**

**Part A — response is an error:**
- Assert: JSON-RPC `error` key is present (query rejected).
- Assert: no `result` key present.

**Part B — error.code == -32602:**
- Assert: `error.code` is exactly `-32602` (INVALID_PARAMS).

**Part C — message contains E-QUERY-045:**
- Assert: `error.message` contains the string `"E-QUERY-045"`.

**Part D — message contains both 257 and 256 (boundary diagnostic):**
- Assert: `error.message` contains `"257"` (the actual key length reported).
- Assert: `error.message` contains `"256"` (the maximum key length cap reported).
  This confirms the message is from the E-QUERY-045(b) length-specific branch, not a generic E-QUERY-045(a) message.

**BDD supplement:**

**Given** prism is built from the S-JSON-EXTRACT-UDF-001 story branch
**And** prism is running in MCP stdio mode
**When** `tools/call query` is issued with query `FROM claroty_alerts | SELECT json_extract_string(raw_extensions, '<257-char-literal>')` via MCP stdio
  (where `<257-char-literal>` is a single-quoted string of exactly 257 ASCII characters)
**Then** the response has `error` (not `result`)
**And** `error.code == -32602`
**And** `error.message` contains `"E-QUERY-045"`
**And** `error.message` contains `"257"` and `"256"`

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
   query = f"FROM claroty_alerts | SELECT json_extract_string(raw_extensions, '{key_257}')"
   ```

6. Issue the query via MCP:
   ```json
   {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query","arguments":{"query":"FROM claroty_alerts | SELECT json_extract_string(raw_extensions, 'aaaaa...a')", "client_id":"<test_client>"}}}
   ```
   (The literal must be exactly 257 'a' characters.)
   Capture the full wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.025 | EC-11-025-007 / §Error Cases E-QUERY-045(b): 257-byte key exceeds 256-byte cap; rejected at plan time; message includes key_len=257 and max_len=256 | Parts A, B, C, D |
| ADR-066 | §B3: literal-key plan gate fires before execution; §D3: 256-byte cap (CWE-400); §F: message format `"key is {key_len} bytes, which exceeds the {max_len}-byte maximum (CWE-400)"` | Part D: 257 and 256 in message |

---

## Verification Approach

1. Parse wire-level JSON-RPC response.

2. **Part A:** Assert `error` key present. If `result` present: FAIL (256-byte gate not implemented; UDF executed the 257-char key lookup).

3. **Part B:** Extract `error.code`. Assert exactly `-32602`.
   If different (e.g., -32600, -32603): likely DataFusion unknown-function or internal error — UDF registration or gate routing issue.

4. **Part C:** Extract `error.message`. Search for substring `"E-QUERY-045"`.
   If absent: FAIL (plan gate not generating structured E-QUERY-045 code in message).

5. **Part D:** In `error.message`, search for the substring `"257"` (key length) AND the substring `"256"` (max length).
   - If `"257"` absent: the message may be a generic E-QUERY-045 without the specific length diagnostic.
   - If `"256"` absent: the cap value is not being reported in the message.
   Both `"257"` and `"256"` must be present for full credit.

6. All assertions on serialized wire bytes.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response has error (prerequisite)** (weight: 0.10):
  Full credit (1.0): query rejected (error response).
  Zero credit (0.0): query returned a result (length gate entirely absent). SETUP-FAILURE if prism startup failed.

- **error.code == -32602 (Part B)** (weight: 0.35):
  Full credit (1.0): exactly -32602.
  Zero credit (0.0): different error code (wrong error routing or DataFusion unknown-function code).

- **E-QUERY-045 in message (Part C)** (weight: 0.30):
  Full credit (1.0): `"E-QUERY-045"` present in message.
  Zero credit (0.0): absent (gate active but not generating E-QUERY-045 code).

- **257 and 256 both in message (Part D — length diagnostic)** (weight: 0.25):
  Full credit (1.0): both `"257"` and `"256"` present in message.
  Half credit (0.5): one of the two present (partial message implementation).
  Zero credit (0.0): neither present (generic E-QUERY-045 without length info, or wrong branch).

---

## Edge Conditions

- **256-byte key must NOT be rejected (boundary correctness):** The evaluator should optionally
  issue a second query with exactly 256 'a' characters to confirm the boundary is inclusive (256 = max_len = accepted, 257 > max_len = rejected). A 256-byte key query should NOT return E-QUERY-045(b). This is an optional verification; the primary assertion uses 257 bytes.

- **UTF-8 multi-byte characters:** The 256-byte cap is on UTF-8 BYTES, not characters. The scenario
  uses ASCII characters (1 byte each) to avoid ambiguity. If the implementation counts characters
  instead of bytes, a multi-byte character test would expose the gap — but that is out of scope here.

- **Query string escaping in JSON:** The 257 'a' characters must be delivered as a JSON string value
  in the `query` field. Standard JSON encoding: surround with double quotes, no special escaping needed
  for repeated 'a' characters.

- **Message format may vary slightly:** The exact message format per ADR-066 §F is:
  `"E-QUERY-045: json_extract_string key is 257 bytes, which exceeds the 256-byte maximum (CWE-400)."`
  The assertion looks for the numeric substrings "257" and "256" within the message, not the exact
  full string, to tolerate minor formatting variations.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-JEX-001-002 (satisfaction: X.XX) — 257-byte literal key did not return E-QUERY-045(b) with length diagnostic; check check_json_extract_key_literal plan gate 256-byte cap (CWE-400) implementation and message format with key_len/max_len substitution (BC-2.11.025 EC-11-025-007 + ADR-066 §B3 + §D3 + §F)"`

Do NOT disclose: the specific error code observed, the exact message text, or whether
the failure was in the gate itself vs the message formatting.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-JSON-EXTRACT-UDF-001 branch. No DTU. Synthetic query with exactly 257-byte literal key. |
| corpus_size | Single MCP query call; plan-time rejection; zero sensor contact. Boundary test at the 256/257 byte threshold. |
| known_edge_cases | Key byte count off-by-one: must be exactly 257 bytes (not 256, not 258). 256-byte key should be accepted (boundary inclusive). UTF-8 encoding: use ASCII-only to ensure 1 byte per character. |
| false_positive_threshold | Near-zero: the specific E-QUERY-045 message with both "257" and "256" is only emitted by the implemented plan gate |
| false_negative_threshold | Near-zero: pre-patch (no UDF) gives "unknown function" not E-QUERY-045(b); no plan gate gives success result not an error |

**Known-good corpus:** post-fix binary with 256-byte cap gate implemented. Expected: -32602 with E-QUERY-045 and both "257"/"256" in message.

**Known-problematic corpus:** pre-patch develop HEAD. Expected: DataFusion "unknown function json_extract_string" (UDF not registered; no length check attempted).

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w3-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-040 group for S-JSON-EXTRACT-UDF-001. CWE-400 key-length cap boundary test: 257-byte literal key rejected with E-QUERY-045(b) carrying key_len=257 and max_len=256 in message. Wire-level -32602 + E-QUERY-045 + "257" + "256" assertions. No DTU required. BC-2.11.025 EC-11-025-007 + ADR-066 §D3 + §F. SINGLE-USE HIDDEN. |
