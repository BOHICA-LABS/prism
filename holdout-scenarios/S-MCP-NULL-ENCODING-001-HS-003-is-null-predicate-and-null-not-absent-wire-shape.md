---
document_type: holdout-scenario
level: L3
id: "HS-NULL-001-003"
title: "IS NULL predicate correctly identifies null string cells, and null column values appear as explicit JSON null (not absent key) in wire output"
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
  - ".factory/specs/behavioral-contracts/BC-2.11.001-query-mcp-tool.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md"
input-hash: "TBD"
traces_to: "BC-2.11.001"
behavioral_contracts:
  - BC-2.11.001
  - BC-2.16.003
verification_properties: []
lifecycle_status: active
introduced: "S-MCP-NULL-ENCODING-001"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-MCP-NULL-ENCODING-001 (HS-035 group). Tests the end-to-end behavioral impact of the Issue 7a fix: (1) IS NULL predicate operates on true Arrow null cells (not the string 'null'), so it finds rows that genuinely have null sensor data; (2) null-not-absent wire-shape discipline (BC-2.11.001 EC-11-079): null cells appear as explicit JSON null key (not absent key) in MCP tool output. This scenario exercises the full chain from sensor null → Arrow null cell → IS NULL DataFusion predicate + explicit_nulls wire serialization. BC-2.11.001 EC-11-079 + BC-2.16.003 EC-016-013-006 amended. Test-writer and implementer must NOT read this file."
---

