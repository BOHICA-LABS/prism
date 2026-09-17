---
document_type: story
story_id: S-MCP-ENVELOPE-DESCRIBE-001
title: "Fix prism_describe total_results counting and add missing virtual field column descriptors"
wave: 2
epic_id: E-BETA3-REMEDIATION
version: "1.1"
status: ready
producer: story-writer
phase: 3
priority: P0
points: 3
tdd_mode: strict
target_module: prism-mcp
subsystems: ["SS-10"]
# Subsystem anchor justification:
#   SS-10 (MCP Server) owns crates/prism-mcp/src/safety_envelope.rs,
#   crates/prism-mcp/src/tools/prism_describe.rs, and
#   crates/prism-mcp/tests/mcp_prism_describe.rs. All files touched by this
#   story live within SS-10's module boundary. No other subsystem is crossed.
crates_touched: [prism-mcp]
estimated_days: 0.5
depends_on: []
# depends_on anchor justification:
#   No hard product-story dependencies — both fixes are isolated to prism-mcp.
#   The delta analysis marks this story as "(no deps)" with no spec pre-work
#   required (beta3-remediation-delta-analysis.md §Batch 0; S-MCP-ENVELOPE-DESCRIBE-001
#   is not gated on any Batch 0 spec pre-work).
#   Soft merge-ordering recommendation: merge S-MCP-TOOL-GATE-001 first to reduce
#   merge-conflict risk on server.rs (TOOL-GATE has lower blast radius). This is
#   a merge-ordering preference, NOT a compile-time dependency.
blocks:
  - S-BETA3-RELEASE-001
# blocks anchor justification:
#   S-BETA3-RELEASE-001: W2 must merge before the beta.3 release bundle assembles.
risk: LOW
# Risk justification:
#   Issue 3 fix (safety_envelope.rs): additive `tables` arm in wrap() total_results
#   counting. No existing behavior changes for query/sensor tool paths.
#   Issue 5 fix (prism_describe.rs): additive ColumnDescriptor append after _sensor.
#   Tests asserting column count must be updated but no behavioral regression.
behavioral_contracts:
  - BC-2.10.012
  - BC-2.11.012
# BC status: AMENDMENTS ACTIVE (D-2544).
#   BC-2.10.012 v1.10 now contains:
#     (a) EC-10-032: prism_describe total_results = tables.len() (not 0).
#         SafetyEnvelopeBuilder.wrap() tables-arm resolves total_results per §Response envelope.
#     (b) Five synthesized ColumnDescriptors in §Response shape (OQ-003 extended), in exact order:
#         class_uid, _sensor, _client, _source_table, _source_type.
#   ADR-058 v2.44: §G OQ-003 now enumerates all five synthesized ColumnDescriptors.
#   BC-2.11.012 v1.11: active — canonical sensor-table virtual field set (exactly four:
#     _sensor, _client, _source_table, _source_type). No amendment required; used as
#     conformance contract only.
#   Spec-First Gate S-7.01 satisfied; story is unblocked for test-writer dispatch.
verification_properties: []
assumption_validations: []
risk_mitigations: []
---

# S-MCP-ENVELOPE-DESCRIBE-001: Fix prism_describe total_results Counting and Add Missing Virtual Field Descriptors

## Authority

**beta3-remediation-delta-analysis.md §Issue 3 + §Issue 5 + §S-MCP-ENVELOPE-DESCRIBE-001** is the
authoritative design decision for this story. Read Part 1 §Issue 3, Part 1 §Issue 5, and Part 3
§S-MCP-ENVELOPE-DESCRIBE-001 in full before implementing.
Path: `.factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md`

**BC-2.10.012 v1.10** (prism_describe Schema Discovery Tool) governs the response shape,
synthesized column set, response envelope, and all prism_describe postconditions.
Path: `.factory/specs/behavioral-contracts/BC-2.10.012-prism-describe-schema-discovery-tool.md`

**BC-2.11.012 v1.11** (Virtual Fields in Queries) defines the canonical sensor-table virtual
field set: exactly four fields (`_sensor`, `_client`, `_source_table`, `_source_type`).
This story implements conformance with §Invariants "sensor-table virtual field set: exactly four".
Path: `.factory/specs/behavioral-contracts/BC-2.11.012-virtual-fields.md`

