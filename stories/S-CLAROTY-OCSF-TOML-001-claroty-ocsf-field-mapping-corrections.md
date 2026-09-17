---
document_type: story
story_id: S-CLAROTY-OCSF-TOML-001
title: "Claroty xDome OCSF Field-Mapping Corrections (Issues 8, 11, 12, 13)"
wave: 3
epic_id: E-BETA3-REMEDIATION
version: "1.0"
status: draft
producer: story-writer
phase: 3
priority: P0
points: 5
tdd_mode: strict
target_module: prism-sensors
subsystems:
  - SS-16  # SS-16 owns the Spec Engine (TOML spec parsing, ocsf_field routing, column
            # normalization) per ARCH-INDEX Subsystem Registry. Owns Issues 11, 12, 13
            # TOML surface and the ocsf_field_to_arrow_name promotion mechanism.
  - SS-07  # SS-07 owns Sensor Adapters (spec_driven_adapter.rs coercion path, DTU
            # integration layer) per ARCH-INDEX Subsystem Registry. Owns Issue 8
            # coercion path + Issue 13 DTU struct and fixture.
crates_touched:
  - prism-sensors       # claroty.sensor.toml: ocsf_field additions (Issues 11, 12),
                        # severity_id column + body_template (Issue 13)
  - prism-bin           # spec_driven_adapter.rs: build_column_array String arm (Issue 8)
  - prism-dtu-claroty   # types.rs ClarotyAlert + fixtures/alerts.json (Issue 13)
estimated_days: 3
depends_on: []
# depends_on justification: No hard compile-time dependency on sibling story
# S-CLAROTY-OCSF-STATUS-001 (issues 9/10 ADR-058 perimeter). Merge-ordering
# RECOMMENDATION only: if S-MCP-NULL-ENCODING-001 is not yet merged, prefer
# merging it first to avoid conflict risk on spec_driven_adapter.rs.
blocks:
  - S-BETA3-RELEASE-001
# blocks justification: S-BETA3-RELEASE-001 depends on S-CLAROTY-OCSF-TOML-001
# because all beta.3 Monroe demo OCSF-correctness defects for issues 8/11/12/13
# must be resolved before the release gating story can proceed to holdout evaluation.
risk: LOW
# Risk justification: TOML-only changes (Issues 11/12) are zero-risk to compiled
# code. Issue 8 code change is confined to a single match arm in build_column_array
# (<= 3 lines). Issue 13 DTU struct addition is additive (Option<u32> with
# serde(default) -- no breaking deserialization change).
behavioral_contracts:
  # BC status: BC-2.16.003 v1.32 ACTIVE -- covers integer->string coercion
  # (EC-016-013-004), Tier-1 OCSF field promotion mechanism, and
  # Column-to-OCSF routing. BC-2.02.005 v1.7 ACTIVE -- governs Claroty-specific
  # field mapping to OCSF (9 data sources including alerts severity_id).
  # BC-2.16.017 v1.2 ACTIVE -- governs device_vulnerability_relations table spec.
  # Story-specific RG anchors (RG-COT-001..009) to be embedded in BC-2.16.003
  # during PO spec-gate review. Spec-First Gate S-7.01: status remains draft
  # until story-specific anchors are confirmed in BCs.
  - BC-2.16.003  # Column-to-OCSF Mapping at Query Time (v1.32) -- primary mechanism
  - BC-2.02.005  # Claroty xDome Field Mapping to OCSF (v1.7) -- Claroty field anchor
  - BC-2.16.017  # Claroty device-vulnerability-relations Table (v1.2) -- Issue 12
verification_properties:
  - VP-003  # OCSF normalization fidelity -- exercises Tier-1 ocsf_field column path
  - VP-007  # Sensor TOML spec correctness -- TOML spec changes gated by VP-007
assumption_validations: []
risk_mitigations: []
---

# S-CLAROTY-OCSF-TOML-001: Claroty xDome OCSF Field-Mapping Corrections (Issues 8, 11, 12, 13)

## Authority

Source documents (read in this order for authoritative scope; do NOT rely on summaries):

| Document | Path | Authoritative For |
|----------|------|-------------------|
| Delta Analysis (Batch-0 frozen spec) | `.factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md` | §Issue 8, §Issue 11, §Issue 12, §Issue 13 — RG list and fix designs |
| BC-2.16.003 v1.32 | `.factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md` | EC-016-013-004, Tier-1 OCSF promotion mechanism, ocsf_field_to_arrow_name |
| BC-2.02.005 v1.7 | `.factory/specs/behavioral-contracts/BC-2.02.005-claroty-field-mapping.md` | Claroty OCSF field mapping table (all 9 data sources), severity_id Integer_t |
| BC-2.16.017 v1.2 | `.factory/specs/behavioral-contracts/BC-2.16.017-claroty-device-vulnerability-relations-table.md` | §PC3 device_uid composite PK, §PC5 no DTU for this table |
| Claroty sensor TOML | `crates/prism-sensors/specs/claroty.sensor.toml` | Authoritative current column declarations for all affected tables |
| DTU types | `crates/prism-dtu-claroty/src/types.rs` | ClarotyAlert and ClarotyVulnerability struct definitions |
| DTU alerts route | `crates/prism-dtu-claroty/src/routes/alerts.rs` | list_alerts handler, static fixture path, generated-records path |
| DTU alerts fixture | `crates/prism-dtu-claroty/fixtures/alerts.json` | Static fixture served on non-seeded path |
| Spec adapter | `crates/prism-bin/src/spec_driven_adapter.rs` | build_column_array String arm for EC-016-013-004 coercion |

