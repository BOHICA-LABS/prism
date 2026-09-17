---
document_type: holdout-scenario
level: L3
id: "HS-NULL-001-002"
title: "Array-typed string columns with null elements produce compact JSON-list without 'null' string elements in raw_extensions"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "E-BETA3-REMEDIATION"
story_source: "S-MCP-NULL-ENCODING-001"
version: "1.0"
status: active
used: false
last_evaluated: null
last_eval_satisfaction: null
single_use: true
producer: product-owner
timestamp: "2026-09-16T00:00:00Z"
modified: "2026-09-16"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md"
input-hash: "TBD"
traces_to: "BC-2.16.003"
behavioral_contracts:
  - BC-2.16.003
verification_properties: []
lifecycle_status: active
introduced: "S-MCP-NULL-ENCODING-001"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-MCP-NULL-ENCODING-001 (HS-035 group). Validates Issue 7b fix: Value::Array arm in ColumnType::String branch of build_column_array was serializing Value::Null elements via to_string() producing the element 'null' in compact JSON-list strings. Fix: filter out null elements before joining. Discriminating assertion: raw_extensions for list columns must NOT contain compact JSON-list values that include the string element 'null' (e.g., '[\"null\"]' pattern). Also asserts that valid non-null list column values are unchanged (EC-016-013-026 non-regression). BC-2.16.003 EC-016-013-041 new. Test-writer and implementer must NOT read this file."
---

