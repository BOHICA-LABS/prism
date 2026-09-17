---
document_type: holdout-scenario
level: L3
id: "HS-DESC-DEDUP-001-002"
title: "After class_uid exclusion at least one Claroty table falls through to Tier 3 count-recent example"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "E-BETA3-REMEDIATION"
story_source: "S-DESCRIBE-EXAMPLE-DEDUP-001"
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
  - ".factory/specs/behavioral-contracts/BC-2.10.012-prism-describe-schema-discovery-tool.md"
input-hash: "TBD"
traces_to: "BC-2.10.012"
behavioral_contracts:
  - BC-2.10.012
verification_properties: []
lifecycle_status: active
introduced: "S-DESCRIBE-EXAMPLE-DEDUP-001"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-DESCRIBE-EXAMPLE-DEDUP-001 (HS-036 group). Validates Tier 3 fallthrough (BC-2.10.012 §Auto-generated example queries Tier 3 count-recent): when all Integer/Float columns for a Claroty table are excluded (only class_uid exists as numeric), and a Datetime column is present, the function MUST produce a count-recent example containing NOW(). Discriminating: pre-patch these tables all produce GROUP BY class_uid (Tier 2), so zero tables contain NOW() in their example_query; post-fix one or more Claroty tables fall through to Tier 3 and contain NOW(). Wire-level count assertion on NOW() presence. DTU NOT required. Test-writer and implementer must NOT read this file."
---

