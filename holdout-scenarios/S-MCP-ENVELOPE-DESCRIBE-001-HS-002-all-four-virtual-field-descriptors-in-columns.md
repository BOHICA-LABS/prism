---
document_type: holdout-scenario
level: L3
id: "HS-DESC-001-002"
title: "prism_describe columns array contains all four virtual field descriptors (_sensor, _client, _source_table, _source_type) for every returned table"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "E-BETA3-REMEDIATION"
story_source: "S-MCP-ENVELOPE-DESCRIBE-001"
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
  - ".factory/specs/behavioral-contracts/BC-2.10.012-prism-describe-schema-discovery-tool.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.012-virtual-fields.md"
input-hash: "TBD"
traces_to: "BC-2.10.012"
behavioral_contracts:
  - BC-2.10.012
  - BC-2.11.012
verification_properties: []
lifecycle_status: active
introduced: "S-MCP-ENVELOPE-DESCRIBE-001"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-MCP-ENVELOPE-DESCRIBE-001 (HS-034 group). Validates Issue 5 fix: build_column_descriptors_ocsf() OQ-003 block was only appending class_uid + _sensor; the three additional virtual fields (_client, _source_table, _source_type) were absent. Discriminating: pre-patch has only class_uid + _sensor in synthesized set; post-fix has all five (class_uid, _sensor, _client, _source_table, _source_type). Wire-level assertion on names in columns array. BC-2.10.012 §Response shape OQ-003 amended postcondition + BC-2.11.012 §Invariants. Test-writer and implementer must NOT read this file."
---