Architecture section files this story depends on:
- `architecture/module-decomposition.md` (SS-16 Spec Engine, SS-07 Sensor Adapters)
- `architecture/adr/ADR-058-ocsf-field-name-routing.md` (§K5 Claroty-specific decisions)

## Problem Statement

The beta.2 Monroe demo live-test triage surfaced four OCSF correctness defects in the
Claroty xDome sensor spec. This story addresses the TOML-and-code-layer defects (issues
8, 11, 12, 13). Issues 9 and 10 (ADR-058 perimeter amendment for `devices.retired` and
`is_online`) are handled in sibling story S-CLAROTY-OCSF-STATUS-001 with no overlap.

### Issue 8 — `alerts.id` integer-to-string coercion path not verified (EC-016-013-004)

**Root Cause:** `ClarotyAlert.id` is declared `u32` in the DTU struct; the Claroty API
returns `id` as a JSON integer. The TOML already declares `column_type = "string"` with
`ocsf_field = "finding_info.uid"` (correct per OCSF finding_info.uid which is String_t).
The coercion documented in EC-016-013-004 must fire in `build_column_array`'s
`ColumnType::String` match arm when it encounters a `serde_json::Value::Number` input.
If that arm falls through to a default, the `finding_info_uid` Arrow column receives
`None`/null for every alert whose `id` is an integer. No test currently drives this
specific input shape end-to-end.

**Fix Design:** In `crates/prism-bin/src/spec_driven_adapter.rs`, locate the
`ColumnType::String` branch of `build_column_array` (or equivalent dispatch). Verify
the arm `Value::Number(n) => Some(n.to_string())` exists. If absent, add it before any
wildcard/catch-all arm. The `to_string()` call on `serde_json::Number` preserves the
integer's decimal representation (`123u32` -> `"123"`, not `"123.0"`).

**Scope Boundary:** TOML does not change (`column_type = "string"` is already correct,
`ocsf_field = "finding_info.uid"` already present). Only `spec_driven_adapter.rs` and
its tests change for this issue.

### Issue 11 — Six entity tables missing `ocsf_field = "time"` on datetime columns

**Root Cause:** The following tables have datetime columns but no `ocsf_field` declaration,
causing those columns to aggregate into `raw_extensions` (Tier-2) instead of projecting
as a first-class OCSF `time` Arrow column (Tier-1):

| Table | Column | Current State | Fix |
|-------|--------|--------------|-----|
| `vulnerabilities` | `published_date` | Tier-2, no ocsf_field | Add `ocsf_field = "time"` |
| `organization_zones` | `last_update` | Tier-2, no ocsf_field | Add `ocsf_field = "time"` |
| `organization_zone_policies` | `last_updated` | Tier-2, no ocsf_field | Add `ocsf_field = "time"` |
| `organization_firewall_groups` | `last_update` | Tier-2, no ocsf_field | Add `ocsf_field = "time"` |
| `organization_firewall_policies` | `last_updated` | Tier-2, no ocsf_field | Add `ocsf_field = "time"` |
| `organization_acl_policies` | `policy_last_updated` | Tier-2, no ocsf_field | Add `ocsf_field = "time"` |

**Spelling note (implementer must confirm against TOML):** `last_update` has NO trailing
'd' for `organization_zones` and `organization_firewall_groups`; `last_updated` HAS a
trailing 'd' for `organization_zone_policies` and `organization_firewall_policies`;
`policy_last_updated` HAS a trailing 'd' for `organization_acl_policies`.

Tables `servers` and `server_interfaces` are explicitly OUT OF SCOPE: neither table
contains a datetime column and therefore cannot receive `ocsf_field = "time"`.

`ocsf_field_to_arrow_name("time") = "time"`. After this fix, `WHERE time = '...'`
becomes the standard PrismQL form; raw_extensions extraction of these fields is no
longer required and is no longer supported.

**SAP-2 status:**
- `vulnerabilities.published_date`: SAP-2 PASS. `ClarotyVulnerability.published_date:
  Option<String>` exists in DTU struct. Wire emits ISO-8601 string. TOML `column_type
  = "datetime"` with `timestamp_formats = ["iso8601"]`. Compatible; no DTU change.
- All other Issue 11 tables: SAP-2 N/A. No DTU exists for entity_management tables
  (D-2200 deferral; BC-2.16.017 §PC5 precedent for this pattern).

### Issue 12 — `device_vulnerability_relations.device_uid` stuck in raw_extensions

**Root Cause:** The `device_uid` column in `device_vulnerability_relations` lacks
`ocsf_field`, making it Tier-2. Analysts cannot JOIN or filter on `device_uid` using
standard PrismQL column syntax; they must use `raw_extensions` extraction, which is
fragile and prevents OCSF-aware tooling from recognizing the field.

**Fix Design:** Add `ocsf_field = "device.uid"` to the `device_uid` column entry in
`[[tables.columns]]` for `device_vulnerability_relations`.
`ocsf_field_to_arrow_name("device.uid") = "device_uid"`. The Arrow column name is
unchanged (still `device_uid`), but it becomes a first-class Tier-1 column. No §J1
shadow collision (raw column name equals the derived Arrow name; A=B case).

**SAP-2 status:** N/A. `device_vulnerability_relations` has no DTU
(BC-2.16.017 §PC5; D-2200 deferral).

### Issue 13 — `severity_id` absent from `ClarotyAlert` struct and Claroty alerts TOML

