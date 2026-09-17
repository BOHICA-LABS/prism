---
document_type: holdout-scenario
level: L3
id: "HS-COSTOML-001-001"
title: "claroty_alerts wire rows carry severity_id as JSON integer and finding_info_uid as JSON string"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "E-BETA3-REMEDIATION"
story_source: "S-CLAROTY-OCSF-TOML-001"
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
  - ".factory/stories/S-CLAROTY-OCSF-TOML-001-claroty-ocsf-field-mapping-corrections.md"
input-hash: "TBD"
traces_to: "BC-2.16.003"
behavioral_contracts:
  - BC-2.16.003
  - BC-2.02.005
  - BC-2.16.013
verification_properties: []
lifecycle_status: active
introduced: "S-CLAROTY-OCSF-TOML-001"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-OCSF-TOML-001 (HS-039 group). Compound discriminating test covering Issues 8 and 13: (a) finding_info_uid is a JSON string in wire output (integer coercion via EC-016-013-004; pre-patch: null because Value::Number arm missing from String branch); (b) severity_id is a JSON integer in wire output (pre-patch: absent because ClarotyAlert struct has no severity_id field). Claroty DTU required (fixtures/alerts.json must be seeded with severity_id per story T-F02 equivalent). Wire-level assertions on serialized MCP query response. Test-writer and implementer must NOT read this file."
---

