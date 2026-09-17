---
document_type: holdout-scenario
level: L3
id: "HS-QTT-001-002"
title: "COUNT(*) aggregate query total_available equals the aggregate row count not the upstream sensor total"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "E-BETA3-REMEDIATION"
story_source: "S-QUERY-TRUE-TOTAL-001"
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
  - ".factory/stories/S-QUERY-TRUE-TOTAL-001-thread-upstream-sensor-total-count-through-pagination-pipeline.md"
input-hash: "TBD"
traces_to: "BC-2.11.001"
behavioral_contracts:
  - BC-2.11.001
verification_properties: []
lifecycle_status: active
introduced: "S-QUERY-TRUE-TOTAL-001"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-QUERY-TRUE-TOTAL-001 (HS-037 group). Validates BC-2.11.001 EC-11-095 MUST-9: aggregating plans (GROUP BY / aggregate functions like COUNT) must NOT have total_available inflated by upstream_total; the gate any_early_stopped=false must close for aggregate plans because all sensor rows are fetched before aggregation. Discriminating: a broken gate (always-open) gives total_available=upstream_count (e.g. 1200+ for claroty_devices) instead of total_rows=1 for COUNT(*). Claroty DTU required. Wire-level JSON assertion. Test-writer and implementer must NOT read this file."
---