**Root Cause:** `ClarotyAlert` in `crates/prism-dtu-claroty/src/types.rs` has no
`severity_id` field. The Claroty API returns `severity_id` as an integer on alert
objects. Without the struct field, serde silently drops the value on deserialization.
The TOML `claroty_alerts` table also lacks a `severity_id` column entry. OCSF Detection
Finding (class 2004) requires `severity_id` as Integer_t. The field appears in the
DTU record generator (`generator.rs`) but never reaches the wire because the struct
drops it.

**SAP-2 mismatch (BLOCKER for TOML column addition):** `ClarotyAlert.severity_id` is
NOT present in the struct. DTU wire currently emits no `"severity_id"` key on the
static fixture path. DTU struct fix (Phase B) is a prerequisite for TOML column
addition (Phase C-5); the implementer MUST NOT add the TOML column before the struct
and fixture are updated.

**Fix Design:**
1. Add `pub severity_id: Option<u32>` with `#[serde(default)]` to `ClarotyAlert` in
   `types.rs`.
2. Update `crates/prism-dtu-claroty/fixtures/alerts.json` to add `"severity_id": 3`
   (representative integer) to each fixture alert record.
3. Add `"severity_id"` to the `body_template` `fields` array in `claroty.sensor.toml`
   for the `alerts` table.
4. Add a new `[[tables.columns]]` entry under the `alerts` table:
   `name = "severity_id"`, `column_type = "integer"`, `ocsf_field = "severity_id"`.
   `ocsf_field_to_arrow_name("severity_id") = "severity_id"` -> Tier-1 Arrow column.

## Narrative

As a SOC analyst using PrismQL, I want Claroty xDome alert and entity data to appear
under correct OCSF field names so that I can write standard OCSF-aware queries (e.g.,
`WHERE severity_id > 3`, `WHERE time > now() - interval '24 hours'`, `WHERE
finding_info_uid = '123'`) without extracting from `raw_extensions` or working around
integer/string type mismatches in alert identifier fields.

## Behavioral Contracts

| BC | Version Pin | Relevant Clauses |
|----|-------------|-----------------|
| BC-2.16.003 | v1.32 | EC-016-013-004 (integer->string coercion); Tier-1 OCSF field promotion via `ocsf_field`; `ocsf_field_to_arrow_name()` naming convention; `ocsf_column_naming = true` routing |
| BC-2.02.005 | v1.7 | Claroty xDome OCSF field mapping (all 9 data sources); severity_id as Integer_t on OCSF Detection Finding class 2004 |
| BC-2.16.017 | v1.2 | §PC3 device_uid as composite PK join key; §PC5 no DTU for device_vulnerability_relations (SAP-2 N/A for Issue 12) |

## Acceptance Criteria

### AC-001 — Issue 8: Integer alert ID value coerces to string in String column

When `build_column_array` processes a `ColumnType::String` column and receives a
`serde_json::Value::Number` input (e.g., integer `123` from `ClarotyAlert.id: u32`),
the resulting Arrow `StringArray` cell MUST contain the string representation `"123"`,
not `None`/null. The coercion MUST preserve the integer's decimal representation;
floating-point forms (`"123.0"`) are incorrect.

_(traces to BC-2.16.003 EC-016-013-004 postcondition: a JSON integer value in a
String-typed column is normalized to its string representation at the spec parser
boundary; the `Value::Number(n) => Some(n.to_string())` arm is the load-bearing
implementation)_

### AC-002 — Issue 8: `finding_info_uid` Arrow column is Utf8 type; wire value is JSON string

Querying `claroty_alerts` via PrismQL returns an Arrow schema in which the
`finding_info_uid` column (mapped from TOML `alerts.id` via `ocsf_field =
"finding_info.uid"`) has Arrow data type `Utf8` (String), not `Int32`/`Int64`. The
serialized MCP JSON response for data rows MUST represent `finding_info_uid` values as
JSON strings (`"123"`), not JSON integers (`123`).

_(traces to BC-2.16.003 §Postconditions: Tier-1 columns use ocsf_field_to_arrow_name
as Arrow field name; column_type = "string" maps to Arrow Utf8; wire-shape assertion
discipline SID-2 applies)_

### AC-003 — Issue 11: Six entity tables expose OCSF `time` as Tier-1 Arrow column

For each of the six tables (`claroty_vulnerabilities`, `claroty_organization_zones`,
`claroty_organization_zone_policies`, `claroty_organization_firewall_groups`,
`claroty_organization_firewall_policies`, `claroty_organization_acl_policies`), a
PrismQL query returns an Arrow schema that includes a standalone column named `"time"`
with a Datetime Arrow type. The raw source column name (`published_date`, `last_update`,
`last_updated`, `policy_last_updated`) MUST NOT appear as a standalone top-level Arrow
column; it is promoted out of `raw_extensions` and into `time`.

_(traces to BC-2.16.003 §Postconditions: Tier-1 OCSF field promotion -- a column
with ocsf_field declared uses ocsf_field_to_arrow_name() as its Arrow field name;
"time" -> Arrow column "time" of Datetime type)_

### AC-004 — Issue 12: `device_uid` is a queryable Tier-1 column in device_vulnerability_relations

A PrismQL query against `claroty_device_vulnerability_relations` returns an Arrow schema
with `device_uid` as a standalone top-level column, not accessible only via
`raw_extensions` extraction. The column MUST be filterable with
`WHERE device_uid = 'abc-123'` without any `json_extract_string` or raw_extensions
workaround.

_(traces to BC-2.16.017 §PC3: device_uid is the composite PK join key for this table
and must be queryable as a first-class column; BC-2.16.003 §Postconditions: Tier-1
promotion via ocsf_field = "device.uid" -> Arrow column "device_uid")_

