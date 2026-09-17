---
document_type: holdout-scenario
level: L3
id: "HS-COSTOML-001-002"
title: "device_vulnerability_relations.device_uid queryable as Tier-1 column and vulnerabilities.time column present as Tier-1"
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
traces_to: "BC-2.16.017"
behavioral_contracts:
  - BC-2.16.017
  - BC-2.16.003
verification_properties: []
lifecycle_status: active
introduced: "S-CLAROTY-OCSF-TOML-001"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-OCSF-TOML-001 (HS-039 group). Compound test covering Issues 11 and 12: (a) SELECT device_uid FROM claroty_device_vulnerability_relations succeeds (no E-QUERY-038) and device_uid is a top-level column not inside raw_extensions (BC-2.16.017 EC-016-017-007); (b) SELECT time FROM claroty_vulnerabilities succeeds (no E-QUERY-038) and time column is present as a Datetime Tier-1 column (BC-2.16.003 EC-016-013-043). NO DTU required for either table (BC-2.16.017 §PC5 + D-2200 deferral; live monroe sensor used). SETUP-FAILURE if live sensor unavailable or tables have zero rows. Test-writer and implementer must NOT read this file."
---

# HS-COSTOML-001-002: device_vulnerability_relations.device_uid queryable as Tier-1 column and vulnerabilities.time column present as Tier-1

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-OCSF-TOML-001 (HS-039 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:**
  - BC-2.16.017 §PC3 + EC-016-017-007: `device_uid` must be a queryable Tier-1 column in `claroty_device_vulnerability_relations`; `SELECT device_uid` must NOT return E-QUERY-038.
  - BC-2.16.003 EC-016-013-043/044: six entity tables gain a Tier-1 `time` column via `ocsf_field = "time"` on their datetime source columns. Vulnerability table (`published_date → time`) is the primary representative.
**Gate:** Story-level holdout gate (HS-039) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This compound scenario validates two TOML-only promotion fixes (Issues 11 and 12) via the live Claroty sensor. No DTU is required — neither `device_vulnerability_relations` nor `vulnerabilities` has a DTU clone (per BC-2.16.017 §PC5 and D-2200). The evaluator queries the live monroe sensor.

### Part A — Issue 12: device_uid is Tier-1 in device_vulnerability_relations

`device_uid` was previously Tier-2 (raw_extensions only) because no `ocsf_field` was declared on the column. The fix adds `ocsf_field = "device.uid"` to promote it to Tier-1 (`ocsf_field_to_arrow_name("device.uid") = "device_uid"` — same Arrow name, but now a first-class column).

**Pre-patch (Issue 12 defect):** `SELECT device_uid FROM claroty_device_vulnerability_relations` returns E-QUERY-038 with `"device_uid"` listed in `available_columns` (it is accessible but only via raw_extensions extraction, not as a top-level column name).

Wait — re-examining: the pre-patch behavior is that `device_uid` routes to `raw_extensions`. When queried directly as `SELECT device_uid`, it returns E-QUERY-038 because the Arrow schema has no `device_uid` column (it's inside the `raw_extensions` JSON blob).

**Post-fix:** `SELECT device_uid` succeeds. Rows have `device_uid` as a top-level column key with a non-null string value (device UUID).

### Part B — Issue 11: vulnerabilities table has a Tier-1 time column

`vulnerabilities.published_date` was previously Tier-2. The fix adds `ocsf_field = "time"` to the `published_date` column. `ocsf_field_to_arrow_name("time") = "time"` — the Arrow column is named `time`.

**Pre-patch (Issue 11 defect):** `SELECT time FROM claroty_vulnerabilities` returns E-QUERY-038 because there is no `time` Tier-1 column in the schema.

**Post-fix:** `SELECT time` succeeds. Rows have `time` as a top-level column key with a Datetime value (ISO-8601 string from `published_date`).

**Four wire-level assertions (two per issue):**

**Part A-1 — SELECT device_uid is non-error:**
- Assert: `FROM claroty_device_vulnerability_relations SELECT device_uid LIMIT 5` returns a non-error MCP response.
- Discriminating: pre-patch returns E-QUERY-038.

**Part A-2 — device_uid is a top-level column in rows:**
- Assert: at least one row has `device_uid` as a direct top-level key with a non-null string value.
- Assert: `device_uid` is NOT inside a raw_extensions JSON blob (it is a first-class column).

**Part B-1 — SELECT time FROM claroty_vulnerabilities is non-error:**
- Assert: `FROM claroty_vulnerabilities SELECT time LIMIT 5` returns a non-error MCP response.
- Discriminating: pre-patch returns E-QUERY-038.

**Part B-2 — time is a top-level column in rows:**
- Assert: at least one row has `time` as a direct top-level key with a non-null value (ISO-8601 string or equivalent datetime representation).

**BDD supplement:**

**Given** prism is built from the S-CLAROTY-OCSF-TOML-001 story branch
**And** the live Claroty sensor (monroe) is configured and accessible
**When** `tools/call query {"query": "FROM claroty_device_vulnerability_relations SELECT device_uid", "client_id": "<client>", "limit": 5}` is issued
**Then** the response is not a JSON-RPC error
**And** at least one row has `device_uid` as a top-level non-null string column
**When** `tools/call query {"query": "FROM claroty_vulnerabilities SELECT time", "client_id": "<client>", "limit": 5}` is issued
**Then** the response is not a JSON-RPC error
**And** at least one row has `time` as a top-level non-null column

---

## Setup Instructions

1. Build prism from the S-CLAROTY-OCSF-TOML-001 story branch.

2. No DTU is required. The live Claroty sensor (monroe tenant) is used for both tables.
   Neither `device_vulnerability_relations` nor `vulnerabilities` has a DTU clone
   (BC-2.16.017 §PC5; D-2200 deferral).

3. Configure `prism.toml` with a test client that has access to the live monroe Claroty sensor.
   SETUP-FAILURE condition: live sensor credentials not available, sensor unreachable, or
   both tables return zero rows. Record SETUP-FAILURE; do not score behavioral assertions.

4. Start prism in MCP stdio mode. Complete `initialize` handshake.

5. Issue the device_uid query (Part A):
   ```
   {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query","arguments":{"query":"FROM claroty_device_vulnerability_relations SELECT device_uid","client_id":"<test_client>","limit":5}}}
   ```
   Capture the full wire-level response.

6. Issue the vulnerabilities time query (Part B):
   ```
   {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"query","arguments":{"query":"FROM claroty_vulnerabilities SELECT time","client_id":"<test_client>","limit":5}}}
   ```
   Capture the full wire-level response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.017 | EC-016-017-007: `device_uid` is a Tier-1 queryable column in `claroty_device_vulnerability_relations`; not restricted to raw_extensions extraction | Parts A-1 and A-2 |
| BC-2.16.003 | EC-016-013-045: Tier-1 promotion of `device_uid` via `ocsf_field = "device.uid"` → Arrow column `device_uid` (§K5 §Postconditions mechanism) | Part A: top-level column |
| BC-2.16.003 | EC-016-013-043/044: Tier-1 OCSF `time` column from `published_date` datetime source via `ocsf_field = "time"` on vulnerabilities table | Parts B-1 and B-2 |

---

## Verification Approach

1. **Part A — device_uid:**
   Parse Part A response. If JSON-RPC error: inspect message.
   - If message contains "device_uid" and ("not found" or "E-QUERY-038"): record FAIL (Issue 12 not fixed).
   - If sensor unavailable or 0 rows: record SETUP-FAILURE.
   - If non-error: extract `rows` array. For each row, assert `device_uid` is a top-level key with
     a non-null string value. Assert it does NOT appear as a key inside `raw_extensions` string only.

2. **Part B — time:**
   Parse Part B response. If JSON-RPC error: inspect message.
   - If message contains "time" and ("not found" or "E-QUERY-038"): record FAIL (Issue 11 not fixed).
   - If sensor unavailable or 0 rows: record SETUP-FAILURE.
   - If non-error: extract `rows` array. For at least one row, assert `time` is a top-level key with
     a non-null value (ISO-8601 string or datetime). Assert it is NOT nested inside raw_extensions.

3. All assertions on the serialized JSON wire bytes.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Part A non-error (Issue 12 discriminator)** (weight: 0.40):
  Full credit (1.0): non-error response from device_vulnerability_relations SELECT device_uid.
  Zero credit (0.0): E-QUERY-038 "device_uid not found" error (pre-patch behavior — TOML fix not effective).
  SETUP-FAILURE: sensor unavailable; score 0.0 and flag separately.

- **device_uid as top-level column in rows (Part A-2)** (weight: 0.20):
  Full credit (1.0): at least one row has `device_uid` as a top-level non-null string.
  Zero credit (0.0): rows present but no `device_uid` key found (routing issue).

- **Part B non-error (Issue 11 discriminator)** (weight: 0.30):
  Full credit (1.0): non-error response from vulnerabilities SELECT time.
  Zero credit (0.0): E-QUERY-038 "time not found" error (pre-patch behavior — ocsf_field = "time" not added).
  SETUP-FAILURE: sensor unavailable; score 0.0 and flag separately.

- **time as top-level column in rows (Part B-2)** (weight: 0.10):
  Full credit (1.0): at least one row has `time` as a top-level non-null value.
  Zero credit (0.0): rows present but no `time` column found.

---

## Edge Conditions

- **Zero device_vulnerability_relations rows on live sensor:** Both parts may hit SETUP-FAILURE.
  Record separately from behavioral assertion failure. Previous HS-026 (S-CLAROTY-DEVVULNREL-001)
  was CONSUMED against monroe with live data — this table had rows at the time.

- **vulnerabilities.time NULL for all rows:** Some vulnerabilities may have null `published_date`.
  A null `time` in the wire response is still a top-level column (just null-valued). The assertion is
  on column presence in the schema, not on non-null values specifically. Part B-2 requires non-null
  for full credit; if all rows are null-time, record half credit (column exists but all-null).

- **device_uid present as both Tier-1 and inside raw_extensions:** After the fix, `device_uid`
  is Tier-1 and should NOT also appear inside `raw_extensions`. If it appears in both places,
  record PARTIAL (Part A-2 non-regression concern). The expected post-fix behavior is Tier-1 ONLY.

- **SELECT time returns raw_extensions column only:** If `time` is still inside raw_extensions
  (partial fix), Part B-2 may score zero credit. The evaluator should check the key is top-level,
  not inside a `raw_extensions` JSON blob.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-COSTOML-001-002 (satisfaction: X.XX) — one or both TOML Tier-1 promotion fixes not effective at wire level; check (a) device_uid ocsf_field = 'device.uid' in claroty.sensor.toml device_vulnerability_relations table (BC-2.16.017 EC-016-017-007 Issue 12) and (b) published_date ocsf_field = 'time' in vulnerabilities table (BC-2.16.003 EC-016-013-043 Issue 11)"`

Do NOT disclose: the specific E-QUERY-038 error text, the row values observed, or which
specific table (Part A vs Part B) failed.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-CLAROTY-OCSF-TOML-001 branch + live Claroty sensor (monroe tenant). No DTU required. |
| corpus_size | claroty_device_vulnerability_relations and claroty_vulnerabilities tables; live data from monroe |
| known_edge_cases | Zero rows on live tenant for one or both tables: SETUP-FAILURE for that Part. Previous G3 (HS-026) confirmed device_vulnerability_relations has live data on monroe. |
| false_positive_threshold | Near-zero: E-QUERY-038 for device_uid and time are the exact pre-patch failures; non-error for both confirms the TOML fixes landed |
| false_negative_threshold | Near-zero: TOML-only fixes either work (non-error query) or don't (E-QUERY-038 persists) |

**Known-good corpus:** post-fix binary with `ocsf_field = "device.uid"` on device_uid column and `ocsf_field = "time"` on published_date column in claroty.sensor.toml. Expected: both queries succeed with top-level columns.

**Known-problematic corpus:** pre-patch develop HEAD. Expected: E-QUERY-038 for both `device_uid` and `time` SELECT queries on their respective tables.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w3-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-039 group for S-CLAROTY-OCSF-TOML-001. Compound test covering Issues 11 (vulnerabilities time Tier-1) and 12 (device_uid Tier-1). BC-2.16.017 EC-016-017-007 + BC-2.16.003 EC-016-013-043. No DTU — live monroe sensor. SINGLE-USE HIDDEN. |
