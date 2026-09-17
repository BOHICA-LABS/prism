---
document_type: holdout-scenario
level: L3
id: "HS-JEX-001-003"
title: "json_extract_string with valid literal key does not produce DataFusion unknown-function error; UDF registration probe"
category: "behavioral-correctness"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-JSON-EXTRACT-UDF-001 (HS-040 group). UDF registration probe using a VALID query syntax (valid literal key, no gate violation): if UDF NOT registered, error contains 'unknown function json_extract_string' (DataFusion plan-time); if UDF IS registered, error is either sensor-related (runtime, no data without DTU) OR success (if sensor available). Discriminating assertion: error message does NOT contain 'unknown function' paired with 'json_extract_string'. NO DTU required. The scenario also validates the dot-in-key literal behavior (AC-010): 'a.b' key is accepted by the plan gate (no E-QUERY-045), confirming the gate does not over-reject valid literal keys with dots. Test-writer and implementer must NOT read this file."
---

# HS-JEX-001-003: json_extract_string with valid literal key does not produce DataFusion unknown-function error; UDF registration probe

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-JSON-EXTRACT-UDF-001 (HS-040 group)
**Must Pass:** YES (P1 — important gate; not P0 but required for merge)
**BC Traced:**
  - BC-2.11.025 §Postconditions: UDF registered under name `"json_extract_string"` at engine construction with input types `(Utf8, Utf8)`, return type `Utf8` (nullable), `Volatility::Immutable`.
  - BC-2.11.025 EC-11-025-009 (AC-010): `'a.b'` key literal treated as top-level key lookup, NOT a nested JSONPath expression. Plan gate must NOT reject a valid dotted literal key.
**Gate:** Story-level holdout gate (HS-040) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario is the UDF registration probe: it confirms that a syntactically valid `json_extract_string` call with a proper literal key passes through the plan gate (no plan-gate rejection for valid syntax) and reaches either execution (if sensor data is available) or a sensor error (if no DTU). Either outcome is acceptable; what is NOT acceptable is a DataFusion "unknown function" error.

This scenario also serves as an implicit AC-010 test: the key used is `'a.b'` (a literal string containing a dot). The plan gate must NOT interpret this as a nested path expression and must NOT return E-QUERY-045. A valid literal key with a dot must be accepted by the gate, not rejected.

**Two valid outcomes post-fix:**

1. **Success (sensor available):** The query returns a non-error result with rows containing the extracted string column. This happens if the Claroty DTU is running or the live sensor is accessible.

2. **Sensor error (no DTU, no live sensor):** The query fails at runtime with a sensor-related error (e.g., E-SENSOR-NNN for connection refused, or a similar sensor-unavailable error). This is acceptable because it proves the UDF is registered (plan gate passed) and the error is at the sensor contact stage, NOT at the UDF-registration plan stage.

**Pre-patch behavior (what this scenario catches):**

DataFusion encounters `json_extract_string` at plan time and does not recognize it as a registered UDF. The response is a JSON-RPC error with a message containing `"unknown function"` and `"json_extract_string"`. This error occurs BEFORE any sensor contact — it is a plan-time failure, not a runtime failure.

**Three assertions:**

**Part A — response error (if any) does NOT contain "unknown function json_extract_string":**
- This is the primary discriminating assertion.
- If the response is an error: assert `error.message` does NOT contain both `"unknown function"` and `"json_extract_string"` together (case-insensitive for "unknown function").
- If the response is a success: Part A passes automatically (UDF is clearly registered and executed).

**Part B — plan gate did NOT fire for a valid literal key with a dot:**
- Assert: the response (whether success or sensor error) is NOT an E-QUERY-045 error.
- Specifically: `error.message` (if error present) does NOT contain `"E-QUERY-045"`.
- This confirms the gate correctly allows `'a.b'` as a valid literal key (AC-010 boundary).

**Part C — error is sensor-related if no data available (acceptable outcome):**
- If the response is an error: assert it is a runtime/sensor error (not a plan-time UDF registration error). Acceptable error categories: sensor unavailable, connection refused, fetch timeout, E-SENSOR-NNN. The key distinction is that the error happens AFTER the plan gate passes, meaning the UDF was recognized.

**BDD supplement (sensor unavailable path):**

**Given** prism is built from the S-JSON-EXTRACT-UDF-001 story branch
**And** prism is running in MCP stdio mode (no DTU required)
**When** `tools/call query {"query": "FROM claroty_alerts | SELECT json_extract_string(raw_extensions, 'a.b')", "client_id": "<test_client>"}` is issued via MCP stdio
**Then** EITHER the response is a non-error success (sensor available)
     **OR** the response is a sensor-related error (connection refused, E-SENSOR-NNN, etc.)
**And** in either case, the response does NOT contain "unknown function json_extract_string"
**And** the response does NOT contain "E-QUERY-045"

---

## Setup Instructions

1. Build prism from the S-JSON-EXTRACT-UDF-001 story branch.

2. NO DTU required (see acceptable outcomes above). The test works with or without a running DTU.

3. Configure `prism.toml` with a test client that includes the Claroty sensor in scope.
   The sensor connection does not need to be live for this test to be discriminating.

4. Start prism in MCP stdio mode. Complete `initialize` handshake.
   SETUP-FAILURE condition: prism fails to start or MCP handshake fails.

5. Issue the query with the dotted literal key `'a.b'`:
   ```
   {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query","arguments":{"query":"FROM claroty_alerts | SELECT json_extract_string(raw_extensions, 'a.b')","client_id":"<test_client>"}}}
   ```
   Note: `'a.b'` is a quoted string literal with a dot — a valid literal key per AC-010.
   Capture the full wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.025 | §Postconditions: UDF registered at engine construction under name "json_extract_string" with (Utf8, Utf8) input signature | Part A: no "unknown function" error |
