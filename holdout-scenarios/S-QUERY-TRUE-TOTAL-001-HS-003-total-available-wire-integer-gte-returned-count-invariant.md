---
document_type: holdout-scenario
level: L3
id: "HS-QTT-001-003"
title: "total_available is always a wire-level JSON integer and always greater than or equal to returned row count"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-QUERY-TRUE-TOTAL-001 (HS-037 group). Validates BC-2.11.001 EC-11-095 MUST-8 (hard invariant: total_available >= total_rows in all cases) AND the wire-shape discipline (total_available must be a JSON integer, not null, not absent, not string). Tests two query shapes: (1) early-stopped (FROM claroty_devices LIMIT=25) and (2) no-total-count-path backward-compat (FROM claroty_alerts LIMIT=25 with total_count_path absent or table uses small fixture). Both must satisfy the integer type assertion and the >= invariant. Claroty DTU required. Wire-level JSON assertions. Test-writer and implementer must NOT read this file."
---

# HS-QTT-001-003: total_available is always a wire-level JSON integer and always greater than or equal to returned row count

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-QUERY-TRUE-TOTAL-001 (HS-037 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.11.001 §Postconditions EC-11-095 MUST-8 (hard invariant) and
wire-shape discipline per CLAUDE.md §Conventions:
- MUST-8: "`total_available >= total_rows` in all cases, including when the sensor
  under-reports (`upstream_total < total_rows`). The `.max(total_rows)` in the gated
  formula enforces this floor."
- Wire-shape: "NULL vs absent vs empty distinctions MUST be asserted at the wire level."
  `total_available` must be a JSON integer, not JSON `null`, not the string `"null"`,
  not absent from the response object.
**Gate:** Story-level holdout gate (HS-037) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates **two cross-cutting invariants** that apply to EVERY query response
regardless of whether `total_count_path` is declared or `any_early_stopped` is true:

**Invariant 1 — Wire-shape integrity:**
`total_available` MUST be present in the `query_context` JSON object as an integer-typed
value. It must NOT be:
- Absent (key missing from JSON)
- JSON `null`
- The string literal `"null"` (the Issue 7a string-null encoding defect from S-MCP-NULL-ENCODING-001 — analogous regression must not occur here)
- A floating-point number (should be an integer count)

This catches plumbing bugs where adding the `upstream_total` field inadvertently changed
the serialization of `total_available`, introduced a null path, or left the field absent
in certain execution branches.

**Invariant 2 — Hard total_available >= returned_results floor:**
For any query, `total_available >= returned_results` (the returned row count) must always hold.
The gated formula's `.max(total_rows)` clause enforces this even when a sensor under-reports
`upstream_total < total_rows`. This catches a missing `.max()` in the formula.

**Two query pairs exercising both invariants across execution paths:**

**Pair A — Early-stopped query (gate-open path):**
`FROM claroty_devices LIMIT 25` (same as HS-QTT-001-001).
- Asserts: `total_available` is JSON integer; `total_available >= 25`.
- The gate-open path exercises the `upstream_total.map(|n| n.max(total_rows)).unwrap_or(total_rows)` branch.

**Pair B — No-total-count-path query (backward-compat path):**
`FROM claroty_alerts LIMIT 25`.
- If `total_count_path` is not declared for `claroty_alerts` (or if the DTU serves fewer
  than 25 alert records so no early-stop fires), `any_early_stopped = false` and
  `upstream_total = None` → `total_available = total_rows`.
- Asserts: `total_available` is JSON integer; `total_available >= returned_results`.
- The backward-compat path exercises the `else { total_rows }` branch (gate closed) AND
  the `upstream_total = None → unwrap_or(total_rows)` fallback.

**Note on Pair B:** The story adds `total_count_path = "total"` to claroty_alerts AS WELL
AS claroty_devices (both are paginated tables). If the Claroty DTU alerts fixture has fewer
than 25 records (the pre-existing DTU has 10 alerts), no early-stop fires for Pair B, so
`any_early_stopped = false` and the gate is closed. `total_available = total_rows = 10`.
The invariant `10 >= 10` holds.

**Three assertions per query pair (six total):**

For each query (Pair A and Pair B):
1. `total_available` is present as a JSON key in `query_context`.
2. The JSON type of `total_available` is `number` (integer), not `null`, not absent, not string.
3. `total_available >= returned_results` (the hard floor invariant).

**BDD supplement:**

**Given** prism is built from the S-QUERY-TRUE-TOTAL-001 story branch
**And** the Claroty DTU is running
**When** `tools/call query {"query": "FROM claroty_devices", "client_id": "<test>", "limit": 25}` is issued
**Then** `query_context.total_available` is present as a JSON integer in the response
**And** `query_context.total_available >= query_context.returned_results`
**When** `tools/call query {"query": "FROM claroty_alerts", "client_id": "<test>", "limit": 25}` is issued
**Then** `query_context.total_available` is present as a JSON integer in the response
**And** `query_context.total_available >= query_context.returned_results`

---

## Setup Instructions

1. Build prism from the S-QUERY-TRUE-TOTAL-001 story branch.

2. Start the Claroty DTU (same instance as HS-QTT-001-001 and HS-QTT-001-002).

3. Prepare the same `prism.toml` Claroty test client configuration.

4. Start prism in MCP stdio mode. Complete the MCP `initialize` handshake.

5. Issue TWO sequential `query` tool calls:

   **Pair A:**
   ```
   {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"query","arguments":{"query":"FROM claroty_devices","client_id":"<test_client>","limit":25}}}
   ```

   **Pair B:**
   ```
   {"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"query","arguments":{"query":"FROM claroty_alerts","client_id":"<test_client>","limit":25}}}
   ```

   Capture both wire-level JSON responses.
   SETUP-FAILURE condition: either query returns a JSON-RPC error, or `query_context` is
   absent from either response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.001 | EC-11-095 MUST-8: hard invariant total_available >= total_rows in all cases including sensor under-report | Part 3 (both pairs): total_available >= returned_results |
| BC-2.11.001 | EC-11-095 MUST-4 (devices) / MUST-6 (alerts no-path path): total_available is an integer in both paths | Parts 1/2 (both pairs): JSON integer type check |
| BC-2.11.001 | EC-11-079 (wire-shape discipline): NULL vs absent vs empty distinctions at wire level | Part 2 (both pairs): not null, not absent, not string |

---

## Verification Approach

1. Parse both wire-level JSON-RPC responses. Verify each is non-error. Extract `query_context`
   from each. If either is missing: SETUP-FAILURE.

For EACH of the two `query_context` objects (Pair A and Pair B):

2. **Part 1 — Field presence:**
   Assert `total_available` key is present in the `query_context` JSON object.
   (Not absent — the key must exist.) If absent: record FAIL for this pair.

3. **Part 2 — Wire-type correctness:**
   Assert the JSON value at `total_available` is of JSON type `number`.
   - NOT `null` (JSON null literal).
   - NOT a string (e.g., `"25"` or `"null"`) — must be a bare integer.
   - NOT a floating-point value (e.g., `25.0`) — must be an integer count.
   If the value is not a JSON integer: record FAIL with the actual type and value.

4. **Part D — Hard floor invariant:**
   Assert `query_context.total_available >= query_context.returned_results`.
   Extract `returned_results` as an integer.
   Assert `total_available >= returned_results`.
   If violated: record FAIL with both values.

5. Aggregate: both pairs must satisfy all three parts. Any FAIL in either pair fails
   the scenario. Partial credit if only one pair fails (Section: Evaluation Rubric below).

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Both responses non-error with query_context (prerequisite)** (weight: 0.10):
  Full credit (1.0): both Pair A and Pair B return non-error responses with query_context.
  Partial credit (0.5): one response is non-error, the other is SETUP-FAILURE.
  Zero credit (0.0): both fail or missing query_context in both (SETUP-FAILURE).

- **total_available is present (field presence — both pairs)** (weight: 0.20):
  Full credit (1.0): key present in both query_context objects.
  Partial credit (0.5): key present in one, absent in the other.
  Zero credit (0.0): key absent in both (plumbing regression — field removed or never added).

- **total_available is JSON integer (wire-type correctness — both pairs)** (weight: 0.30):
  Full credit (1.0): both values are JSON integers (number type, not null, not string).
  Partial credit (0.5): one is correct, the other is null or absent.
  Zero credit (0.0): both are null, absent, or string-encoded (wire-shape regression).

- **Hard floor invariant (total_available >= returned_results — both pairs)** (weight: 0.40):
  Full credit (1.0): invariant holds for both pairs.
  Partial credit (0.5): invariant holds for one pair, violated for the other (edge case).
  Zero credit (0.0): invariant violated for both pairs.

---

## Edge Conditions

- **Claroty DTU alerts fixture has exactly 0 records:** Pair B query returns 0 rows.
  `returned_results = 0`. `total_available = 0`. Invariant `0 >= 0` holds. Parts 1/2/3
  all pass (total_available is 0, an integer, and >= 0).

- **claroty_alerts also has total_count_path declared and early-stop fires:** If the DTU
  serves more than 25 alerts AND `total_count_path` is declared on claroty_alerts (which
  the story adds), Pair B becomes an early-stopped query with `total_available = N > 25`.
  The invariant `N >= 25` still holds. All parts pass.

- **Sensor under-report edge case for Pair A:** If the Claroty DTU's `"total"` field returns
  a value LESS than the actual returned row count (e.g., `"total": 10` but 25 rows fetched),
  the `.max(total_rows)` clause in the gated formula ensures `total_available = 25` (not 10).
  Part 4 invariant `25 >= 25` holds. This DTU behavior is unlikely in practice but the
  invariant check is designed to catch it.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-QTT-001-003 (satisfaction: X.XX) — total_available wire-shape or floor invariant violation; check: (1) total_available is serialized as a JSON integer not null/absent/string in the query response envelope, (2) engine Step 6 gated formula includes .max(total_rows) floor clause (BC-2.11.001 EC-11-095 MUST-8: total_available >= returned_results in all query paths including backward-compat no-total-count-path case)"`

Do NOT disclose: which pair failed (devices or alerts), the specific observed value of
total_available or returned_results, or the DTU's total count.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-QUERY-TRUE-TOTAL-001 branch + Claroty DTU (claroty_devices + claroty_alerts fixtures) |
| corpus_size | Two queries: (A) claroty_devices LIMIT=25 (early-stopped, gate open); (B) claroty_alerts LIMIT=25 (backward-compat or gate closed depending on fixture size) |
| known_edge_cases | Alerts fixture 0 records: total_available=0 >= returned_results=0 — passes invariant. Alerts fixture > 25 with total_count_path: Pair B becomes early-stopped, still passes invariant. |
| false_positive_threshold | Near-zero: JSON integer type vs null/absent/string is unambiguous. Floor invariant violation (total_available < returned_results) is a strict arithmetic check. |
| false_negative_threshold | Near-zero: the plumbing regression of setting total_available=null or omitting the field entirely is caught by Parts 1/2 regardless of arithmetic correctness. |

**Known-good corpus:** prism binary with full upstream_total plumbing and correct serialization.
Expected: both pairs give total_available as JSON integer >= returned_results.

**Known-problematic corpus (regression patterns):**
- Missing `upstream_total` field added to `FetchOutput`: serialization path returns null or
  absent for `total_available` — Parts 1/2 fail.
- Missing `.max(total_rows)` in gated formula: sensor under-report not floored to total_rows —
  Part 4 fails (total_available < returned_results when sensor under-reports).
- `total_available` serialized as string due to serde annotation change: Part 2 fails.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w2-arch-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-037 group for S-QUERY-TRUE-TOTAL-001. Hard invariant + wire-shape tests: total_available is a JSON integer (not null/absent/string) AND total_available >= returned_results for both an early-stopped query (claroty_devices LIMIT=25) and a backward-compat query (claroty_alerts LIMIT=25). BC-2.11.001 EC-11-095 MUST-8 (floor invariant) + wire-shape discipline. Claroty DTU required. SINGLE-USE HIDDEN. |