**ADR-058 v2.44 §G** (OQ-003 synthesized columns) defines which columns are appended as
synthesized descriptors. The OQ-003 block in prism_describe.rs must be extended to five
columns per the amendment (D-2544).
Path: `.factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md`

> NOTE: BC-2.10.012 v1.10 and ADR-058 v2.44 amendments are now active per D-2544.
> Spec-First Gate S-7.01 is satisfied; story is unblocked for test-writer dispatch.

---

## Problem Statement

Two distinct defects were identified in the beta.2 Monroe demo live-test log (D-2520) against
the Claroty xDome tenant (jea-readapi). Both are confined to `crates/prism-mcp`.

### Issue 3 — prism_describe total_results: 0 in safety_envelope.wrap()

`safety_envelope::wrap()` in `safety_envelope.rs` computes `total_results` via two arms:
1. `results.as_array()` → bare array length
2. `results.get("rows").and_then(|v| v.as_array())` → query tool `{rows: [...]}` shape

`prism_describe` returns an Object `{client_id, tables: [...], pql_hints: [...]}` — neither
arm matches. The `else { 0 }` branch fires unconditionally, producing `_meta.total_results: 0`
regardless of how many tables the client has. The LLM agent receives `"0 results found"` in the
prose summary even when the client has 9 Claroty tables.

**Fix:** Add a third arm in `wrap()`:
```rust
} else if let Some(tables_arr) = results.get("tables").and_then(|v| v.as_array()) {
    tables_arr.len() as u64
}
```
This arm fires only for the `prism_describe` shape; no existing behavior changes for query/sensor paths.

### Issue 5 — 3 of 4 virtual fields absent from prism_describe column list

`build_column_descriptors_ocsf` in `prism_describe.rs` appends only two synthesized descriptors
after spec-derived columns (OQ-003):
1. `class_uid` (Integer, nullable=false) — OCSF class identifier
2. `_sensor` (String, nullable=false) — sensor type provenance

The virtual fields `_client`, `_source_table`, and `_source_type` — which are injected into
every sensor RecordBatch by `inject_virtual_fields` per BC-2.11.012 — are absent. An LLM agent
calling `prism_describe` cannot discover these columns and therefore cannot write queries that
filter on client, source table, or data-delivery path.

**Fix:** Append three additional synthesized `ColumnDescriptor` entries after `_sensor`:
```rust
ColumnDescriptor { name: "_client".to_string(), col_type: ColumnType::String, nullable: false, description: None }
ColumnDescriptor { name: "_source_table".to_string(), col_type: ColumnType::String, nullable: false, description: None }
ColumnDescriptor { name: "_source_type".to_string(), col_type: ColumnType::String, nullable: false, description: None }
```
This matches the `VirtualField` variants injected at query time (BC-2.11.012 §Invariants).

---

## Narrative

As an LLM agent using `prism_describe` to plan a PrismQL query, I want the response to show
the correct number of tables in `_meta.total_results` and include all four virtual field columns
in each table's `columns` array, so that I can understand query scope and use `_client`,
`_source_table`, and `_source_type` filters without being surprised by missing columns.

---

## Behavioral Contracts

| BC | Title | Version at Authoring | Scope in This Story |
|----|-------|---------------------|---------------------|
| BC-2.10.012 | `prism_describe` Schema Discovery Tool (L2) | v1.10 | §Response envelope: total_results = tables.len() for prism_describe shape (EC-10-032); §Response shape: five synthesized column descriptors (OQ-003 extended — class_uid, _sensor, _client, _source_table, _source_type) |
| BC-2.11.012 | Virtual Fields in Queries — `_sensor`, `_client`, `_source_table`, `_source_type` | v1.11 (active, no amendment) | §Invariants: sensor-table virtual field set exactly four; this story implements conformance for the prism_describe schema-discovery surface |

---

## Acceptance Criteria

### AC-001 — prism_describe `_meta.total_results` equals the table count

When `prism_describe` returns a response with N tables in the `tables` array,
`_meta.total_results` in the envelope equals N (not 0). For a client configured with
3 tables (e.g., claroty_alerts, claroty_devices, claroty_audit_logs), the envelope must
carry `"total_results": 3`.

Wire-shape assertion (SID-2): serialize the full response to JSON and assert
`response["_meta"]["total_results"] == 3` on the serialized bytes, not only on the
pre-serialization Rust u64 value.

