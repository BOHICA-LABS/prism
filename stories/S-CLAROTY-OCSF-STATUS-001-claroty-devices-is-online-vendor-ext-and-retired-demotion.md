---
document_type: story
story_id: S-CLAROTY-OCSF-STATUS-001
title: "Fix Claroty devices.is_online OCSF vendor-extension mapping and devices.retired status_code demotion"
wave: 3
epic_id: E-BETA3-REMEDIATION
version: "1.1"
status: ready
producer: story-writer
phase: 3
priority: P0
points: 3
tdd_mode: strict
target_module: prism-spec-engine
subsystems: ["SS-16", "SS-02"]
# Subsystem anchor justifications:
#   SS-16 (Spec Engine) owns the sensor TOML parsing pipeline
#   (`crates/prism-spec-engine/src/spec_parser.rs`, `crates/prism-sensors/specs/claroty.sensor.toml`)
#   and `build_column_array` in `crates/prism-bin/src/spec_driven_adapter.rs`, which routes
#   TOML column declarations (including `ocsf_field`) to Arrow field names. Both TOML changes
#   (`retired` demotion and `is_online` vendor-extension mapping) are SS-16 config changes
#   that take effect inside SS-16's spec-engine column-routing pipeline.
#
#   SS-02 (OCSF Normalization) is the behavioral owner of the OCSF field-path routing
#   decisions codified in ADR-058. The `is_online → device.is_online` vendor-extension
#   path and the `retired` demotion to `raw_extensions` are both OCSF-normalization
#   decisions per ADR-058 §K5. SS-02 is the semantic owner; SS-16 is the implementation
#   owner. Both subsystems are correctly listed because the story spans both the
#   normalization decision layer (SS-02) and the spec-engine execution layer (SS-16).
crates_touched:
  - prism-sensors
  - prism-bin
  - prism-dtu-claroty
estimated_days: 0.5
depends_on: []
# depends_on anchor justification:
#   No hard compile-time product-story dependencies. Both TOML changes are
#   pure-config — no new Rust production code is required. The existing
#   `ColumnType::Boolean` arm in `build_column_array` already handles boolean
#   passthrough; adding `ocsf_field = "device.is_online"` to the TOML routes
#   `is_online` to a Tier-1 `device_is_online: Boolean` Arrow column via the
#   existing OCSF-routing pipeline with no code changes (ADR-058 §K5 D-2522
#   Option A, ratified 2026-09-16).
#
#   Spec pre-work (ADR-058 §K5 D-2522 Option A ratified; BC-2.16.003 v1.28+
#   amended for ECs 034..040) is complete per Batch-0 gate. Story is unblocked
#   for test-writer dispatch.
#
#   Soft ordering: merge this story BEFORE S-CLAROTY-OCSF-REMEDIATION-001
#   (issues 8, 11, 12, 13) to avoid conflicting TOML changes in
#   `claroty.sensor.toml`. Both stories touch that file; serial merge ordering
#   prevents diffs from conflicting in the `[devices]` table block.
blocks:
  - S-BETA3-RELEASE-001
  - S-CLAROTY-OCSF-REMEDIATION-001
# blocks anchor justifications:
#   S-BETA3-RELEASE-001: this story is a W3 fix; all W3 stories must be merged to
#   develop before S-BETA3-RELEASE-001 can dispatch the beta.3 release bundle.
#
#   S-CLAROTY-OCSF-REMEDIATION-001: that story covers issues 8, 11, 12, 13 for
#   the Claroty OCSF remediation wave and also touches `claroty.sensor.toml`
#   devices table. This story's `retired` demotion and `is_online` mapping must
#   land first so REMEDIATION-001 does not need to re-work those columns.
behavioral_contracts:
  - BC-2.16.003
# BC status: ACTIVE v1.34 (per BC-INDEX current pin).
#   BC-2.16.003 v1.28 added EC-016-013-034..039 anchored to S-CLAROTY-OCSF-STATUS-001
#   RG-COS-001..006 and RG-COS-008; BC-2.16.003 v1.29 added EC-016-013-040 anchored to
#   RG-COS-007 and restructured EC-016-013-039 for canonical RG-COS-006/RG-COS-008
#   separation. All 8 RG-COS anchors are present in BC-2.16.003 v1.34.
#   v1.34 adds EC-016-013-042..045 (unrelated to this story); EC-016-013-034..040 semantics unchanged.
#   Spec-First Gate S-7.01 satisfied: behavioral_contracts is non-empty with canonical
#   BC-S.SS.NNN pattern.
verification_properties: []
assumption_validations: []
risk_mitigations: []
---

# S-CLAROTY-OCSF-STATUS-001: Fix Claroty devices.is_online OCSF Vendor-Extension Mapping and devices.retired Status-Code Demotion

## Authority

**ADR-058 v2.44 §K5 D-2522 Amendment Notes** is the primary authoritative source for
this story's decisions and mandate anchors. Read §K5 "§K5 D-2522 Option A (ratified
2026-09-16): `is_online` Vendor-Extension Tier-1 Boolean Column" and "§K5 D-2522
Amendment Notes — `retired` Demotion to raw_extensions" in full before implementing.
Path: `.factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md`

**BC-2.16.003 v1.34** (Column-to-OCSF Mapping at Query Time — §Postconditions EC-016-013-034
through EC-016-013-040) governs the full behavioral contract for this story. The 8
EC clauses covering `is_online` Boolean passthrough, `retired` raw_extensions demotion,
spec-validity, prism_describe shape, WHERE predicate, and SAP-2 DTU parity are the
authoritative acceptance criteria.
Path: `.factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md`

**beta3-remediation-delta-analysis.md §Issue 9 and §Issue 10** provide the defect
description, architect adjudication, and design rationale.
Path: `.factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md`

> NOTE: ADR-058 §K5 and BC-2.16.003 are FROZEN per the Batch-0 adversarial gate
> (BC-5.39.001 3-CLEAN passed). These spec files MUST NOT be amended by the
> implementer — any spec discrepancy routes to product-owner/architect via the
> orchestrator.

> CRITICAL NOTE — RESCINDED v2.35 DESIGN: ADR-058 v2.35 described a
> `BooleanToOcsfStatusCoercion` normalizer producing dual `status: Utf8` +
> `status_id: Int32` columns from `is_online`. That design was RESCINDED at v2.36
> (D-2522-CORRECTION) after confirming OCSF v1.7.0 `inventory_info.status_id` is
> an EVENT outcome field, not device operational state. The frozen spec (v2.37+) is
> vendor-extension Boolean only: ONE Arrow column `device_is_online: Boolean`, direct
> passthrough, NO coercion. Do NOT implement the rescinded v2.35 design.

