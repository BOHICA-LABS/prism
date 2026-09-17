---
document_type: holdout-scenario
level: L3
id: "HS-NULL-001-001"
title: "Lock-in regression guard: string-typed null sensor values materialize as wire JSON null, not the four-character string 'null'"
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
  - ".factory/specs/behavioral-contracts/BC-2.11.001-query-mcp-tool.md"
input-hash: "TBD"
traces_to: "BC-2.16.003"
behavioral_contracts:
  - BC-2.16.003
  - BC-2.11.001
verification_properties: []
lifecycle_status: active
introduced: "S-MCP-NULL-ENCODING-001"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-MCP-NULL-ENCODING-001 (HS-035 group). Lock-in regression guard for Issue 7a (ALREADY FIXED on develop per D-1110): build_column_array ColumnType::String arm has Value::Null => None (Arrow null cell) as its first explicit match. This scenario validates the current-correct behavior holds post-story-merge and would catch a regression. Discriminating assertion: WHERE source_ip = 'null' returns 0 rows (pre-fix: rows with null source_ip were stored as the string 'null' and matched this predicate). BC-2.16.003 EC-016-013-006 amended + BC-2.11.001 EC-11-079. Test-writer and implementer must NOT read this file."
---

# HS-NULL-001-001: Lock-in regression guard — string-typed null sensor values produce wire JSON null, not the string "null"

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-MCP-NULL-ENCODING-001 (HS-035 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.003 EC-016-013-006 (amended): Path A `build_column_array` returns
`None` (Arrow null cell) for `Value::Null` string input — NOT `Some("null")`. BC-2.11.001
EC-11-079 (null-not-absent): null Arrow cells serialize as JSON `null` (explicit key with
null value), NOT as an absent key, per `WriterBuilder::with_explicit_nulls(true)`.
**Lock-in status:** Issue 7a is ALREADY FIXED on develop (commit `fff6e28ba`, June 2026,
per D-1110 remove-uncertainty finding). This scenario is a REGRESSION GUARD — it verifies
the existing correct behavior survives the story-branch changes and confirms the full
end-to-end chain is intact.
**Gate:** Story-level holdout gate (HS-035) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates that the **Issue 7a fix is preserved** and the **full null-encoding
chain** (sensor null → Arrow null cell → wire JSON null, not absent) is intact after all
story changes.

The chain under test:
1. Claroty DTU emits a JSON null for an optional string column (e.g., `source_ip: null`).
2. `build_column_array` `ColumnType::String` arm hits `Value::Null => None` (Arrow null cell).
3. `WriterBuilder::with_explicit_nulls(true)` serializes the Arrow null cell as JSON `null`.
4. Wire output: `{"source_ip": null}` — NOT `{"source_ip": "null"}`, NOT `{}` (absent key).

**The regression this scenario catches:** If a code change accidentally removes or moves the
`Value::Null => None` arm in the String branch of `build_column_array` to a position after the
wildcard `other => other.to_string()` arm, null inputs would be stringified to "null" again.
An LLM agent reading the wire output would see `"source_ip": "null"` (a 4-char string) and
could not distinguish it from a legitimate sensor that reports "null" as a location.

**Two assertions in this scenario:**

**Part A — No `"null"` string values in nullable string columns:**
Run `SELECT source_ip FROM claroty_alerts LIMIT 10`.
For each row in the wire result, check that `source_ip` is NOT the JSON string `"null"`.
`source_ip` must be either: a valid IP string (non-null sensor data) OR JSON `null`
(Arrow null cell serialized by explicit_nulls) OR absent (tolerated with a process-gap note
if explicit_nulls is not enabled, but preferred absent: see Part B).

**Part B — WHERE col = 'null' predicate returns zero rows:**
Run `SELECT source_ip FROM claroty_alerts WHERE source_ip = 'null' LIMIT 1`.
Assert: the result contains 0 rows.
**This is the primary discriminating assertion.** Pre-fix: null source_ip values were stored
as the string "null" and this predicate would match them. Post-fix: no row has source_ip equal
to the four-character string "null" (real sensors do not report "null" as an IP address).

**BDD supplement (Part B):**

**Given** prism is built from the S-MCP-NULL-ENCODING-001 story branch
**And** the Claroty DTU is running and configured for client `"holdout-null-test"`
**When** `tools/call query {client_id: "holdout-null-test", query: "SELECT source_ip FROM claroty_alerts WHERE source_ip = 'null' LIMIT 1"}` is issued
**Then** the response is not a JSON-RPC error
**And** the rows array in the wire response contains exactly 0 entries

---

## Setup Instructions

1. Build prism from the S-MCP-NULL-ENCODING-001 story branch (standard build).

2. Start the Claroty DTU on its standard port. The DTU must be running for this scenario
   (unlike Story A scenarios, this scenario requires live MCP query execution).
   SETUP-FAILURE if the Claroty DTU is not reachable.

3. Prepare a `prism.toml` configuring a client `"holdout-null-test"` against the Claroty DTU
   with at least the `claroty_alerts` table loaded.

4. Start prism in MCP stdio mode. SETUP-FAILURE if prism fails to start.

5. Complete MCP `initialize` handshake.

6. **Step A — projection query:**
   `{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"query","arguments":{"client_id":"holdout-null-test","query":"SELECT source_ip FROM claroty_alerts LIMIT 10"}}}`
   Capture the full raw wire-level JSON response. SETUP-FAILURE if the query itself errors
   (e.g., `source_ip` column not available — check claroty.sensor.toml for the column).

7. **Step B — predicate query:**
   `{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"query","arguments":{"client_id":"holdout-null-test","query":"SELECT source_ip FROM claroty_alerts WHERE source_ip = 'null' LIMIT 1"}}}`
   Capture the full raw wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.003 | EC-016-013-006 amended: Path A `build_column_array` returns None (Arrow null cell) for Value::Null string input | Part A: no "null" string in source_ip column values |
| BC-2.11.001 | EC-11-079: null-not-absent wire-shape — Arrow null cell → JSON null key, NOT absent key | Part A: null values appear as explicit null, not absent |
| BC-2.16.003 | §Invariants null-vs-absent: "a column present with Value::Null is placed in its destination as a JSON null value" | Part B: WHERE source_ip = 'null' returns 0 rows |

---

## Verification Approach

1. Verify responses are not JSON-RPC errors for both Step A and Step B queries.

2. **Part A — row-level null encoding check (Step A response):**
   Parse `result.content[0].text` as JSON. Locate the `rows` array.
   For each row object in `rows`:
     Check the `source_ip` value. It MUST NOT equal the JSON string `"null"`.
     Acceptable values: any non-null JSON string (valid IP), JSON `null`, or absent key.
   If any row has `source_ip == "null"` (the string): record FAIL on Part A dimension.
   Note the number of rows returned (for context).

3. **Part B — predicate zero-row assertion (Step B response):**
   Parse `result.content[0].text` as JSON. Extract the `rows` array.
   Assert `rows.length == 0`.
   If `rows.length > 0`: record FAIL on Part B dimension — at least one row has
   `source_ip` stored as the literal string "null" (Issue 7a regression).

4. All assertions on PARSED JSON from `result.content[0].text` wire bytes.

5. **SETUP-FAILURE clause:** If the Step A query returns 0 rows (DTU has no claroty_alerts
   fixture data), record SETUP-FAILURE for Part A (not a behavioral FAIL). Part B can still
   execute: a zero-row alerts fixture means `WHERE source_ip = 'null'` also returns 0 rows
   correctly — record as PASS with a note about the empty fixture.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Queries non-error (prerequisite)** (weight: 0.10):
  Full credit (1.0): both Step A and Step B queries return non-error responses.
  Zero credit (0.0): either query errors (SETUP-FAILURE if DTU unreachable; FAIL if column unavailable).

- **Part A — no "null" string in source_ip values** (weight: 0.30):
  Full credit (1.0): no row has source_ip == "null" (JSON string), OR 0 rows returned (SETUP
  note but dimension passes trivially).
  Zero credit (0.0): at least one row has source_ip == "null" (Issue 7a regression).

- **Part B — WHERE source_ip = 'null' returns 0 rows** (weight: 0.60):
  Full credit (1.0): rows.length == 0.
  Zero credit (0.0): rows.length > 0 — Issue 7a regression: at least one row stores source_ip
  as the string "null" and the predicate finds it.

---

## Edge Conditions

- **DTU has no claroty_alerts rows:** Part A is SETUP-FAILURE (trivially clean). Part B passes
  correctly (0 rows, null predicate finds nothing). Record SETUP-FAILURE for Part A with note.

- **source_ip column not in claroty.sensor.toml spec:** SELECT source_ip errors. Check that
  the story branch spec file has source_ip defined as a nullable String column. If absent,
  use another nullable String column available in the spec.

- **all source_ip values are non-null in DTU fixture:** Part A passes (no "null" string values).
  Part B passes (predicate finds no rows). The lock-in is still valid — the fix is confirmed
  to not incorrectly stringify non-null values.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-NULL-001-001 (satisfaction: X.XX) — Issue 7a regression detected in null string encoding; check build_column_array ColumnType::String arm (BC-2.16.003 EC-016-013-006 amended: Value::Null must return None [Arrow null cell], not Some('null') [the 4-char string]; null-not-absent EC-11-079: explicit_nulls serialization must produce key:null not absent)"`

Do NOT disclose: the specific predicate tested, the exact row count, or which query was used.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Claroty DTU (prism-dtu-claroty alerts fixture) via MCP stdio against S-MCP-NULL-ENCODING-001 branch build |
| corpus_size | up to 10 claroty_alerts rows from DTU fixture |
| known_edge_cases | empty DTU fixture: SETUP-FAILURE for row-level check; predicate check still valid |
| false_positive_threshold | Near-zero: a legitimate source_ip value of the string "null" is not a real IP address |
| false_negative_threshold | Zero: pre-fix behavior stores null as "null" string; predicate match is deterministic |

**Known-good corpus:** S-MCP-NULL-ENCODING-001 branch with Issue 7a fix preserved (develop
HEAD already has it). Expected: 0 rows where source_ip == "null" string; WHERE source_ip = 'null'
returns 0 rows.

**Known-problematic corpus:** A hypothetical regression where Value::Null => None arm is
removed or moved after the wildcard. Expected: rows with null source_ip appear as "null" string;
WHERE source_ip = 'null' returns N > 0 rows.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w2-holdout-authoring | 2026-09-16 | product-owner | Initial authoring. HS-035 group for S-MCP-NULL-ENCODING-001. Lock-in regression guard for Issue 7a (already fixed on develop per D-1110). Primary assertion: WHERE source_ip = 'null' returns 0 rows — discriminates pre-fix (string "null" stored as value, predicate matches) from post-fix (Arrow null cell, predicate finds nothing). BC-2.16.003 EC-016-013-006 amended + BC-2.11.001 EC-11-079. SINGLE-USE. |