(traces to BC-2.10.012 §Response envelope amended postcondition: total_results = tables.len();
relies on `wrap()` tables-arm addition in safety_envelope.rs)

### AC-002 — All four virtual field descriptors present in every table's `columns` array

Every `TableDescriptor` returned by `prism_describe` includes all four virtual field
`ColumnDescriptor` entries as synthesized columns (appended after spec-derived columns):
- `_sensor` (String, nullable: false) — already present, must remain
- `_client` (String, nullable: false) — NEW: added by this story
- `_source_table` (String, nullable: false) — NEW: added by this story
- `_source_type` (String, nullable: false) — NEW: added by this story

The assertion must verify that `columns.iter().any(|c| c.name == "_client")`,
`_source_table`, and `_source_type` are all true for a table returned by a real
`prism_describe` call (not just the ColumnDescriptor builder in isolation).

(traces to BC-2.10.012 §Response shape OQ-003 amended postcondition: five synthesized columns;
BC-2.11.012 §Invariants: sensor-table virtual field set exactly four: _sensor, _client,
_source_table, _source_type)

### AC-003 — Wire-shape JSON includes total_results at the correct value

The serialized JSON envelope from a `prism_describe` call with N > 0 tables must include
a `"total_results"` field in `_meta` whose value equals the count of entries in the
`tables` array. This assertion must operate on the bytes of the serialized JSON string,
not on intermediate Rust struct fields, per SID-2 wire-shape assertion discipline.

Example: `let json = serde_json::to_string(&envelope).unwrap(); let v: serde_json::Value =
serde_json::from_str(&json).unwrap(); assert_eq!(v["_meta"]["total_results"], 3);`

(traces to BC-2.10.012 §Response envelope amended postcondition; BC-2.09.008 §total_results
field wire semantics)

### AC-004 — Existing example_query behavior is unaffected

Adding three new synthesized `ColumnDescriptor` entries to the `columns` array must not
change `build_example_with_note` output for any table. The priority ladder (Tier 1: severity-IEQ,
Tier 2: aggregate, Tier 3: count-recent, Tier 4: column-free fallback) must produce identical
queries before and after the virtual field additions.

Specifically: `_client`, `_source_table`, and `_source_type` are String-typed. They must
NOT trigger the Tier 2 aggregate template (which fires on Integer/Float columns only).
`class_uid` and `_sensor` behavior is unchanged.

(traces to BC-2.10.012 §Auto-generated example queries — priority ladder postcondition
unchanged; virtual field ColumnType::String columns must not be aggregate-example targets)

---

## Red Gate Test List (SAC-1 — BC-5.38.001)

All three failing tests reside in the EXISTING file
`crates/prism-mcp/tests/mcp_prism_describe.rs`.

- **RG-DESC-001**: `test_BC_2_10_012_prism_describe_total_results_equals_table_count`
  Assert: calling `prism_describe` for a client configured with 3 tables returns an envelope
  where `_meta.total_results == 3`. Currently FAILS: `wrap()` hits `else { 0 }` for the
  `{client_id, tables, pql_hints}` object shape; returns `total_results: 0`.
  Covers AC-001.

- **RG-DESC-002**: `test_BC_2_10_012_virtual_fields_all_four_present_in_describe_columns`
  Assert: the `columns` array for any table returned by `prism_describe` includes entries
  for `_client`, `_source_table`, and `_source_type` (in addition to the already-present
  `_sensor` and `class_uid`). Currently FAILS: only `class_uid` and `_sensor` are appended
  in the OQ-003 block; `_client`, `_source_table`, `_source_type` are absent.
  Covers AC-002.

- **RG-DESC-003**: `test_BC_2_10_012_wire_shape_total_results_in_json`
  Assert: `serde_json::to_string(&envelope)` followed by `json["_meta"]["total_results"] == N`
  on the serialized bytes (SID-2 wire-shape requirement). Currently FAILS because
  `total_results` is 0 (same root cause as RG-DESC-001).
  Covers AC-003.

**BC-5.38.001 density check:** 3 failing Red Gate tests / 4 ACs = **0.75** — satisfies the
≥ 0.5 threshold. AC-004 is a regression guard on existing behavior (currently GREEN); it does
not require a new failing test but an existing test must be confirmed GREEN after the addition
of three new column descriptors to verify no priority-ladder breakage.