---

## Problem Statement

During the beta.2 Monroe demo live-test against the Claroty xDome tenant (jea-readapi),
two defects were observed in the `claroty_devices` table output (D-2520 Issues 9 and 10):

### Issue 9 — devices.retired: boolean written to string status_code field

`claroty_devices` declared `retired` column with `ocsf_field = "status_code"`. OCSF
`inventory_info.status_code` is a free-text string (`string_t`). Writing boolean
`true`/`false` to a string field caused type-contract violations and confusion at
the MCP consumer layer. The live test confirmed runtime failures from this mismatch.

**Fix (ADR-058 §K5 D-2522):** Remove `ocsf_field = "status_code"` from the `retired`
column in `claroty.sensor.toml`. The `retired` value routes to `raw_extensions` under
its native key, preserving the boolean value without type conflict.

### Issue 10 — devices.is_online: no OCSF mapping despite clear semantic meaning

`is_online` (bool) had no `ocsf_field` assignment — it was Tier-2 (raw_extensions only).
OCSF v1.7.0 `device` object has no native `is_online`, `state_id`, or operational
connectivity field. The correct approach is a vendor extension.

**Fix (ADR-058 §K5 D-2522 Option A, ratified 2026-09-16):** Add
`ocsf_field = "device.is_online"` to the `is_online` column in `claroty.sensor.toml`.
This is a vendor-extended path (not in OCSF v1.7.0 schema) consistent with the
`device.type_category` / `device.type_label` precedent at ADR-058 §J3. The spec-engine
emits `device_is_online: Boolean (nullable)` as a first-class Arrow column via the
existing `ColumnType::Boolean` arm in `build_column_array`. ONE Arrow column. No
coercion normalizer step. No `ocsf_status_id_field` TOML key.

### Implementation Mechanism (PURE-TOML)

Both fixes are **pure-TOML config changes** to `crates/prism-sensors/specs/claroty.sensor.toml`:

1. On `retired` column: remove `ocsf_field = "status_code"` → column becomes Tier-2
2. On `is_online` column: add `ocsf_field = "device.is_online"` → column becomes Tier-1

No new Rust production code is required. The existing `ColumnType::Boolean` arm in
`build_column_array` (`crates/prism-bin/src/spec_driven_adapter.rs`) already handles
boolean extraction. The existing OCSF routing pipeline already calls
`ocsf_field_to_arrow_name("device.is_online")` → `"device_is_online"` for Tier-1 columns.

The 8 Red Gate tests verify the behavioral effect of these TOML changes, including
DTU fixture seeding (F11 obligation) to exercise null/absent `is_online` states.

---

## Narrative

As an LLM analyst agent querying `claroty_devices` via `prism_query`, I want
`device_is_online` to be a first-class queryable Boolean column (enabling
`WHERE device_is_online = true/false`) and `retired` to route cleanly to `raw_extensions`
(no type mismatch), so that device operational state is queryable and the type-contract
violation in beta.2 is resolved.

---

## Behavioral Contracts

| BC | Title | Version at Authoring | Scope in This Story |
|----|-------|---------------------|---------------------|
| BC-2.16.003 | Column-to-OCSF Mapping at Query Time — Map Sensor Columns to OCSF Fields Per Spec | v1.34 | §Postconditions EC-016-013-034..040: `is_online` Boolean passthrough (RG-COS-001..003), `retired` raw_extensions demotion (RG-COS-004), prism_describe shape (RG-COS-005), TOML spec-validity (RG-COS-006), WHERE predicate (RG-COS-007), SAP-2 DTU parity (RG-COS-008) |

---

## Acceptance Criteria

### AC-001 — is_online=true produces device_is_online=true in Arrow RecordBatch

When a DTU fixture device has `"is_online": true`, the spec-engine pipeline MUST produce
a `device_is_online: Boolean` Arrow column with value `true` in the corresponding row.
The `device_is_online` column MUST appear in the top-level Arrow schema as a first-class
column (NOT inside `raw_extensions`). The assertion MUST be on the Arrow RecordBatch
column schema and row value (SID-2 wire-shape discipline applies: also assert the
serialized JSON row carries `"device_is_online": true`, not `"is_online": true`).

(traces to BC-2.16.003 EC-016-013-034 postcondition: `is_online=true` →
`device_is_online=true` in Arrow RecordBatch; ADR-058 §K5 D-2522 Option A mandate table
row 1)

### AC-002 — is_online=false produces device_is_online=false in Arrow RecordBatch

When a DTU fixture device has `"is_online": false`, the spec-engine pipeline MUST produce
`device_is_online: Boolean` Arrow column value `false` in the corresponding row. Same
first-class column requirement as AC-001. Wire-shape assertion: serialized JSON row carries
`"device_is_online": false`.

(traces to BC-2.16.003 EC-016-013-035 postcondition: `is_online=false` →
`device_is_online=false` in Arrow RecordBatch; ADR-058 §K5 D-2522 Option A mandate table
row 2)

### AC-003 — is_online=null or absent produces device_is_online=null Arrow cell

When a DTU fixture device has `"is_online": null` (JSON null) or the `"is_online"` key
is absent from the JSON object, the spec-engine pipeline MUST produce a null Arrow cell
in the `device_is_online: Boolean (nullable)` column for that row. The null MUST be a
proper Arrow null cell, NOT a literal `"null"` string (BC-2.11.001 EC-11-079 null-not-
absent discipline). Wire-shape assertion: the serialized JSON row MUST NOT contain the
key `"device_is_online"` OR MUST contain `"device_is_online": null` (absent-key or null,
never the string `"null"`).

(traces to BC-2.16.003 EC-016-013-036 postcondition: `is_online=null/absent` →
`device_is_online=null` Arrow null cell; ADR-058 §K5 D-2522 Option A mandate table row 3)

### AC-004 — retired column demoted to raw_extensions; no status_code Arrow column emitted

The `retired` column MUST have `ocsf_field = "status_code"` removed from
`claroty.sensor.toml`. After this change, `retired` MUST route to `raw_extensions` as
a boolean value under its native key `"retired"`. No first-class `status_code` Arrow
column MUST be emitted with `retired` as its source. The `raw_extensions` blob for a
device with `retired = true` MUST contain `"retired": true` (or equivalent per
`raw_extensions` serialization rules). Wire-shape assertion: serialized JSON row carries
`"retired"` inside the `raw_extensions` JSON string, NOT as a top-level `"status_code"` key.

(traces to BC-2.16.003 EC-016-013-037 postcondition: `retired` demoted to `raw_extensions`;
`ocsf_field = "status_code"` removed; ADR-058 §K5 D-2522 mandate table row 4)