### AC-005 — Issue 13: `ClarotyAlert` struct deserializes `severity_id` from API response

A `ClarotyAlert` value deserialized from JSON `{"id": 1, "severity_id": 3, ...}` MUST
populate `severity_id` as `Some(3u32)`. A `ClarotyAlert` deserialized from JSON that
lacks a `severity_id` key entirely MUST produce `severity_id: None` (not a
deserialization error), courtesy of `#[serde(default)]`.

_(traces to BC-2.16.013 v1.46 §Postconditions: the DTU struct must expose all
TOML-declared columns as deserializable fields; absence of an optional field in the
response JSON must not cause deserialization failure)_

### AC-006 — Issue 13: DTU wire emits `severity_id` for alert records on both fixture paths

A request to the DTU Claroty alerts endpoint returns JSON where each alert object
includes a `"severity_id"` key with a non-null integer value. This MUST hold on BOTH
the static fixture path (serving `fixtures/alerts.json`) AND the generated-records path.
Wire-level assertion: `response_json[0]["severity_id"].is_i64()` MUST be true.

_(traces to BC-2.16.013 v1.46 §Postconditions: DTU wire must emit all TOML-declared
columns; wire-shape assertion discipline SID-2 requires asserting on serialized JSON
output, not only pre-serialization Rust structs; both static-fixture and
generated-records paths must be verified per SAP-2 probe rule 6)_

### AC-007 — Issue 13: `severity_id` Arrow column present with Integer type in query output

A PrismQL query against `claroty_alerts` returns an Arrow schema that includes
`severity_id` as a standalone Tier-1 column with Arrow type `Int64`. Seeded records
from the DTU MUST return non-null `severity_id` values. The serialized MCP JSON
response for data rows MUST include `"severity_id"` as a JSON integer.

_(traces to BC-2.02.005 v1.7 §Postconditions: Claroty Detection Finding OCSF field
mapping includes severity_id as Integer_t; BC-2.16.003 §Postconditions: Tier-1
column_type = "integer" maps to Arrow Int64; wire-shape assertion discipline SID-2)_

### AC-008 — Regression guard: existing `claroty_alerts` query structure is unchanged

A PrismQL query against `claroty_alerts` that returns paginated results MUST still
include `total_available` (integer >= 0) and `is_truncated` (boolean) in the serialized
MCP JSON response. The fields `detected_time`, `alert_type_name`, `category`, `status`,
and `description` MUST remain present and non-null for seeded alert records. No
previously-present Tier-1 column is removed or demoted to `raw_extensions` by this
story's changes.

_(traces to BC-2.16.003 §Invariants: ocsf_field additions are additive; existing Tier-1
columns are not affected by adding new ocsf_field mappings to other columns or to other
tables)_

## Red Gate Test List

**BC-5.38.001 Density Check:** 9 Red Gate tests / 8 ACs = **1.125** (>= 0.50 threshold
satisfied). Enumerated Red Gate tests below MUST be authored as failing
(`assert!(false, "RED GATE: ...")` or `todo!()`) before any implementation code is
written. Red-then-green ordering per SAC-1 and TDD Iron Law BC-8.30.001 strict mode:
test-authoring tasks (Phase A) MUST complete and all RG-COT-001..008 verified FAILING
before the implementer receives Phase B/C/D tasks.

**Embedding test-writing inline inside implementation tasks inverts TDD ordering and is
a defect under SAC-1.**

| ID | Test Name | Currently FAILS Because | AC Coverage |
|----|-----------|------------------------|-------------|
| RG-COT-001 | `test_cot_rg001_string_column_integer_json_value_coerces_to_string_cell` | `build_column_array` String arm has no verified `Value::Number => Some(n.to_string())` case with test coverage | AC-001 |
| RG-COT-002 | `test_cot_rg002_alerts_finding_info_uid_is_utf8_in_wire_response` | No test asserts serialized JSON has `finding_info_uid` as a JSON string type (not integer); wire-shape assertion absent | AC-002 |
| RG-COT-003 | `test_cot_rg003_claroty_alert_severity_id_deserializes_from_json` | `ClarotyAlert` lacks `severity_id` field; `serde_json::from_str` with `severity_id` in JSON produces None (field ignored) instead of Some(3) | AC-005 |
| RG-COT-004 | `test_cot_rg004_dtu_alerts_static_fixture_emits_severity_id` | DTU `fixtures/alerts.json` lacks `"severity_id"` key; static path returns objects without the field | AC-006 |
| RG-COT-005 | `test_cot_rg005_claroty_alerts_severity_id_in_arrow_output` | TOML lacks `severity_id` column declaration; Arrow schema for `claroty_alerts` has no `severity_id` column | AC-007 |
| RG-COT-006 | `test_cot_rg006_device_vuln_relations_device_uid_is_tier1_not_raw_extensions` | TOML `device_uid` column lacks `ocsf_field`; column aggregates into `raw_extensions` rather than as standalone Arrow column | AC-004 |
| RG-COT-007 | `test_cot_rg007_vulnerabilities_time_column_present_in_arrow_schema` | TOML `published_date` lacks `ocsf_field = "time"`; no `time` column exists in `claroty_vulnerabilities` Arrow schema | AC-003 |
| RG-COT-008 | `test_cot_rg008_organization_zones_time_column_from_last_update` | TOML `last_update` lacks `ocsf_field = "time"`; no `time` column exists in `claroty_organization_zones` Arrow schema | AC-003 |
| RG-COT-009 | `test_cot_rg009_existing_claroty_alerts_pagination_fields_unchanged` | NOT failing -- write as a passing regression guard test FIRST; must survive all subsequent story changes | AC-008 |