**Red-then-green task ordering:** RG-DESC-001..003 must be written and confirmed RED by
test-writer BEFORE implementer begins any implementation tasks. See §Tasks.

---

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| `wrap()` total_results counting | `crates/prism-mcp/src/safety_envelope.rs` | Pure (arithmetic on Value shape) |
| OQ-003 synthesized-column append | `crates/prism-mcp/src/tools/prism_describe.rs` — `build_column_descriptors_ocsf()` | Pure (Vec append) |
| Red Gate + wire-shape tests | `crates/prism-mcp/tests/mcp_prism_describe.rs` | Pure (tests) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Client has 0 tables (empty schema) | `tables: []` → `total_results: 0` (wrap() tables-arm returns 0 for empty array) — correct; no change needed for zero-table path |
| EC-002 | `prism_query` tool result `{rows: [...]}` shape after fix | Unchanged: `wrap()` rows-arm still fires; tables-arm is never reached for query results |
| EC-003 | Table with Integer column AND _client/_source_table/_source_type appended | Tier 2 (aggregate) example fires on the Integer column; String-typed virtual fields are NOT picked as aggregate target — priority ladder unchanged |
| EC-004 | Table with ONLY String and virtual-field columns (no Integer/Float, no severity-vocab, no Datetime) | Falls through to Tier 4 (column-free fallback: `SELECT * FROM <t> LIMIT 25`) — _client/_source_table/_source_type (String) do not satisfy Tier 1/2/3 conditions |
| EC-005 | `_source_type` column descriptor: nullable = false | Per BC-2.11.012 §Invariants empty-MemTable schema parity note: the describe surface must declare nullable=false (matching the populated-path injection behavior, not the empty-path nullable=true schema) |
| EC-006 | Injection scanner scan_target for prism_describe after fix | scan_target remains None for prism_describe (neither as_array() nor get("rows") matches); DI-006 says no injection scan required for server-authored catalog — correct behavior, no change |

---

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story file | ~5,000 | |
| BC-2.10.012 v1.10 (active — primary AC source) | ~20,000 | Large; read §Response envelope + §Response shape + §OQ-003 |
| BC-2.11.012 v1.11 (virtual fields — §Invariants, §Postconditions) | ~8,000 | Read full; §Invariants is the authoritative field set |
| `safety_envelope.rs` (full) | ~5,000 | wrap() function + struct definitions |
| `crates/prism-mcp/src/tools/prism_describe.rs` (full) | ~30,000 | Large; includes build_column_descriptors_ocsf OQ-003 block and all tests |
| `crates/prism-mcp/tests/mcp_prism_describe.rs` (existing test file) | ~20,000 | Add RG-DESC-001..003 to existing file |
| beta3-remediation-delta-analysis.md §Issue 3 + §Issue 5 + §S-MCP-ENVELOPE-DESCRIBE-001 | ~4,000 | Reference for fix design |
| ADR-058 v2.44 §G (OQ-003 section only) | ~2,000 | Read §G only; skip other sections |
| **Total estimated** | **~94,000** | Well within one context window |

---

## Tasks

### Red Gate tests (to be written by test-writer BEFORE implementation — SAC-1)

All three tests are added to the EXISTING file `crates/prism-mcp/tests/mcp_prism_describe.rs`.
All non-trivial function bodies in stubs use `todo!()` until Green Gate phase.

- [ ] **RG-DESC-001**: `test_BC_2_10_012_prism_describe_total_results_equals_table_count`
  Set up a `PrismServer` (or call `handle_prism_describe` directly via the test infrastructure
  in `mcp_infrastructure.rs`) with a client configured for 3 tables
  (e.g., claroty_alerts, claroty_devices, claroty_audit_logs). Invoke `prism_describe` and
  assert `envelope._meta.total_results == 3`. Must FAIL before the `wrap()` fix lands.

- [ ] **RG-DESC-002**: `test_BC_2_10_012_virtual_fields_all_four_present_in_describe_columns`
  Invoke `prism_describe` for a client with at least one table. Assert that the `columns`
  array for the returned table includes entries for ALL of: `_sensor`, `_client`,
  `_source_table`, `_source_type`. Use `columns.iter().any(|c| c.name == "_client")` etc.
  Must FAIL before the OQ-003 block extension lands.