### AC-005 — prism_describe reports device_is_online: Boolean for Claroty devices table

A `prism_describe` call with a client spec that includes the `claroty_devices` table MUST
return a column descriptor for `device_is_online` with `column_type = "Boolean"` (or
equivalent display string) in the columns array for that table. The column MUST NOT be
absent from the describe response. The `retired` column MUST NOT appear as `status_code`
in the describe response. Wire-shape assertion (SID-2): serialize the describe response
to JSON; assert `"device_is_online"` key present in the columns array with Boolean type.

(traces to BC-2.16.003 EC-016-013-038 postcondition: `prism_describe` MUST include
`device_is_online: Boolean` in Claroty devices column schema; ADR-058 §K5 D-2522 Option A
mandate table row 5)

### AC-006 — TOML spec-validity: column_type = "boolean" + ocsf_field = "device.is_online" parses without ValidationError

When the spec-engine parses `claroty.sensor.toml` with the amended `is_online` entry
(`column_type = "boolean"` + `ocsf_field = "device.is_online"`), it MUST NOT raise a
`ValidationError` (which would block spec loading). It MAY emit a `ValidationWarning`
(non-blocking) for the unrecognized vendor-extension path per `validate_ocsf_field_path`
behavior. The spec-load MUST succeed (return `Ok(...)` from `parse_and_validate_spec_toml`
or equivalent) with at most warnings, consistent with the `device.type_category` /
`device.type_label` vendor-extension precedent (ADR-058 §J3).

(traces to BC-2.16.003 EC-016-013-039 TOML spec-validity clause: `ocsf_field = "device.is_online"`
on Boolean column MUST NOT raise ValidationError; MUST produce at most ValidationWarning;
ADR-058 §K5 D-2522 Option A mandate table row 6)

### AC-007 — WHERE device_is_online = true and = false filter correctly

A PrismQL query `FROM claroty_devices WHERE device_is_online = true` MUST execute without
error and return only rows where the `device_is_online` Arrow column is `true`. A query
`FROM claroty_devices WHERE device_is_online = false` MUST return only rows where
`device_is_online` is `false`. Rows with `device_is_online = null` MUST be excluded
from both WHERE predicates (SQL three-valued logic). The end-to-end test MUST exercise
this from the `prism_query` MCP tool surface (SAP-3 spec-arm reachability), not just
from a synthetic DataFusion AST.

(traces to BC-2.16.003 EC-016-013-040 postcondition: `WHERE device_is_online = true/false`
executes and filters correctly at query layer; ADR-058 §K5 D-2522 Option A mandate table
row 7)

### AC-008 — SAP-2 DTU parity: ClarotyDevice.is_online wire value maps to device_is_online Boolean column

`ClarotyDevice.is_online` in `crates/prism-dtu-claroty/src/types.rs` MUST be `Option<bool>`
(to support `true`, `false`, and `null`/absent wire states). The DTU route handler for
devices MUST emit `"is_online": true`, `"is_online": false`, and `"is_online": null` (or
absent key) in the fixture JSON. The spec-engine pipeline MUST convert these wire states to
`device_is_online: Boolean` Arrow cells `true`, `false`, and `null` respectively. This SAP-2
DTU parity check verifies the fixture wire shape matches the production TOML declaration
end-to-end. Wire-shape assertion: read DTU route fixture JSON directly; confirm `is_online`
field is present and has the expected value in each seeded device record.

(traces to BC-2.16.003 EC-016-013-039 SAP-2 DTU parity clause: `ClarotyDevice.is_online`
on DTU wire MUST match `device_is_online: Boolean` Arrow column after spec-engine pipeline;
ADR-058 §K5 D-2522 Option A mandate table row 8)

---

## Red Gate Test List (SAC-1 — BC-5.38.001)

All 8 failing Red Gate tests reside in a NEW file:
`crates/prism-bin/tests/bc_2_16_003_claroty_devices_status_tests.rs`

This file exercises the full pipeline from `claroty.sensor.toml` TOML declaration →
spec-engine parsing → `build_column_array` → Arrow RecordBatch → MCP wire output.
Following the pattern of `crates/prism-bin/tests/bc_2_16_015_claroty_vulnerabilities_wire_shape.rs`.

- **RG-COS-001**: `test_cos_rg001_is_online_true_produces_device_is_online_true`
  Assert: a DTU-backed `claroty_devices` query where the DTU fixture has `"is_online": true`
  produces an Arrow RecordBatch with column `device_is_online: Boolean` containing `true`.
  Also assert serialized JSON row carries `"device_is_online": true` (SID-2).
  Currently FAILS: `is_online` has no `ocsf_field` mapping, routes to `raw_extensions` as
  `"is_online": true` — no first-class `device_is_online` column exists. Covers AC-001.

- **RG-COS-002**: `test_cos_rg002_is_online_false_produces_device_is_online_false`
  Assert: DTU fixture with `"is_online": false` → `device_is_online: Boolean` = `false`.
  Wire-shape assertion: `"device_is_online": false` in serialized row JSON.
  Currently FAILS: same root cause as RG-COS-001. Covers AC-002.

- **RG-COS-003**: `test_cos_rg003_is_online_null_produces_device_is_online_null`
  Assert: DTU fixture with `"is_online": null` (requires `Option<bool>` in `ClarotyDevice`)
  → `device_is_online: Boolean` Arrow null cell. Wire-shape assertion: serialized JSON row
  has `"device_is_online": null` (not the string `"null"`, not absent — explicit JSON null
  per BC-2.11.001 EC-11-079).
  Currently FAILS: `is_online: bool` (not `Option<bool>`) cannot represent null; also no
  `ocsf_field` mapping. Covers AC-003.

- **RG-COS-004**: `test_cos_rg004_retired_demoted_to_raw_extensions`
  Assert: after the TOML fix, a `claroty_devices` query produces a row where `retired = true`
  appears inside `raw_extensions` (e.g., `"retired": true` in the `raw_extensions` JSON blob).
  Assert NO top-level `"status_code"` Arrow column exists with the boolean `retired` value.
  Wire-shape assertion: serialize the row; assert `"status_code"` key is NOT a top-level
  column emitted from the `retired` Boolean source; assert `"retired"` key appears in
  `raw_extensions` blob.
  Currently FAILS: `retired` has `ocsf_field = "status_code"`, producing a type-mismatch
  (boolean written to string field). Covers AC-004.