# HS-COSTOML-001-001: claroty_alerts wire rows carry severity_id as JSON integer and finding_info_uid as JSON string

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-OCSF-TOML-001 (HS-039 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:**
  - BC-2.16.003 EC-016-013-004: `Value::Number(n)` input to a String-typed column coerces to `n.to_string()` (integer `id` → string `finding_info_uid`)
  - BC-2.16.003 EC-016-013-042: `severity_id` is an Int64 Tier-1 column in `claroty_alerts` schema
  - BC-2.02.005 §Postconditions: Claroty Detection Finding OCSF field mapping; `severity_id` as Integer_t
  - BC-2.16.013 §Postconditions: DTU wire emits `severity_id`; both fixture paths covered
**Gate:** Story-level holdout gate (HS-039) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates two independent defect fixes in a single compound assertion, both affecting the `claroty_alerts` wire output:

### Part A — Issue 8: finding_info_uid is a JSON string (not null, not integer)

`ClarotyAlert.id` is a `u32` integer on the wire. The TOML declares `column_type = "string"` with `ocsf_field = "finding_info.uid"`. The coercion path in `build_column_array`'s `ColumnType::String` branch must handle `Value::Number` inputs by converting to their decimal string representation.

**Pre-patch (Issue 8 defect):** The `Value::Number` arm is absent from the String match branch. `ClarotyAlert.id = 123` (integer) produces a null Arrow cell in the `finding_info_uid` column. Every alert's `finding_info_uid` is null in the wire response.

**Post-fix:** `finding_info_uid = "123"` (string) in the wire response for each alert. The integer is preserved as its decimal string form.

### Part B — Issue 13: severity_id is a JSON integer (not absent)

`ClarotyAlert` previously had no `severity_id` struct field. The Claroty API returns `"severity_id": 3` (an integer) on each alert object. Without the struct field, serde silently drops the value. The TOML previously had no `severity_id` column entry.

**Pre-patch (Issue 13 defect):** `severity_id` is absent from every row in the `claroty_alerts` wire output. The column does not exist in the Arrow schema.

**Post-fix:** `severity_id` is a JSON integer (e.g., `3`) in the wire response for each seeded alert.

**Four compound assertions on the serialized MCP wire bytes:**

**Part A-1 — finding_info_uid is a JSON string:**
- Assert: at least one row has `finding_info_uid` as a JSON string (not null, not a JSON integer).
- Assert: no row has `finding_info_uid` as a JSON integer (e.g., `123` not `"123"` — wrong type).

**Part A-2 — finding_info_uid not null for alerts with non-null IDs:**
- Assert: the `finding_info_uid` value for a row whose DTU fixture has a non-null integer `id` is NOT null in the wire response.

**Part B-1 — severity_id present as JSON integer:**
- Assert: at least one row has `severity_id` as a JSON integer (a positive integer, e.g., 1–5).
- Assert: `severity_id` is NOT absent from the row object (pre-patch behavior).

**Part B-2 — severity_id not null:**
- Assert: for rows from the seeded fixture (which has `"severity_id": 3`), `severity_id` is NOT null.

**BDD supplement:**

**Given** prism is built from the S-CLAROTY-OCSF-TOML-001 story branch
**And** the Claroty DTU is running with an updated `fixtures/alerts.json` that includes `"severity_id": 3` on each alert record
**When** `tools/call query {"query": "FROM claroty_alerts", "client_id": "<test_client>", "limit": 5}` is issued via MCP stdio
**Then** the response is not a JSON-RPC error
**And** at least one row has `finding_info_uid` as a JSON string (e.g. `"123"`, not `123`)
**And** at least one row has `severity_id` as a JSON integer (e.g. `3`, not absent, not null)

---

## Setup Instructions

1. Build prism from the S-CLAROTY-OCSF-TOML-001 story branch.

2. Start the Claroty DTU (`cargo run -p prism-dtu-claroty` or equivalent). The DTU must
   serve `/api/v1/alerts` with fixtures that include:
   - Alert records with an integer `"id"` field (e.g., `"id": 12345`)
   - Alert records with `"severity_id": 3` (or any positive integer — the story seeds this in Phase B)
   The standard alerts fixture after story implementation includes both.

   SETUP-FAILURE condition: DTU cannot be started as a standalone process, OR the fixture
   does not contain `severity_id` integer values, OR alerts fixture is empty. Record
   SETUP-FAILURE and do not score behavioral assertions.

3. Configure `prism.toml` with a test client pointing at the DTU Claroty alerts endpoint.

4. Start prism in MCP stdio mode. Complete `initialize` handshake.

5. Issue the query:
   ```
   {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query","arguments":{"query":"FROM claroty_alerts","client_id":"<test_client>","limit":5}}}
   ```
   Capture the full wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.003 | EC-016-013-004: `Value::Number(n)` in String column coerces to `n.to_string()` | Parts A-1 and A-2: finding_info_uid as string |
| BC-2.16.003 | EC-016-013-042: `severity_id` Int64 Tier-1 column in claroty_alerts schema | Parts B-1 and B-2: severity_id as integer |
| BC-2.02.005 | §Postconditions: Claroty alerts `severity_id` as Integer_t OCSF Detection Finding field | Part B: severity_id present and integer-typed |
| BC-2.16.013 | §Postconditions: DTU wire emits `severity_id`; fixture path and generated-records path both covered | Part B: severity_id from DTU fixture reaches wire |

---

## Verification Approach

1. Parse wire-level JSON-RPC response. Assert non-error.

2. Parse `result.content[0].text` as JSON to obtain the query response envelope.

3. Extract `rows` array. If empty: SETUP-FAILURE.

4. **Part A-1 — finding_info_uid is a JSON string:**
   For at least one row, locate key `finding_info_uid`.
   - Assert: JSON type is `string` (not null, not number).
   - Assert: no row has `finding_info_uid` as a JSON number (integer type in JSON).

5. **Part A-2 — finding_info_uid not null for integer-id alerts:**
   For rows where the DTU fixture has an integer `id`, `finding_info_uid` should be a
   non-null string like `"12345"`. Assert at least one row has non-null `finding_info_uid`.
   (If all alerts happen to have null-equivalent ids, record as PARTIAL.)

6. **Part B-1 — severity_id present as JSON integer:**
   For at least one row, locate key `severity_id`.
   - If absent from all rows: record FAIL (pre-patch behavior — field not in struct).
   - If present: assert JSON type is `number` (integer type), not null, not string.

7. **Part B-2 — severity_id not null for seeded records:**
   For rows from the seeded fixture (which includes `"severity_id": 3`),
   assert `severity_id` value is `> 0` (positive integer, not null).

8. All assertions on the serialized JSON wire bytes from `result.content[0].text`.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response is non-error (prerequisite)** (weight: 0.10):
  Full credit (1.0): non-error result, rows array non-empty.
  Zero credit (0.0): error response or empty rows (SETUP-FAILURE).

- **finding_info_uid is a JSON string (Part A — Issue 8 discriminator)** (weight: 0.40):
  Full credit (1.0): at least one row has `finding_info_uid` as a JSON string; no rows have it as an integer.
  Half credit (0.5): finding_info_uid present but as null (coercion partially implemented).
  Zero credit (0.0): `finding_info_uid` absent from all rows, or consistently null (pre-patch: Value::Number arm missing).

- **severity_id present as JSON integer (Part B — Issue 13 discriminator)** (weight: 0.40):
  Full credit (1.0): at least one row has `severity_id` as a JSON integer > 0.
  Half credit (0.5): `severity_id` key present but null (struct field exists but fixture not seeded).
  Zero credit (0.0): `severity_id` absent from all rows (pre-patch: struct field missing, serde dropped value).

- **severity_id not null for seeded records (Part B-2)** (weight: 0.10):
  Full credit (1.0): seeded records carry non-null integer severity_id.
  Zero credit (0.0): seeded records have null severity_id (deserialization not working).

---

## Edge Conditions

- **DTU fixture has severity_id absent (not seeded):** Part B yields SETUP-FAILURE.
  The Story's Phase B task must update `fixtures/alerts.json` to add `"severity_id": 3` per AC-006.
  If this seeding step was skipped, Part B FAIL is a legitimate behavioral failure, not a setup issue.

- **finding_info_uid string contains decimal form of integer:** `id = 12345` → `finding_info_uid = "12345"`.
  The evaluator must NOT require any particular string format beyond being a valid decimal string.

- **Integer id = 0 edge case:** If a fixture alert has `"id": 0`, `finding_info_uid` should be `"0"`.
  The assertion is on JSON string type, not on value content.

- **Alerts fixture uses generated-records path (not static fixture):** Per BC-2.16.013 §Postconditions
  SAP-2 probe rule 6, both static-fixture and generated-records paths must carry severity_id.
  The evaluator should run one query; both paths are exercised by the DTU in normal operation.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-COSTOML-001-001 (satisfaction: X.XX) — claroty_alerts wire output missing expected column types; check (a) build_column_array String arm for Value::Number coercion (BC-2.16.003 EC-016-013-004 Issue 8) and (b) ClarotyAlert struct severity_id field + fixture seeding + TOML column entry (BC-2.16.003 EC-016-013-042 + BC-2.16.013 §Postconditions Issue 13)"`

Do NOT disclose: the specific finding_info_uid values observed, the severity_id values
observed, or which issue (8 vs 13) failed independently.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-CLAROTY-OCSF-TOML-001 branch + Claroty DTU alerts fixture with `"severity_id": 3` seeded per story Phase B |
| corpus_size | claroty_alerts table; 5 alert rows from DTU fixture |
| known_edge_cases | DTU standalone binary unavailable (SETUP-FAILURE for full MCP mode; story has RG tests as compensating coverage). Fixture seeding omitted (Part B FAIL — legitimate implementation gap). |
| false_positive_threshold | Near-zero: finding_info_uid as string requires both the struct and the Value::Number coercion arm; severity_id as integer requires both struct field and TOML column. |
| false_negative_threshold | Near-zero: pre-patch finding_info_uid is null for integer-id alerts; pre-patch severity_id absent from row entirely |

**Known-good corpus:** post-fix binary with coercion arm + ClarotyAlert.severity_id field + TOML column + seeded fixture. Expected: finding_info_uid as string, severity_id as integer.

**Known-problematic corpus:** pre-patch develop HEAD. Expected: finding_info_uid null (Number arm missing); severity_id absent (struct field missing).

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w3-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-039 group for S-CLAROTY-OCSF-TOML-001. Compound test covering Issues 8 (finding_info_uid string coercion) and 13 (severity_id integer Tier-1 column). BC-2.16.003 EC-016-013-004 + EC-016-013-042. Claroty DTU required. SINGLE-USE HIDDEN. |