# HS-NULL-001-002: Array-typed string columns with null elements produce filtered compact JSON-list (no "null" string elements) in raw_extensions

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-MCP-NULL-ENCODING-001 (HS-035 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.003 new EC-016-013-041: `Value::Null` elements within a
`Value::Array` input in the String arm of `build_column_array` MUST be omitted from the
compact JSON-list string output. An all-null array produces `"[]"`. A mixed array
`[null, "192.168.1.1", null]` produces `"[\"192.168.1.1\"]"`. The string `"null"` must
NOT appear as an element in any compact JSON-list value stored under `raw_extensions`.
**Gate:** Story-level holdout gate (HS-035) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the **Issue 7b fix** — null-element filtering in the `Value::Array`
arm of `build_column_array` — via end-to-end inspection of `raw_extensions` in the MCP wire
output.

**Context:** Tier-2 string columns that receive `Value::Array` inputs from the sensor API
are serialized as compact JSON-list strings and stored under `raw_extensions` in the OCSF
record. Pre-fix, `Value::Null` elements within those arrays were passed through
`other.to_string()`, producing the string element `"null"` in the compact list
(e.g., `["null"]` or `["null","192.168.1.1"]`). Post-fix, null elements are filtered before
joining, so `[null]` → `"[]"` and `[null, "192.168.1.1", null]` → `"[\"192.168.1.1\"]"`.

**The defect this scenario catches:** Pre-fix `raw_extensions` contains compact JSON-list
values where at least one element is the string `"null"`. For example, a Claroty device
with a partially-populated `ip_list` (some interfaces having null IPs) would produce
`"[\"null\",\"192.168.1.1\"]"` instead of `"[\"192.168.1.1\"]"`. Downstream agents
parsing this compact list would encounter the string `"null"` as an apparent IP address.

**Three assertions in this scenario:**

**Part A — raw_extensions does not contain the string "null" as a list element:**
Query `SELECT raw_extensions FROM claroty_devices LIMIT 10`.
Parse each row's `raw_extensions` JSON object. For each key whose value is a compact
JSON-list string (`[...]`), parse that inner list and assert no element equals the string `"null"`.

**Part B — Non-null list column values are preserved (EC-016-013-026 regression guard):**
In the same query result, for any row where `raw_extensions` has a non-empty compact list value
(e.g., `ip_list` with at least one IP), assert the non-null elements are present and the
list is a valid JSON array string. This guards against the fix accidentally discarding non-null elements.

**Part C — Asserting valid list structure (structural health check):**
All compact JSON-list values in `raw_extensions` must be valid JSON arrays when parsed.
A malformed list (e.g., `[,"192.168.1.1"]`) would indicate a serialization error in the fix.

**BDD supplement (Part A):**

**Given** prism is built from the S-MCP-NULL-ENCODING-001 story branch
**And** the Claroty DTU is running for client `"holdout-null-test"`
**When** `tools/call query {client_id: "holdout-null-test", query: "SELECT raw_extensions FROM claroty_devices LIMIT 10"}` is issued
**Then** the response is not a JSON-RPC error
**And** for every row in the result, `raw_extensions` is valid JSON
**And** for every compact JSON-list value in any raw_extensions object, no element equals the string `"null"`

---

## Setup Instructions

1. Build prism from the S-MCP-NULL-ENCODING-001 story branch.

2. Start the Claroty DTU. The DTU fixtures for claroty_devices include list columns
   (`ip_list`, `mac_list`, `vlan_list`, `network_list`, etc.). These are string-typed Tier-2
   columns that aggregate via the `Value::Array` arm.
   SETUP-FAILURE if Claroty DTU is not reachable.

3. Configure client `"holdout-null-test"` with `claroty_devices` table available.

4. Start prism in MCP stdio mode. Complete MCP initialize handshake.

5. **Step A query:**
   `{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"query","arguments":{"client_id":"holdout-null-test","query":"SELECT raw_extensions FROM claroty_devices LIMIT 10"}}}`
   Capture raw wire-level JSON response.

6. **Step B query (non-null regression guard):**
   `{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"query","arguments":{"client_id":"holdout-null-test","query":"SELECT raw_extensions FROM claroty_devices WHERE raw_extensions IS NOT NULL LIMIT 5"}}}`
   Capture raw wire-level JSON response.

Note on SETUP-FAILURE condition for Part A: if the DTU returns 0 devices OR all device
rows have empty list columns (no arrays at all in raw_extensions), record SETUP-FAILURE for
Part A. Part B is still verifiable if at least one row has non-null raw_extensions.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.003 | EC-016-013-041 new: Value::Null elements in Value::Array arm omitted; all-null array → "[]" | Part A: no "null" string element in compact JSON-list values |
| BC-2.16.003 | §Invariants null-vs-absent: nulls are not fabricated as string data | Part A: "null" element absence enforces the invariant |
| BC-2.16.003 | EC-016-013-026: Value::Array arm for non-null elements unchanged (ENRICH-1 path) | Part B: non-null elements still present after fix |

---

## Verification Approach

1. Verify Step A response is non-error. Parse `result.content[0].text` as JSON. Extract `rows`.
   If `rows.length == 0`: SETUP-FAILURE for Part A (no device rows in DTU fixture).

2. **Part A — no "null" element in any compact list:**
   For each row in `rows`:
     Extract the `raw_extensions` field value.
     If `raw_extensions` is null or absent: skip this row (no compact lists to check).
     Parse `raw_extensions` as a JSON object.
     For each value in the object:
       If the value is a string that starts with `[` and ends with `]` (compact JSON-list):
         Attempt to parse the inner content as a JSON array.
         If the inner content IS a valid JSON array: check each element.
           Assert no element equals the JSON string `"null"`.
           If any element == `"null"` (the string): record FAIL on Part A with the key name
           and the offending value.
         If the inner content is NOT a valid JSON array: record as FINDING (malformed list; Part C).

3. **Part B — non-null elements preserved:**
   Using the same rows from Step A (or Step B response):
   Find any compact JSON-list value with at least one element.
   Assert that element is not empty and is a valid non-null string.
   If all compact lists are `"[]"` (all-null arrays): record SETUP-FAILURE for Part B
   (DTU has all-null list columns; non-regression cannot be verified).

4. **Part C — structural validity:**
   All compact JSON-list values MUST be parseable as JSON arrays. If any is malformed:
   record FAIL on Part C with the malformed value.

5. All assertions on PARSED JSON from wire bytes (`result.content[0].text`).

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response non-error and rows >= 1 (prerequisite)** (weight: 0.10):
  Full credit (1.0): non-error response with rows.length >= 1.
  Zero credit (0.0): error response or SETUP-FAILURE (no rows).

- **Part A — no "null" string element in compact JSON-list values** (weight: 0.60):
  Full credit (1.0): no compact list in any row's raw_extensions contains the string element "null";
  OR no compact lists present in any row (SETUP-FAILURE for Part A — record note but do not penalize).
  Zero credit (0.0): at least one compact list contains `"null"` as an element — Issue 7b defect.
  SETUP-FAILURE (0.5 credited, not zero): DTU has 0 rows or 0 compact list columns.

- **Part B — non-null elements preserved** (weight: 0.20):
  Full credit (1.0): at least one non-empty compact list found with valid non-null elements; OR
  SETUP-FAILURE noted (all lists are "[]" or no lists found — not penalized).
  Zero credit (0.0): a compact list exists that should have non-null elements but has `"[]"` or is empty
  (fix accidentally discards all elements, not just null ones — EC-016-013-026 regression).

- **Part C — structural validity of compact lists** (weight: 0.10):
  Full credit (1.0): all compact list strings parse as valid JSON arrays.
  Zero credit (0.0): at least one compact list is malformed (serialization error in fix).

---

## Edge Conditions

- **DTU has no device rows:** SETUP-FAILURE for Part A and Part B. Notify with the fixture
  details. This scenario may need to be re-run against a DTU with device fixture data.

- **All device rows have no array columns (all list columns are scalar or absent):** SETUP-FAILURE
  for Part A. If `ip_list`, `mac_list`, `vlan_list`, etc. are all absent from the raw_extensions
  for all rows, the compact-list assertion cannot be exercised.

- **DTU fixture has no null array elements (all lists have only non-null values):** Part A passes
  trivially (no "null" elements present). Part B passes (non-null elements preserved). The
  SETUP-FAILURE note is recorded for context but the scenario passes.

- **Mix of null and non-null elements in one list:** The fix should filter the nulls and retain
  the non-nulls. The combined result should contain only the non-null elements.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-NULL-001-002 (satisfaction: X.XX) — array null element filtering gap in raw_extensions; check build_column_array Value::Array arm ColumnType::String branch (BC-2.16.003 EC-016-013-041: Value::Null elements must be omitted from compact JSON-list output; all-null array must produce '[]'; non-null elements must be preserved per EC-016-013-026)"`

Do NOT disclose: which specific column or key had the failing value, the exact compact list
content observed, or which row was problematic.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Claroty DTU (prism-dtu-claroty devices fixture) via MCP stdio — story branch build |
| corpus_size | up to 10 claroty_devices rows; list columns: ip_list, mac_list, vlan_list, network_list, etc. |
| known_edge_cases | DTU fixture with no null array elements: Part A trivially passes (no regressions to find) |
| false_positive_threshold | Near-zero: a legitimate sensor value equal to the string "null" is implausible for IP/MAC/VLAN lists |
| false_negative_threshold | Near-zero: pre-fix always stringifies null elements to "null"; post-fix never does |

**Known-good corpus:** S-MCP-NULL-ENCODING-001 branch with Issue 7b fix applied.
Expected: no `"null"` string element in any compact JSON-list in raw_extensions; non-null
elements unchanged; valid JSON array structure throughout.

**Known-problematic corpus:** pre-patch develop HEAD (before this story's 7b fix).
Expected: compact JSON-list values containing `"null"` as string elements (e.g., `["null"]`
for a device with all-null ip_list, or `["null","192.168.1.1"]` for mixed).

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w2-holdout-authoring | 2026-09-16 | product-owner | Initial authoring. HS-035 group for S-MCP-NULL-ENCODING-001. Issue 7b (array null element filtering): compact JSON-list values in raw_extensions must not contain the string element "null"; all-null arrays produce "[]"; non-null elements preserved (EC-016-013-026 guard). BC-2.16.003 EC-016-013-041 new. SINGLE-USE. |