# HS-DESC-001-002: prism_describe columns array contains all four virtual field descriptors for every returned table

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-MCP-ENVELOPE-DESCRIBE-001 (HS-034 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.10.012 §Response shape OQ-003 amended postcondition: each `TableDescriptor`
in the prism_describe response must include all five synthesized column descriptors appended
after spec-derived columns, in order: `class_uid`, `_sensor`, `_client`, `_source_table`,
`_source_type`. BC-2.11.012 §Invariants: sensor-table virtual field set is exactly four:
`_sensor`, `_client`, `_source_table`, `_source_type`.
**Gate:** Story-level holdout gate (HS-034) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the **OQ-003 synthesized-column extension fix** in
`build_column_descriptors_ocsf()` in `prism_describe.rs`
(BC-2.10.012 §Response shape OQ-003; S-MCP-ENVELOPE-DESCRIBE-001 Issue 5 / AC-002).

When `prism_describe` returns a response with at least one `TableDescriptor`:

1. Each table's `columns` array must contain entries for ALL FOUR virtual field names:
   `_sensor`, `_client`, `_source_table`, `_source_type`.
2. Additionally, `class_uid` must be present (was already appended pre-fix; must not regress).
3. The synthesized entries appear AFTER the spec-derived columns (appended last).

**The defect this scenario catches:** Pre-patch `build_column_descriptors_ocsf()` OQ-003 block
appends only two synthesized descriptors: `class_uid` (OCSF class ID) and `_sensor` (sensor
type provenance). The three additional virtual fields injected by `inject_virtual_fields` at
query time — `_client`, `_source_table`, `_source_type` — are absent from the schema catalog.
An LLM agent calling `prism_describe` cannot discover these fields and therefore cannot write
queries that filter on client, source table, or data-delivery path.

**Discriminating assertion:** The presence of `_client`, `_source_table`, and `_source_type`
in `columns[*].name` for any returned table. Pre-patch: these three names are absent.
Post-fix: all three names are present.

**Three assertions in this scenario:**

**Part A — All four virtual field names present:**
- From any `tables[i].columns` array, extract the set of column names.
- Assert ALL FOUR of these names appear: `_sensor`, `_client`, `_source_table`, `_source_type`.
- This covers BC-2.11.012 §Invariants: "sensor-table virtual field set: exactly four."

**Part B — class_uid regression guard:**
- Assert `class_uid` is also present in `columns[*].name` for the same table.
- This guards against a regression where the fix accidentally removes the pre-existing synthesized entry.

**Part C — No raw col.name shadowing:**
- Assert that `_client`, `_source_table`, and `_source_type` appear exactly ONCE each in the
  `columns` array (they are synthesized; the spec should not also define them as raw columns).
- If any of the three names appears more than once, record a finding (duplicate synthesized entry).

**BDD supplement (Part A):**

**Given** prism is built from the S-MCP-ENVELOPE-DESCRIBE-001 story branch
**And** a test client is configured with at least one Claroty sensor table
**When** `tools/call prism_describe {client_id: "<test_client>"}` is issued via MCP stdio
**Then** the response is not a JSON-RPC error
**And** the `columns` array for any returned table contains an entry with `name: "_sensor"`
**And** the `columns` array for any returned table contains an entry with `name: "_client"`
**And** the `columns` array for any returned table contains an entry with `name: "_source_table"`
**And** the `columns` array for any returned table contains an entry with `name: "_source_type"`
**And** the `columns` array for any returned table contains an entry with `name: "class_uid"`

---

## Setup Instructions

1. Build prism from the S-MCP-ENVELOPE-DESCRIBE-001 story branch (standard build).

2. Prepare a `prism.toml` configuring at least one client with at least one Claroty sensor
   table (the standard `claroty.sensor.toml` is sufficient). Client ID: `"holdout-describe-test"`.
   The DTU does NOT need to be running — prism_describe reads from loaded specs at boot only.

3. Start prism in MCP stdio mode. SETUP-FAILURE if prism fails to start.

4. Complete MCP `initialize` handshake (same as HS-DESC-001-001 §Setup step 4).

5. Issue `prism_describe` call (same as HS-DESC-001-001 §Setup step 5).
   Capture the full raw wire-level JSON response.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.10.012 | §Response shape OQ-003 amended postcondition: five synthesized ColumnDescriptors (class_uid, _sensor, _client, _source_table, _source_type) appended after spec-derived columns | Part A: _client, _source_table, _source_type present |
| BC-2.11.012 | §Invariants: sensor-table virtual field set is exactly four (_sensor, _client, _source_table, _source_type) | Part A: all four virtual names present |
| BC-2.10.012 | §Response shape OQ-003 existing postcondition: class_uid appended (pre-fix; must not regress) | Part B: class_uid present |

---

## Verification Approach

1. Parse the wire-level JSON-RPC response. Verify non-error (same check as HS-DESC-001-001).

2. Parse `result.content[0].text` as JSON. Locate `tables` array. Verify `tables.length >= 1`.
   If `tables.length == 0`: SETUP-FAILURE (no tables configured for test client).

3. Select `tables[0]` (first table) for the assertions. Extract `tables[0].columns` as an array.

4. **Part A — Virtual field name presence:**
   Extract the set of `name` values from `tables[0].columns`.
   For each of: `"_sensor"`, `"_client"`, `"_source_table"`, `"_source_type"`:
     Assert the name is present in the extracted set.
     If any name is absent: record FAIL on Part A dimension with the missing name(s).

5. **Part B — class_uid regression guard:**
   Assert `"class_uid"` is present in the extracted name set.
   If absent: record FAIL on Part B dimension (regression of pre-existing OQ-003 entry).

6. **Part C — No duplicate synthesized entries:**
   Count occurrences of `"_client"`, `"_source_table"`, `"_source_type"` in the columns array.
   Assert each count equals exactly 1.
   If any count > 1: record a FINDING (non-blocking; note the duplicate).

7. Apply Part A and Part B checks to ALL tables in the `tables` array (not just `tables[0]`)
   if time permits. The synthesized columns should appear in every table uniformly.

8. All assertions operate on the PARSED JSON content from `result.content[0].text` —
   not on pre-serialization Rust struct inspection.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response is non-error and tables.length >= 1 (prerequisite)** (weight: 0.10):
  Full credit (1.0): response valid, tables array non-empty.
  Zero credit (0.0): error response or empty tables (SETUP-FAILURE).

- **All four virtual field names present in tables[0].columns** (weight: 0.55):
  Full credit (1.0): `_sensor`, `_client`, `_source_table`, `_source_type` all present.
  Partial credit (0.5): exactly 1 name absent (_sensor present, one of the three new names missing).
  Partial credit (0.2): 2 or 3 of the new names absent (_sensor present but most new ones missing;
  partial fix).
  Zero credit (0.0): all three new names absent AND `_sensor` absent (pre-patch state without _sensor).

- **class_uid present (regression guard)** (weight: 0.25):
  Full credit (1.0): `class_uid` present.
  Zero credit (0.0): `class_uid` absent — OQ-003 pre-existing entry regressed.

- **No duplicate synthesized entries** (weight: 0.10):
  Full credit (1.0): each of `_client`, `_source_table`, `_source_type` appears exactly once.
  Partial credit (0.5): one name duplicated.
  Zero credit (0.0): two or more duplicates.

---

## Edge Conditions

- **table with no spec-derived columns (all synthesized):** Highly unlikely with standard
  claroty.sensor.toml, but if present, the synthesized entries should still appear.

- **_sensor absent pre-fix:** Not possible for existing tests since _sensor was already in the
  OQ-003 block. If _sensor is absent post-fix, that is a regression of the pre-existing behavior.

- **only class_uid + _sensor present (no new virtual fields):** This is the pre-patch state.
  Record FAIL on Part A dimension (post-patch MUST have _client, _source_table, _source_type).

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-DESC-001-002 (satisfaction: X.XX) — virtual field column descriptor gap in prism_describe schema output; check build_column_descriptors_ocsf() OQ-003 block (BC-2.10.012 §Response shape OQ-003: five synthesized ColumnDescriptors required; BC-2.11.012 §Invariants: sensor-table virtual field set must be exactly four)"`

Do NOT disclose: which specific field names were found absent, the exact column count, or
which table was inspected.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-MCP-ENVELOPE-DESCRIBE-001 branch + standard claroty.sensor.toml |
| corpus_size | columns array length per table (spec-derived + 5 synthesized; exact count depends on spec) |
| known_edge_cases | pre-patch state: only class_uid + _sensor in synthesized set (2 of 5); this scenario catches missing 3 |
| false_positive_threshold | Near-zero: presence of specific string names in JSON array is unambiguous |
| false_negative_threshold | Zero: pre-patch cannot produce _client/_source_table/_source_type in columns |

**Known-good corpus:** S-MCP-ENVELOPE-DESCRIBE-001 story branch with OQ-003 extension applied.
Expected: `_sensor`, `_client`, `_source_table`, `_source_type`, `class_uid` all in `columns[*].name`.

**Known-problematic corpus:** pre-patch develop HEAD. Expected: `class_uid` and `_sensor` present;
`_client`, `_source_table`, `_source_type` absent — the Issue 5 defect.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w2-holdout-authoring | 2026-09-16 | product-owner | Initial authoring. HS-034 group for S-MCP-ENVELOPE-DESCRIBE-001. Issue 5 (missing virtual fields): prism_describe columns array must contain all four virtual field descriptors. Discriminates pre-patch (only class_uid+_sensor) from post-fix (all five: class_uid, _sensor, _client, _source_table, _source_type). BC-2.10.012 §Response shape OQ-003 + BC-2.11.012 §Invariants. SINGLE-USE. |