# HS-DESC-DEDUP-001-002: After class_uid exclusion at least one Claroty table falls through to Tier 3 count-recent example

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-DESCRIBE-EXAMPLE-DEDUP-001 (HS-036 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.10.012 §Auto-generated example queries Tier 3 count-recent postcondition:
"Used when Tiers 1 and 2 did NOT fire AND a Datetime-typed column exists; query uses
`SELECT COUNT(*) FROM <table> WHERE <datetime_col> > NOW() - INTERVAL '1h'` form."
EC-10-033 causes Tier 2 to NOT fire for tables whose only numeric column is `class_uid`,
which makes Tier 3 the active path for those tables (if they have a Datetime column).
**Gate:** Story-level holdout gate (HS-036) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the **Tier 3 fallthrough behavior triggered by the EC-10-033 fix**
(BC-2.10.012 §Auto-generated example queries Tier 3; S-DESCRIBE-EXAMPLE-DEDUP-001 AC-003).

Claroty tables like `claroty_alerts`, `claroty_devices`, and `claroty_audit_logs` have:
- `class_uid` as their only Integer column (synthesized, now excluded by EC-10-033)
- Datetime columns (e.g., `detected_time`, `last_seen`, `timestamp`)

After the exclusion fix:
1. Tier 1 check: no `severity` column → skip.
2. Tier 2 check: no non-excluded Integer/Float column → `agg_col = None` → skip.
3. Tier 3 check: Datetime column exists → **fire**: produce count-recent example with `NOW()`.

**The defect this scenario catches:** A partial or incorrect implementation of the fix that
excludes `class_uid` from `agg_col` but fails to let the function fall through to Tier 3.
Symptoms: an empty `example_query`, a malformed query, or an unexpected Tier 4 (`SELECT *
LIMIT 25`) for a table that has a Datetime column and should produce Tier 3.

**Discriminating assertion:** Count the number of Claroty table entries in the prism_describe
response whose `example_query` contains the substring `NOW()`.

- **Pre-patch:** All Claroty tables produce `GROUP BY class_uid` (Tier 2). Zero tables
  contain `NOW()`. The assertion `count >= 1` FAILS.
- **Post-fix:** Tables with Datetime columns and no domain Integer fall through to Tier 3.
  At least one table's example_query contains `NOW()`. The assertion `count >= 1` PASSES.

**Two assertions in this scenario:**

**Part A — At least one Tier 3 count-recent example exists:**
- Parse `result.content[0].text` as JSON.
- Extract `tables` array.
- Count elements where `example_query` contains the substring `NOW()`.
- Assert: count >= 1.
  (If count == 0: either pre-patch behavior survived, or the fallthrough is broken.)

**Part B — Each Tier 3 example has the correct structural form:**
- For each table where `example_query` contains `NOW()`:
  - Assert the example_query contains `COUNT(*)` (the count-recent template uses COUNT).
  - Assert the example_query contains `WHERE` (the temporal filter is a WHERE clause).
  - Assert the example_query contains `INTERVAL '1h'` (the canonical 1-hour lookback window
    per BC-2.10.012 §Auto-generated example queries Tier 3 template).
  - Assert the example_query does NOT contain `GROUP BY` (Tier 3 is a COUNT aggregate,
    not a GROUP BY aggregate — these are distinct query shapes).

**BDD supplement (Part A):**

**Given** prism is built from the S-DESCRIBE-EXAMPLE-DEDUP-001 story branch
**And** a test client is configured with the Claroty sensor
**When** `tools/call prism_describe {"client_id": "<test_client>"}` is issued via MCP stdio
**Then** the response is not a JSON-RPC error
**And** `result.content[0].text` parses as valid JSON with a non-empty `tables` array
**And** at least one entry in `tables` has an `example_query` containing the substring `"NOW()"`
**And** each such Tier 3 example also contains `"COUNT(*)"`, `"WHERE"`, and `"INTERVAL '1h'"`
**And** no Tier 3 example contains `"GROUP BY"`

---

## Setup Instructions

1. Build prism from the S-DESCRIBE-EXAMPLE-DEDUP-001 story branch. No special feature flags
   required.

2. Prepare a `prism.toml` that configures a test client with the Claroty sensor (same config
   as HS-DESC-DEDUP-001-001 — `client_id = "holdout-claroty-dedup"`). The standard
   `claroty.sensor.toml` spec must include at least one table that has a Datetime column
   and no domain Integer/Float column other than `class_uid`. The core Claroty tables
   (claroty_alerts, claroty_devices, claroty_audit_logs) satisfy this condition.
   **Note:** DTU NOT required — prism_describe reads the TOML spec at boot.

3. Start prism in MCP stdio mode. Complete the MCP `initialize` handshake as in
   HS-DESC-DEDUP-001-001.

4. Issue the `prism_describe` tool call:
   ```
   {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"prism_describe","arguments":{"client_id":"holdout-claroty-dedup"}}}
   ```
   Capture the full wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.10.012 | §Auto-generated example queries Tier 3 count-recent: fires when Tiers 1/2 did NOT fire AND a Datetime column exists | Part A: at least one NOW() example in tables |
| BC-2.10.012 | §Auto-generated example queries Tier 3 template: `SELECT COUNT(*) FROM <t> WHERE <dt> > NOW() - INTERVAL '1h'` | Part B: COUNT(*) + WHERE + INTERVAL '1h' + no GROUP BY |
| BC-2.10.012 | EC-10-033: class_uid excluded from agg_col → Tier 2 skipped → Tier 3 fires for Datetime tables | Part A/B combined: fallthrough path is active |

---

## Verification Approach

1. Parse the wire-level JSON-RPC response. Verify non-error (same as HS-DESC-DEDUP-001-001
   Step 1).

2. Extract `result.content[0].text` as JSON. Extract `tables` array. If empty: SETUP-FAILURE.

3. **Part A — Count Tier 3 examples:**
   Initialize counter `tier3_count = 0`.
   For each table entry `t` in `tables`:
     If `t.example_query` contains substring `NOW()`: increment `tier3_count`.
   Assert `tier3_count >= 1`.
   If `tier3_count == 0`: record FAIL — the Tier 3 fallthrough is not active.

4. **Part B — Validate Tier 3 structure:**
   For each table entry where example_query contains `NOW()`:
     Assert: example_query contains `COUNT(*)`.
     Assert: example_query contains `WHERE`.
     Assert: example_query contains `INTERVAL '1h'`.
     Assert: example_query does NOT contain `GROUP BY`.
   Record any violations with the table name and actual example_query text.

5. All assertions are on the serialized JSON from `result.content[0].text`.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response is non-error (prerequisite)** (weight: 0.10):
  Full credit (1.0): non-error result, non-empty tables array.
  Zero credit (0.0): error or empty tables (SETUP-FAILURE).

- **At least one Tier 3 count-recent example (Part A)** (weight: 0.60):
  Full credit (1.0): `tier3_count >= 1`.
  Zero credit (0.0): `tier3_count == 0` — pre-patch behavior or broken fallthrough.
  (No partial credit: either Tier 3 fires or it does not.)

- **Structural correctness of Tier 3 examples (Part B)** (weight: 0.30):
  Full credit (1.0): every NOW()-containing example also has COUNT(*) + WHERE + INTERVAL '1h' + no GROUP BY.
  Partial credit (0.5): at least one Tier 3 example is correctly formed but at least one is malformed.
  Zero credit (0.0): all NOW()-containing examples are missing one or more required components.

---

## Edge Conditions

- **All Claroty tables have a `severity` column (Tier 1 fires everywhere):** If every
  Claroty table happens to have a `severity` String column, Tier 1 fires for all of them
  before reaching Tier 3. `tier3_count == 0` in this case. This is SETUP-FAILURE, not a
  behavioral FAIL — verify the test client uses standard `claroty.sensor.toml` which has
  tables without severity columns (e.g., `claroty_devices`, `claroty_audit_logs`).

- **No Claroty table has a Datetime column:** Tier 3 requires a Datetime column; without
  one, the function falls to Tier 4 (`SELECT * LIMIT 25`). If no table has a Datetime
  column, `tier3_count == 0` — SETUP-FAILURE. Standard `claroty.sensor.toml` includes
  tables with Datetime columns; verify spec loading.

- **Tier 4 examples present alongside Tier 3:** Some tables may fall to Tier 4 (no Datetime,
  no domain Integer/Float — only String columns). Those tables are not checked by Part A
  (they lack `NOW()`). Their presence is acceptable as long as Part A's count >= 1.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-DESC-DEDUP-001-002 (satisfaction: X.XX) — prism_describe Tier 3 fallthrough missing; check build_example_with_note: after EXAMPLE_EXCLUDED_COLS exclusion, agg_col=None tables with a Datetime column must fall through to the Tier 3 count-recent branch (BC-2.10.012 §Auto-generated example queries Tier 3: SELECT COUNT(*) FROM <t> WHERE <dt_col> > NOW() - INTERVAL '1h')"`

Do NOT disclose: which specific table produced (or failed to produce) the Tier 3 example,
or the actual count of Tier 3 examples found.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-DESCRIBE-EXAMPLE-DEDUP-001 branch + standard claroty.sensor.toml |
| corpus_size | All Claroty tables in claroty.sensor.toml; at minimum claroty_alerts, claroty_devices, claroty_audit_logs (known to have Datetime columns, no domain Integer) |
| known_edge_cases | Tables with severity column: Tier 1 fires, no NOW() — not counted in tier3_count (acceptable). Tables with no Datetime: Tier 4 SELECT * LIMIT 25 — not counted (acceptable). |
| false_positive_threshold | Zero: NOW() is an unambiguous substring absent from Tier 1/2/4 examples |
| false_negative_threshold | Zero: pre-patch never produces NOW() in any example; any post-fix Claroty table with Datetime will contain NOW() |

**Known-good corpus:** prism binary with EXAMPLE_EXCLUDED_COLS fix applied.
Expected: tier3_count >= 1; structural assertions pass.

**Known-problematic corpus:** pre-patch prism binary.
Expected: tier3_count == 0 — all Claroty tables produce GROUP BY class_uid (Tier 2);
Part A assertion fails.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w2-arch-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-036 group for S-DESCRIBE-EXAMPLE-DEDUP-001. Tier 3 fallthrough verification: after class_uid exclusion, Claroty tables with Datetime columns must produce count-recent examples containing NOW(). Discriminating: pre-patch zero tables contain NOW() (all have GROUP BY class_uid); post-fix count >= 1. BC-2.10.012 §Auto-generated example queries Tier 3 + EC-10-033. DTU NOT required. SINGLE-USE HIDDEN. |