- **RG-COS-005**: `test_cos_rg005_prism_describe_reports_device_is_online_boolean`
  Assert: `prism_describe` response for `claroty_devices` table includes a column descriptor
  with `name = "device_is_online"` and Boolean type. Serialize the describe response to JSON;
  assert `"device_is_online"` is present in the columns array (SID-2 wire-shape discipline).
  Currently FAILS: `is_online` has no `ocsf_field`, so `device_is_online` is never emitted as
  a Tier-1 Arrow column and is absent from `prism_describe` schema. Covers AC-005.

- **RG-COS-006**: `test_cos_rg006_toml_spec_validity_no_validation_error`
  Assert: loading the amended `claroty.sensor.toml` (with `ocsf_field = "device.is_online"`
  on the `is_online` column) via `parse_and_validate_spec_toml` (or equivalent spec-engine
  entry point) returns `Ok(...)` or a result with at most `ValidationWarning` entries and
  zero `ValidationError` entries. The spec-load MUST succeed.
  Currently FAILS (by definition — the TOML amendment is what this story implements; without
  the change, `is_online` has no `ocsf_field` and the vendor-extension path is not exercised).
  Covers AC-006.

- **RG-COS-007**: `test_cos_rg007_where_device_is_online_true_predicate`
  Assert: PrismQL `FROM claroty_devices WHERE device_is_online = true` (exercised via
  `prism_query` MCP tool surface — SAP-3 reachability, not synthetic AST) returns only
  rows where the DTU fixture has `"is_online": true`. Assert rows with `"is_online": false`
  are excluded. Assert the query executes without error (no `E-QUERY-NNN` raised for a
  valid Boolean column predicate).
  Currently FAILS: `device_is_online` column does not exist; query would return
  E-QUERY-NNN (column not found) or equivalent DataFusion error. Covers AC-007.

- **RG-COS-008**: `test_cos_rg008_sap2_dtu_parity_is_online_option_bool_wire`
  Assert SAP-2 DTU parity: read `crates/prism-dtu-claroty/src/types.rs` to confirm
  `ClarotyDevice.is_online` is `Option<bool>`. Verify the DTU `fixtures/devices.json`
  contains at least one device with `"is_online": true`, one with `"is_online": false`,
  and one with `"is_online": null` (or a route handler that emits these states). Assert
  the spec-engine pipeline converts these three wire states to `device_is_online: Boolean`
  Arrow cells `true`, `false`, and `null` respectively (combining with RG-COS-001/002/003
  coverage at the parity level).
  Currently FAILS: `ClarotyDevice.is_online: bool` (not `Option<bool>`); null/absent state
  not representable; `device_is_online` column not in Arrow output. Covers AC-008.

**BC-5.38.001 density check:** 8 failing Red Gate tests / 8 ACs = **1.00** — satisfies
the ≥ 0.5 threshold.

**Red-then-green task ordering:** RG-COS-001..008 MUST all be written and confirmed RED
by the test-writer BEFORE the implementer begins any TOML or DTU changes. See §Tasks.

---

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| Claroty sensor TOML spec (TOML config change — primary fix) | `crates/prism-sensors/specs/claroty.sensor.toml` — `devices` table `retired` and `is_online` columns | Pure (config data) |
| Boolean column routing in spec-engine | `crates/prism-bin/src/spec_driven_adapter.rs` — `build_column_array` `ColumnType::Boolean` arm | Pure (data transformation; no code change required) |
| DTU types (F11 — struct change for null/absent) | `crates/prism-dtu-claroty/src/types.rs` — `ClarotyDevice.is_online: bool` → `Option<bool>` | Pure (data definition) |
| DTU devices fixture (F11 — seed fixture states) | `crates/prism-dtu-claroty/fixtures/devices.json` — add devices with `is_online: true`, `false`, `null` | Pure (fixture data) |
| DTU devices route (F11 — fixture loading path) | `crates/prism-dtu-claroty/src/routes/devices.rs` — verify `load_devices_fixture()` + serde handles `Option<bool>` | Effectful (HTTP handler) |
| Red Gate integration tests | `crates/prism-bin/tests/bc_2_16_003_claroty_devices_status_tests.rs` (NEW) | Pure (tests) |

> NOTE: The `ColumnType::Boolean` arm in `build_column_array` handles boolean extraction
> unchanged. NO new production code is required in `spec_driven_adapter.rs`. The TOML
> `ocsf_field = "device.is_online"` addition routes `is_online` through the existing
> OCSF-routing pipeline. `ocsf_field_to_arrow_name("device.is_online")` → `"device_is_online"`
> via the underscore-flattening convention (ADR-058 §C). This is confirmed clean: no shadow
> collision with any of the 20 devices `col.name` values (ADR-058 §K5 §J3 shadow-check,
> verified 2026-09-16).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | DTU emits `"is_online": null` (JSON null) for a device | `device_is_online` Arrow null cell; wire JSON has `"device_is_online": null`; NOT the string `"null"` (BC-2.11.001 EC-11-079) |
| EC-002 | DTU omits `"is_online"` key entirely for a device | Same as EC-001 — Arrow null cell per `Option<bool>` serde default None; `device_is_online` null in wire shape |
| EC-003 | `retired = true` device: `raw_extensions` must carry `"retired": true` | After demotion, `raw_extensions` JSON blob includes `"retired": true`; NO `status_code` Arrow column for this source |
| EC-004 | `retired = false` device: verify false value reaches raw_extensions cleanly | `raw_extensions` carries `"retired": false`; no type confusion from the prior boolean-to-string conversion |
| EC-005 | Shadow-name check: `device_is_online` vs. existing 20 devices `col.name` values | `device_is_online` does NOT match any of the 20 `col.name` values in the devices table (ADR-058 §J3 shadow-check CLEAN, verified 2026-09-16); no column collision |
| EC-006 | ValidationWarning (not ValidationError) for vendor-extended `device.is_online` path | `validate_ocsf_field_path("device.is_online")` returns a `ValidationWarning` (non-blocking) per the `device.type_category`/`device.type_label` precedent; spec load succeeds |
| EC-007 | `WHERE device_is_online = null` in PrismQL | NULL comparison in SQL three-valued logic: `WHERE device_is_online IS NULL` is the correct form; `= null` behavior per DataFusion defaults (not a gate contract for this story — edge behavior is DataFusion standard SQL semantics) |
| EC-008 | Existing `prism_describe` tests that assert on devices column count | Tests asserting exact column count for `claroty_devices` must be updated to include the new `device_is_online` column (column count increases by 1); `status_code` emitted from `retired` must NOT appear. |