## Architecture Mapping

| Component | File | Module | Pure/Effectful | Issue |
|-----------|------|--------|---------------|-------|
| `build_column_array` String arm | `crates/prism-bin/src/spec_driven_adapter.rs` | SS-07 Sensor Adapters | Pure (JSON Value -> Arrow cell) | 8 |
| Claroty sensor spec | `crates/prism-sensors/specs/claroty.sensor.toml` | SS-16 Spec Engine | Config | 11, 12, 13 |
| `ClarotyAlert` struct | `crates/prism-dtu-claroty/src/types.rs` | SS-07 Sensor Adapters (DTU) | Pure (serde deserialize) | 13 |
| DTU alerts fixture | `crates/prism-dtu-claroty/fixtures/alerts.json` | SS-07 Sensor Adapters (DTU) | Config | 13 |
| DTU alerts route (read-only) | `crates/prism-dtu-claroty/src/routes/alerts.rs` | SS-07 Sensor Adapters (DTU) | Effectful (HTTP handler) | 13 (verify only) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|------------------|
| EC-001 | `alerts.id` is a UUID string in some Claroty firmware versions (EC-016-013-004 poly case) | `Value::String(s) => Some(s)` arm already handles this; no change needed; RG-COT-001 must cover BOTH integer AND UUID-string input shapes |
| EC-002 | `severity_id` absent from a live Claroty API response (older firmware) | `#[serde(default)]` on `Option<u32>` produces `None`; Arrow cell is null; no deserialization error; pagination/other fields unaffected |
| EC-003 | Organization entity datetime column is null in API response | Tier-1 `ocsf_field = "time"` with a null source value: Arrow Datetime cell = null (None); not an error per BC-2.16.003 null-handling invariant |
| EC-004 | `device_uid` is null in `device_vulnerability_relations` (BC-2.16.017 §PC3 valid case) | After Tier-1 promotion, null `device_uid` produces a null Arrow cell; not an error; null is permitted per §PC3 |
| EC-005 | `ocsf_field_to_arrow_name("device.uid")` must produce "device_uid" via dot-to-underscore convention | Implementation MUST call the canonical `ocsf_field_to_arrow_name` function; MUST NOT hard-code `"device_uid"` as the output string inline |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~4,500 |
| BC-2.16.003 v1.32 (Column-to-OCSF Mapping) | ~6,000 |
| BC-2.02.005 v1.7 (Claroty field mapping) | ~3,500 |
| BC-2.16.017 v1.2 (device_vulnerability_relations) | ~2,500 |
| BC-2.16.013 v1.46 (DTU-parity authoring contract) | ~3,000 |
| `claroty.sensor.toml` (1,695 lines) | ~6,000 |
| `crates/prism-bin/src/spec_driven_adapter.rs` | ~4,500 |
| `crates/prism-dtu-claroty/src/types.rs` | ~2,500 |
| `crates/prism-dtu-claroty/src/routes/alerts.rs` | ~2,000 |
| `crates/prism-dtu-claroty/fixtures/alerts.json` | ~1,000 |
| ADR-058 §K5 (Claroty OCSF routing decisions) | ~2,000 |
| Test files (new + existing) | ~3,000 |
| Tool outputs (nextest, cargo check) | ~2,000 |
| **Total estimate** | **~42,500 tokens** |

At 200k context window, this story consumes ~21% -- at the upper boundary of the
20-30% ceiling. If `spec_driven_adapter.rs` is larger than estimated, split Issue 8
into a micro-story before dispatch.

## Tasks

**Dispatch order: Phase A (test-writer) MUST complete and all RG-COT-001..008
verified FAILING before Phase B/C/D (implementer) dispatches.**

### Phase A -- Red Gate Tests (test-writer)

- [ ] **A-1** Write `test_cot_rg001_string_column_integer_json_value_coerces_to_string_cell`
      in `crates/prism-bin/tests/spec_adapter_coercion.rs` (create file if absent): call
      the `ColumnType::String` branch of `build_column_array` with
      `Value::Number(serde_json::Number::from(123u32))`; assert result == `Some("123")`.
      Also write a companion UUID-string shape arm: `Value::String("abc-def".into())`
      -> `Some("abc-def")`. MUST FAIL on unmodified code (no Number arm exists).

- [ ] **A-2** Write `test_cot_rg002_alerts_finding_info_uid_is_utf8_in_wire_response` as
      an integration test: query `claroty_alerts` against DTU; deserialize MCP JSON
      response; assert `response.data[0]["finding_info_uid"]` is a `serde_json::Value::String`,
      not `Value::Number`. MUST FAIL if coercion fires nowhere.

- [ ] **A-3** Write `test_cot_rg003_claroty_alert_severity_id_deserializes_from_json` in
      `crates/prism-dtu-claroty/src/types.rs` `#[cfg(test)] mod tests` block:
      `serde_json::from_str::<ClarotyAlert>(include minimal valid JSON with
      "severity_id": 3)` and assert `.severity_id == Some(3)`. Also test default path:
      JSON without `severity_id` key -> `.severity_id == None`. MUST FAIL until struct
      field added.

- [ ] **A-4** Write `test_cot_rg004_dtu_alerts_static_fixture_emits_severity_id` in
      `crates/prism-dtu-claroty/tests/`: start DTU; call alerts endpoint; parse response
      JSON array; assert `json_array[0].get("severity_id").is_some()` and
      `.is_i64()` is true. Cover BOTH fixture paths (static + generated) per SAP-2 rule 6.
      MUST FAIL until fixture updated AND struct field added.