# HS-QTT-001-002: COUNT(*) aggregate query total_available equals the aggregate row count not the upstream sensor total

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-QUERY-TRUE-TOTAL-001 (HS-037 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.11.001 §Postconditions EC-11-095 MUST-9:
"Aggregating plan (GROUP BY/HAVING): `total_available = total_rows` even with
`total_count_path` declared. The upstream item count predates aggregation and does not
apply to the number of output aggregate buckets. The gate (`any_early_stopped = false`)
naturally produces `total_available = total_rows` for complete aggregations."
ADR-060 §D8.11.5: for aggregate plans all sensor rows are fetched before GROUP BY, so
`any_early_stopped = false` (Condition A gate suppresses early-stop for aggregating plans).
**Gate:** Story-level holdout gate (HS-037) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the **gate-closed path of the gated formula**
(BC-2.11.001 EC-11-095 MUST-9; S-QUERY-TRUE-TOTAL-001 AC-009).

The gated formula (ADR-060 §D8.11.5) is:
```
total_available = if any_early_stopped {
    upstream_total.map(|n| n.max(total_rows)).unwrap_or(total_rows)
} else {
    total_rows                     // <- gate CLOSED
}
```

For `SELECT COUNT(*) FROM claroty_devices`:
- All device records must be fetched before COUNT can be computed → `any_early_stopped = false`.
- The gated formula gate is CLOSED → `total_available = total_rows`.
- `total_rows` for COUNT(*) = 1 (one aggregate output row with the count value).
- Therefore: `total_available = 1`, `returned_results = 1`.

**The defect this scenario catches — broken gate (always-open):**

If the gate logic has an inversion bug (gate always open regardless of `any_early_stopped`),
or if `upstream_total` is applied unconditionally:
```
total_available = upstream_total.map(|n| n.max(1)).unwrap_or(1)
```
Result: `total_available = N` (upstream total for claroty_devices, potentially > 1000).
But the actual query output is a single COUNT(*) row with value 1. `total_available = 1000`
is semantically wrong — the LLM agent would be told "1000 results available" for a query
that returned 1 aggregated result.

**Discriminating assertion:**

- `query_context.total_available == query_context.returned_results == 1` (or the actual
  COUNT value, which for a full-table COUNT(*) is 1 row regardless of device count).
- If `total_available > 1` AND `returned_results == 1`: the gate is broken (over-counts).
- If `total_available == 1`: gate is correct (closed for aggregate plan).

**Three assertions in this scenario:**

**Part A — total_available equals returned_results for aggregate queries:**
- Assert `query_context.total_available` is a JSON integer >= 1.
- Assert `query_context.returned_results == 1` (COUNT(*) returns exactly 1 row).
- Assert `query_context.total_available == query_context.returned_results`.
  (Both must be 1 for a COUNT(*) across all records.)

**Part B — is_truncated is false:**
- Assert `query_context.is_truncated == false`.
- A COUNT(*) over all records completes without truncation; `is_truncated = false` is expected.
- If `is_truncated == true`: the aggregate plan unexpectedly early-stopped.

**Part C — total_available is NOT inflated to upstream device count:**
- Supplementary guard: assert `query_context.total_available < 100`.
  (Even if the DTU has 1200 devices, COUNT(*) returns 1 row — total_available of 1. A
  broken gate would give a value in the hundreds or thousands. The threshold of 100 is
  generous: any value >= 100 is definitively broken for a COUNT(*) result.)

**BDD supplement:**

**Given** prism is built from the S-QUERY-TRUE-TOTAL-001 story branch
**And** the Claroty sensor TOML declares `total_count_path = "total"` on the `devices` table
**And** the Claroty DTU is running
**When** `tools/call query {"query": "SELECT COUNT(*) FROM claroty_devices", "client_id": "<test_client>"}` is issued via MCP stdio (no explicit limit)
**Then** the response is not a JSON-RPC error
**And** `query_context.returned_results == 1` (one aggregate row)
**And** `query_context.total_available == 1` (gate closed: equals total_rows)
**And** `query_context.is_truncated == false`
**And** `query_context.total_available < 100` (not inflated to upstream device count)

---

## Setup Instructions

1. Build prism from the S-QUERY-TRUE-TOTAL-001 story branch.

2. Start the Claroty DTU (same instance used for HS-QTT-001-001 if run sequentially).
   The DTU must serve the devices endpoint.

3. Prepare a `prism.toml` with the same Claroty test client configuration as
   HS-QTT-001-001 (`total_count_path = "total"` declared on claroty_devices in
   `claroty.sensor.toml`).

4. Start prism in MCP stdio mode. Complete the MCP `initialize` handshake.

5. Issue the `query` tool call without a LIMIT (or with a very high limit such as 10000):
   ```
   {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"query","arguments":{"query":"SELECT COUNT(*) FROM claroty_devices","client_id":"<test_client>"}}}
   ```
   Capture the full wire-level JSON response bytes.
   SETUP-FAILURE condition: prism errors, query returns a non-result error code, or the
   response does not contain `query_context`.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.001 | EC-11-095 MUST-9: aggregating plan total_available = total_rows even with total_count_path declared | Part A: total_available == returned_results == 1 |
| BC-2.11.001 | EC-11-095 MUST-10: is_truncated formula unchanged by upstream_total | Part B: is_truncated == false for complete aggregate |
| BC-2.11.001 | EC-11-095 MUST-7: early-stop gate any_early_stopped=false → total_available=total_rows even when upstream_total=Some(N) | Part C: total_available < 100 (not inflated) |

---

## Verification Approach

1. Parse wire-level JSON-RPC response. Verify non-error. If error: SETUP-FAILURE.

2. Parse `result.content[0].text` as JSON. Extract `query_context`. If absent: SETUP-FAILURE.

3. **Part A — total_available == returned_results:**
   Assert `query_context.returned_results == 1`.
   If `returned_results != 1`: record the value (COUNT(*) should always return 1 row;
   a different value suggests the query did not execute as a full aggregate).
   Assert `query_context.total_available` is a JSON integer.
   Assert `query_context.total_available == query_context.returned_results`.
   If `total_available != returned_results`: record FAIL — gate is broken (over-counts
   or under-counts).

4. **Part B — is_truncated:**
   Assert `query_context.is_truncated == false`.

5. **Part C — not inflated:**
   Assert `query_context.total_available < 100`.
   This is a supplementary guard in case Part A passes vacuously (e.g., returned_results
   is unexpectedly 100 due to a COUNT result artifact). The threshold 100 is generous.

6. All assertions on serialized JSON from `result.content[0].text`.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response is non-error (prerequisite)** (weight: 0.10):
  Full credit (1.0): non-error result, query_context present.
  Zero credit (0.0): error or missing context (SETUP-FAILURE).

- **total_available == returned_results (Part A — gate-closed verification)** (weight: 0.55):
  Full credit (1.0): total_available == returned_results == 1.
  Partial credit (0.3): total_available == returned_results but both > 1 (unusual COUNT behavior).
  Zero credit (0.0): total_available != returned_results (gate is broken — upstream total applied incorrectly).

- **is_truncated == false (Part B)** (weight: 0.20):
  Full credit (1.0): is_truncated is JSON boolean false.
  Zero credit (0.0): is_truncated == true (unexpected truncation for aggregate plan).

- **Not inflated to upstream count (Part C)** (weight: 0.15):
  Full credit (1.0): total_available < 100.
  Zero credit (0.0): total_available >= 100 (gate overcorrection — upstream total applied to aggregate result).

---

## Edge Conditions

- **COUNT(*) returns 0 (empty table in DTU):** If the Claroty DTU devices fixture is empty,
  COUNT(*) returns 1 row with value 0. `returned_results = 1`, `total_available = 1`.
  Parts A/B/C all pass. Acceptable (not SETUP-FAILURE; the assertion is on structure not content).

- **DTU unreachable during device fetch:** If the aggregate requires fetching all device
  records but the DTU is unreachable, the query errors. SETUP-FAILURE.

- **total_available absent from response:** If `total_available` is absent from the wire
  JSON (not present as a key, not even null), Part A assertion on JSON integer fails.
  This is a separate defect (missing field) — record as FAIL with the observation that
  the field is absent.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-QTT-001-002 (satisfaction: X.XX) — COUNT(*) query total_available not equal to aggregate row count; check engine Step 6 gated formula gate condition: any_early_stopped must be false for aggregating plans (ADR-060 §D8.11.5 Condition A via ast_is_reducing_plan() suppresses early-stop; BC-2.11.001 EC-11-095 MUST-9: total_available = total_rows for aggregate, not upstream sensor count)"`

Do NOT disclose: the specific total_available value, the upstream device count from the DTU,
or whether the failure was in the gate condition or the formula application.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-QUERY-TRUE-TOTAL-001 branch + Claroty DTU (claroty_devices fixture; total field > 25) |
| corpus_size | claroty_devices table; COUNT(*) aggregate result = 1 output row |
| known_edge_cases | Empty DTU fixture: COUNT returns 1 row with value 0; total_available=1 (acceptable). DTU with exactly 1 device: same structure. |
| false_positive_threshold | Zero: total_available == 1 vs > 100 is unambiguous for COUNT(*) |
| false_negative_threshold | Near-zero: a broken always-open gate gives total_available = N (hundreds/thousands); correct gate gives 1 |

**Known-good corpus:** prism binary with gate-closed path correctly implemented.
Expected: total_available = 1 = returned_results; is_truncated = false.

**Known-problematic corpus (broken gate):** a build where the gated formula applies
`upstream_total.map(|n| n.max(total_rows))` unconditionally (missing `if any_early_stopped`
check). Expected: total_available = upstream_device_count (> 100 for a real Claroty tenant),
not 1. Part A and Part C both fail.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w2-arch-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-037 group for S-QUERY-TRUE-TOTAL-001. Gate-closed path verification: COUNT(*) on claroty_devices must give total_available == returned_results == 1 (not inflated to upstream device count). Catches a broken gate where upstream_total is applied unconditionally. BC-2.11.001 EC-11-095 MUST-9 + ADR-060 §D8.11.5. Claroty DTU required. SINGLE-USE HIDDEN. |
