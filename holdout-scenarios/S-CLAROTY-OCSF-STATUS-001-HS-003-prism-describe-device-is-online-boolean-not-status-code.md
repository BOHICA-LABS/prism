---
document_type: holdout-scenario
level: L3
id: "HS-COS-001-003"
title: "prism_describe reports device_is_online as Boolean column for claroty_devices; no status_code from Boolean retired source"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-OCSF-STATUS-001 (HS-038 group). Validates BC-2.16.003 EC-016-013-038: prism_describe returns device_is_online with Boolean column_type in the claroty_devices column schema; no status_code column from the boolean retired source appears in the schema. Wire-level assertion on the serialized prism_describe JSON response. NO DTU required (prism_describe reads the TOML spec at boot; no sensor query performed). Discriminating: pre-patch: device_is_online absent from describe output (column does not exist); status_code may appear. Post-fix: device_is_online present with Boolean type; status_code not emitted from retired boolean source. Test-writer and implementer must NOT read this file."
---

# HS-COS-001-003: prism_describe reports device_is_online as Boolean column for claroty_devices; no status_code from Boolean retired source

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-OCSF-STATUS-001 (HS-038 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.003 §Postconditions EC-016-013-038
  "`prism_describe` MUST include `device_is_online: Boolean` in the Claroty devices column schema (Tier-1 vendor-extended column from `ocsf_field = 'device.is_online'`). The `retired` column MUST NOT appear as `status_code` in the describe response."
**Gate:** Story-level holdout gate (HS-038) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the schema-visibility half of the `device_is_online` fix: not only that queries return the column (HS-COS-001-001), but that `prism_describe` correctly advertises it to LLM agents who consult the schema before writing queries.

`prism_describe` reads the TOML spec at boot and constructs column descriptors from the
declared columns. This scenario does NOT require the Claroty DTU to be running — it
tests the spec-engine's ability to build the schema from the amended TOML file.

**Pre-patch behavior (what this scenario catches):**

- `device_is_online` is absent from the `prism_describe` column list for the `claroty_devices` table (the column has no `ocsf_field` mapping, so it is not promoted to a named Tier-1 Arrow column visible in the describe output).
- A `status_code` column may appear in the describe output with a Boolean source (from the `retired` field's now-removed `ocsf_field = "status_code"` declaration).

**Post-fix behavior:**

- `device_is_online` appears in the columns array for the `claroty_devices` table with `col_type` = `"Boolean"` (or equivalent representation).
- No `status_code` column exists in the schema whose Arrow type is Boolean (the retired field is now Tier-2, so it does not project as a named Tier-1 column).

**Three wire-level assertions:**

**Part A — device_is_online present in columns array:**
- Parse the `prism_describe` response for `claroty_devices` table.
- Assert: the `columns` array contains an object with `name = "device_is_online"`.
- Assert: that object's `col_type` (or equivalent field) is `"Boolean"` or contains `"bool"` (case-insensitive).

**Part B — device_is_online NOT present as a string type:**
- Assert: no column with `name = "device_is_online"` has `col_type = "String"` or `"Utf8"`.
  (A string-typed `device_is_online` would indicate the Boolean passthrough is broken.)

**Part C — no Boolean-source status_code column:**
- Assert: no column in the `claroty_devices` schema has `name = "status_code"` with `col_type = "Boolean"` or `col_type = "Integer"` (a boolean-source status_code column from the retired field is the pre-patch defect).
- Note: a `status_code` String column from a different source is not prohibited; only a Boolean-source one is the defect.

**BDD supplement:**

**Given** prism is built from the S-CLAROTY-OCSF-STATUS-001 story branch
**And** the Claroty sensor TOML has `ocsf_field = "device.is_online"` on `is_online` and no `ocsf_field = "status_code"` on `retired`
**When** `tools/call prism_describe {"client_id": "<test_client>"}` is issued via MCP stdio
**Then** the response is not a JSON-RPC error
**And** the `claroty_devices` table's columns array contains a column named `"device_is_online"` with Boolean type
**And** no column named `"status_code"` with a Boolean Arrow type appears in the `claroty_devices` schema

---

## Setup Instructions

1. Build prism from the S-CLAROTY-OCSF-STATUS-001 story branch.
   (DTU NOT required — `prism_describe` reads TOML at boot; no live sensor query.)

2. Configure `prism.toml` with a test client that includes the Claroty sensor spec.
   The spec-engine will read `crates/prism-sensors/specs/claroty.sensor.toml`; the
   `claroty_devices` table must have the amended column declarations.

3. Start prism in MCP stdio mode. Complete `initialize` handshake.
   SETUP-FAILURE condition: prism fails to start or MCP handshake fails.

4. Issue the `prism_describe` tool call:
   ```
   {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"prism_describe","arguments":{"client_id":"<test_client>"}}}
   ```
   Capture the full wire-level JSON response bytes.

5. From the response, locate the entry for `claroty_devices` in the `tables` array.
   Extract its `columns` array.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.003 | EC-016-013-038: `prism_describe` includes `device_is_online: Boolean` in Claroty devices column schema | Part A and B |
| BC-2.16.003 | EC-016-013-037: `retired` demoted to raw_extensions; `ocsf_field = "status_code"` removed → no Boolean-source status_code column in schema | Part C |

---

## Verification Approach

1. Parse wire-level JSON-RPC response. Assert non-error.

2. Parse `result.content[0].text` as JSON to obtain the describe response.

3. Locate the `claroty_devices` table entry in the `tables` array. If not found: SETUP-FAILURE.

4. Extract the `columns` array for `claroty_devices`.

5. **Part A — device_is_online in columns:**
   Search for a column object where `name == "device_is_online"`.
   - If absent: record FAIL (pre-patch behavior).
   - If present: extract the `col_type` (or `column_type`) field.
   - Assert `col_type` contains "bool" or "Boolean" (case-insensitive comparison).

6. **Part B — not a String type:**
   For the `device_is_online` column found in Part A:
   Assert `col_type` does NOT equal "String", "Utf8", or "Text".
   A string-typed `device_is_online` indicates the Boolean passthrough pipeline is broken.

7. **Part C — no Boolean-source status_code:**
   Search the columns array for any column where `name == "status_code"`.
   If found: check `col_type`. If `col_type` is "Boolean" or "bool": record FAIL.
   If `status_code` is present but is a String type from a different source: acceptable.
   If `status_code` is absent entirely: full credit.

8. All assertions on the serialized JSON wire bytes from `result.content[0].text`.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response is non-error (prerequisite)** (weight: 0.10):
  Full credit (1.0): non-error result, claroty_devices table found in response.
  Zero credit (0.0): error response or claroty_devices absent (SETUP-FAILURE).

- **device_is_online present with Boolean type (Part A — primary discriminator)** (weight: 0.60):
  Full credit (1.0): column named `device_is_online` present with Boolean/bool col_type.
  Half credit (0.5): column present but col_type is unexpected (not Boolean — implementation issue).
  Zero credit (0.0): `device_is_online` absent from columns array (pre-patch behavior — TOML fix not effective).

- **device_is_online not String-typed (Part B)** (weight: 0.15):
  Full credit (1.0): not String-typed (or column absent — Part A covers absence).
  Zero credit (0.0): String-typed device_is_online (passthrough broken — would give string "true"/"false").

- **No Boolean-source status_code (Part C)** (weight: 0.15):
  Full credit (1.0): no Boolean-source status_code column in claroty_devices schema.
  Zero credit (0.0): Boolean-typed status_code present (retired demotion not effective).

---

## Edge Conditions

- **col_type field naming varies (col_type vs column_type vs type):** The evaluator must
  check the actual field name in the `prism_describe` response structure. The assertion is
  on the value being Boolean/bool, regardless of the exact field name used.

- **Multiple tables returned in prism_describe:** The evaluator must locate the
  `claroty_devices` table specifically. Other tables' column schemas are irrelevant.

- **vendor-extension validation warning in prism startup logs:** AC-006 of the story
  specifies that a `ValidationWarning` (not `ValidationError`) is acceptable for the
  vendor-extended `device.is_online` path. If prism emits a warning log on boot, this
  is NOT a failure — the describe response must still reflect the column.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-COS-001-003 (satisfaction: X.XX) — prism_describe claroty_devices column schema missing device_is_online Boolean column; check claroty.sensor.toml ocsf_field = 'device.is_online' and validate_ocsf_field_path produces at most ValidationWarning (BC-2.16.003 EC-016-013-038 + ADR-058 §K5 D-2522 Option A; prism_describe AC-005)"`

Do NOT disclose: which specific columns were seen in the describe output, the total count
of columns returned, or the exact col_type value observed.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-CLAROTY-OCSF-STATUS-001 branch + amended claroty.sensor.toml TOML spec. No DTU required. |
| corpus_size | prism_describe output for claroty_devices table; column descriptor array. |
| known_edge_cases | ValidationWarning from validate_ocsf_field_path on vendor-extended path: acceptable (non-blocking); column must still appear. device_is_online name shadow check: confirmed clean (no 20-column name collision per ADR-058 §J3 verified 2026-09-16). |
| false_positive_threshold | Zero: device_is_online column in describe output is the definitive result of the TOML fix |
| false_negative_threshold | Near-zero: pre-patch TOML has no ocsf_field on is_online → column absent from describe; post-fix TOML has ocsf_field → column present |

**Known-good corpus:** post-fix binary with `ocsf_field = "device.is_online"` in TOML. Expected: `device_is_online` in columns array with Boolean type.

**Known-problematic corpus:** pre-patch develop HEAD. Expected: `device_is_online` absent from claroty_devices column list; possibly `status_code` with Boolean type from `retired` field.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w3-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-038 group for S-CLAROTY-OCSF-STATUS-001. Tests prism_describe schema-visibility of device_is_online Boolean column. No DTU required. BC-2.16.003 EC-016-013-038. SINGLE-USE HIDDEN. |