- [ ] **RG-DESC-003**: `test_BC_2_10_012_wire_shape_total_results_in_json`
  Invoke `prism_describe` for a client with N > 0 tables. Serialize the returned
  `ResponseEnvelope` to JSON via `serde_json::to_string()`. Parse back and assert
  `json["_meta"]["total_results"] == N` (SID-2: wire-shape assertion on serialized bytes).
  Must FAIL before the `wrap()` fix lands.

- [ ] **Regression confirmation**: Verify that existing tests in `mcp_prism_describe.rs`
  remain GREEN (none should be broken by the Red Gate additions). In particular confirm
  that `test_BC_2_10_012_prism_describe_happy_path_catalog` and the `build_example_with_note`
  tests (inline in `prism_describe.rs`) remain GREEN with the stubs in place.

### Implementation tasks (to be executed by implementer after Red Gate)

#### Phase A — Fix wrap() total_results counting (Issue 3)

- [ ] **T-A01**: In `crates/prism-mcp/src/safety_envelope.rs`, extend the `total_results`
  computation in `wrap()` to add a third arm for `prism_describe`'s `{tables: [...]}` shape:
  ```rust
  // For prism_describe payloads of shape {client_id, tables: [...], pql_hints}: length of tables.
  // BC-2.10.012 §Response envelope (amended): total_results = tables.len().
  } else if let Some(tables_arr) = results.get("tables").and_then(|v| v.as_array()) {
      tables_arr.len() as u64
  } else {
      0
  };
  ```
  The new arm must be inserted BETWEEN the existing `rows` arm and the `else { 0 }` fallback.
  Verify no change to the `scan_target` block (scan_target remains None for prism_describe;
  DI-006 — no injection scan for server-authored catalog).

- [ ] **T-A02**: Run `cargo test -p prism-mcp test_BC_2_10_012_prism_describe_total_results_equals_table_count`
  — must be GREEN after T-A01.

- [ ] **T-A03**: Run `cargo test -p prism-mcp test_BC_2_10_012_wire_shape_total_results_in_json`
  — must be GREEN after T-A01.

- [ ] **T-A04**: Run existing `cargo test -p prism-mcp` (no new features flag). All existing
  tests must remain GREEN — specifically tests for query tool `rows` shape
  (`bc_2_09_001_test.rs`, `bc_2_09_008_test.rs`) must not be affected.

#### Phase B — Add missing virtual field column descriptors (Issue 5)

- [ ] **T-B01**: In `crates/prism-mcp/src/tools/prism_describe.rs`, in the function
  `build_column_descriptors_ocsf()`, extend the OQ-003 synthesized-column append block to
  add three new `ColumnDescriptor` entries AFTER the existing `_sensor` entry:
  ```rust
  // OQ-003 extension (beta.3 S-MCP-ENVELOPE-DESCRIBE-001): _client, _source_table, _source_type
  // complete the four-field virtual set per BC-2.11.012 §Invariants. These fields are injected
  // by inject_virtual_fields at query time; prism_describe must advertise them for LLM discovery.
  // BC-2.10.012 §Response shape (amended): five synthesized columns total.
  descriptors.push(ColumnDescriptor {
      name: "_client".to_string(),
      col_type: ColumnType::String,
      nullable: false,
      description: None,
  });
  descriptors.push(ColumnDescriptor {
      name: "_source_table".to_string(),
      col_type: ColumnType::String,
      nullable: false,
      description: None,
  });
  descriptors.push(ColumnDescriptor {
      name: "_source_type".to_string(),
      col_type: ColumnType::String,
      nullable: false,
      description: None,
  });
  ```
  Use the same pattern as the existing `_sensor` push (lines around `OQ-003 (AC-015)`
  comment in `build_column_descriptors_ocsf`). Verify `nullable: false` matches the
  populate-path injection (BC-2.11.012 §Invariants empty-MemTable schema parity note).

- [ ] **T-B02**: Run `cargo test -p prism-mcp test_BC_2_10_012_virtual_fields_all_four_present_in_describe_columns`
  — must be GREEN after T-B01.

- [ ] **T-B03**: Run all inline `#[cfg(test)]` tests in `prism_describe.rs` to confirm
  `build_example_with_note` priority ladder is unchanged. Specifically:
  - Existing aggregate tests that pick an Integer/Float column must still pick the same column
  - No test should produce a query on `_client`, `_source_table`, or `_source_type` as
    an aggregate GROUP BY target (they are String-typed)

