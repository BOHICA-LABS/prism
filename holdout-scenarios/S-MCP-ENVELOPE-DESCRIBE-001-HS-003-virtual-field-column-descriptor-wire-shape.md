---
document_type: holdout-scenario
level: L3
id: "HS-DESC-001-003"
title: "prism_describe newly-added virtual field column descriptors have nullable=false in wire JSON and total_results + columns count are self-consistent"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-MCP-ENVELOPE-DESCRIBE-001 (HS-034 group). Validates the WIRE SHAPE of the three newly-added virtual field ColumnDescriptors: each must carry nullable=false in the serialized JSON. Also validates combined self-consistency: _meta.total_results == tables.length (Issue 3 fix) AND each table has the four virtual fields in columns (Issue 5 fix). This scenario exercises both fixes together in one call, providing a compound correctness gate. BC-2.11.012 §Invariants empty-MemTable schema parity note: prism_describe ColumnDescriptor uses nullable=false (populated-path behavior). Test-writer and implementer must NOT read this file."
---

# HS-DESC-001-003: Virtual field column descriptors have nullable=false wire shape, and both fixes (total_results + virtual fields) are self-consistent in a single response

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-MCP-ENVELOPE-DESCRIBE-001 (HS-034 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.11.012 §Invariants (empty-MemTable schema parity note): the prism_describe
`ColumnDescriptor` for virtual fields MUST declare `nullable: false` (reflecting populated-path
injection behavior, NOT the empty-table pre-registration `nullable: true` schema). Also BC-2.10.012
§Response shape (EC-005 from story edge cases): `_source_type` column descriptor: `nullable = false`.
This scenario also verifies compound self-consistency across both Issue 3 and Issue 5 fixes in
one `prism_describe` response.
**Gate:** Story-level holdout gate (HS-034) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the **wire shape of newly-added virtual field ColumnDescriptors** and the
**compound self-consistency of both fixes** in a single `prism_describe` call
(BC-2.10.012; S-MCP-ENVELOPE-DESCRIBE-001 ACs for Issues 3 + 5 combined; AC-005 story edge case).

**Part A — nullable=false for the three new virtual field column descriptors:**
The three new synthesized ColumnDescriptors appended by the OQ-003 extension
(`_client`, `_source_table`, `_source_type`) must carry `nullable: false` in the wire JSON.
This matches the populated-path inject_virtual_fields behavior (the virtual fields are always
non-null in a query result). Per BC-2.11.012 §Invariants parity note:
> "the describe surface should reflect the populated-path behavior (nullable=false)"

**Part B — Compound self-consistency gate (both fixes verified in one response):**
Issue a single `prism_describe` call and verify:
1. `_meta.total_results > 0` AND `_meta.total_results == tables.length` (Issue 3 fix)
2. At least one table has `_client`, `_source_table`, `_source_type` in its `columns` (Issue 5 fix)

This combination in one call confirms that neither fix interferes with the other.

**The defect this scenario catches:** An implementation that adds the virtual field names but
sets `nullable: true` (or omits the `nullable` field entirely) would fail Part A. An
implementation that fixes one issue but not both (e.g., correct virtual fields but still
`total_results: 0`) would fail Part B.

**Discriminating assertion (Part A):** `nullable` field on `_client`, `_source_table`,
`_source_type` column descriptors in the wire JSON equals `false` (JSON boolean false).

**BDD supplement (combined):**

**Given** prism is built from the S-MCP-ENVELOPE-DESCRIBE-001 story branch
**And** a test client is configured with at least one Claroty sensor table
**When** `tools/call prism_describe {client_id: "<test_client>"}` is issued via MCP stdio
**Then** the response is not a JSON-RPC error
**And** `_meta.total_results > 0` AND `_meta.total_results == tables.length`
**And** the entry for `_client` in `tables[0].columns` has `nullable: false` (JSON boolean false)
**And** the entry for `_source_table` in `tables[0].columns` has `nullable: false`
**And** the entry for `_source_type` in `tables[0].columns` has `nullable: false`

---

## Setup Instructions

Same as HS-DESC-001-001 §Setup steps 1–5. The DTU does NOT need to be running.
Use the same `holdout-describe-test` client with standard claroty.sensor.toml.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.012 | §Invariants empty-MemTable schema parity: describe surface reflects populated-path nullable=false | Part A: nullable=false on _client, _source_table, _source_type |
| BC-2.10.012 | §Response shape EC-005 (story edge case): `_source_type` ColumnDescriptor nullable=false | Part A: _source_type nullable=false |
| BC-2.10.012 | §Response envelope EC-10-032: total_results == tables.len() | Part B combined gate dimension 1 |
| BC-2.10.012 | §Response shape OQ-003 amended: _client, _source_table, _source_type present | Part B combined gate dimension 2 |

---

## Verification Approach

1. Parse the wire-level JSON-RPC response. Verify non-error. Parse `result.content[0].text`.

2. **Part B — Compound self-consistency (both fixes):**
   a. Assert `_meta.total_results == tables.length > 0` (Issue 3).
   b. Assert `tables[0].columns` contains `_client`, `_source_table`, `_source_type` (Issue 5).
   Record which sub-dimensions pass/fail independently.

3. **Part A — nullable=false for each new virtual field:**
   For `tables[0].columns`, find the entry where `name == "_client"`:
     Extract its `nullable` field value.
     Assert `nullable == false` (JSON boolean false, NOT `null`, NOT `true`, NOT absent).
   Repeat for `_source_table` and `_source_type`.
   If any entry is missing (name not found), record FAIL on Part A (name absence also fails
   Part B; both dimensions affected).
   If `nullable` field is absent: record a FINDING (structural gap — field must be present).
   If `nullable == true`: record FAIL — violates BC-2.11.012 §Invariants parity note.

4. **Bonus assertion — _sensor nullable=false (non-regression):**
   Find the entry where `name == "_sensor"` and check `nullable == false` (was already correct
   pre-fix; verify not regressed by the extension).

5. All assertions on PARSED JSON from `result.content[0].text` wire bytes.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response non-error and tables non-empty (prerequisite)** (weight: 0.10):
  Full credit (1.0): valid response, tables.length >= 1.
  Zero credit (0.0): error or empty (SETUP-FAILURE).

- **Part B — total_results == tables.length > 0** (weight: 0.25):
  Full credit (1.0): both conditions satisfied.
  Zero credit (0.0): total_results == 0 with tables.length >= 1 (Issue 3 not fixed).

- **Part B — _client + _source_table + _source_type present in columns[*].name** (weight: 0.25):
  Full credit (1.0): all three names present.
  Partial credit (0.3): 1–2 names present.
  Zero credit (0.0): all three absent (Issue 5 not fixed).

- **Part A — nullable == false for all three new virtual fields** (weight: 0.40):
  Full credit (1.0): all three entries have `nullable: false` in wire JSON.
  Partial credit (0.5): one or two entries have `nullable: false`; remaining have `nullable: true`
  or field absent.
  Zero credit (0.0): all three have `nullable: true` or field absent — BC-2.11.012 §Invariants
  parity note violated.

---

## Edge Conditions

- **nullable field absent from ColumnDescriptor:** Record as FINDING (structural gap).
  Score 0.0 for that entry's nullable dimension — the field must be explicitly present to
  serve as a schema advertisement for LLM agents.

- **nullable == true on _source_type:** Per story edge case EC-005: this is the specific
  error that would occur if the fix incorrectly used the empty-MemTable schema (which uses
  `nullable: true` on the pre-registration path). The prism_describe ColumnDescriptor must
  use the populated-path value (false).

- **Both issues still broken:** Part A FAIL implies Part B Issue 5 also fails; both dimensions
  score zero. The compound gate catches both simultaneously.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-DESC-001-003 (satisfaction: X.XX) — virtual field column descriptor wire shape or compound-fix self-consistency gap; check (1) nullable field on synthesized ColumnDescriptors in build_ocsf_column_descriptors() [BC-2.11.012 §Invariants: nullable=false required, not true]; (2) safety_envelope wrap() tables-arm plus OQ-003 extension producing consistent _meta.total_results and columns in same response [BC-2.10.012 §Response envelope EC-10-032 + §Response shape OQ-003]"`

Do NOT disclose: whether Part A or Part B failed specifically, the exact nullable values
observed, or which virtual field was problematic.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-MCP-ENVELOPE-DESCRIBE-001 branch + standard claroty.sensor.toml |
| corpus_size | per-table column count (N spec columns + 5 synthesized); exact N depends on claroty spec table |
| known_edge_cases | nullable: true on new virtual fields = BC-2.11.012 §Invariants violation; absent nullable field = structural gap |
| false_positive_threshold | Near-zero: nullable boolean value is unambiguous |
| false_negative_threshold | Near-zero: pre-patch cannot produce _client/_source_table/_source_type (Part B fails) and correct nullable requires explicit fix |

**Known-good corpus:** S-MCP-ENVELOPE-DESCRIBE-001 branch with both fixes applied.
Expected: total_results > 0 and == tables.length; _client/_source_table/_source_type present
with nullable=false.

**Known-problematic corpus:** partial fix (virtual field names added but nullable: true
left as default, OR total_results still 0). Catches incomplete implementation.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w2-holdout-authoring | 2026-09-16 | product-owner | Initial authoring. HS-034 group for S-MCP-ENVELOPE-DESCRIBE-001. Compound gate: (1) nullable=false for new virtual field column descriptors in wire JSON [BC-2.11.012 §Invariants parity note]; (2) both Issue 3 + Issue 5 fixes self-consistent in one prism_describe call. Discriminates partial/incorrect implementations. SINGLE-USE. |