# HS-NULL-001-003: IS NULL predicate correctly identifies Arrow null cells; null values appear as explicit JSON null (not absent key) in wire output

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-MCP-NULL-ENCODING-001 (HS-035 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.11.001 EC-11-079 (null-not-absent wire-shape): when a column value is an
Arrow null cell, the MCP tool output MUST contain the column key with value `null` (JSON null),
NOT omit the key entirely. `WriterBuilder::with_explicit_nulls(true)` ensures this. Also
BC-2.16.003 EC-016-013-006 amended: the upstream `build_column_array` fix feeds EC-11-079 —
once null inputs produce Arrow null cells, explicit_nulls serializes them as `null`, not absent.
**Gate:** Story-level holdout gate (HS-035) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario tests the **end-to-end behavioral impact** of the Issue 7a null-encoding fix:
the IS NULL predicate and the null-not-absent wire-shape requirement.

**The full chain under test:**
1. Claroty DTU emits JSON `null` for an optional string column.
2. `build_column_array` produces Arrow null cell (`None`).
3. DataFusion `IS NULL` predicate correctly evaluates to true for that cell.
4. `WriterBuilder::with_explicit_nulls(true)` serializes the null cell as `null` (not absent key).
5. Wire output row: `{"source_ip": null}` — key present, value is JSON `null`.

**Why this matters:** Pre-fix, the chain was broken at step 2 (null input became the string "null"),
making steps 3 and 4 produce wrong results:
- `WHERE source_ip IS NULL` returned 0 rows (no true null cells existed).
- The wire output showed `"source_ip": "null"` (a 4-char string), which an LLM agent
  could not distinguish from a sensor that genuinely reported "null" as a string value.

**Three assertions in this scenario:**

**Part A — IS NULL predicate correctly finds null rows (if any exist in DTU data):**
Run `SELECT source_ip FROM claroty_alerts WHERE source_ip IS NULL LIMIT 5`.
The result is either:
  - N rows with `source_ip: null` in the wire output — correct behavior (true null cells).
  - 0 rows — acceptable if the DTU fixture has no null source_ip values; record SETUP-FAILURE
    for Part A and proceed to Part B.

**Part B — null-not-absent wire shape (EC-11-079):**
Run `SELECT source_ip FROM claroty_alerts LIMIT 10`.
For any row where `source_ip` is null in the Arrow column, assert the wire JSON contains
`"source_ip": null` (key PRESENT, value JSON `null`) — NOT `{}` (key absent).
This directly verifies `WriterBuilder::with_explicit_nulls(true)` is wired and active.

**Part C — IS NULL + IS NOT NULL cover the full set of rows:**
Run `SELECT COUNT(*) FROM claroty_alerts WHERE source_ip IS NULL` → N_null rows.
Run `SELECT COUNT(*) FROM claroty_alerts WHERE source_ip IS NOT NULL` → N_not_null rows.
Run `SELECT COUNT(*) FROM claroty_alerts` → N_total rows.
Assert `N_null + N_not_null == N_total` (no rows are "lost" between the two predicates).
Pre-fix failure: N_null would be 0 (string "null" doesn't satisfy IS NULL), making
N_null + N_not_null < N_total when some rows have null source_ip in the raw data.

**BDD supplement (Part B):**

**Given** prism is built from the S-MCP-NULL-ENCODING-001 story branch
**And** the Claroty DTU is running for client `"holdout-null-test"` with claroty_alerts loaded
**When** `tools/call query {client_id: "holdout-null-test", query: "SELECT source_ip FROM claroty_alerts LIMIT 10"}` is issued
**Then** the response is not a JSON-RPC error
**And** for every row where source_ip is null in the sensor data, the wire JSON contains `"source_ip": null` (key present, not absent)
**And** no row contains `"source_ip": "null"` (the string — not JSON null)

---

## Setup Instructions

1. Build prism from the S-MCP-NULL-ENCODING-001 story branch.

2. Start the Claroty DTU. SETUP-FAILURE if DTU is not reachable.

3. Configure client `"holdout-null-test"` with claroty_alerts. Start prism in MCP stdio mode.
   Complete MCP initialize handshake.

4. **Step A — IS NULL predicate:**
   `{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"query","arguments":{"client_id":"holdout-null-test","query":"SELECT source_ip FROM claroty_alerts WHERE source_ip IS NULL LIMIT 5"}}}`

5. **Step B — projection for null-not-absent check:**
   `{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"query","arguments":{"client_id":"holdout-null-test","query":"SELECT source_ip FROM claroty_alerts LIMIT 10"}}}`

6. **Step C — partition coverage check:**
   Issue three count queries sequentially (id=4, id=5, id=6):
   - `SELECT COUNT(*) FROM claroty_alerts WHERE source_ip IS NULL` (id=4)
   - `SELECT COUNT(*) FROM claroty_alerts WHERE source_ip IS NOT NULL` (id=5)
   - `SELECT COUNT(*) FROM claroty_alerts` (id=6)

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.001 | EC-11-079 null-not-absent: Arrow null cell → JSON null key (not absent), explicit_nulls=true | Part B: source_ip key present with null value |
| BC-2.16.003 | EC-016-013-006 amended: Value::Null → Arrow null cell (feeds EC-11-079 chain) | Part A: IS NULL finds true null cells |
| BC-2.16.003 | §Invariants null-vs-absent: null placed as JSON null value, not absent | Part B and Part C: no "missing" rows |

---

## Verification Approach

1. Verify all responses are non-error.

2. **Part A — IS NULL predicate (Step A response):**
   Parse `result.content[0].text` as JSON. Extract `rows`.
   For each row in `rows`, assert that `source_ip` is JSON `null` (not a non-null string, not absent).
   If `rows.length == 0`: record SETUP-FAILURE for Part A (DTU has no null source_ip rows).
   Note: a 0-row result is acceptable behavior; the scenario continues to Part B.

3. **Part B — null-not-absent wire shape (Step B response):**
   Parse `result.content[0].text` as JSON. Extract `rows`.
   Identify any row where the raw_extensions or source_ip might reveal a null value.
   For each row, check: IF `source_ip` is null in the sensor data (inferred from the IS NULL
   result in Step A — if Step A returned rows, those same row IDs should have null source_ip
   in Step B), THEN `source_ip` key MUST be PRESENT in the wire JSON row object with value `null`,
   NOT absent.
   Assert: no row has `source_ip` == `"null"` (the string "null").

4. **Part C — partition coverage (Step C responses):**
   Extract the count values from id=4, id=5, id=6 responses.
   Parse the count from the rows (e.g., `rows[0]["COUNT(*)"]` or similar column name).
   Assert: count_null + count_not_null == count_total.
   If the assertion fails: record FAIL on Part C dimension (a gap between IS NULL and IS NOT NULL
   coverage indicates null cells are being "lost" — possible regression of explicit_nulls or
   Arrow null handling).

5. All assertions on PARSED JSON from wire bytes.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Responses non-error (prerequisite)** (weight: 0.10):
  Full credit (1.0): all queries return non-error responses.
  Zero credit (0.0): any query errors (SETUP-FAILURE if DTU unreachable).

- **Part A — IS NULL finds null cells correctly** (weight: 0.20):
  Full credit (1.0): either N > 0 rows returned with source_ip: null in wire JSON, OR
  0 rows returned (SETUP-FAILURE noted; DTU has no null source_ip — not a behavioral fail).
  Zero credit (0.0): N > 0 rows returned but source_ip values are non-null (IS NULL returned
  wrong rows — DataFusion predicate operating on incorrect Arrow types).
  Note: if Step A returns 0 rows while Part C Count-null > 0, record FAIL (IS NULL not finding
  the null cells).

- **Part B — null-not-absent wire shape (explicit_nulls)** (weight: 0.40):
  Full credit (1.0): any null cells that exist appear as key:null in wire JSON (not absent).
  OR no null cells observed (SETUP-FAILURE noted; pass credited).
  Zero credit (0.0): null cells appear as absent keys in wire JSON — explicit_nulls not active.

- **Part C — partition coverage N_null + N_not_null == N_total** (weight: 0.30):
  Full credit (1.0): the three counts sum correctly.
  Zero credit (0.0): N_null + N_not_null != N_total — rows are lost between IS NULL and IS NOT NULL
  coverage (pre-fix: rows with null source_ip stored as "null" string are in IS NOT NULL,
  not IS NULL; sum never equals total if those exist).

---

## Edge Conditions

- **DTU has no rows with null source_ip:** Part A is SETUP-FAILURE (0 rows). Part C: N_null=0,
  N_not_null=N_total, sum is correct. Parts B and C still verify correct behavior.

- **COUNT(*) column name in wire output varies:** The column name for COUNT(*) aggregates may
  appear as `COUNT(*)`, `count`, or a function-specific alias. Parse the first (and only) column
  in the first (and only) row of each count query result.

- **No rows in claroty_alerts at all:** SETUP-FAILURE for all parts. N_total=0; assertions
  pass trivially. Record SETUP-FAILURE with the empty DTU fixture.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-NULL-001-003 (satisfaction: X.XX) — null predicate or null-not-absent wire-shape gap; check (1) IS NULL predicate operating on correct Arrow null cells [BC-2.16.003 EC-016-013-006 amended: Value::Null => None in build_column_array]; (2) WriterBuilder explicit_nulls=true serializing null cells as key:null not absent [BC-2.11.001 EC-11-079]"`

Do NOT disclose: the specific counts, the predicate used, or which rows revealed the failure.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Claroty DTU (prism-dtu-claroty alerts fixture) via MCP stdio — S-MCP-NULL-ENCODING-001 branch build |
| corpus_size | up to 10 claroty_alerts rows from DTU fixture; 3 count queries for partition coverage |
| known_edge_cases | DTU has no null source_ip: IS NULL returns 0 rows (SETUP-FAILURE noted; partition coverage still tested) |
| false_positive_threshold | Near-zero: IS NULL predicate behavior and key-present vs key-absent are unambiguous |
| false_negative_threshold | Near-zero: pre-fix IS NULL on "null" string cells returns wrong results; partition count gap is detectable |

**Known-good corpus:** S-MCP-NULL-ENCODING-001 branch with Issue 7a fix preserved (develop
already has it). Expected: IS NULL returns rows with key:null; N_null+N_not_null==N_total;
source_ip absent from wire when null with explicit_nulls enabled.

**Known-problematic corpus:** A hypothetical regression where Value::Null => None is removed.
Expected: IS NULL finds 0 rows even when source_ip is null; partition count gap > 0;
source_ip appears as "null" string in wire output.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w2-holdout-authoring | 2026-09-16 | product-owner | Initial authoring. HS-035 group for S-MCP-NULL-ENCODING-001. End-to-end behavioral chain: IS NULL predicate correctness + null-not-absent wire shape (EC-11-079) + partition coverage (N_null + N_not_null == N_total). Tests the full sensor-null → Arrow-null → predicate + explicit_nulls chain. BC-2.11.001 EC-11-079 + BC-2.16.003 EC-016-013-006 amended. SINGLE-USE. |
