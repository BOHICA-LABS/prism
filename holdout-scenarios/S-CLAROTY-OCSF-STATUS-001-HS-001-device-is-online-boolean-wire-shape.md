---
document_type: holdout-scenario
level: L3
id: "HS-COS-001-001"
title: "claroty_devices rows carry device_is_online as JSON boolean (or null), not is_online or string true/false"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "E-BETA3-REMEDIATION"
story_source: "S-CLAROTY-OCSF-STATUS-001"
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
  - ".factory/stories/S-CLAROTY-OCSF-STATUS-001-claroty-devices-is-online-vendor-ext-and-retired-demotion.md"
input-hash: "TBD"
traces_to: "BC-2.16.003"
behavioral_contracts:
  - BC-2.16.003
verification_properties: []
lifecycle_status: active
introduced: "S-CLAROTY-OCSF-STATUS-001"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-OCSF-STATUS-001 (HS-038 group). Validates BC-2.16.003 EC-016-013-034..036: is_online values true/false/null produce device_is_online Boolean Arrow cells at the serialized wire level. Also validates EC-016-013-037 (retired demoted to raw_extensions: no top-level status_code column from boolean source). Discriminating: pre-patch has no device_is_online column (routes to raw_extensions); post-fix has first-class device_is_online: Boolean. Claroty DTU required (T-F02 fixture seeding: is_online true/false/null states). Test-writer and implementer must NOT read this file."
---