---

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story file | ~14,000 | |
| BC-2.16.003 v1.34 — EC-016-013-034..040 + §Postconditions OCSF routing clauses | ~10,000 | Load §Postconditions + EC-016-013-034..040; skip unrelated EC rows |
| ADR-058 §K5 (§K5 D-2522 Amendment Notes + §K5 mandate table) | ~8,000 | Load §K5 only; skip earlier §A..§J sections |
| `crates/prism-sensors/specs/claroty.sensor.toml` (devices table) | ~12,000 | Load `[[claroty_devices]]` section only (~120 lines) |
| `crates/prism-bin/src/spec_driven_adapter.rs` (ColumnType::Boolean arm in build_column_array) | ~8,000 | Load `build_column_array` function only; full file is ~800 lines |
| `crates/prism-dtu-claroty/src/types.rs` (ClarotyDevice struct) | ~3,000 | Load `ClarotyDevice` struct definition (~50 lines) |
| `crates/prism-dtu-claroty/src/routes/devices.rs` (load_devices_fixture + handler) | ~6,000 | Load fixture loading path and route handler |
| `crates/prism-dtu-claroty/fixtures/devices.json` (current fixture) | ~4,000 | Skim structure to understand seeding requirements |
| `crates/prism-bin/tests/bc_2_16_015_claroty_vulnerabilities_wire_shape.rs` (test pattern reference) | ~8,000 | Skim for test setup / DTU fixture loading pattern |
| New test file `bc_2_16_003_claroty_devices_status_tests.rs` | ~10,000 | 8 test functions |
| delta-analysis.md §Issue 9 + §Issue 10 (design rationale) | ~4,000 | Reference for root cause and fix design |
| **Total estimated (full implementation)** | **~87,000** | Well within one context window |

**Phase-scoped loading recommendation:** The test-writer should load TOML + BC + ADR-058 §K5
to write the Red Gate tests. The implementer needs only the TOML file + `types.rs` + fixtures
to execute the PURE-TOML + DTU struct changes. Loading the full `spec_driven_adapter.rs` is
needed only to confirm the `ColumnType::Boolean` arm requires no changes (read-and-confirm
pattern, not write-and-modify).

---

## Tasks

### Red Gate tests (to be written by test-writer BEFORE implementation — SAC-1)

All 8 tests are in the NEW file:
`crates/prism-bin/tests/bc_2_16_003_claroty_devices_status_tests.rs`

All non-trivial function bodies in test scaffolding use assertions that will fail until
the TOML changes and DTU struct changes land. Tests must FAIL (or fail to compile) before
any fix is applied.

Tests follow the pattern in `crates/prism-bin/tests/bc_2_16_015_claroty_vulnerabilities_wire_shape.rs`
for DTU-backed pipeline integration tests. Each test loads the production
`claroty.sensor.toml` (via the spec-engine), starts a DTU process or loads fixture data,
runs the query pipeline, and asserts on Arrow RecordBatch column shape + serialized JSON
(SID-2).

- [ ] **RG-COS-001**: Write `test_cos_rg001_is_online_true_produces_device_is_online_true`.
  Assert: `device_is_online: Boolean = true` in Arrow RecordBatch for a device with
  `"is_online": true` in fixture. Wire-shape JSON assertion: `"device_is_online": true`
  in serialized row. Must FAIL before TOML fix (no `device_is_online` column exists).

- [ ] **RG-COS-002**: Write `test_cos_rg002_is_online_false_produces_device_is_online_false`.
  Assert: `device_is_online: Boolean = false` for fixture `"is_online": false`. Wire-shape
  assertion: `"device_is_online": false`. Must FAIL before TOML fix.

- [ ] **RG-COS-003**: Write `test_cos_rg003_is_online_null_produces_device_is_online_null`.
  Assert: `device_is_online` is Arrow null for fixture `"is_online": null`. Wire-shape
  assertion: `"device_is_online": null` in JSON (NOT string `"null"`; NOT absent key unless
  explicit_nulls behavior; verify per BC-2.11.001 EC-11-079 null-not-absent discipline).
  Must FAIL before both TOML fix AND DTU struct change (needs `Option<bool>`).

- [ ] **RG-COS-004**: Write `test_cos_rg004_retired_demoted_to_raw_extensions`.
  Assert: no top-level `"status_code"` Boolean-source column in Arrow schema for devices.
  Assert: `"retired"` key present inside `raw_extensions` JSON blob for a device with
  `retired = true`. Wire-shape assertion: serialize row; assert `raw_extensions` contains
  `"retired": true`; assert no top-level `"status_code"` key emitted from the `retired`
  Boolean source. Must FAIL before TOML fix (currently `status_code` exists from `retired`).

- [ ] **RG-COS-005**: Write `test_cos_rg005_prism_describe_reports_device_is_online_boolean`.
  Assert: `prism_describe` response for client with `claroty_devices` table includes
  column name `"device_is_online"` with Boolean type in columns array. Use
  `crates/prism-mcp/tests/mcp_prism_describe.rs` pattern for describe call setup.
  Wire-shape: serialize describe response; assert `"device_is_online"` present in column
  list. Must FAIL before TOML fix.

- [ ] **RG-COS-006**: Write `test_cos_rg006_toml_spec_validity_no_validation_error`.
  Assert: loading the amended `claroty.sensor.toml` via spec-engine parse function returns
  a result with zero `ValidationError` entries (at most `ValidationWarning`). Use the
  spec-engine TOML parsing API directly (unit-level test; no DTU process needed). Must
  FAIL in documentation (this test confirms the spec-validity of the fix itself; before
  the TOML fix, `ocsf_field = "device.is_online"` doesn't exist and this test's precondition
  is unexercised — add a negative test that WITH the old TOML the `retired → status_code`
  produces no ValidationError, and WITH the new TOML the `is_online → device.is_online`
  also produces no ValidationError, confirming both are spec-valid).

- [ ] **RG-COS-007**: Write `test_cos_rg007_where_device_is_online_true_predicate`.
  Assert: `prism_query(query="FROM claroty_devices WHERE device_is_online = true")` via
  MCP tool surface (SAP-3 reachability — PrismQL → pipe-SQL-emitter → DataFusion path).
  Verify only rows with `"is_online": true` in fixture are returned. Assert no
  `E-QUERY-NNN` column-not-found error. Must FAIL before TOML fix (`device_is_online`
  column absent; query returns error).

- [ ] **RG-COS-008**: Write `test_cos_rg008_sap2_dtu_parity_is_online_option_bool_wire`.
  SAP-2 parity check: read `crates/prism-dtu-claroty/src/types.rs` at compile time
  (or via a unit test that verifies serde deserialization). Assert `ClarotyDevice.is_online`
  is `Option<bool>` by verifying `serde_json::from_str::<ClarotyDevice>(r#"{"is_online": null, ...}"#)`
  succeeds (all other required fields filled). Assert the spec-engine pipeline converts
  `true` / `false` / `null` wire states to corresponding `device_is_online` Boolean Arrow
  cells. Must FAIL before DTU struct change (`bool` not `Option<bool>`;
  null deserialization panics).