- [ ] **A-5** Write `test_cot_rg005_claroty_alerts_severity_id_in_arrow_output` as
      end-to-end integration test: PrismQL `SELECT severity_id FROM claroty_alerts`
      against seeded DTU; assert Arrow schema contains `severity_id` with type Int64;
      assert at least one non-null value. MUST FAIL until TOML column added.

- [ ] **A-6** Write `test_cot_rg006_device_vuln_relations_device_uid_is_tier1_not_raw_extensions`
      in `crates/prism-bin/tests/`: query `claroty_device_vulnerability_relations`; parse
      Arrow schema; assert `device_uid` appears as standalone column; assert
      `raw_extensions` (if column present) does NOT contain `"device_uid"` key.
      MUST FAIL until `ocsf_field = "device.uid"` added.

- [ ] **A-7** Write `test_cot_rg007_vulnerabilities_time_column_present_in_arrow_schema`
      in `crates/prism-bin/tests/`: query `claroty_vulnerabilities` against DTU; parse
      Arrow schema; assert column named `"time"` exists with Datetime type; assert
      `"published_date"` does NOT appear as a standalone top-level column.
      MUST FAIL until `ocsf_field = "time"` added.

- [ ] **A-8** Write `test_cot_rg008_organization_zones_time_column_from_last_update`
      in `crates/prism-bin/tests/`: query `claroty_organization_zones` (no DTU -- use
      mock or spec-parse test); parse resulting Arrow schema; assert `"time"` column
      present; assert `"last_update"` not a standalone column.
      MUST FAIL until `ocsf_field = "time"` added.

- [ ] **A-9** Write `test_cot_rg009_existing_claroty_alerts_pagination_fields_unchanged`
      as regression guard: query `claroty_alerts` against DTU; parse MCP JSON response;
      assert `response["total_available"].is_i64()` is true; assert
      `response["is_truncated"].is_boolean()` is true; assert `detected_time`,
      `alert_type_name` present and non-null in first data row. Write GREEN (passing
      on unmodified code); verify it passes before story changes begin.

- [ ] **A-10** Run `cargo nextest run -p prism-bin -p prism-dtu-claroty --no-fail-fast`;
       confirm RG-COT-001..008 FAIL, RG-COT-009 PASSES. Report RED Gate density:
       8 failing / 8 ACs = 1.0 >= 0.5 satisfied. Do NOT proceed to Phase B until
       this confirmation is in hand.

### Phase B -- DTU Struct + Fixture (implementer; Issue 13 prerequisite)

- [ ] **B-1** In `crates/prism-dtu-claroty/src/types.rs`, add `pub severity_id:
      Option<u32>` with `#[serde(default)]` to `ClarotyAlert`. Verify `#[non_exhaustive]`
      attribute is present on `ClarotyAlert` (existing; confirm before commit). No new
      public type is added; `EXPECTED_SYMBOLS` count does not change.

- [ ] **B-2** Update `crates/prism-dtu-claroty/fixtures/alerts.json`: add
      `"severity_id": 3` to each alert object in the fixture array. Verify JSON parses
      cleanly. Verify the static fixture path in `list_alerts` route serves from this
      file (check `include_str!` path in route handler).

- [ ] **B-3** Run `test_cot_rg003` and `test_cot_rg004` -- MUST NOW PASS.
      Run `just iter prism-dtu-claroty` -- all tests green.

### Phase C -- TOML Corrections (implementer; Issues 8, 11, 12, 13)

- [ ] **C-1** **Issue 8 -- Coercion verification:** In
      `crates/prism-bin/src/spec_driven_adapter.rs`, locate the `ColumnType::String`
      branch in `build_column_array`. Verify `Value::Number(n) => Some(n.to_string())`
      exists. If absent, add it before any wildcard/catch-all arm. Confirm the arm does
      NOT use f64 intermediate (`123u32 -> 123.0 -> "123.0"` is wrong).
      Run `test_cot_rg001` -- MUST NOW PASS.

- [ ] **C-2** **Issue 11 -- Add `ocsf_field = "time"` to 6 tables:** In
      `crates/prism-sensors/specs/claroty.sensor.toml`, for each table+column pair add
      `ocsf_field = "time"` to the `[[tables.columns]]` entry. Exact columns (confirm
      spelling against current TOML before editing):
      - `vulnerabilities` table -> `published_date` column
      - `organization_zones` table -> `last_update` column (NO trailing 'd')
      - `organization_zone_policies` table -> `last_updated` column (WITH trailing 'd')
      - `organization_firewall_groups` table -> `last_update` column (NO trailing 'd')
      - `organization_firewall_policies` table -> `last_updated` column (WITH trailing 'd')
      - `organization_acl_policies` table -> `policy_last_updated` column

- [ ] **C-3** Run `test_cot_rg007` and `test_cot_rg008` -- MUST NOW PASS.

- [ ] **C-4** **Issue 12 -- Promote `device_uid` to Tier-1:** In `claroty.sensor.toml`,
      locate the `device_vulnerability_relations` table's `device_uid` column entry.
      Add `ocsf_field = "device.uid"`. Run `test_cot_rg006` -- MUST NOW PASS.

- [ ] **C-5** **Issue 13 -- Add `severity_id` column to TOML (ONLY after Phase B
      complete):** SAP-2 gate: verify struct fix from Phase B is committed before this
      step.
      (a) In `claroty.sensor.toml`, locate the `alerts` table's `body_template` value.
          Add `"severity_id"` to the `fields` array within the JSON body template.
      (b) Add a new `[[tables.columns]]` entry for the `alerts` table:
          ```toml
          [[tables.columns]]
          name = "severity_id"
          column_type = "integer"
          ocsf_field = "severity_id"
          ```
      Run `test_cot_rg005` -- MUST NOW PASS.