| BC-2.11.025 | EC-11-025-009 / AC-010: `'a.b'` dot-containing key is a valid top-level literal key; plan gate must NOT reject it (no E-QUERY-045) | Part B: no E-QUERY-045 for dotted literal |
| ADR-066 | §E: UDF registration contract at QueryEngine::new / SessionContext construction | Part A: UDF known to DataFusion |

---

## Verification Approach

1. Parse wire-level JSON-RPC response.

2. Determine outcome type:
   - **Success:** `result` key present, no `error`. Parts A and B pass by definition.
   - **Error:** `error` key present. Proceed to assertions.

3. **Part A — not an unknown-function error:**
   If error: extract `error.message`.
   Check for the pattern: message contains ("unknown function" OR "unregistered") AND "json_extract_string".
   - If pattern found: record FAIL (pre-patch behavior — UDF not registered).
   - If pattern absent: record PASS for Part A.

4. **Part B — not E-QUERY-045 for the dotted literal key:**
   If error: check `error.message` for `"E-QUERY-045"`.
   - If E-QUERY-045 present: record FAIL (plan gate over-rejected a valid literal key with a dot — AC-010 gate error).
   - If E-QUERY-045 absent: record PASS for Part B.

5. **Part C — characterize the error (informational):**
   If error: note whether it appears to be sensor-related (connection error, E-SENSOR-NNN, timeout)
   vs. a plan-time error. This characterization is informational, not a PASS/FAIL criterion.
   If the error is a sensor error: this confirms UDF registration is correct and plan gate passed.

6. All assertions on the serialized wire bytes.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Not an unknown-function error (Part A — primary UDF-registration discriminator)** (weight: 0.65):
  Full credit (1.0): no "unknown function json_extract_string" in response.
  Zero credit (0.0): "unknown function json_extract_string" found in error message (pre-patch: UDF not registered).
  Note: if response is a non-error success, this dimension gets full credit automatically.

- **Not an E-QUERY-045 for dotted literal key (Part B — plan-gate boundary / AC-010)** (weight: 0.25):
  Full credit (1.0): E-QUERY-045 absent from response (plan gate did not over-reject valid dotted key).
  Zero credit (0.0): E-QUERY-045 present (plan gate incorrectly rejected `'a.b'` as a non-literal or as a nested path — implementation bug).

- **Response makes sense (Part C — sanity check)** (weight: 0.10):
  Full credit (1.0): response is either a success or a recognizable sensor/runtime error.
  Zero credit (0.0): response is a completely unexpected panic, internal server error, or malformed JSON.

---

## Edge Conditions

- **Live sensor IS available (DTU running or live monroe sensor configured):** The query
  executes end-to-end. The `'a.b'` key lookup returns null for all rows (no JSON object
  key named literally `"a.b"` exists in real Claroty data). Null results are acceptable.
  The evaluator should check the extracted column is present in the rows (even if all null).

- **'a.b' in query string parsing:** The PrismQL parser must accept `'a.b'` as a string literal
  (single-quoted, with dot inside). If the parser rejects dotted literals as invalid syntax
  (a parse error before the plan gate), this is a different failure mode. Record separately.

- **Pipe syntax `|` in query:** The pipe operator `|` is standard PrismQL pipe syntax. If the
  parser rejects the pipe-mode syntax, this indicates a different issue. Record separately.

- **Column alias in result:** DataFusion may name the output column
  `json_extract_string(raw_extensions,Utf8("a.b"))` in the result schema. This is acceptable.
  The evaluator does not need to assert on the exact output column name.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-JEX-001-003 (satisfaction: X.XX) — valid literal-key json_extract_string call produced unexpected response; check (a) UDF registration in QueryEngine::new / SessionContext construction (BC-2.11.025 §Postconditions + ADR-066 §E) and (b) plan gate must NOT reject literal keys containing dots (BC-2.11.025 EC-11-025-009 AC-010)"`

Do NOT disclose: the specific error message content observed, whether the sensor was
reachable, or the query result rows.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-JSON-EXTRACT-UDF-001 branch. No DTU required. Valid syntactic query with dotted literal key `'a.b'`. |
| corpus_size | Single MCP query call; either plan gate passes (no E-QUERY-045) and reaches sensor stage (or success if sensor available) |
| known_edge_cases | Live sensor available: query succeeds with null values (no "a.b" top-level key in real Claroty JSON). No sensor: sensor error expected and acceptable. Parse error on dotted key: different failure mode, record separately. |
| false_positive_threshold | Near-zero: absence of "unknown function json_extract_string" in the response is the discriminating fact; only a registered UDF passes plan time without this error |
| false_negative_threshold | Low: pre-patch always emits "unknown function json_extract_string"; post-fix never does (UDF registered) |

**Known-good corpus:** post-fix binary with UDF registered and plan gate allowing dotted literal keys. Expected: non-error success OR sensor error — never "unknown function json_extract_string".

**Known-problematic corpus:** pre-patch develop HEAD. Expected: "unknown function json_extract_string" error from DataFusion at plan time.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w3-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-040 group for S-JSON-EXTRACT-UDF-001. UDF registration probe using valid dotted literal key 'a.b'. Two acceptable outcomes: success OR sensor error. Discriminating: no "unknown function json_extract_string" (UDF registered). AC-010 boundary: plan gate must accept dotted literals. No DTU required. BC-2.11.025 §Postconditions + EC-11-025-009 + ADR-066 §E. SINGLE-USE HIDDEN. |
