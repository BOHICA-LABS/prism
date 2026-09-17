---
document_type: holdout-scenario
level: L3
id: "HS-DESC-DEDUP-001-003"
title: "No prism_describe example_query across any sensor uses GROUP BY targeting a synthesized metadata column name"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-DESCRIBE-EXAMPLE-DEDUP-001 (HS-036 group). Cross-sensor universal prohibition: for ALL sensors in the describe response (Claroty, CrowdStrike, Cyberint, Armis), no table's example_query may use GROUP BY with any of the five synthesized column names (class_uid, _sensor, _client, _source_table, _source_type). Complements HS-DESC-DEDUP-001-001 (Claroty-specific) with a global invariant check. Includes non-regression guard: tables with a severity column must produce a Tier 1 WHERE severity example, NOT a GROUP BY example — excluding class_uid must not suppress Tier 1. DTU NOT required. Test-writer and implementer must NOT read this file."
---

# HS-DESC-DEDUP-001-003: No prism_describe example_query across any sensor uses GROUP BY targeting a synthesized metadata column name

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-DESCRIBE-EXAMPLE-DEDUP-001 (HS-036 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.10.012 §Auto-generated example queries EC-10-033:
The `EXAMPLE_EXCLUDED_COLS` const applies globally to ALL sensors, not just Claroty.
The five synthesized metadata column names `{class_uid, _sensor, _client, _source_table,
_source_type}` must NEVER appear as the GROUP BY target in any sensor's example queries,
regardless of whether they are Integer-typed (`class_uid`) or String-typed (the others).
**Gate:** Story-level holdout gate (HS-036) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the **universal scope of the EC-10-033 prohibition**
(BC-2.10.012 §Auto-generated example queries EC-10-033;
S-DESCRIBE-EXAMPLE-DEDUP-001 AC-001 / AC-002 / Architecture Compliance Rule 2).

Where HS-DESC-DEDUP-001-001 checks Claroty-specific GROUP BY class_uid, this scenario
extends the check to:

1. **All five synthesized column names:** `class_uid`, `_sensor`, `_client`, `_source_table`,
   `_source_type`. The last four are String-typed and cannot reach Tier 2's Integer/Float
   predicate; they are excluded defensively in `EXAMPLE_EXCLUDED_COLS`. This scenario
   catches the case where a future TOML change introduces an Integer-typed virtual column
   that would otherwise become an agg_col candidate.

2. **All sensors in the describe response:** Not only Claroty tables, but every table from
   every sensor the test client configures (CrowdStrike, Cyberint, Armis, any others).
   `EXAMPLE_EXCLUDED_COLS` is a module-level const — it applies in all code paths.

3. **Non-regression guard — Tier 1 not suppressed:** If a table has a `severity` String
   column, EC-10-033 must NOT suppress the Tier 1 (severity-IEQ) example. The fix is in
   the Tier 2 `agg_col` selection; Tier 1 runs BEFORE Tier 2. An incorrect implementation
   that gates Tier 1 on the exclusion check would produce a broken behavior where severity-
   column tables fall to Tier 3/4 instead of producing the Tier 1 example.

**Discriminating assertion:**

For any table that uses GROUP BY in its example_query, the GROUP BY field must be a
domain-data column — NOT one of the five synthesized names.

- **Pre-patch (Claroty tables):** Many tables have `GROUP BY class_uid` → assertion fails.
- **Post-fix:** No table has GROUP BY targeting any synthesized name → assertion passes.

**Three assertions in this scenario:**

**Part A — Global synthesized-name prohibition in GROUP BY:**
- For ALL tables in ALL sensors in the response:
  - If `example_query` contains `GROUP BY`: extract the identifier immediately following
    `GROUP BY ` (up to the next whitespace or end of string).
  - Assert that identifier is NOT in `{class_uid, _sensor, _client, _source_table, _source_type}`.
- Equivalently: no `example_query` contains any of these literal substrings:
  - `GROUP BY class_uid`
  - `GROUP BY _sensor`
  - `GROUP BY _client`
  - `GROUP BY _source_table`
  - `GROUP BY _source_type`

**Part B — Non-regression: Tier 1 fires for severity-column tables (if any):**
- For each table whose `example_query` contains the substring `severity`:
  - Assert: the example_query contains `WHERE` (Tier 1 uses a WHERE clause, not GROUP BY).
  - Assert: the example_query does NOT contain `GROUP BY` (Tier 1 must have fired, not Tier 2).
- If no table in the response has a severity-related example, skip Part B (not a failure).

**Part C — All examples are syntactically valid SELECT strings:**
- For ALL tables: assert `example_query` is a non-empty string beginning with `SELECT`.
- This is a basic non-regression guard that the fix did not introduce empty or null queries
  in the non-Claroty sensor path.

**BDD supplement (Part A):**

**Given** prism is built from the S-DESCRIBE-EXAMPLE-DEDUP-001 story branch
**And** a test client is configured with Claroty and optionally other sensors
**When** `tools/call prism_describe {"client_id": "<test_client>"}` is issued via MCP stdio
**Then** the response is not a JSON-RPC error
**And** `result.content[0].text` parses as valid JSON with a non-empty `tables` array
**And** no entry in `tables` has an `example_query` containing `"GROUP BY class_uid"`
**And** no entry in `tables` has an `example_query` containing `"GROUP BY _sensor"`
**And** no entry in `tables` has an `example_query` containing `"GROUP BY _client"`
**And** no entry in `tables` has an `example_query` containing `"GROUP BY _source_table"`
**And** no entry in `tables` has an `example_query` containing `"GROUP BY _source_type"`
**And** every entry with a severity-referencing example uses `WHERE`, not `GROUP BY`

---

## Setup Instructions

1. Build prism from the S-DESCRIBE-EXAMPLE-DEDUP-001 story branch. No special feature flags.

2. Prepare a `prism.toml` configuring a test client with the Claroty sensor (same as
   HS-DESC-DEDUP-001-001). Optionally configure additional sensors (CrowdStrike, Cyberint,
   Armis) with the test client if available — this scenario strengthens with more sensor
   coverage, but Claroty alone is sufficient for the primary assertion.
   **Note:** DTU NOT required — `prism_describe` reads TOML specs at boot.

3. Start prism in MCP stdio mode. Complete the MCP `initialize` handshake.

4. Issue the `prism_describe` tool call:
   ```
   {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"prism_describe","arguments":{"client_id":"holdout-claroty-dedup"}}}
   ```
   Capture the full wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.10.012 | §Auto-generated example queries EC-10-033: EXAMPLE_EXCLUDED_COLS applies globally to all sensors | Part A: universal prohibition across all tables |
| BC-2.10.012 | §Auto-generated example queries Tier 1 severity-IEQ priority: Tier 1 fires BEFORE Tier 2 and is unaffected by agg_col exclusion | Part B: Tier 1 non-regression for severity tables |
| BC-2.10.012 | §Auto-generated example queries: every table produces a non-empty SELECT example | Part C: structural validity |

---

## Verification Approach

1. Parse the wire-level JSON-RPC response. Verify non-error. Extract tables array.
   If tables is empty: SETUP-FAILURE.

2. **Part A — Universal synthesized-name prohibition:**
   Initialize `violations = []`.
   For each table entry `t` in `tables`:
     For each name in `["class_uid", "_sensor", "_client", "_source_table", "_source_type"]`:
       If `t.example_query` contains `"GROUP BY " + name`:
         Append `{table: t.name, column: name, query: t.example_query}` to violations.
   Assert: `violations` is empty.
   If non-empty: record FAIL with the count of violations (not the specific content).

3. **Part B — Tier 1 non-regression:**
   For each table entry `t` where `t.example_query` contains `"severity"`:
     Assert: `t.example_query` contains `"WHERE"`.
     Assert: `t.example_query` does NOT contain `"GROUP BY"`.
   If no tables have severity in their example: skip Part B, record N/A.

4. **Part C — Structural validity:**
   For each table entry `t`:
     Assert: `t.example_query` is a non-empty string beginning with `"SELECT"`.
   Record any violations.

5. All assertions are on the parsed JSON from `result.content[0].text`.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response is non-error (prerequisite)** (weight: 0.10):
  Full credit (1.0): non-error result, non-empty tables array.
  Zero credit (0.0): error or empty tables (SETUP-FAILURE).

- **Global synthesized-name prohibition in GROUP BY (Part A)** (weight: 0.65):
  Full credit (1.0): zero violations across all tables and all five synthesized names.
  Partial credit (0.3): exactly one violation (single missed table; partial fix).
  Zero credit (0.0): two or more violations.

- **Tier 1 non-regression (Part B)** (weight: 0.15):
  Full credit (1.0): all severity-referencing examples use WHERE not GROUP BY. Or N/A (no such tables).
  Zero credit (0.0): any severity-column table produces a GROUP BY example (Tier 1 suppressed — severe regression).

- **Structural validity (Part C)** (weight: 0.10):
  Full credit (1.0): all examples are non-empty SELECTs.
  Zero credit (0.0): any table has an empty, null, or non-SELECT example.

---

## Edge Conditions

- **No sensor tables have synthesized columns with Integer type (only class_uid does):**
  The checks for `GROUP BY _sensor` etc. are defensive and will vacuously pass for
  String-typed columns. Part A nonetheless applies them in case future TOML specs change
  the type of a synthesized column.

- **Multi-sensor response:** If the evaluator configures CrowdStrike or Cyberint in addition
  to Claroty, those sensors' tables also appear in the response. Part A covers all of them.
  CrowdStrike tables may have `severity` columns — Part B covers the non-regression there.

- **Part B N/A (no severity-column tables found):** Acceptable. The Tier 1 non-regression
  guard only triggers when a severity-related example is present. Record N/A for Part B;
  use full credit weight on Parts A and C.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-DESC-DEDUP-001-003 (satisfaction: X.XX) — prism_describe example_query contains synthesized column name in GROUP BY clause; check EXAMPLE_EXCLUDED_COLS coverage in build_example_with_note (BC-2.10.012 §Auto-generated example queries EC-10-033: all five synthesized column names must be excluded from agg_col selection across all sensors)"`

Do NOT disclose: which specific table or column name triggered the violation, or whether
the failure was on the Claroty or another sensor's tables.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-DESCRIBE-EXAMPLE-DEDUP-001 branch + standard claroty.sensor.toml (and optionally crowdstrike.sensor.toml, cyberint.sensor.toml if configured) |
| corpus_size | All sensor tables registered in the test client's sensor configuration |
| known_edge_cases | String-typed synthesized columns cannot be agg_col candidates by type predicate; Part A checks them anyway (defensive). Part B N/A if no severity-column tables present. |
| false_positive_threshold | Near-zero: exact substring match `GROUP BY class_uid` etc. is unambiguous |
| false_negative_threshold | Near-zero: pre-patch Claroty tables ALL contain GROUP BY class_uid; post-fix none do |

**Known-good corpus:** prism binary with EXAMPLE_EXCLUDED_COLS fix applied to
`build_example_with_note`. Expected: zero Part A violations; Part B N/A or passes.

**Known-problematic corpus:** pre-patch prism binary. Expected: all Claroty tables without
domain Integer/Float columns have `GROUP BY class_uid` — at minimum 3+ violations
(claroty_alerts, claroty_devices, claroty_audit_logs). Part A fails.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w2-arch-holdout-authoring | 2026-09-17 | product-owner | Initial authoring. HS-036 group for S-DESCRIBE-EXAMPLE-DEDUP-001. Universal synthesized-column GROUP BY prohibition across all sensors and all five excluded names. Includes Tier 1 non-regression guard (severity tables must still produce WHERE severity form, not GROUP BY). BC-2.10.012 EC-10-033. DTU NOT required. SINGLE-USE HIDDEN. |
