---
document_type: holdout-scenario
level: L3
id: "HS-QTT-001-001"
title: "claroty_devices LIMIT 25 wire response carries total_available as integer greater than 25"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-QUERY-TRUE-TOTAL-001 (HS-037 group). Validates BC-2.11.001 EC-11-095 MUST-4: when total_count_path is declared and any_early_stopped=true, total_available in the MCP wire response must reflect the upstream sensor total (not the returned row count). Discriminating: pre-patch total_available=25 (equals limit=25); post-fix total_available=N (true upstream total >> 25). Claroty DTU required. Wire-level JSON assertion on query_context.total_available. Test-writer and implementer must NOT read this file."
---

# HS-QTT-001-001: claroty_devices LIMIT 25 wire response carries total_available as integer greater than 25

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-QUERY-TRUE-TOTAL-001 (HS-037 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.11.001 §Postconditions Dual-limit-semantics Override, EC-11-095
MUST-4: "For a single sensor with `total_count_path` declared: when `any_early_stopped = true`
and the sensor reports upstream total N >= total_rows, the MCP wire response MUST carry
`total_available = N`. Assertion MUST be on serialized JSON wire bytes."
**Gate:** Story-level holdout gate (HS-037) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the **primary fix in the pagination pipeline**:
thread `PaginationCursor.total_count` → `FetchOutput.upstream_total` → `FanOutResult.upstream_total`
→ `MaterializationOutput.upstream_total` → engine Step 6 gated formula →
`query_context.total_available` in the MCP wire response.

This is exactly the beta.2 Monroe demo failure case: `query(query="FROM claroty_devices",
limit=25)` returned `total_available: 25` when the Claroty tenant has 1,200+ devices.
An analyst cannot determine dataset completeness from a `total_available` equal to the
limit — it is indistinguishable from a dataset of exactly 25 records.

**The defect this scenario catches (pre-patch behavior):**

1. `FetchOutput` has no `upstream_total` field.
2. Even if it did, engine Step 6 uses `total_available = total_rows` unconditionally.
3. Result: `total_available = 25` (equals `total_rows` which equals `limit` for
   an early-stopped query), NOT the true upstream count.

**Post-fix behavior (gated formula, ADR-060 §D8.11.5):**

```
total_available = if any_early_stopped {
    upstream_total.map(|n| n.max(total_rows)).unwrap_or(total_rows)
} else {
    total_rows
}
```

With `any_early_stopped = true` and `upstream_total = Some(N)` where N > 25:
`total_available = N`.

**Discriminating assertion:** `query_context.total_available > 25`.

- **Pre-patch:** `total_available = 25` → assertion fails.
- **Post-fix:** `total_available = N` (actual upstream count, > 25) → assertion passes.

**Four assertions on the serialized MCP response:**

**Part A — total_available is an integer greater than 25:**
- Assert `query_context.total_available` is a JSON integer (not null, not absent, not string).
- Assert `query_context.total_available > 25`.

**Part B — returned row count equals the limit:**
- Assert `query_context.returned_results == 25` (the engine returned exactly what was requested).
- Assert `rows` array has length 25.

**Part C — is_truncated reflects early-stop (not total_available comparison):**
- Assert `query_context.is_truncated == true`.
- This confirms `is_truncated` is driven by `any_early_stopped`, NOT by
  `total_available > returned_results`. Pre-patch: `total_available = 25 = returned_results`,
  so an incorrect implementation deriving is_truncated from that comparison would return
  `is_truncated = false`. Post-fix: `is_truncated = true` regardless (driven by `any_early_stopped`).

**Part D — hard invariant:**
- Assert `query_context.total_available >= query_context.returned_results`.
  (BC-2.11.001 EC-11-095 MUST-8: total_available >= total_rows in all cases.)

**BDD supplement:**

**Given** prism is built from the S-QUERY-TRUE-TOTAL-001 story branch
**And** the Claroty sensor TOML declares `total_count_path = "total"` on the `devices` table
**And** the Claroty DTU is running and serves a `devices` endpoint with a `"total"` field > 25
**When** `tools/call query {"query": "FROM claroty_devices", "client_id": "<test_client>", "limit": 25}` is issued via MCP stdio
**Then** the response is not a JSON-RPC error
**And** `query_context.total_available` is a JSON integer > 25
**And** `query_context.returned_results == 25`
**And** `rows` array length == 25
**And** `query_context.is_truncated == true`
**And** `query_context.total_available >= query_context.returned_results`

---

## Setup Instructions

1. Build prism from the S-QUERY-TRUE-TOTAL-001 story branch.

2. Start the Claroty DTU (`cargo run -p prism-dtu-claroty` or equivalent). The DTU must
   serve the `/api/v1/devices` endpoint with a response body of the form:
   `{"devices": [...N records...], "total": N, "page": 1}` where N > 25.
   The DTU serves its built-in fixture data by default.

3. Prepare a `prism.toml` that configures a test client with the Claroty sensor.
   The `claroty.sensor.toml` spec (as modified by this story) must declare
   `total_count_path = "total"` on the `devices` table.

4. Start prism in MCP stdio mode.
   SETUP-FAILURE condition: prism fails to start, DTU is unreachable, or MCP handshake fails.

5. Complete the MCP `initialize` handshake.

6. Issue the `query` tool call with limit=25:
   ```
   {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query","arguments":{"query":"FROM claroty_devices","client_id":"<test_client>","limit":25}}}
   ```
   Capture the full wire-level JSON response bytes.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.001 | EC-11-095 MUST-4: single-sensor with total_count_path: MCP total_available = max(upstream_total, total_rows) when early-stopped | Part A: total_available > 25 |
| BC-2.11.001 | EC-11-095 MUST-10: is_truncated formula UNCHANGED by upstream_total; driven by any_early_stopped | Part C: is_truncated = true |
| BC-2.11.001 | EC-11-095 MUST-8: hard invariant total_available >= total_rows in all cases | Part D: total_available >= returned_results |

---

## Verification Approach

1. Parse wire-level JSON-RPC response. Verify non-error (`result` present, `error` absent).
   If error: SETUP-FAILURE.

2. Parse `result.content[0].text` as JSON to obtain the query response envelope.

3. Extract `query_context` object. If absent: SETUP-FAILURE.

4. **Part A — total_available type and value:**
   - Assert `query_context.total_available` is present (key exists in JSON).
   - Assert the JSON type is `number` (integer), not `null`, not string `"null"`, not absent.
   - Assert the integer value > 25.
   - If `total_available == 25`: record FAIL — pre-patch behavior (total_available == limit).

5. **Part B — returned row count:**
   - Extract `rows` array (or equivalent field per wire schema). Assert length == 25.
   - Assert `query_context.returned_results == 25`.

6. **Part C — is_truncated:**
   - Assert `query_context.is_truncated == true` (JSON boolean `true`, not string, not null).

7. **Part D — hard invariant:**
   - Assert `query_context.total_available >= query_context.returned_results`.
   - Since Part B asserts returned_results == 25 and Part A asserts total_available > 25,
     this follows. The explicit check guards against edge cases.

8. All assertions are on the serialized JSON wire bytes from `result.content[0].text`.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response is non-error (prerequisite)** (weight: 0.10):
  Full credit (1.0): non-error result, query_context present.
  Zero credit (0.0): error response or missing query_context (SETUP-FAILURE).

- **total_available > 25 as integer (Part A — primary discriminator)** (weight: 0.55):
  Full credit (1.0): `total_available` is integer > 25.
  Zero credit (0.0): `total_available == 25` (pre-patch behavior), or null, or absent.

- **Returned rows == 25 (Part B)** (weight: 0.15):
  Full credit (1.0): rows.length == 25 and returned_results == 25.
  Zero credit (0.0): row count != 25 (unexpected limit behavior — SETUP check needed).

- **is_truncated == true (Part C)** (weight: 0.10):
  Full credit (1.0): is_truncated is JSON boolean true.
  Zero credit (0.0): is_truncated == false (early-stop not detected or incorrectly computed).

- **Hard invariant (Part D)** (weight: 0.10):
  Full credit (1.0): total_available >= returned_results.
  Zero credit (0.0): total_available < returned_results (invariant violated — severe defect).

---

## Edge Conditions

- **DTU fixture has exactly 25 device records:** If the Claroty DTU devices fixture happens
  to contain exactly 25 records, the API response page is partial (partial page → no
  early-stop). With `any_early_stopped = false`, the gated formula gives
  `total_available = total_rows = 25` (gate closed, correct behavior). But then Part A
  fails because `total_available == 25` is not `> 25`. This is SETUP-FAILURE, not
  a behavioral failure — verify the DTU fixture contains more than 25 device records.
  The standard Claroty DTU fixture should serve enough records to trigger early-stop at
  limit=25 (the beta.2 demo scenario involves ~1,200 devices).

- **total_count_path key missing from DTU response:** If the Claroty DTU's devices response
  body does not contain the `"total"` key, `upstream_total = None` and `total_available =
  total_rows = 25`. Part A fails. This indicates either the TOML `total_count_path` key
  is missing, or the DTU is not returning the `"total"` field.

- **total_available is null or absent:** The wire-shape discipline requires `total_available`
  to be a JSON integer (not null, not absent). If absent: score Part A as zero credit.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-QTT-001-001 (satisfaction: X.XX) — query total_available not reflecting upstream sensor count; check total_count_path TOML field, FetchOutput.upstream_total plumbing, and engine Step 6 gated formula (BC-2.11.001 EC-11-095 MUST-4 + ADR-060 §D8.11.5: when any_early_stopped=true and upstream_total=Some(N), total_available must equal N not total_rows)"`

Do NOT disclose: the specific total_available value observed, the expected upstream total
count from the DTU, or whether the failure was in the TOML configuration, adapter plumbing,
or engine Step 6.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-QUERY-TRUE-TOTAL-001 branch + Claroty DTU (claroty_devices fixture with total field > 25) |
| corpus_size | claroty_devices table; the DTU's built-in device fixture; upstream total field value from DTU response |
| known_edge_cases | DTU fixture with exactly 25 devices: SETUP-FAILURE (no early-stop, total_available=25 ambiguous). Sentinel usize::MAX from PaginationCursor before first API response: must be filtered (EC-006 in story edge cases). |
| false_positive_threshold | Zero: total_available == 25 is the exact pre-patch defect; > 25 is the exact post-fix behavior |
| false_negative_threshold | Near-zero: pre-patch always gives total_available = limit = 25; post-fix gives the actual upstream total |

**Known-good corpus:** prism binary with full upstream_total propagation chain implemented
and `total_count_path = "total"` declared in claroty.sensor.toml for the devices table.
Expected: total_available = N (actual device count from DTU) > 25.

**Known-problematic corpus:** pre-patch prism binary (develop HEAD before this story).
Expected: total_available = 25 (equals limit) — the beta.2 Monroe demo failure.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w2-arch-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-037 group for S-QUERY-TRUE-TOTAL-001. Primary discriminating test: claroty_devices LIMIT=25 with total_count_path declared and early-stop firing must produce total_available > 25 in the MCP wire response. Pre-patch total_available=25 (equals limit); post-fix total_available=N (true upstream count). BC-2.11.001 EC-11-095 MUST-4 + ADR-060 §D8.11.5. Claroty DTU required. SINGLE-USE HIDDEN. |