- [ ] **C-6** Run `test_cot_rg002` (wire-shape assertion for finding_info_uid) -- MUST
       NOW PASS (coercion from C-1 fires on integer id from DTU).

- [ ] **C-7** Run `cargo nextest run -p prism-bin -p prism-sensors -p prism-dtu-claroty
       --no-fail-fast` -- all 9 RG-COT tests green; confirm regression guard green.

### Phase D -- Final Verification and Delivery

- [ ] **D-1** Run `just check` (full workspace) -- no new warnings, no new clippy lints,
       all tests green.

- [ ] **D-2** SAP-2 post-fix parity sweep for `claroty_alerts` table: for each of the
       TOML `[[tables.columns]]` entries under `alerts` (including newly added
       `severity_id`), confirm the field exists in `ClarotyAlert` struct AND appears
       in the `list_alerts` route emission (both static-fixture path and
       generated-records path). Produce a per-column verdict line in the PR description.

- [ ] **D-3** Add CHANGELOG entry under `[Unreleased] > Fixed` in `CHANGELOG.md`:
       ```
       - Fixed Claroty xDome OCSF field mapping: `finding_info_uid` integer-to-string
         coercion now fires per EC-016-013-004; `severity_id` added to alerts as
         OCSF Detection Finding Integer_t field; `time` OCSF field added to 6 entity
         tables (vulnerabilities, organization_zones, organization_zone_policies,
         organization_firewall_groups, organization_firewall_policies,
         organization_acl_policies); `device_uid` promoted to Tier-1 queryable column
         in device_vulnerability_relations (beta.3 Monroe demo issues 8/11/12/13)
       ```

- [ ] **D-4** Open PR targeting `develop` with title:
       `fix(claroty): OCSF field-mapping corrections -- severity_id, time, device_uid, coercion (beta3 issues 8/11/12/13)`

- [ ] **D-5** Dispatch holdout-evaluator for story-level holdout gate (blocking before
       demo recording per CLAUDE.md §Story-level holdout gate).

## Previous Story Intelligence

This is the first story in epic E-BETA3-REMEDIATION targeting the Claroty OCSF TOML
surface (issues 8/11/12/13). Sibling story S-CLAROTY-OCSF-STATUS-001 (issues 9/10:
`devices.retired`/`is_online` ADR-058 perimeter amendment) runs in parallel with no
file conflicts.

**Lessons from prior Claroty stories applied to this story:**

- **DEFECT-CLAROTY-SORTBY-DETERMINISM-001:** TOML table column fixes must be tested
  end-to-end via PrismQL query round-trips, not only via TOML parse-level unit tests.
  Sort order and schema interactions can mask column promotion bugs at the parse level.

- **S-ADR058-OCSF-ROUTING-001:** `ocsf_field_to_arrow_name()` is the canonical naming
  function. Never hard-code the derived string inline -- always call the function. A
  hard-coded `"device_uid"` that bypasses the function will break when the naming
  convention changes.

