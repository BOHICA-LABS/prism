---
document_type: holdout-scenario
level: L3
id: "HS-COS-001-002"
title: "WHERE device_is_online = true executes without E-QUERY-038 and returns only true-online devices"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-CLAROTY-OCSF-STATUS-001 (HS-038 group). Validates BC-2.16.003 EC-016-013-040: WHERE device_is_online = true/false executes end-to-end via MCP prism_query surface without E-QUERY-038 column-not-found error. Discriminating: pre-patch returns E-QUERY-038 (device_is_online column does not exist); post-fix returns filtered rows. Claroty DTU required. SAP-3 reachability: must exercise from MCP query tool surface, not synthetic AST. Test-writer and implementer must NOT read this file."
---

# HS-COS-001-002: WHERE device_is_online = true executes without E-QUERY-038 and returns only true-online devices

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-CLAROTY-OCSF-STATUS-001 (HS-038 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.003 §Postconditions EC-016-013-040
  "WHERE device_is_online = true executes correctly at the query layer and returns only rows where the Arrow Boolean cell is true."
**Gate:** Story-level holdout gate (HS-038) — runs after LOCAL 3-CLEAN convergence, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario tests the query-layer end of the `device_is_online` fix: not only that the column exists in the schema (HS-COS-001-001 covers that), but that it is FILTERABLE via a PrismQL WHERE predicate exercised through the MCP `query` tool public API surface.

**The defect this scenario catches (pre-patch behavior):**

`FROM claroty_devices WHERE device_is_online = true` returns a JSON-RPC error because `device_is_online` is not a recognized column — it does not exist in the Arrow schema before the TOML fix. The error is typically E-QUERY-038 ("column device_is_online not found") or a DataFusion "Column not found" runtime error.

**Post-fix behavior:**

The same query executes successfully. Only rows where the DTU fixture has `"is_online": true` are returned. Rows where `is_online` is false or null are excluded (SQL three-valued logic: NULL comparison returns UNKNOWN, which is treated as false in a WHERE clause).

**Three wire-level assertions:**

**Part A — query succeeds (no E-QUERY-038 or DataFusion column error):**
- Assert: the JSON-RPC response has `result` (not `error`).
- Assert: `result.content[0].text` parses as a valid query response (not an error envelope).
- If error: assert the error.message does NOT contain "not found" or "E-QUERY-038" for "device_is_online"
  (which would indicate the column still doesn't exist post-fix).

**Part B — all returned rows have device_is_online = true:**
- Assert: every row in the `rows` array has `device_is_online` with JSON boolean value `true`.
- Assert: no returned row has `device_is_online` = `false` or `null`.

**Part C — anti-regression: query with false also executes (non-error):**
- Issue a second query: `FROM claroty_devices WHERE device_is_online = false`.
- Assert: response is non-error (same non-error gate as Part A).
- This confirms the fix applies to both boolean directions, not just a special-case for `true`.

**BDD supplement:**

**Given** prism is built from the S-CLAROTY-OCSF-STATUS-001 story branch
**And** the Claroty DTU is running with fixtures containing at least one device with `is_online=true` and one with `is_online=false`
**When** `tools/call query {"query": "FROM claroty_devices WHERE device_is_online = true", "client_id": "<test_client>"}` is issued via MCP stdio
**Then** the response is not a JSON-RPC error
**And** every returned row has `device_is_online` equal to JSON boolean `true`
**When** a second query `FROM claroty_devices WHERE device_is_online = false` is issued
**Then** the second response is also not a JSON-RPC error

---

## Setup Instructions

1. Build prism from the S-CLAROTY-OCSF-STATUS-001 story branch.

2. Start the Claroty DTU with updated fixtures (T-F02: `fixtures/devices.json` seeded with
   at least one `is_online=true` device and one `is_online=false` device).

   SETUP-FAILURE condition: DTU not startable as standalone process, or fixture has no
   is_online=true devices. Record SETUP-FAILURE; do not score behavioral assertions.

3. Configure `prism.toml` with a test client pointing at the DTU.

4. Start prism in MCP stdio mode. Complete `initialize` handshake.

5. Issue the WHERE true query:
   ```
   {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query","arguments":{"query":"FROM claroty_devices WHERE device_is_online = true","client_id":"<test_client>"}}}
   ```
   Capture the full wire-level response.

6. Issue the WHERE false query (for Part C anti-regression):
   ```
   {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"query","arguments":{"query":"FROM claroty_devices WHERE device_is_online = false","client_id":"<test_client>"}}}
   ```
   Capture the full wire-level response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.003 | EC-016-013-040: `WHERE device_is_online = true/false` executes at the query layer without error; filters correctly | Parts A, B, C |
| BC-2.16.003 | EC-016-013-034..035: device_is_online is a first-class Boolean Tier-1 column (prerequisite for filterability) | Implicit — filterability requires the column to exist in the schema |

---

## Verification Approach

1. Parse the `WHERE device_is_online = true` response.
   - If JSON-RPC error: check `error.message`. If it contains "device_is_online" and ("not found" OR "E-QUERY-038"): record FAIL for Part A (pre-patch behavior confirmed).
   - If JSON-RPC error with a different message (e.g., sensor unavailable): record SETUP-FAILURE.
   - If non-error: proceed to Part B.

2. **Part B — filter correctness:**
   Extract `rows` array from `result.content[0].text` parsed as JSON.
   For each row, assert `row["device_is_online"] == true` (JSON boolean true, not string).
   If any row has `device_is_online != true` (false or null): record FAIL.

3. **Part C — false query non-error:**
   Parse the `WHERE device_is_online = false` response.
   Assert: non-error (no JSON-RPC `error` key, or `result` present).
   Record the result (may be empty rows if no false-devices are in fixture — that is acceptable; what matters is non-error).

4. All assertions on serialized wire bytes.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Query executes without column-not-found error (Part A — primary discriminator)** (weight: 0.60):
  Full credit (1.0): response is non-error (result present, no JSON-RPC error).
  Zero credit (0.0): JSON-RPC error referencing "device_is_online" as not found (pre-patch behavior).
  SETUP-FAILURE (0.0): sensor unavailable; record separately.

- **All returned rows have device_is_online = true (Part B — filter correctness)** (weight: 0.25):
  Full credit (1.0): all rows pass the boolean-true assertion.
  Partial credit (0.5): rows returned but some have non-true values (filter not working).
  Zero credit (0.0): no rows returned (SETUP-FAILURE if fixture has no true-state devices).

- **false-query also non-error (Part C — anti-regression)** (weight: 0.15):
  Full credit (1.0): WHERE false query is also non-error.
  Zero credit (0.0): WHERE false query errors while WHERE true succeeded (asymmetric bug).

---

## Edge Conditions

- **Fixture has no is_online=true devices:** Part B yields zero rows. Score Part B as SETUP-FAILURE
  (cannot distinguish pre-patch column error from empty-result empty rows without seeing an actual row).
  The Part A assertion (non-error) is still valid and discriminating.

- **DTU returns is_online for all devices as null:** All rows excluded by WHERE true predicate
  (SQL three-valued logic excludes NULLs). Zero rows returned. Part B: SETUP-FAILURE for filter-correctness
  (no evidence one way or the other). Part A non-error assertion still discriminating.

- **prism_query returns is_truncated=true:** Acceptable. The scenario does not constrain pagination.
  The filter-correctness assertion applies to the rows returned, regardless of truncation.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-COS-001-002 (satisfaction: X.XX) — WHERE device_is_online = true returned an error; check device_is_online column exists in claroty_devices Arrow schema after TOML fix; BC-2.16.003 EC-016-013-040 SAP-3 reachability via MCP query surface (ADR-058 §K5 D-2522 Option A)"`

Do NOT disclose: the specific error message text observed, the number of rows returned,
or whether the false-direction predicate succeeded or failed.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-CLAROTY-OCSF-STATUS-001 branch + Claroty DTU devices fixture with is_online=true and is_online=false devices (T-F02) |
| corpus_size | claroty_devices table; WHERE predicate filters to is_online=true rows only |
| known_edge_cases | Fixture with only null is_online: WHERE true returns empty rows (SETUP-FAILURE for Part B; Part A non-error still discriminating). |
| false_positive_threshold | Zero: a successful query (non-error) for a column that previously did not exist is definitive proof of the fix |
| false_negative_threshold | Near-zero: pre-patch always errors on device_is_online column reference; post-fix always succeeds |

**Known-good corpus:** post-fix binary with TOML mapping in place. Expected: non-error query response with rows containing `device_is_online: true`.

**Known-problematic corpus:** pre-patch develop HEAD. Expected: JSON-RPC error because `device_is_online` column does not exist in the Arrow schema.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w3-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-038 group for S-CLAROTY-OCSF-STATUS-001. Tests WHERE device_is_online = true/false predicate end-to-end via MCP query tool surface (SAP-3 reachability). BC-2.16.003 EC-016-013-040. Claroty DTU required. SINGLE-USE HIDDEN. |