- [ ] **T-B04**: If any existing test in `mcp_prism_describe.rs` asserts a specific column
  count (e.g., `columns.len() == 5`), update it to the new count (5 + 3 = 8 for a two-Tier-1
  table that previously had 2 spec columns + class_uid + _sensor). Grep the test file for
  `columns.len()` and `assert_eq!.*columns` before committing.

#### Phase C — Final verification

- [ ] **T-C01**: Run `cargo test -p prism-mcp --no-fail-fast`. All tests must pass including
  RG-DESC-001..003 (GREEN) and all pre-existing tests (no regression).

- [ ] **T-C02**: Run `just check` (full workspace). Must exit 0. The `safety_envelope.rs`
  change and `prism_describe.rs` change are both single-crate; no cross-crate compilation
  breakage expected.

#### Phase D — CHANGELOG

- [ ] **T-D01** (BEFORE creating the PR): Add a CHANGELOG entry under `[Unreleased] > Fixed`:
  ```markdown
  - Fix `prism_describe` response: `_meta.total_results` now equals the count of tables
    returned (was always 0 due to `safety_envelope::wrap()` missing a `tables`-key arm
    for the prism_describe object shape). Resolves beta.2 live-test issue 3.
  - Fix `prism_describe` column list: all four virtual fields (`_sensor`, `_client`,
    `_source_table`, `_source_type`) are now advertised as synthesized ColumnDescriptors
    per BC-2.11.012 §Invariants. Previously only `class_uid` and `_sensor` were appended
    (OQ-003 block). LLM agents can now discover and filter on `_client`, `_source_table`,
    `_source_type`. Resolves beta.2 live-test issue 5.
  ```

---

## Previous Story Intelligence

**S-MCP-TOOL-GATE-001** is the Wave 1 story in the same epic (E-BETA3-REMEDIATION). It gates
40 operations stubs behind a default-off Cargo feature and is recommended to merge before this
story to reduce merge-conflict risk on `server.rs`. However, this story does NOT touch `server.rs`
— its files are `safety_envelope.rs` and `tools/prism_describe.rs` — so parallelism is safe if
worktrees are used.

**Key context from the beta.2 live-test log (D-2520):**
- Both issues were identified against the Claroty xDome tenant (jea-readapi) during the Monroe demo.
- Issue 3 (total_results: 0) was visible in the MCP trace as `"0 results found"` even for a
  client with 9 Claroty tables.
- Issue 5 (missing virtual fields) caused the LLM agent to write queries without `_client` or
  `_source_type` filters because the column catalog did not advertise them.
- Both fixes are low-risk additive changes; no existing behavioral contracts are violated.

**From mcp_prism_describe.rs history:**
- The test file has been hardened multiple times (round-1, round-2 wiring corrections per the
  file header). New RG-DESC-001..003 tests must follow the same pattern: drive the REAL
  production path (not leaf renderers directly). Use `mcp_infrastructure.rs` helpers or call
  `handle_prism_describe` with a properly wired `query_engine` / `config_manager`.

---

## Architecture Compliance Rules

1. **Do NOT add a `tables` arm to scan_target.** The injection scanner `scan_target` block
   in `wrap()` has two arms: bare array and `{rows: [...]}`. For `prism_describe`, scan_target
   remains None — the schema catalog is server-authored (operator TOML specs via
   resolved_spec_map / config_manager), not sensor data. DI-006 explicitly states no
   injection scan is required for the discovery response. Only the `total_results` counting
   block receives the new `tables` arm.

2. **Append synthesized columns in exact order.** Per BC-2.10.012 §column ordering guidance
   (confirmed/unconfirmed first) and OQ-003 extension (five synthesized columns appended last
   in the order: class_uid, _sensor, _client, _source_table, _source_type). Do not reorder
   existing class_uid or _sensor entries.

3. **nullable: false for virtual field descriptors.** BC-2.11.012 §Invariants empty-MemTable
   schema parity note specifies nullable=true on the empty-table pre-registration path
   (append_virtual_fields_to_schema). The prism_describe ColumnDescriptor is the schema
   ADVERTISEMENT surface — it should reflect the populated-path behavior (nullable=false).
   This matches the existing `_sensor` ColumnDescriptor in the OQ-003 block.