- [ ] **Regression confirmation**: Existing tests in `crates/prism-spec-engine/tests/bc_2_16_003_test.rs`
  and `crates/prism-bin/tests/bc_2_16_015_claroty_vulnerabilities_wire_shape.rs` must
  remain GREEN after the TOML fix. Verify `just iter prism-spec-engine` and
  `just iter prism-bin` after fix is applied.

---

### Implementation tasks (to be executed by implementer AFTER Red Gate — SAC-1)

#### Phase F — F11 DTU Fixture Seeding (PREREQUISITE — execute before TOML fix)

The test infrastructure requires the Claroty DTU to emit `is_online` in multiple states
(true, false, null, absent). The current `ClarotyDevice.is_online: bool` cannot represent
null or absent. This DTU struct change must precede the TOML fix so that RG-COS-003 and
RG-COS-008 can exercise the null path.

- [ ] **T-F01** (F11 DTU FIXTURE — types.rs): In `crates/prism-dtu-claroty/src/types.rs`,
  change `ClarotyDevice.is_online` from `bool` to `Option<bool>`:
  ```rust
  // Before:
  pub is_online: bool,
  // After (F11 fixture obligation — S-CLAROTY-OCSF-STATUS-001):
  #[serde(skip_serializing_if = "Option::is_none")]
  pub is_online: Option<bool>,
  ```
  The `skip_serializing_if` ensures that `None` serializes as absent key (for RG-COS-003
  absent-key path). Use `#[serde(default)]` on the field if needed for deserialization
  of records that lack the key. Verify the `load_devices_fixture()` path in
  `crates/prism-dtu-claroty/src/routes/devices.rs` still compiles after this change.
  Run `cargo check -p prism-dtu-claroty` — must exit 0.

  SAP-2 obligation (AC-008): after this change, run `cargo test -p prism-dtu-claroty`
  to confirm no existing DTU tests break. The change from `bool` to `Option<bool>` is
  backward-compatible for `true`/`false` values; only `null`/absent are now representable.

- [ ] **T-F02** (F11 DTU FIXTURE — devices.json): Seed `crates/prism-dtu-claroty/fixtures/devices.json`
  with at least THREE additional fixture device entries covering the required states:
  1. A device entry with `"is_online": true` (if not already present)
  2. A device entry with `"is_online": false` (if not already present)
  3. A device entry with `"is_online": null` (explicit JSON null; exercises RG-COS-003)
  
  The absent-key state (`"is_online"` key missing entirely) is exercised by the null
  path since `Option<bool>` with `#[serde(default)]` produces `None` for both null and
  absent. If a distinct absent-key device entry is needed for test isolation, add one.
  
  Confirm the fixture file is valid JSON after edits. Verify the embedded fixture loads
  correctly: `cargo test -p prism-dtu-claroty -- --test fixture_loads`.

- [ ] **T-F03** (F11 DTU FIXTURE — route handler): In `crates/prism-dtu-claroty/src/routes/devices.rs`,
  verify `load_devices_fixture()` and the fixture-gen path both handle `Option<bool>`
  `is_online` correctly. The static fixture path loads from `fixtures/devices.json` via
  `include_str!("../../fixtures/devices.json")`. After T-F01 changes `is_online` to
  `Option<bool>`, confirm `serde_json::from_value::<Vec<ClarotyDevice>>(...)` still
  deserializes the fixture array without error. Run the DTU unit tests to confirm.

#### Phase T — TOML Configuration Changes (core fix)

- [ ] **T-T01** (TOML fix — retired demotion): In `crates/prism-sensors/specs/claroty.sensor.toml`,
  locate the `retired` column entry in the `[[claroty_devices.columns]]` section (current
  state: `name = "retired"`, `column_type = "boolean"`, `ocsf_field = "status_code"`).
  Remove the `ocsf_field = "status_code"` line. Update the comment to reflect the
  ADR-058 §K5 D-2522 demotion rationale:
  ```toml
  name = "retired"
  column_type = "boolean"
  # Tier-2: demoted to raw_extensions per ADR-058 §K5 D-2522 (beta.3).
  # inventory_info.status_code is string_t in OCSF v1.7.0; writing boolean
  # "true"/"false" caused type-mismatch at the MCP consumer layer (D-2520 Issue 9).
  # No superior OCSF boolean-retirement field exists; raw_extensions preserves
  # the boolean value without type conflict.
  # Anchor: S-CLAROTY-OCSF-STATUS-001 RG-COS-004.
  ```
  
  SAP-2 check: `ClarotyDevice.retired: bool` in `crates/prism-dtu-claroty/src/types.rs`
  is the wire source. No struct change needed — `retired` is already `bool` on the DTU
  wire and the raw_extensions path handles bool values. Confirm no DTU change is needed.

- [ ] **T-T02** (TOML fix — is_online vendor-extension mapping): In
  `crates/prism-sensors/specs/claroty.sensor.toml`, locate the `is_online` column entry
  in the `[[claroty_devices.columns]]` section (current state: `name = "is_online"`,
  `column_type = "boolean"`, no `ocsf_field`). Add `ocsf_field = "device.is_online"` and
  update the comment:
  ```toml
  name = "is_online"
  column_type = "boolean"
  ocsf_field = "device.is_online"   # vendor-extended; not in OCSF v1.7.0 schema
  # Tier-1 (vendor-extended): ocsf_field_to_arrow_name("device.is_online") → "device_is_online"
  # Arrow: device_is_online: Boolean (nullable). Direct boolean passthrough via
  # ColumnType::Boolean arm in build_column_array; no coercion step.
  # ValidationWarning (non-blocking) from validate_ocsf_field_path (vendor-ext path).
  # Consistent with device.type_category / device.type_label precedent (ADR-058 §J3).
  # D-2522 Option A ratified 2026-09-16. Shadow: device_is_online not in 20 col.names.
  # Anchor: S-CLAROTY-OCSF-STATUS-001 RG-COS-001..003, RG-COS-005..008.
  ```
  
  Confirm no code change is needed in `build_column_array` — the `ColumnType::Boolean`
  arm handles this passthrough automatically once the TOML route is established.

- [ ] **T-T03** (Verify spec-engine parses amended TOML): After T-T01 + T-T02, run
  `cargo test -p prism-spec-engine -- bc_2_16_003` to confirm the amended TOML parses
  without ValidationError. Also run `just iter prism-spec-engine` to verify no existing
  spec-engine tests regress.