# HS-COS-001-001: claroty_devices rows carry device_is_online as JSON boolean (or null), not is_online or string true/false

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-OCSF-STATUS-001 (HS-038 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.003 §Postconditions EC-016-013-034..037
  EC-034: `is_online=true` → `device_is_online=true` in Arrow RecordBatch (MUST-1)
  EC-035: `is_online=false` → `device_is_online=false` in Arrow RecordBatch (MUST-2)
  EC-036: `is_online=null/absent` → `device_is_online` null Arrow cell (MUST-3)
  EC-037: `retired` demoted to raw_extensions; no top-level `status_code` column from Boolean source (MUST-4)
**Gate:** Story-level holdout gate (HS-038) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the TOML fix in S-CLAROTY-OCSF-STATUS-001 takes effect at the serialized MCP wire level:

1. `is_online` is no longer routed to `raw_extensions`; it projects as a first-class `device_is_online: Boolean` Arrow column.
2. The three Boolean wire states (true, false, null) are correctly represented as JSON booleans or JSON null — never as the strings `"true"`, `"false"`, or `"null"`.
3. The `retired` boolean no longer appears as a top-level `status_code` column; it routes to `raw_extensions`.

**Pre-patch behavior (what this scenario catches):**

- `device_is_online` is absent from the row objects entirely — `is_online` appears inside the raw_extensions JSON blob as `"is_online": true` or `"is_online": false`.
- A top-level `status_code` column exists with the boolean `retired` value force-cast to a string, causing a type-contract violation.

**Post-fix behavior:**

- Each row has a top-level `device_is_online` key with value `true`, `false`, or `null` (JSON boolean or JSON null — never the string `"null"` per EC-11-079 null-not-absent invariant).
- `is_online` does NOT appear as a standalone top-level key.
- `retired` does NOT appear as a top-level `status_code` key; it appears inside the `raw_extensions` JSON string under its native key `"retired"`.

**Four wire-level assertions:**

**Part A — device_is_online present as boolean (or null) in at least one row:**
- Assert: at least one row in the response has a `device_is_online` key.
- Assert: the JSON type of `device_is_online` is boolean (`true` or `false`) or `null` (JSON null) — NOT the string `"true"`, `"false"`, or `"null"`.

**Part B — is_online NOT a standalone top-level row key:**
- Assert: no row object has `is_online` as a direct top-level key. If `is_online` appears, it must be inside the `raw_extensions` JSON string blob, not as a first-class column.

**Part C — status_code column absent as Boolean-source column:**
- Assert: no row contains a top-level `status_code` key whose value is a JSON boolean (the retired v1 behaviour). The `status_code` column should not be emitted from the `retired` boolean source.

**Part D — retired value in raw_extensions:**
- For a row where the DTU fixture has `"retired": true` (or `"retired": false`), assert the `raw_extensions` JSON string contains `"retired":true` (or `"retired":false`).

**BDD supplement:**

**Given** prism is built from the S-CLAROTY-OCSF-STATUS-001 story branch
**And** the Claroty DTU is running with the updated `devices.json` fixture seeded with at least one device where `is_online=true`, one where `is_online=false`, and one where `is_online=null`
**When** `tools/call query {"query": "FROM claroty_devices", "client_id": "<test_client>", "limit": 10}` is issued via MCP stdio
**Then** the response is not a JSON-RPC error
**And** at least one row has `device_is_online` as a JSON boolean (`true` or `false`) or JSON null
**And** no row has `is_online` as a direct top-level row key
**And** no row has `status_code` as a top-level key from a boolean source
**And** for rows where `retired` appears, it is inside the `raw_extensions` string, not as a standalone `status_code`

---

## Setup Instructions

1. Build prism from the S-CLAROTY-OCSF-STATUS-001 story branch.

2. Start the Claroty DTU (`cargo run -p prism-dtu-claroty` or equivalent). The DTU must
   serve `/api/v1/devices` with a fixture that includes at least three device records:
   - One with `"is_online": true`
   - One with `"is_online": false`
   - One with `"is_online": null` (explicit JSON null) or the key absent
   The story's T-F02 task seeds `fixtures/devices.json` with these states.

   SETUP-FAILURE condition (DTU): if the Claroty DTU cannot be started as a standalone
   process, or if the fixture does not contain the required is_online states (true/false/null),
   record as SETUP-FAILURE. The structural assertions (Parts A–D) may still be evaluated
   via the prism-bin integration test suite if MCP-stdio mode is unavailable.

3. Prepare a `prism.toml` with a test client configured for the Claroty sensor, pointing
   at the running DTU.

4. Start prism in MCP stdio mode.
   SETUP-FAILURE condition (prism): prism fails to start or MCP handshake fails.

5. Complete the MCP `initialize` handshake.

6. Issue the `query` tool call:
   ```
   {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query","arguments":{"query":"FROM claroty_devices","client_id":"<test_client>","limit":10}}}
   ```
   Capture the full wire-level JSON response bytes.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.003 | EC-016-013-034: `is_online=true` → `device_is_online=true` Arrow Boolean cell | Part A: JSON boolean true present in row |
| BC-2.16.003 | EC-016-013-035: `is_online=false` → `device_is_online=false` Arrow Boolean cell | Part A: JSON boolean false present in row |
| BC-2.16.003 | EC-016-013-036: `is_online=null/absent` → `device_is_online` null Arrow cell; wire null not absent (EC-11-079) | Part A: JSON null present for null-state device |
| BC-2.16.003 | EC-016-013-037: `retired` demoted to raw_extensions; no top-level Boolean-source `status_code` | Parts C and D |

---

## Verification Approach

1. Parse wire-level JSON-RPC response. Verify non-error (`result` present, `error` absent).
   If error: record error message; SETUP-FAILURE if "sensor not configured" or "fetch failed".

2. Parse `result.content[0].text` as JSON to obtain the query response envelope.

3. Extract the `rows` array. If empty or absent: SETUP-FAILURE (no data).

4. **Part A — device_is_online wire type:**
   For each row object, look for the key `device_is_online`.
   - Assert at least one row has the key.
   - For rows that have it: assert the JSON type is `true`, `false`, or `null` (JSON null).
   - Assert: no row has `device_is_online` as the string `"true"`, `"false"`, or `"null"`.

5. **Part B — is_online not a standalone key:**
   For each row object: assert `is_online` is NOT a direct top-level key with a JSON boolean value.
   If `raw_extensions` is present in the row, optionally inspect it — `is_online` may appear there (acceptable).

6. **Part C — status_code column not from Boolean source:**
   For each row object: look for `status_code` key with a JSON boolean value.
   Assert: no such key/value pair exists (no Boolean-typed `status_code` from the `retired` field).
   Note: other `status_code` values (strings from a different source) are acceptable if they exist.

7. **Part D — retired in raw_extensions:**
   Locate a row where `raw_extensions` is present and non-null.
   Parse the `raw_extensions` JSON string. Assert the parsed object contains `"retired"` key.
   If no rows have a non-null `raw_extensions`, record as PARTIAL (cannot verify Part D).

8. All assertions are on the serialized JSON wire bytes from `result.content[0].text`.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response is non-error (prerequisite)** (weight: 0.10):
  Full credit (1.0): non-error result, rows array non-empty.
  Zero credit (0.0): error response or empty rows array (SETUP-FAILURE).

- **device_is_online present as boolean/null in rows (Part A — primary discriminator)** (weight: 0.55):
  Full credit (1.0): at least one row has `device_is_online` as JSON boolean or JSON null; zero rows have `device_is_online` as a string.
  Half credit (0.5): column present but only for some states (e.g., true/false present, null absent).
  Zero credit (0.0): `device_is_online` absent from all rows (pre-patch behavior — column not in schema).

- **is_online NOT a standalone row key (Part B)** (weight: 0.15):
  Full credit (1.0): no row has `is_online` as a direct top-level key.
  Zero credit (0.0): `is_online` appears as a top-level key in any row (old raw-passthrough path still active).

- **status_code not a Boolean-source column (Part C)** (weight: 0.10):
  Full credit (1.0): no row has a boolean-valued `status_code` top-level key.
  Zero credit (0.0): boolean-valued `status_code` present (pre-patch retired-to-status_code mapping active).

- **retired in raw_extensions (Part D)** (weight: 0.10):
  Full credit (1.0): `raw_extensions` parsed JSON contains `"retired"` key.
  Half credit (0.5): cannot verify (no row with parseable raw_extensions containing retired).
  Zero credit (0.0): `raw_extensions` present but does NOT contain `"retired"` key.

---

## Edge Conditions

- **DTU fixture has no is_online=null device:** If the fixture only has true/false states (no null),
  Part A can still be partially satisfied (true + false verified). Record as PARTIAL credit for the null arm.
  This is SETUP-FAILURE for the null-state arm, not a behavioral failure.

- **All devices are retired=false:** Part D may only see `"retired": false` in raw_extensions.
  Still satisfies Part D (the key is present with the expected boolean value).

- **device_is_online present but value is null for ALL rows:** If every device has null `is_online`,
  only the null arm of Part A is exercisable. Part A still passes at half-credit minimum.

- **raw_extensions is JSON-serialized as a string (Arrow Utf8 per ADR-058 §I2):** If `raw_extensions`
  in the row is a JSON string (not a JSON object), the evaluator must parse it as a JSON string first,
  then parse the inner value as a JSON object to inspect `"retired"`.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-COS-001-001 (satisfaction: X.XX) — device_is_online column not present as Boolean at wire level; check claroty.sensor.toml is_online ocsf_field = 'device.is_online' change and DTU fixture is_online states (BC-2.16.003 EC-016-013-034..037 + ADR-058 §K5 D-2522 Option A)"`

Do NOT disclose: which specific rows were inspected, the raw_extensions contents, or whether
the failure was in the TOML config, DTU fixture seeding, or build_column_array routing.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-CLAROTY-OCSF-STATUS-001 branch + Claroty DTU with updated devices fixture (is_online: true/false/null states seeded by T-F02) |
| corpus_size | claroty_devices table; standard DTU fixture with 3+ device entries covering the three Boolean states |
| known_edge_cases | DTU fixture seeded with only true/false (no null): Part A partial satisfaction; is_online key absent vs JSON null: both map to Arrow null per Option<bool> serde default |
| false_positive_threshold | Zero: presence of `device_is_online` as a JSON boolean is the exact post-fix behavior and was completely absent pre-patch |
| false_negative_threshold | Near-zero: pre-patch has no `device_is_online` column at all; any non-null Boolean value in that key proves the fix landed |

**Known-good corpus:** prism binary with `ocsf_field = "device.is_online"` in claroty.sensor.toml and DTU with `Option<bool>` is_online fixture data. Expected: `device_is_online` in every row as boolean/null; no `is_online` standalone column.

**Known-problematic corpus:** pre-patch develop HEAD. Expected: no `device_is_online` key in row objects; `is_online` buried in `raw_extensions` JSON blob.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w3-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-038 group for S-CLAROTY-OCSF-STATUS-001. Primary discriminating test: device_is_online Boolean wire shape (true/false/null) at MCP serialized-JSON level. Validates EC-016-013-034..037 (is_online mapping + retired demotion). Claroty DTU required (T-F02 fixture seeding). SINGLE-USE HIDDEN. |