- **SAP-2 probe (from PLUGIN-MIGRATION-001-D pass-3):** Verify the DTU WIRE EMISSION
  SITE (the route handler's response JSON), not only the Rust struct definition. A
  field on the struct but absent from `fixtures/alerts.json` never reaches the client
  on the static-fixture path. This is exactly the failure mode for Issue 13 -- the
  generator might emit `severity_id` in raw JSON blobs for the generated path, but
  the static fixture path serves a hand-crafted JSON file that lacks the field.
  Both paths must be tested per SAP-2 rule 6.

- **Trailing-'d' column name drift:** Organization entity tables have inconsistent
  column naming (`last_update` vs `last_updated`). Task C-2 lists exact spellings.
  Read the current TOML before editing -- do not guess from table name patterns.

## Architecture Compliance Rules

Extracted from `architecture/adr/ADR-058-ocsf-field-name-routing.md` §K5 and
BC-2.16.003 §Invariants:

1. **`ocsf_column_naming = true` routing (ADR-058 §K5):** When `ocsf_column_naming =
   true` is set on a Claroty table (all Claroty tables default to this), every column
   with `ocsf_field` declared is Tier-1. Arrow field name comes from
   `ocsf_field_to_arrow_name()` only. Never bypass.

2. **`#[non_exhaustive]` on `ClarotyAlert` (CLAUDE.md §Conventions):** The struct
   already carries `#[non_exhaustive]`; adding a field does not require updating
   `EXPECTED_SYMBOLS` (that gate checks for unregistered NEW public types, not for
   new fields on existing types). Confirm attribute presence before committing.

3. **`#[serde(default)]` on optional DTU fields:** All optional DTU response struct
   fields added in this story MUST use `#[serde(default)]` so that live API responses
   that omit the field do not fail deserialization. Never add `#[serde(deny_unknown_fields)]`
   to any DTU response type.

4. **`Value::Number` -> String coercion form (EC-016-013-004):** Use
   `n.to_string()` on `serde_json::Number`. Do NOT parse through `f64` first -- large
   integers lose precision and produce `"123.0"` form instead of `"123"`.

5. **No `println!` in production code (CLAUDE.md §Conventions):** Any diagnostic
   emission in `spec_driven_adapter.rs` must use `tracing::debug!` with structured
   fields, not `println!`.

6. **Forbidden dependency:** `prism-bin` MUST NOT gain a dependency on
   `prism-dtu-claroty` as a result of this story. The coercion fix in
   `spec_driven_adapter.rs` operates on `serde_json::Value` only -- it has no
   knowledge of Claroty-specific types. If `Cargo.toml` for `prism-bin` gains
   `prism-dtu-claroty` as a dependency, the build MUST fail with an architectural
   violation note. (This is an enforcement note, not a CI gate -- the implementer
   must not introduce the dependency.)

7. **No `reqwest::Client::new()` without `.timeout()` (CLAUDE.md §Conventions):**
   This story does not add new HTTP clients, but if any test scaffolding creates a
   client, it must set a 30-second timeout per ADR-050 / TD-S-PLUGIN-PREREQ-B-005.

## Library & Framework Requirements

| Library | Version Pin | Usage |
|---------|-------------|-------|
| `serde_json` | workspace pin (1.x) | `Value::Number` -> String in spec_driven_adapter.rs; fixture JSON parse in tests |
| `serde` | workspace pin (1.x) | `#[serde(default)]` on `ClarotyAlert.severity_id` |
| `arrow` (via DataFusion) | workspace pin | Arrow StringArray / Int64Array / Date64Array schema assertions in tests |
| `axum` | workspace pin | DTU route handler verification (no version change) |

No new library dependencies are introduced by this story. All required libraries are
already present in the workspace Cargo.toml.

## File Structure Requirements

**Files to modify:**

| File | Change Type | Issue |
|------|-------------|-------|
| `crates/prism-bin/src/spec_driven_adapter.rs` | Modify: add `Value::Number(n) => Some(n.to_string())` arm to String branch of `build_column_array` (<= 3 lines) | 8 |
| `crates/prism-sensors/specs/claroty.sensor.toml` | Modify: add `ocsf_field = "time"` to 6 datetime columns; add `ocsf_field = "device.uid"` to device_uid; add `severity_id` column + body_template | 11, 12, 13 |
| `crates/prism-dtu-claroty/src/types.rs` | Modify: add `pub severity_id: Option<u32>` with `#[serde(default)]` to `ClarotyAlert` | 13 |
| `crates/prism-dtu-claroty/fixtures/alerts.json` | Modify: add `"severity_id": 3` to each alert fixture record | 13 |
| `CHANGELOG.md` | Modify: add `[Unreleased] > Fixed` entry | D-3 |

**Files to create (new):**

| File | Purpose |
|------|---------|
| `crates/prism-bin/tests/spec_adapter_coercion.rs` | Unit tests for `build_column_array` coercion path (RG-COT-001, RG-COT-002) |

**Files NOT to touch:**

- `.factory/specs/behavioral-contracts/` -- state-manager registers BC updates after merge
- `.factory/specs/architecture/adr/` -- no ADR amendments in this story (ADR-058 perimeter unchanged)
- `crates/prism-dtu-claroty/src/routes/alerts.rs` -- route handler does not change;
  fixture update propagates automatically via `include_str!` or equivalent path
- Any other TOML sensor specs (crowdstrike, cyberint, armis) -- out of scope

## SAP-2 TOML-to-DTU Parity Pre-Confirmation

Confirmed before story authoring per SAP-2 standing probe (CLAUDE.md §SAP-2).
Emit-site authority rule applied: `crates/prism-dtu-claroty/src/routes/alerts.rs`
`list_alerts` handler was read directly, not inferred from struct definitions alone.

| Table | Column | TOML type | DTU Struct Field | DTU Rust Type | Wire Type | SAP-2 Verdict |
|-------|--------|-----------|-----------------|--------------|-----------|---------------|
| `alerts` | `id` | string | `ClarotyAlert.id` | `u32` | JSON integer | INTENTIONAL MISMATCH -- EC-016-013-004; Issue 8 fix verifies coercion fires |
| `alerts` | `severity_id` (new) | integer | `ClarotyAlert.severity_id` | ABSENT | ABSENT | BLOCKER -- struct field absent; Phase B adds it before Phase C-5 |
| `vulnerabilities` | `published_date` | datetime | `ClarotyVulnerability.published_date` | `Option<String>` | ISO-8601 string | PASS -- compatible with datetime + iso8601 chain (arm (b) per SAP-2) |
| `device_vulnerability_relations` | `device_uid` | string | N/A (no DTU) | N/A | N/A | N/A -- D-2200, BC-2.16.017 §PC5 |
| `organization_zones` | `last_update` | datetime | N/A (no DTU) | N/A | N/A | N/A |
| `organization_zone_policies` | `last_updated` | datetime | N/A | N/A | N/A | N/A |
| `organization_firewall_groups` | `last_update` | datetime | N/A | N/A | N/A | N/A |
| `organization_firewall_policies` | `last_updated` | datetime | N/A | N/A | N/A | N/A |
| `organization_acl_policies` | `policy_last_updated` | datetime | N/A | N/A | N/A | N/A |

**SAP-2 BLOCKERS resolved by this story:**
1. `severity_id` absent from `ClarotyAlert` -- fixed in Phase B (struct + fixture).
2. `alerts.id` integer on wire vs string in TOML -- intentional per EC-016-013-004;
   Issue 8 (Phase C-1) verifies the coercion arm fires.

No additional SAP-2 blockers identified. All other table+column pairings above are
either PASS or N/A (no DTU for entity_management tables per D-2200).

## History

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-16 | story-writer | Initial draft from beta3-remediation-delta-analysis.md Batch-0 frozen spec (issues 8/11/12/13); SAP-2 parity confirmation embedded; SAC-1 RG list (RG-COT-001..009) included; density check 1.125 |