4. **ColumnType::String for _client, _source_table, _source_type.** Per BC-2.11.012
   §Postconditions: virtual fields are string-typed. ColumnType::String is the canonical
   sensor schema API (prism_core::column::ColumnType per CLAUDE.md conventions). Do not
   use the retired prism_spec_engine::types::ColumnType variants.

5. **No `--no-verify` hook bypass.** Per CLAUDE.md non-negotiable git rules. If a pre-commit
   hook fails, fix the root cause.

6. **No volatile line-number cites in code comments.** Per TD-VSDD-091 / TD-VSDD-092: code
   comments added by this story must reference function names and BC section anchors, not
   `safety_envelope.rs:NNN` line numbers.

---

## Library & Framework Requirements

All versions pinned in workspace `Cargo.toml` — use workspace pins, not standalone versions.

| Dependency | Version | Note |
|-----------|---------|------|
| `serde_json` | workspace pin | `Value::get("tables").as_array()` in wrap() fix |
| `prism_core::column::ColumnType` | workspace pin | Use `ColumnType::String` / `ColumnType::Integer` — canonical variants only (CLAUDE.md §ColumnType canonical naming) |
| Rust toolchain | per `rust-toolchain.toml` | Stable channel; edition 2024 |

No new crate dependencies are introduced by this story.

---

## File Structure Requirements

### Files to MODIFY

| File | Change | Phase |
|------|--------|-------|
| `crates/prism-mcp/src/safety_envelope.rs` | Add `tables` arm in `wrap()` `total_results` block (between `rows` arm and `else { 0 }`) | Phase A |
| `crates/prism-mcp/src/tools/prism_describe.rs` | Add three `ColumnDescriptor` pushes after `_sensor` in `build_column_descriptors_ocsf()` OQ-003 block | Phase B |
| `crates/prism-mcp/tests/mcp_prism_describe.rs` | Add RG-DESC-001..003 tests; update any column-count assertions if needed | Red Gate + Phase B |
| `CHANGELOG.md` | Add [Unreleased] > Fixed entry (T-D01) | Phase D |

### Files NOT to touch

- `crates/prism-mcp/src/server.rs` — out of scope (Wave 1 story S-MCP-TOOL-GATE-001)
- `crates/prism-mcp/src/tools/operations.rs` — out of scope
- `crates/prism-query/` — virtual field injection is out of scope; this story only advertises them
- `crates/prism-core/src/virtual_fields.rs` — read-only reference; no modification
- `.factory/specs/behavioral-contracts/BC-2.10.012-*` — amendment is product-owner work; implementer does NOT modify BCs
- `.factory/specs/architecture/decisions/ADR-058-*` — amendment is architect work; implementer does NOT modify ADRs

### Forbidden Dependencies

- `prism_spec_engine::types::ColumnType` — RETIRED shadow enum per ADR-024 (CLAUDE.md §Forbidden patterns). Use `prism_core::column::ColumnType` exclusively. If this module appears in `prism-mcp`'s import tree for ColumnType resolution, it is a bug to be fixed in-scope.

---

## Holdout Authoring Note

`behavioral_contracts: [BC-2.10.012, BC-2.11.012]` is non-empty. Per the story-level holdout
gate protocol (D-1715/D-1716, human-approved 2026-07-13), the product-owner must author 2–4
HIDDEN, SINGLE-USE holdout scenarios for this story at story-materialization time (the same
touchpoint as the remove-uncertainty pass). Holdout scenarios should exercise:
- A `prism_describe` call for a client with N > 0 tables, asserting `total_results == N` at
  the wire level
- The `columns` array for a returned table containing all four virtual field descriptors
- A `prism_query` call using `_client` or `_source_table` in a WHERE clause after `prism_describe`
  has advertised those columns (end-to-end discoverability scenario)

Holdout scenarios are stored in the holdout directory that test-writer/implementer never read.

---

## History

| Version | Date | Change |
|---------|------|--------|
| 1.1 | 2026-09-16 | F3 BC/ADR pin propagation (D-2544): BC-2.10.012 v1.9→v1.10; ADR-058 §G reference updated to v2.44. AMENDMENT PENDING annotations removed from frontmatter comment, §Authority, §Behavioral Contracts table, and §Token Budget. |
| 1.0 | 2026-09-16 | Initial story decomposition |
