---
document_type: holdout-scenario
level: L3
id: "HS-DESC-DEDUP-001-001"
title: "prism_describe returns no example_query containing GROUP BY class_uid for any Claroty table"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-DESCRIBE-EXAMPLE-DEDUP-001 (HS-036 group). Validates EC-10-033: build_example_with_note MUST NOT select class_uid as the Tier-2 aggregate GROUP BY column. Discriminating: pre-patch all Claroty tables produce 'SELECT class_uid, COUNT(*) FROM <table> GROUP BY class_uid ORDER BY COUNT(*) DESC LIMIT 10' because class_uid is the first (often only) Integer column; post-fix none do because EXAMPLE_EXCLUDED_COLS filters it. Wire-level assertion on serialized prism_describe tables[*].example_query strings. DTU NOT required — prism_describe reads TOML spec at boot. Test-writer and implementer must NOT read this file."
---

# HS-DESC-DEDUP-001-001: prism_describe returns no example_query containing GROUP BY class_uid for any Claroty table

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-DESCRIBE-EXAMPLE-DEDUP-001 (HS-036 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.10.012 §Auto-generated example queries Tier 2 aggregate template
EC-10-033: synthesized metadata columns (`class_uid`, `_sensor`, `_client`, `_source_table`,
`_source_type`) MUST NOT be used as `agg_col` in the GROUP BY target selection.
**Gate:** Story-level holdout gate (HS-036) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the **primary fix in `build_example_with_note`**
(BC-2.10.012 §Auto-generated example queries Tier 2 EC-10-033;
S-DESCRIBE-EXAMPLE-DEDUP-001 AC-001 / AC-002).

When `prism_describe` executes against a Claroty client:

1. The tool returns a JSON object with a `tables` array, each entry containing `example_query`.
2. For every Claroty table, the `example_query` MUST NOT use `class_uid` as the GROUP BY
   aggregate column.
3. `class_uid` is a synthesized OCSF metadata column appended by `build_ocsf_column_descriptors`
   — not a domain-data column. It holds a constant value for all rows in a given table
   (e.g., 2004 for claroty_alerts, 5001 for claroty_devices). Grouping by it produces a
   single bucket with a count equal to total row count — a semantically meaningless example.

**The defect this scenario catches:** Pre-patch `build_example_with_note` selects `agg_col`
as the first `Integer | Float` column encountered:
```
let agg_col = columns.iter().find(|c| matches!(c.col_type, ColumnType::Integer | ColumnType::Float));
```
For Claroty tables that have no earlier domain Integer/Float column, `class_uid` is the
first (and often only) Integer — so every such table produces the identical query:
`SELECT class_uid, COUNT(*) FROM <table> GROUP BY class_uid ORDER BY COUNT(*) DESC LIMIT 10`

**Discriminating assertion:** Parse `tables[*].example_query` from the wire response.
- Pre-patch: every Claroty table example_query contains the substring `GROUP BY class_uid`.
  Assertion fails.
- Post-fix: no Claroty table example_query contains `GROUP BY class_uid`.
  Assertion passes.

**Two assertions in this scenario:**

**Part A — No GROUP BY class_uid in any table's example_query:**
- Parse `result.content[0].text` as JSON to get the prism_describe response object.
- Extract `tables` as an array.
- For each element `t` in `tables`:
  - Assert `t.example_query` does NOT contain the substring `GROUP BY class_uid`.
  - Assert `t.example_query` does NOT contain the substring `class_uid, COUNT(*)`.
- Both sub-assertions must hold for EVERY table.

**Part B — example_query is a valid non-empty SELECT string:**
- For each `t` in `tables`:
  - Assert `t.example_query` is a non-empty string.
  - Assert `t.example_query` starts with the prefix `SELECT`.
- This is a sanity guard: the fix must not produce empty or malformed queries.

**BDD supplement (Part A):**

**Given** prism is built from the S-DESCRIBE-EXAMPLE-DEDUP-001 story branch
**And** a test client is configured with the Claroty sensor (at least claroty_alerts and claroty_devices tables registered)
**When** `tools/call prism_describe {"client_id": "<test_client>"}` is issued via MCP stdio
**Then** the response is not a JSON-RPC error
**And** `result.content[0].text` parses as valid JSON
**And** for every entry in `tables`, the `example_query` string does not contain `"GROUP BY class_uid"`
**And** for every entry in `tables`, the `example_query` string does not contain `"class_uid, COUNT(*)"`
**And** every `example_query` is a non-empty string starting with `"SELECT"`

---

## Setup Instructions

1. Build prism from the S-DESCRIBE-EXAMPLE-DEDUP-001 story branch. No special feature flags
   required for this scenario.

2. Prepare a `prism.toml` that configures a test client with the Claroty sensor.
   The standard `claroty.sensor.toml` spec shipped with the codebase registers multiple tables
   (claroty_alerts, claroty_devices, claroty_audit_logs, and others from Wave A/B/C expansions).
   Configure a client (`client_id = "holdout-claroty-dedup"`) with the Claroty sensor.
   **Note:** `prism_describe` reads from the loaded sensor TOML spec at boot — it does NOT
   contact the live Claroty API or DTU. The Claroty DTU does NOT need to be running.

3. Start prism in MCP stdio mode with the prepared config.
   SETUP-FAILURE condition: prism fails to start, or MCP handshake does not complete within
   10 seconds.

4. Complete the MCP `initialize` handshake:
   ```
   {"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"holdout-evaluator","version":"1.0"}}}
   ```
   Await a non-error response before proceeding.

5. Issue the `prism_describe` tool call:
   ```
   {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"prism_describe","arguments":{"client_id":"holdout-claroty-dedup"}}}
   ```
   Capture the full raw wire-level JSON response bytes.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.10.012 | §Auto-generated example queries Tier 2 aggregate template EC-10-033: `class_uid` MUST NOT be used as `agg_col` | Part A: no `GROUP BY class_uid` in any example_query |
| BC-2.10.012 | §Auto-generated example queries — example_query is always a non-empty SELECT string | Part B: structural validity of all example_query values |

---

## Verification Approach

1. Parse the wire-level JSON-RPC response. Verify the response is NOT a JSON-RPC error:
   `error` key absent and `result` key present. If error is present: SETUP-FAILURE.

2. Extract `result.content[0].text` as a raw string. Parse it as JSON to obtain the
   prism_describe response object.

3. Extract `tables` as an array. If `tables` is absent, empty, or not an array: SETUP-FAILURE.

4. **Part A — GROUP BY class_uid prohibition (primary assertion):**
   For each table entry `t` in `tables`:
   - Check: does `t.example_query` contain the substring `GROUP BY class_uid`?
   - Check: does `t.example_query` contain the substring `class_uid, COUNT(*)`?
   If EITHER check is true for ANY table: record FAIL for that table with the violating
   `example_query` value.
   Record the total FAIL count. Any FAIL count > 0 causes this scenario to FAIL.

5. **Part B — Structural validity:**
   For each table entry `t` in `tables`:
   - Assert `t.example_query` is a non-empty string.
   - Assert `t.example_query` begins with `SELECT` (case-sensitive; the template always
     produces uppercase SELECT per BC-2.10.012 §Auto-generated example queries).
   If any table has an empty, null, or non-SELECT example_query: record FAIL with details.

6. All assertions are on the PARSED JSON from `result.content[0].text` — wire-level check.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response is non-error (prerequisite)** (weight: 0.10):
  Full credit (1.0): `result` key present, `error` key absent, `result.content[0].text` is valid JSON, `tables` array is non-empty.
  Zero credit (0.0): error response, missing content, empty tables array (SETUP-FAILURE).

- **No GROUP BY class_uid in any example_query (Part A)** (weight: 0.70):
  Full credit (1.0): zero tables contain `GROUP BY class_uid` or `class_uid, COUNT(*)`.
  Partial credit (0.3): at most 1 table contains the pattern (partial implementation).
  Zero credit (0.0): 2 or more tables contain the pattern (fix not applied or incomplete).

- **Structural validity — all examples are non-empty SELECTs (Part B)** (weight: 0.20):
  Full credit (1.0): every `example_query` is a non-empty string beginning with `SELECT`.
  Partial credit (0.5): 1 table has an invalid example but all others are valid.
  Zero credit (0.0): 2 or more tables have empty, null, or non-SELECT examples.

---

## Edge Conditions

- **Client not found / no tables registered:** If the client_id is not recognized or no
  sensor is configured, prism_describe may return an error or an empty tables array.
  If `tables.length == 0`: SETUP-FAILURE (verify prism.toml sensor configuration).

- **Claroty tables with a `severity` column (Tier 1 fires instead of Tier 2):** If a Claroty
  table has a `severity` column, the Tier 1 severity-IEQ example fires instead of Tier 2.
  Such a table would produce `WHERE severity IN (...)` form — which does NOT contain
  `GROUP BY class_uid`. Part A vacuously passes for that table. That is correct behavior.

- **Non-Claroty tables in the response:** If the test client also configures CrowdStrike,
  Cyberint, or Armis sensors, those tables appear in the response. Part A applies to ALL
  tables — no CrowdStrike, Cyberint, or Armis table should use `GROUP BY class_uid` either
  (they are also subject to EC-10-033 via the global `EXAMPLE_EXCLUDED_COLS` constant).

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-DESC-DEDUP-001-001 (satisfaction: X.XX) — prism_describe example_query violation; check build_example_with_note EXAMPLE_EXCLUDED_COLS const and agg_col filter predicate (BC-2.10.012 §Auto-generated example queries Tier 2 EC-10-033: synthesized columns must be excluded from GROUP BY target selection)"`

Do NOT disclose: the specific table name(s) that failed, the exact example_query string,
or the count of failing tables.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-DESCRIBE-EXAMPLE-DEDUP-001 branch + standard claroty.sensor.toml (codebase-shipped spec) |
| corpus_size | All Claroty tables registered in claroty.sensor.toml (minimum: claroty_alerts, claroty_devices, claroty_audit_logs; up to ~9 tables from Wave A/B/C expansions) |
| known_edge_cases | Tables with a `severity` column fire Tier 1 not Tier 2 — vacuously pass Part A. Tables with a domain Float/Integer column other than class_uid fire Tier 2 with that domain column — vacuously pass Part A. |
| false_positive_threshold | Zero: GROUP BY class_uid is an unambiguous substring match |
| false_negative_threshold | Near-zero: pre-patch every Claroty table contains the pattern; any single surviving table is a FAIL |

**Known-good corpus:** prism binary built from S-DESCRIBE-EXAMPLE-DEDUP-001 branch with the
EXAMPLE_EXCLUDED_COLS fix applied. Expected: zero tables with `GROUP BY class_uid`.

**Known-problematic corpus:** prism binary from develop HEAD before this story's fix.
Expected: every Claroty table without domain Integer/Float columns produces
`SELECT class_uid, COUNT(*) FROM <table> GROUP BY class_uid ORDER BY COUNT(*) DESC LIMIT 10`
— Part A fails on every such table.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w2-arch-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-036 group for S-DESCRIBE-EXAMPLE-DEDUP-001. EC-10-033 class_uid exclusion from Tier-2 agg_col: prism_describe example_query must not contain GROUP BY class_uid for any table. Primary discriminating test: pre-patch all Claroty tables produce GROUP BY class_uid; post-fix none do. Wire-level assertions on tables[*].example_query. DTU NOT required. SINGLE-USE HIDDEN. |