#### Phase V — Verification

- [ ] **T-V01**: Run `cargo nextest run -p prism-bin -E 'test(cos)' --no-fail-fast`.
  All 8 RG-COS tests must be GREEN. No existing tests must regress.

- [ ] **T-V02**: Run `just iter prism-bin`. All existing `bc_2_16_015_*` tests and
  `bc_2_01_013_*` tests must remain GREEN (no regression from TOML changes).

- [ ] **T-V03**: Run `just iter prism-dtu-claroty`. Confirm DTU struct change
  (`Option<bool>` for `is_online`) did not break any existing DTU tests.

- [ ] **T-V04**: Run `just check` (full workspace). Must exit 0. Blast radius is
  LOW-MEDIUM: the DTU struct change may require serde default annotation updates; the
  TOML change only affects Claroty devices table routing. Full `just check` is required
  before declaring done.

#### Phase C — CHANGELOG

- [ ] **T-C01** (BEFORE creating the PR): Add a CHANGELOG entry under `[Unreleased] > Fixed`:
  ```markdown
  - Fix Claroty `devices` table OCSF mapping: `is_online` is now a first-class queryable
    Boolean column (`device_is_online: Boolean`) via vendor-extended OCSF path
    `device.is_online` (ADR-058 §K5 D-2522 Option A). Resolves beta.2 live-test issue 10
    (no OCSF mapping for `is_online`).
  - Fix Claroty `devices` table: `retired` column demoted from `ocsf_field = "status_code"`
    to `raw_extensions` (ADR-058 §K5 D-2522). Resolves beta.2 live-test issue 9
    (type-mismatch: boolean `true`/`false` written to string `status_code` field).
  ```

---

## Previous Story Intelligence

**S-ADR058-OCSF-ROUTING-001** (merged) established the OCSF routing pipeline this story
depends on — `ocsf_column_naming = true` at the sensor level, `ocsf_field_to_arrow_name`
conversion, Tier-1/Tier-2 routing, and the `raw_extensions` aggregation path. The
`ColumnType::Boolean` arm in `build_column_array` is part of this baseline. This story's
PURE-TOML fix works BECAUSE that routing pipeline is already in place.

**S-ADR058-OCSF-COERCION-001** (merged) added the `ColumnType::Integer` and `ColumnType::Float`
coercion path. The Boolean case (`ColumnType::Boolean` direct passthrough) predates this story.
Confirm the Boolean arm is not inadvertently sharing any coercion state.

**S-CLAROTY-VULNS-001** (merged) set the precedent for Claroty-specific TOML + test changes.
The test pattern in `crates/prism-bin/tests/bc_2_16_015_claroty_vulnerabilities_wire_shape.rs`
is the primary model for RG-COS-001..008 test structure. Read that file's DTU fixture loading
setup, pipeline invocation, and Arrow column assertion pattern before writing the new test file.

**ADR-058 §K5 D-2522 evolution (important for context):** v2.35 proposed `BooleanToOcsfStatusCoercion`
dual-emit (`status: Utf8` + `status_id: Int32`). This was RESCINDED at v2.36 (OCSF schema
investigation confirmed `inventory_info.status_id` is an event outcome field). The ratified
v2.37+ decision is vendor-extension Boolean only. Do NOT implement the v2.35 design.
The implementer must confirm they are reading ADR-058 §K5 at v2.37+ (current v2.44), not
a cached or earlier version.

**No FetchOutput blast radius:** unlike S-QUERY-TRUE-TOTAL-001 (46+ FetchOutput::new call
sites), this story has NO struct changes to cross-crate data types. The DTU struct change
(`ClarotyDevice.is_online: bool` → `Option<bool>`) is contained within `prism-dtu-claroty`
and its callers (DTU only; `ClarotyDevice` is not used outside the DTU crate in production
paths). Blast radius is LOW.

---

## Architecture Compliance Rules

1. **PURE-TOML for production behavior change.** The `is_online → device_is_online`
   promotion and `retired` demotion are achieved by TOML config changes only. If the
   implementer finds they need to modify `build_column_array` logic, stop and route to
   architect — the ADR-058 §K5 rationale explicitly states "direct boolean passthrough
   via `ColumnType::Boolean` arm; no coercion step."

2. **ONE Arrow column for `is_online` fix.** ADR-058 §K5 D-2522 Option A: ONE Arrow
   column `device_is_online: Boolean (nullable)`. Do NOT emit `status: Utf8` +
   `status_id: Int32` — those were the rescinded v2.35 columns. If any spec or code still
   references `BooleanToOcsfStatusCoercion` or `ocsf_status_id_field`, it is stale and
   must NOT be implemented.

3. **Vendor-extension produces ValidationWarning, NOT ValidationError.** The
   `validate_ocsf_field_path` function must return a non-blocking warning (not error)
   for `"device.is_online"` per the `device.type_category` / `device.type_label` precedent
   (ADR-058 §J3). If the spec-engine currently escalates unknown paths to ValidationError,
   this must be fixed in-scope before this story can pass RG-COS-006. Route to spec-engine
   owner if a code fix is required beyond the TOML-only scope.

4. **Shadow check: `device_is_online` must not collide.** Confirmed CLEAN: `device_is_online`
   does not match any of the 20 `col.name` values in the Claroty devices table (ADR-058 §J3
   shadow-check verified 2026-09-16). Do not proceed if a new devices column with name
   `device_is_online` is added to the TOML in a parallel story before this one merges.

5. **SAP-2 parity check before TOML commit.** Per SAP-2 (CLAUDE.md §Standing Adversary
   Probes): before committing the TOML change adding `ocsf_field = "device.is_online"`,
   confirm `ClarotyDevice.is_online` exists in `types.rs` AND is emitted on the DTU wire.
   After T-F01 changes `is_online` to `Option<bool>`, the SAP-2 check confirms `Option<bool>`
   wire values map to the correct Arrow Boolean column.

6. **`retired` comment must preserve audit trail.** The comment update in T-T01 must include
   the ADR-058 §K5 demotion rationale, beta.2 evidence reference (D-2520 Issue 9), and
   story anchor (S-CLAROTY-OCSF-STATUS-001 RG-COS-004). ADR-058 §K5 D-2522 Amendment Notes
   must be the source of truth — do not paraphrase from memory.

7. **Volatile line-number cites forbidden.** Per TD-VSDD-091/092: code comments must
   reference function names and ADR section anchors (e.g., `ADR-058 §K5 D-2522 Option A`),
   NOT `types.rs:74` line numbers.

8. **`#[non_exhaustive]` for DTU struct.** If `ClarotyDevice` carries `#[non_exhaustive]`,
   the `Option<bool>` field change is backward-compatible for external callers. Confirm
   `ClarotyDevice` has `#[non_exhaustive]` in `types.rs`. If it does NOT and `ClarotyDevice`
   is `pub`, adding a new field (changing `bool` → `Option<bool>` is a field-type change, not
   a new field) may be a semver concern. Verify with `cargo check -p prism-dtu-claroty` and
   `cargo test -p prism-dtu-claroty` before committing.

9. **No `--no-verify` hook bypass.** Per CLAUDE.md non-negotiable git rules. If the pre-commit
   hook fails, fix the root cause.

---

## Library & Framework Requirements

All versions pinned in workspace `Cargo.toml` — use workspace pins, not standalone versions.

| Dependency | Version | Note |
|-----------|---------|------|
| `serde` / `serde_json` | workspace pin | DTU struct `#[serde(skip_serializing_if)]` / `#[serde(default)]` annotations; fixture JSON loading |
| `arrow` | workspace pin | `BooleanArray`, `RecordBatch` types in Red Gate test assertions |
| Rust toolchain | per `rust-toolchain.toml` | Stable channel; edition 2024 |

No new crate dependencies are introduced by this story.

### Forbidden Dependencies

- **`prism_spec_engine::types::ColumnType`** (RETIRED shadow enum, ADR-024) — this story
  does not add new column type references, but if the test setup imports any ColumnType, use
  `prism_core::column::ColumnType::Boolean` (canonical) only.
- **`BooleanToOcsfStatusCoercion`** — this normalizer was proposed in ADR-058 v2.35 and
  RESCINDED at v2.36. If this symbol exists anywhere in the codebase, it is dead code and
  must NOT be referenced. If it does not exist, do not create it.
- **Dual-emit `status: Utf8` + `status_id: Int32`** — the rescinded v2.35 plan. Any code
  path that emits both `status` and `status_id` from the `is_online` boolean source is
  WRONG for this story. Only `device_is_online: Boolean` is emitted.

---

## File Structure Requirements

### Files to CREATE

| File | Purpose |
|------|---------|
| `crates/prism-bin/tests/bc_2_16_003_claroty_devices_status_tests.rs` | 8 Red Gate tests (RG-COS-001..008) |

### Files to MODIFY

| File | Change | Phase |
|------|--------|-------|
| `crates/prism-dtu-claroty/src/types.rs` | Change `ClarotyDevice.is_online: bool` → `Option<bool>` with appropriate serde annotations | Phase F (T-F01) |
| `crates/prism-dtu-claroty/fixtures/devices.json` | Add/update fixture devices with `is_online: true`, `false`, `null` states | Phase F (T-F02) |
| `crates/prism-dtu-claroty/src/routes/devices.rs` | Verify (and fix if needed) `load_devices_fixture()` handles `Option<bool>` is_online | Phase F (T-F03) |
| `crates/prism-sensors/specs/claroty.sensor.toml` | Remove `ocsf_field = "status_code"` from `retired` column; add `ocsf_field = "device.is_online"` to `is_online` column | Phase T (T-T01 + T-T02) |
| `CHANGELOG.md` | Add [Unreleased] > Fixed entries for both fixes | Phase C (T-C01) |

### Files NOT to touch

- `.factory/specs/behavioral-contracts/BC-2.16.003-*` — FROZEN at v1.34; implementer MUST NOT modify
- `.factory/specs/architecture/decisions/ADR-058-*` — FROZEN at v2.44; implementer MUST NOT modify
- `crates/prism-bin/src/spec_driven_adapter.rs` — no production code change needed; read-to-confirm only
- `crates/prism-spec-engine/src/column_mapping.rs` — no code change needed; TOML drives routing
- `crates/prism-mcp/src/tools/prism_describe.rs` — no change needed; prism_describe uses the Arrow schema from the query pipeline, which will pick up `device_is_online` automatically after the TOML fix
- Any other `crates/prism-dtu-*` files — scope is Claroty DTU only; do not touch crowdstrike/armis/cyberint DTU
- `crates/prism-sensors/specs/crowdstrike.sensor.toml` / `armis.sensor.toml` / `cyberint.sensor.toml` — out of scope

---

## Holdout Authoring Note

`behavioral_contracts: [BC-2.16.003]` is non-empty. Per the story-level holdout gate
protocol (D-1715/D-1716, human-approved 2026-07-13), the product-owner must author 2–4
HIDDEN, SINGLE-USE holdout scenarios for this story at story-materialization time (the
same touchpoint as the remove-uncertainty pass). Suggested holdout scenario themes
(DO NOT author here — product-owner owns this):
- A `prism_query` call against `claroty_devices` filtering `WHERE device_is_online = true`,
  verifying only online devices are returned (wire-level Boolean predicate check)
- A `prism_query` call that returns a device with `retired = true`, verifying `retired`
  appears in `raw_extensions` and NOT as a top-level `status_code` column
- A `prism_describe` call verifying `device_is_online: Boolean` in the column list for
  the `claroty_devices` table

Holdout scenarios are stored in the holdout directory that test-writer/implementer never read.

---

## History

| Version | Date | Change |
|---------|------|--------|
| 1.1 | 2026-09-17 | D-1110 remove-uncertainty pass: all DTU/spec-engine symbol claims CONFIRMED against live code (ClarotyDevice.is_online current type bool confirmed; load_devices_fixture at routes/devices.rs confirmed; fixtures/devices.json path confirmed; parse_and_validate_spec_toml, ocsf_field_to_arrow_name, validate_ocsf_field_path all confirmed; no corrections). Re-pinned BC-2.16.003 v1.32 → v1.34 throughout (6 live occurrences in frontmatter comment, §Authority, §Behavioral Contracts table, §Token Budget, Files-NOT-to-touch; §History rows left as historical). v1.34 adds EC-016-013-042..045 unrelated to this story; EC-016-013-034..040 semantics unchanged. |
| 1.0 | 2026-09-17 | Initial story decomposition from Batch-0 frozen spec (ADR-058 §K5 D-2522 Option A ratified v2.37+; BC-2.16.003 v1.28+ EC-016-013-034..040). 8 ACs per ADR-058 §K5 mandate table RG-COS-001..008; 8 Red Gate tests enumerated; F11 DTU fixture obligation captured (T-F01..T-F03 tasks; files: types.rs, fixtures/devices.json, routes/devices.rs). Issues 9+10 split from S-CLAROTY-OCSF-REMEDIATION-001 per architect recommendation (delta-analysis §Part 4 §Item 3). |
