---
document_type: delta-analysis
level: architecture
version: "0.1"
status: draft
producer: architect
timestamp: 2026-09-15T00:00:00Z
cycle: wave-5-e-demo-fidelity
traces_to: STATE.md D-2520
inputs:
  - STATE.md D-2520 (beta.2 live-test triage, 20 issues)
  - SESSION-HANDOFF.md §RESUME SNAPSHOT D-2521
  - crates/prism-mcp/src/server.rs (LIVE_TOOLS / NOT_YET_AVAILABLE_TOOLS)
  - crates/prism-mcp/src/safety_envelope.rs (wrap, total_results)
  - crates/prism-mcp/src/tools/prism_describe.rs (build_example_with_note)
  - crates/prism-mcp/src/tools/operations.rs
  - crates/prism-bin/src/spec_driven_adapter.rs (build_column_array, null encoding)
  - crates/prism-spec-engine/src/column_mapping.rs (map_record, null encoding)
  - crates/prism-query/src/ast.rs (ScalarFunc::JsonExtractString)
  - crates/prism-query/src/engine.rs (UDF registration)
  - crates/prism-query/src/virtual_fields.rs (inject_virtual_fields)
  - crates/prism-sensors/src/pagination.rs (PaginationCursor.total_count)
  - crates/prism-sensors/specs/claroty.sensor.toml (full)
  - crates/prism-dtu-claroty/src/types.rs (ClarotyAlert, ClarotyDevice)
  - .factory/specs/behavioral-contracts/BC-2.10.012-prism-describe-schema-discovery-tool.md
  - .factory/specs/behavioral-contracts/BC-2.10.017-not-yet-available-tools-fast-fail-audit-channel-non-blocking.md
  - .factory/specs/behavioral-contracts/BC-2.11.001-query-mcp-tool.md
  - .factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md
  - .factory/specs/architecture/decisions/ADR-060-limit-aware-early-stop-pagination.md
  - .factory/specs/prd-supplements/interface-definitions.md
  - scripts/install.sh
---

# Beta.3 Remediation Delta Analysis

## Context

This artifact fulfills the F1 delta-analysis step for the beta.3 remediation cycle scoped in
STATE.md D-2520. Source event: 20 issues identified in the beta.2 Monroe demo live-test log
against a real Claroty xDome tenant (jea-readapi). Human decisions recorded in D-2520; three
post-beta.3 fast-follow stubs recorded in D-2521.

**Scope boundary:** this is analysis + design-recommendation only. No existing BCs, ADRs, VPs,
stories, or code are modified here. New spec artifacts (new BC, ADR, VP) referenced below are to
be authored during story materialization by product-owner / architect per the normal F2/F3 flow.

---

## Part 1 — Issue Delta Map (all 20 issues)

### Issue 1 — 40 ops stubs unconditionally compiled → -32003 at runtime

**Wave:** W1

**Defect:** All ~40 prism-operations tool handler methods are registered unconditionally in the
MCP router regardless of whether the operations subsystem is implemented. Any invocation returns
`-32003 not_yet_available_msg`. This poisons the analyst's tool catalog — 40 of 54 tools visibly
listed never do anything.

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| Code (primary) | `crates/prism-mcp/src/server.rs` — all stub handlers (the block beginning at the operations impl), `NOT_YET_AVAILABLE_TOOLS` const, router registration in `build_tool_router` |
| Code (secondary) | `crates/prism-mcp/src/tools/operations.rs` — the operations module stubs |
| Code (Cargo) | `crates/prism-mcp/Cargo.toml` — needs new `operations` feature |
| Spec AMENDMENT | BC-2.10.017 §Postconditions — currently says "Tool names in `NOT_YET_AVAILABLE_TOOLS` are registered in `tools/list` (visible to clients)"; must be amended to reflect the new gate behavior |
| Spec AMENDMENT | BC-2.10.011 — `list_capabilities.not_registered_tools` sync point must reflect feature-gate state |

**Spec gate decision (no new BC/ADR needed):** amend existing BC-2.10.017 and BC-2.10.011.

**Implementation design:** Add `default-off operations` Cargo feature in `prism-mcp`. Gate the
entire operations impl block and all 40 stub handlers with `#[cfg(feature = "operations")]`.
When `operations` is not in features (the default), the methods simply do not exist — they
cannot be registered and cannot return `-32003`. The `NOT_YET_AVAILABLE_TOOLS` const becomes
an empty slice when `!cfg(feature = "operations")`. The `list_capabilities.not_registered_tools`
binding (`not_registered_tools: &[&str] = NOT_YET_AVAILABLE_TOOLS`) picks up the empty slice
automatically.

**Three required sync points after every router change (invariant from BC-2.10.017):**

1. `build_tool_router` registration list
2. `LIVE_TOOLS` / `NOT_YET_AVAILABLE_TOOLS` consts (become empty when feature absent)
3. `list_capabilities.not_registered_tools` binding

**rmcp `#[cfg]` compile-spike requirement (first task in story):** before writing any tests,
verify that `rmcp`'s `#[tool_router]` proc-macro and method-dispatch machinery correctly drops
gated methods when `cfg(feature = "operations")` is absent. Spike writes a minimal test binary
in `crates/prism-mcp/tests/` that asserts tool count == 14 (not 54) with `operations` feature
absent.

**BC-2.10.012 protection:** `prism_describe` must NOT be gated. BC-2.10.012 §Postconditions
§1 explicitly states "`prism_describe` is always registered — it is NOT gated by any feature
flag or capability check." The `LIVE_TOOLS` const must continue to include `"prism_describe"`,
`"prism_query"`, `"list_capabilities"`, and the other 11 live tools regardless of the
`operations` feature state.

**Regression risk:** LOW. Contained to `prism-mcp`. No cross-crate API changes. No existing
Red Gate tests assert tool count == 54; verify no tests assert on `NOT_YET_AVAILABLE_TOOLS`
contents directly.

**Blast radius:** `prism-mcp` only. No SAP-2 DTU parity implication.

---

### Issue 2 — list_capabilities reports 40 stubs as not_registered_tools

**Wave:** W1 (same story as issue 1)

**Defect:** Consequence of issue 1. `list_capabilities` calls
`not_registered_tools: &[&str] = NOT_YET_AVAILABLE_TOOLS`, reporting all 40 stubs as
"not registered" to the LLM agent. This confuses agents into attempting tool invocations.

**Fix:** Resolved automatically when the `operations` feature gate from issue 1 lands. With
`NOT_YET_AVAILABLE_TOOLS` becoming an empty slice, `list_capabilities.not_registered_tools`
is empty and `tools/list` shows only the 14 live tools.

**Artifacts:** Same as issue 1 — no additional delta.

---

### Issue 3 — prism_describe total_results: 0 in safety_envelope.wrap()

**Wave:** W2

**Defect:** `safety_envelope::wrap()` counts `total_results` as 0 for object-shaped payloads
without a `rows` key. `prism_describe` returns an Object `{client_id, tables, pql_hints}` — the
`else { 0 }` branch fires. The envelope reports `"0 results found"` regardless of how many
tables the client has.

**Root cause:** `wrap()` handles bare-Array and `{rows: [...]}` shapes but has no arm for the
`prism_describe` shape. The shape is `{tables: [...], ...}`.

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| Code (primary) | `crates/prism-mcp/src/safety_envelope.rs` — `wrap()` total_results counting logic |
| Code (secondary) | `crates/prism-mcp/src/tools/prism_describe.rs` — call site that invokes `wrap()` |
| Spec AMENDMENT | BC-2.10.012 §Postconditions — add EC for `total_results` = count of tables returned |
| Test | `crates/prism-mcp/tests/mcp_prism_describe.rs` — add wire-shape assertion per SID-2 |

**Fix:** In `wrap()`, add a third counting arm: `results.get("tables").and_then(|v| v.as_array()) → len`. This covers `prism_describe` responses. Alternatively, count could be set at the call site rather than auto-detected inside `wrap()`, which is more explicit. Architect recommendation: add a `tables` arm in `wrap()` rather than bypassing the envelope, to preserve injection-scanning consistency.

**Regression risk:** LOW. No existing behavior changes for query/sensor tool paths.

---

### Issue 4 — example_query identical across tables (SELECT class_uid COUNT...)

**Wave:** W2-arch (spec + code)

**Defect:** `build_example_with_note` falls through to the aggregate branch when no `severity`
String column is present. The aggregate branch picks the FIRST `Integer | Float` column via
`find()`. For Claroty tables without an earlier numeric column, this picks `class_uid` — a
synthesized Integer column appended last, but `find()` returns the first match position in the
actual `columns` iterator ordering. Since `class_uid` is appended at the end, it should be
last, but for tables whose only Integer columns are `class_uid` and possibly `devices_count`,
the first match may still be `class_uid` or `devices_count` — both are semantically wrong as
group-by targets for OCSF "give me a meaningful query" examples.

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| Code (primary) | `crates/prism-mcp/src/tools/prism_describe.rs` — `build_example_with_note` agg_col logic |
| Spec AMENDMENT | BC-2.10.012 §Postconditions EC-10-025 — add exclusion list for synthesized columns in example generation |
| Test | existing `build_example_with_note` unit tests; add regression for class_uid exclusion |

**Fix:** Exclude synthesized / virtual metadata columns from the `agg_col` lookup:
`class_uid`, `_sensor`, `_client`, `_source_table`, `_source_type`. These are
infrastructure columns, not domain-data columns — they should never be the group-by target of
an example query. Use a const exclusion set: `const EXAMPLE_EXCLUDED_COLS: &[&str] = &["class_uid", "_sensor", "_client", "_source_table", "_source_type"]`.

**No new BC needed.** BC-2.10.012 amendment sufficient.

**Regression risk:** LOW. Unit-tested function, no cross-crate callers.

---

### Issue 5 — 3 of 4 virtual fields absent from prism_describe column list

**Wave:** W2 (same story as issue 3)

**Defect:** `prism_describe.rs` appends only `class_uid` (Integer) and `_sensor` (String) as
synthesized columns per OQ-003 (ADR-058 §G). The virtual fields `_client`, `_source_table`,
`_source_type` injected by `inject_virtual_fields` (BC-2.11.012) are absent from the describe
response. LLM agents cannot discover these columns for filtering.

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| Code (primary) | `crates/prism-mcp/src/tools/prism_describe.rs` — the OQ-003 synthesized-column append block |
| Spec AMENDMENT | BC-2.10.012 §Postconditions — add `_client`, `_source_table`, `_source_type` to the required synthesized columns (alongside `class_uid`, `_sensor`); must match BC-2.11.012 virtual field set exactly |
| Spec AMENDMENT | ADR-058 §G — extend OQ-003 to enumerate all five synthesized columns |
| Test | `crates/prism-mcp/tests/mcp_prism_describe.rs` — assert all five columns present |

**Fix:** Append three additional synthesized `ColumnDescriptor` entries after `_sensor`:
- `_client` (String, nullable = false)
- `_source_table` (String, nullable = false)
- `_source_type` (String, nullable = false)

These match the `VirtualField` enum variants in `prism-core/src/virtual_fields.rs`.

**Regression risk:** LOW. Additive change. Tests asserting column count must be updated.

---

### Issue 6 — total_available = post-LIMIT count (not true upstream total)

**Wave:** W2-arch (spec-heavy)

**Defect:** `QueryResult.total_available` currently reports `total_rows` — the number of rows
the engine saw after all sensor fetching, before the LIMIT cap. This equals the LIMIT value when
the query was truncated, giving the misleading signal `is_truncated: true, total_available: 25`
when there are actually 1,200 devices. An analyst cannot judge query completeness.

**Root cause:** `PaginationCursor.total_count` (from `pagination.rs::advance()`) already
captures the upstream API total from `total_count` field in each response page. This value is
never threaded forward to `FetchOutput → FanOutResult → MaterializationOutput → engine Step 6`.

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| Code (primary) | `crates/prism-sensors/src/pagination.rs` — `PaginationCursor.total_count` is the source |
| Code (plumbing) | `crates/prism-bin/src/spec_driven_adapter.rs` — `fetch()` result construction |
| Code (plumbing) | `crates/prism-sensors/src/fanout.rs` — `FetchOutput`, `FanOutResult` structs |
| Code (plumbing) | `crates/prism-query/src/materialization.rs` — `MaterializationOutput` struct |
| Code (plumbing) | `crates/prism-query/src/engine.rs` — engine Step 6 `total_available` assignment |
| Spec AMENDMENT | BC-2.11.001 — `total_available` semantics: "true upstream sensor total when available; lower bound (= rows before LIMIT cap) otherwise" |
| Spec AMENDMENT | ADR-060 §D8 — add §D8.11: upstream-total propagation chain; `total_available = max(total_rows, upstream_total)` when upstream_total known |
| Spec AMENDMENT | `prd-supplements/interface-definitions.md` — `total_available` JSON Schema `description` field |

**Design — TOML `total_count_path` spec field:**

Add an optional `total_count_path: String` field to `[tables]` spec declaration. When set,
`spec_driven_adapter.fetch()` reads the response root at that path to extract the upstream total.
For Claroty: `total_count_path = "total_count"`. For sensors without an upstream total (cursor-
only pagination, no explicit count), omit the field — `upstream_total = None` — and fall back to
current lower-bound behavior.

**Threading chain:**

```
PaginationCursor.total_count (usize)
  ↓ spec_driven_adapter.fetch() captures last page's total_count
FetchOutput { batches, any_early_stopped, any_pipeline_truncated, upstream_total: Option<usize> }
  ↓ OR-aggregate across fan-out targets: upstream_total = max() across sensors
FanOutResult { any_early_stopped, any_pipeline_truncated, upstream_total: Option<usize> }
  ↓
MaterializationOutput { any_early_stopped, any_pipeline_truncated, upstream_total: Option<usize> }
  ↓ engine Step 6
total_available = upstream_total.unwrap_or(total_rows)
```

**Per-sensor behavior:**
- Claroty xDome: `total_count_path = "total_count"` — true upstream total in every response
- Sensors without `total_count_path` (cyberint, armis): lower-bound behavior unchanged

**Regression risk:** HIGH. This touches FetchOutput, FanOutResult, MaterializationOutput
(21+ construction sites for FetchOutput per ADR-060 v1.9 changelog note). Full `just check`
required. SAP-2 not directly implicated (schema-level change, not wire-shape column change).
ADR-060 is a frozen perimeter artifact — amendment requires strict adversarial review per
BC-5.39.001.

**No new BC/ADR needed.** Amendment to existing BC-2.11.001 and ADR-060 sufficient.

---

### Issue 7 — null/list encoding: literal "null" strings + double-encoded ["null"] arrays

**Wave:** W2

**Defect:** Two distinct sub-bugs:

(7a) String columns: when a sensor API returns a JSON `null` for an optional string field,
`build_column_array` in `spec_driven_adapter.rs` materializes it as the literal string `"null"`
instead of an Arrow null cell.

(7b) List columns (raw_extensions): when a Tier-2 column has null values in an array, the
`raw_extensions` JSON-serialization path produces `["null"]` — a JSON array containing the
string `"null"` — rather than `null` (absent key) or `[]` (empty array).

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| Code (primary) | `crates/prism-bin/src/spec_driven_adapter.rs` — `build_column_array` String arm, raw_extensions aggregation |
| Code (secondary) | `crates/prism-spec-engine/src/column_mapping.rs` — `map_record` null handling |
| Spec (informational) | BC-2.11.001 EC-11-079 (null-not-absent wire-shape) — review if already covers this case; if not, add EC |
| Test | Wire-shape assertions per SID-2; must assert on serialized JSON output, not just Arrow structs |

**SAP-2 DTU parity check required:** For any Claroty table columns that are nullable strings,
verify DTU route emissions match the corrected null behavior. Failing to do this means DTU
tests pass but live API fails.

**Regression risk:** MEDIUM. Changing null-vs-string behavior may affect existing tests that
assert on literal `"null"` strings. Grep all test fixture JSON for literal `"null"` string
values before fixing; update tests to assert Arrow null cells.

> **D-1110 remove-uncertainty note (2026-09-17):** Issue 7a (top-level `Value::Null` in a
> String column → Arrow None, not `"null"`) is PRE-FIXED on develop. The `ColumnType::String`
> arm of `build_column_array` has `serde_json::Value::Null => None` as its first explicit
> match arm (added in commit `fff6e28ba`). The code behavior matches BC-2.16.003
> EC-016-013-006 (amended). Only Issue 7b (null ELEMENTS in a `Value::Array` compact-JSON-list
> → filtered; all-null → `"[]"`; EC-016-013-041) requires implementation. See
> S-MCP-NULL-ENCODING-001 §History v1.2 for the discovery record and the reclassification
> of RG-NULL-001 as a lock-in regression guard.

---

### Issue 8 — alerts.finding_info_uid column_type mismatch with live API

**Wave:** W3

**Defect:** `claroty_alerts` table declares `id` with `column_type = "string"` and
`ocsf_field = "finding_info.uid"`. The live Claroty xDome API returns `id` as a JSON integer
(u32). OCSF v1.7.0 `detection_finding.finding_info.uid` is `String_t`. There is a type-at-
boundary mismatch: the API sends integer, the TOML expects string.

The existing comment "Polymorphic ID: Claroty returns IDs as JSON integers or UUID strings.
Normalized to string at the spec parser boundary (EC-016-013-004)" indicates the STRING
declaration is intentional per-spec. The live test failure suggests the pipeline is NOT
performing the integer→string coercion correctly at runtime — the coercion path (EC-016-013-004)
may have a defect or the DTU returns it as integer without coercion.

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| TOML (annotation) | `crates/prism-sensors/specs/claroty.sensor.toml` — `alerts.id` column; confirm `column_type = "string"` is correct per OCSF; add `integer_coerce = true` if needed |
| Code (investigation) | `crates/prism-bin/src/spec_driven_adapter.rs` — EC-016-013-004 integer→string coercion path; verify it fires for `alerts.id` |
| Code (DTU) | `crates/prism-dtu-claroty/src/types.rs` — `ClarotyAlert.id: u32` serializes as integer; verify DTU wire matches coercion expectation |
| Spec AMENDMENT | BC-2.16.003 (column-to-ocsf-mapping) — document the integer→string coercion behavior for `id`-type fields |

**Blast radius:** Claroty alerts only. SAP-2 check: verify `ClarotyAlert.id` serializes as integer in DTU route; TOML says string; coercion must fire.

**Regression risk:** LOW if the coercion path is already implemented. MEDIUM if it's missing (requires implementing it).

---

### Issue 9 — devices.retired: boolean written to string ocsf_field = "status_code"

**Wave:** W3 (same story as issue 10)

**Defect:** `claroty_devices` table `retired` column: `column_type = "boolean"`, `ocsf_field = "status_code"`. OCSF `inventory_info.status_code` is a free-text string. Writing boolean `true`/`false` into a field typed as string is a type-contract violation. The live test confirms this produces unexpected values.

**Architect adjudication (per production-grade mandate):**

`retired` is a device decommissioning flag, not an operational status. It has no clean OCSF
`status_code` equivalent that preserves boolean semantics. The prior NOTE in ADR-058 §K5
("Boolean true/false written to free-text status_code is schema-valid; no clearly superior
standard OCSF field for a boolean retirement flag; no KF assigned") is acknowledged but
superseded by the live-test evidence that this mapping is causing runtime failures.

**Decision:** Remove `ocsf_field = "status_code"` from `retired` column. `retired` becomes
Tier-2 and aggregates into `raw_extensions` under its native key. ADR-058 §K5 NOTE must be
updated to reflect this decision.

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| TOML AMENDMENT | `crates/prism-sensors/specs/claroty.sensor.toml` — `devices.retired` column: remove `ocsf_field = "status_code"` |
| Spec AMENDMENT | ADR-058 §K5 NOTE for `devices.retired → status_code` — update rationale from "schema-valid note-only" to "field demoted to raw_extensions; boolean type incompatible with string status_code at runtime (beta.2 live-test evidence)" |
| Spec AMENDMENT | BC-2.16.003 §Claroty Devices — update status_code source column |
| SAP-2 parity | `crates/prism-dtu-claroty/src/types.rs` `ClarotyDevice.retired: bool` — no struct change needed; DTU emits as JSON bool, TOML now routes to raw_extensions |
| Live re-validation | NO (re-validation required for issue 10 which shares the same table) |

---

### Issue 10 — devices.status_code: is_online should be the operational status source

**Wave:** W3 (same story as issue 9)

**Defect:** `is_online` (bool) has no `ocsf_field` mapping. OCSF Device Inventory Info (5001)
has `status`/`status_id` for operational state. `is_online = true` semantically maps to
"Active"/"Online"; `is_online = false` to "Inactive"/"Offline".

**Architect adjudication:**

The spec engine does not currently support `boolean → enum_string` coercion (e.g., `true →
"Active"`). Introducing this in beta.3 would require a new spec-engine feature, which is
out of scope for a remediation cycle. The correct approach is:

1. `is_online` (bool): keep `column_type = "boolean"`, assign no `ocsf_field` for beta.3.
   Routes to `raw_extensions` as `is_online`. Queryable as `WHERE is_online = true`.
2. No `status_code` mapping for any bool source in devices table in beta.3.
3. Post-beta.3: story `S-OCSF-FIDELITY-CLAROTY-DEVICES-STATUS-001` adds `boolean_to_enum`
   coercion spec support and maps `is_online → status` ("Active"/"Inactive") + `status_id`.

This decision MUST be recorded in ADR-058 §K5 as a named deferral with a concrete story anchor.

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| TOML AMENDMENT | `crates/prism-sensors/specs/claroty.sensor.toml` — `devices.is_online` column: ALREADY has no `ocsf_field`; no change needed |
| Spec AMENDMENT | ADR-058 §K5 — add explicit deferral note for `is_online → status` mapping; anchor to `S-OCSF-FIDELITY-CLAROTY-DEVICES-STATUS-001` (this story must be created as a fast-follow stub, analogous to the S-JSON-EXTRACT-TYPED-001 stubs) |
| Live re-validation | YES — live tenant must confirm `retired` demoted to raw_extensions and `is_online` remains as raw bool; no runtime crash from status_code type mismatch |

**Blast radius:** MEDIUM. Touches ADR-058 frozen perimeter. ADR-058 amendment requires same
adversarial review standard as any perimeter artifact.

---

### Issue 11 — 7 Claroty tables lack top-level OCSF-required `time` column

**Wave:** W3 (same story as issues 8-13)

**Defect:** OCSF requires every event to carry a `time` field (the "event time" timestamp).
7 Claroty tables declare datetime columns but none with `ocsf_field = "time"`:
device_alert_relations, device_vulnerability_relations, servers, server_interfaces, org_zones,
firewall groups, org_acl_policies (exact set to be confirmed by story implementer vs DTU source).

The claroty_alerts table CORRECTLY has `detected_time → ocsf_field = "time"`.
The claroty_audit_logs table CORRECTLY has `timestamp → ocsf_field = "time"`.

**Fix per table (provisional — verify vs DTU `types.rs` before committing):**

| Table | Proposed `time` source column | Rationale |
|-------|-------------------------------|-----------|
| device_alert_relations | `device_alert_detected_time` | Detection event time |
| device_vulnerability_relations | `last_seen` or first datetime column | Most recent observation |
| servers | `last_seen` or equivalent | Inventory scan time |
| server_interfaces | `last_seen` or equivalent | Inventory scan time |
| org_zones | `created_at` or equivalent | Entity management time |
| firewall groups / policies | `created_at` or equivalent | Entity management time |
| org_acl_policies | `created_at` or equivalent | Entity management time |

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| TOML AMENDMENT | `crates/prism-sensors/specs/claroty.sensor.toml` — add `ocsf_field = "time"` to one datetime column per affected table |
| Spec AMENDMENT | BC-2.16.017 through BC-2.16.022 (each affected table BC) — add or update `time` field mapping |
| SAP-2 check | Each DTU route handler for affected tables — verify datetime field exists and is emitted on the wire |
| Live re-validation | YES — live tenant must confirm `time` column present in OCSF output for these tables |

**Blast radius:** MEDIUM. Purely TOML + BC prose; no code changes unless spec engine has a
`time`-required validator that needs updating.

---

### Issue 12 — device_vulnerability_relations join keys in raw_extensions

**Wave:** W3 (same story as issues 8-13)

**Defect:** The composite PK join keys for `claroty_device_vulnerability_relations`
(`device_uid` and `vulnerability_name`) are Tier-2 columns — they aggregate into `raw_extensions`
rather than being top-level queryable Arrow columns. This prevents cross-table JOINs and
WHERE-clause filtering on these keys.

From the TOML: "Tier-2: composite PK join key to claroty_devices (device_uid → claroty_devices.uid)".
The join-key comment exists, but the column lacks an `ocsf_field` that would make it Tier-1.

**Fix:** Promote `device_uid` and `vulnerability_name` (or equivalent join key columns) to
Tier-1 by assigning `ocsf_field` values. Candidate OCSF fields:
- `device_uid` → `device.uid` (OCSF Device object uid)
- `vulnerability_name` → `finding_info.uid` or a custom vendor field if no standard OCSF slot

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| TOML AMENDMENT | `crates/prism-sensors/specs/claroty.sensor.toml` — `device_vulnerability_relations` join key columns |
| Spec AMENDMENT | BC-2.16.017 (claroty-device-vulnerability-relations-table) §Postconditions — add join-key queryability requirement |
| SAP-2 check | `crates/prism-dtu-claroty/src/routes/` device_vulnerability_relations route — verify fields are emitted at top level not just in raw JSON |

**Blast radius:** LOW-MEDIUM. Affects one table, one crate. Test assertions on raw_extensions
content for this table must be updated.

---

### Issue 13 — claroty_alerts missing severity (stale removal; live API returns severity_id)

**Wave:** W3 (same story as issues 8-13)

**Defect:** `severity` was removed from `claroty_alerts` at Gap-CL-005 (2026-05-29) because
the DTU `ClarotyAlert` struct had no severity field. However, the live Claroty xDome API returns
`severity_id` (numeric, e.g. 1-5) in alert records. The DTU generator emits `severity_id` in
generated JSON blobs (confirmed in `generator.rs` line 167, 199) but the `ClarotyAlert` struct
does not declare it (serde drops it on deserialization). The TOML has no `severity_id` column.

**Fix in three parts:**
1. Add `severity_id` to `ClarotyAlert` struct in `crates/prism-dtu-claroty/src/types.rs`
2. Add `severity_id` to the DTU route `body_template` fields list
3. Add `severity_id` column to `claroty_alerts` table in TOML with `column_type = "integer"`
   and appropriate OCSF mapping (`ocsf_field` TBD — OCSF Detection Finding has
   `severity_id` as `Integer_t`)

**OCSF mapping for severity_id:** OCSF v1.7.0 Detection Finding (class 2004) has `severity_id`
as `Integer_t` and `severity` as `String_t`. Map `severity_id` → `ocsf_field = "severity_id"`
with `column_type = "integer"`. Add a companion string `severity` if the API also returns it
(to be confirmed by story implementer vs live DTU).

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| Code AMENDMENT | `crates/prism-dtu-claroty/src/types.rs` — add `pub severity_id: Option<u32>` to `ClarotyAlert` struct |
| Code AMENDMENT | `crates/prism-dtu-claroty/src/routes/alerts.rs` — add `severity_id` to body_template fields request |
| TOML AMENDMENT | `crates/prism-sensors/specs/claroty.sensor.toml` — add `severity_id` column to `claroty_alerts` table |
| Spec AMENDMENT | BC-2.02.005 (claroty-field-mapping) — add `severity_id` mapping row; update §Postconditions |
| Spec AMENDMENT | BC-2.16.003 — add severity_id to Claroty alerts column mapping table |
| SAP-2 check (required) | Verify `ClarotyAlert.severity_id` present in DTU wire response BEFORE adding TOML column |
| Live re-validation | YES — live tenant must confirm `severity_id` present and non-zero in alerts |

**Blast radius:** MEDIUM. DTU struct change + TOML change. `just check` required. The existing
"severity removed" comment in TOML must be updated to reference this fix.

---

### Issue 14 — SLSA attestation orphaned under drbothen org; beta.3 needed

**Wave:** W4

**Defect:** v1.0.0-beta.2 was released before the org rename (drbothen → BOHICA-LABS). The
SLSA attestation was generated against the `drbothen/prism` GitHub Actions workflow. After
org rename, `gh attestation verify --repo BOHICA-LABS/prism` fails because the attestation
references the old org namespace.

**Fix:** Create a new release `v1.0.0-beta.3` under `BOHICA-LABS/prism` with all 20 fixes
bundled. The release.yml workflow will generate a fresh SLSA attestation for BOHICA-LABS.
The old beta.2 attestation remains non-verifiable and should be documented as such in RELEASING.md.

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| Ops | `RELEASING.md` — document that beta.2 attestation is non-verifiable under BOHICA-LABS; add note on beta.3 superseding it |
| Ops | `.github/workflows/release.yml` — no code change; workflow runs correctly under BOHICA-LABS |
| Ops | `RELEASE_PROMOTE_TOKEN` PAT — must be re-scoped for BOHICA-LABS (D-2517 open item (e)) before release dispatch |

**Blast radius:** Release process only. RELEASE_PROMOTE_TOKEN re-scope is a human action.

---

### Issue 15 — install.sh URL concern + main branch stale

**Wave:** W4 (same story as issue 14)

**Defect analysis:** The current `develop` branch `scripts/install.sh` already has
`REPO="BOHICA-LABS/prism"` (verified by code read, 2026-09-15). The `main` branch is a stub
at `bdf24cec8`, 307+ commits behind `develop`. When users run
`curl -fsSL https://raw.githubusercontent.com/BOHICA-LABS/prism/main/scripts/install.sh | bash`,
they get the stale `main` content (which may still reference drbothen).

**Fix:** The develop→main promotion that happens as part of the beta.3 release (bundled into
S-BETA3-RELEASE-001) will update main with the BOHICA-LABS install.sh. No separate code change
needed beyond the promotion.

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| Release process | Develop→main promotion (part of beta.3 release workflow) |
| Verification | After beta.3 tag, verify `raw.githubusercontent.com/BOHICA-LABS/prism/main/scripts/install.sh` has `REPO="BOHICA-LABS/prism"` |

---

### Issue 16 — SETUP.md §9 my-client placeholder → confusing E-SPEC-022 error

**Wave:** W5

**Defect:** SETUP.md §9 uses `my-client` as a literal example `client_id` in the onboarding
walkthrough. When users copy-paste without substituting, the prism validator correctly rejects
it with E-SPEC-022 (org_id not found in registry). But the error message doesn't guide the
user to fix the client_id. The validator IS correct (ADR-029 / BC-2.06.015/016 by design).

**Fix:** Update SETUP.md §9 to use a clearly-distinguished placeholder (e.g.
`<YOUR-CLIENT-ID>`) with an explicit bold note: "Replace `<YOUR-CLIENT-ID>` with the exact
org_id you registered in step 8." Add a note that E-SPEC-022 means the org_id doesn't match.

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| Docs | `docs/SETUP.md` §9 — placeholder clarity + E-SPEC-022 guidance |

**No spec amendment needed.** The validator is correct by design.

---

### Issue 17 — SETUP.md §9 base_url = dashboard host → 403

**Wave:** W5 (same story as issue 16)

**Defect:** SETUP.md §9 example sensor config uses a Claroty xDome dashboard URL as `base_url`.
The correct API endpoint is `https://api.claroty.com` (or tenant-specific subdomain). The
dashboard host returns 403 for API calls.

**Fix:** Replace the example `base_url` with `https://<tenant>.claroty.com` or the canonical
API base URL per xDome documentation. Cross-reference the onboarding runbook in
`.factory/ops/live-tenant-validation-runbook.md` which has the correct URL.

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| Docs | `docs/SETUP.md` §9 — correct the base_url example |
| Docs | `.factory/ops/live-tenant-validation-runbook.md` — verify canonical API base URL is documented |

---

### Issue 18 — gh attestation verify runbook missing "download to CWD first" prereq

**Wave:** W5 (same story as issues 16/17/19)

**Defect:** The `gh attestation verify` command requires the artifact to be downloaded to
the current working directory before verification. The runbook step shows
`gh attestation verify <artifact>` without the prerequisite download step, causing
"file not found" failures for users following the runbook.

**Fix:** Add an explicit download step before the verify command:
```
gh release download v1.0.0-beta.3 --pattern 'prism-*-linux-x86_64.tar.gz' --dir ./verify_tmp
cd verify_tmp
gh attestation verify prism-<version>-linux-x86_64.tar.gz \
  --repo BOHICA-LABS/prism \
  --signer-workflow BOHICA-LABS/prism/.github/workflows/release.yml
```

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| Docs | `.factory/ops/live-tenant-validation-runbook.md` or wherever the attestation verify runbook lives |
| Docs | `RELEASING.md` §Verify attestation — if present, add same fix |

---

### Issue 19 — Additional onboarding docs fixes

**Wave:** W5 (same story as issues 16/17/18)

**Defect:** Based on D-2520 grouping of "4 docs issues" in Group F, the fourth issue is likely
one of: (a) Monroe demo script update needed for beta.3 (if demo script references beta.2
SHA/version); (b) RELEASING.md beta.2 attestation note; (c) capstone runbook URL updates.

**Provisional fix:** The story implementer must audit ALL docs files for drbothen → BOHICA-LABS
residual references and beta.2 → beta.3 version references. The ORG RENAME sweep (D-2516) may
have missed docs-only files. Story scope: complete docs sweep for BOHICA-LABS migration +
beta.3 version references.

**Affected artifacts:** `docs/` directory (SETUP.md, any remaining migration docs);
`.factory/ops/` runbooks; `RELEASING.md`.

---

### Issue 20 — json_extract_string dead AST infra (no UDF registered)

**Wave:** W3 (dedicated story S-JSON-EXTRACT-UDF-001)

**Defect:** The PrismQL parser recognizes `json_extract_string(col, 'key')` syntax
(in `sql_parser.rs` and `pipe_sql_emitter.rs`) and emits it correctly to DataFusion SQL.
However, no `ScalarUDF` named `json_extract_string` is registered with the DataFusion context
in `engine.rs`. Consequences:
- SQL mode: parser succeeds, DataFusion receives an unregistered function call → runtime error
- Pipe mode: PQL converts to SQL via pipe_sql_emitter, same runtime error
- Silent failure in queries — the error message is DataFusion-internal and not structured under
  the prism error taxonomy

Additionally: no literal-key plan-gate exists. A non-literal (dynamic) key expression is
unvalidated at plan time, opening a potential expression injection path.

**NEW SPEC ARTIFACTS REQUIRED:**

- **NEW BC: `BC-2.11.025-json-extract-string-scalar-udf.md`**
  Governs: minimal `json_extract_string(column: String, key: String) → Option<String>` UDF.
  Key contracts: (a) literal-key-only plan gate (E-QUERY-NNN for dynamic key); (b) key length
  ≤ 256 bytes (E-QUERY-NNN for oversized key); (c) top-level keys only in beta.3 (no nested
  path); (d) null-safe: `null` JSON column → `null` return; missing key → `null` return;
  non-object JSON → `null` return; (e) synchronous serde_json execution (no async); (f) UDF
  registered in DataFusion context at engine construction.

- **NEW ADR: `ADR-066-json-extract-scalar-udf.md`**
  Decision: Why synchronous serde_json (not arrow-rs JSON reader): simplicity + correctness
  for single-key extraction; performance is acceptable for LIMIT-bounded result sets.
  Deferred: push-down, nested paths (JSONPath), typed variants (int/float/bool) —
  rationale for each deferral with story anchors to the existing fast-follow stubs.
  Key scope constraints: literal-key-only (injection prevention); 256B cap (CWE-400).

- **NEW VP: `VP-162-json-extract-string-null-safety.md`**
  Property: For any (`column_value: Option<String>`, `key: &str`) where `key.len() ≤ 256` and
  `key` is a literal, `json_extract_string` either returns `Some(string)` or `None` — never
  panics, never returns an error that propagates as an unstructured internal failure.
  Verification method: Kani proof on the UDF's core extraction function (pure function, no I/O).
  Feasibility: HIGH — the function is a pure `serde_json::Value::parse → get(key)` chain;
  Kani's symbolic execution is well-suited.

**Affected artifacts:**

| Kind | Artifact |
|------|----------|
| NEW BC | `BC-2.11.025-json-extract-string-scalar-udf.md` |
| NEW ADR | `ADR-066-json-extract-scalar-udf.md` |
| NEW VP | `VP-162-json-extract-string-null-safety.md` |
| Code (new) | `crates/prism-query/src/json_extract_udf.rs` — ScalarUDF implementation |
| Code AMENDMENT | `crates/prism-query/src/engine.rs` — register UDF at context construction |
| Code AMENDMENT | `crates/prism-query/src/engine.rs` or new `plan_gates.rs` — literal-key plan gate |
| VP-INDEX AMENDMENT | Add VP-162 row; update totals |
| ARCH-INDEX AMENDMENT | Add ADR-066 row; update module-decomposition if new module file |
| BC-INDEX AMENDMENT | Add BC-2.11.025 row |

**Latent dead-path defect fix (mandatory per production-grade default):** The `ScalarFunc::JsonExtractString` AST variant and pipe_sql_emitter arm are already wired. The only missing piece is UDF registration. The plan-gate for non-literal keys is NOT present and must be added — without it, a user can write `json_extract_string(col, some_column)` and the UDF will be called with a runtime-resolved key, bypassing the 256B cap and literal-key-only contract.

**Plan-gate implementation:** In engine plan validation (before DataFusion execution), walk the SQL AST and reject any `json_extract_string` call where the second argument is not a `Literal(String)`. Error code: `E-QUERY-045` (reserve in error taxonomy `prd-supplements/error-taxonomy.md`).

**SAP-1 check:** New UDF registration must include a `BC-2.16.002` Canonical Structured Event
Catalog entry if any `tracing::info!(event_type = ...)` emission is added inside the UDF.

**Reconciliation for D-2521 fast-follow stubs:** S-JSON-EXTRACT-TYPED-001 and
S-JSON-EXTRACT-NESTED-001 reference `depends_on: S-JSON-EXTRACT-UDF-001`. This ID is confirmed
here. Story-writer must update those stubs' `depends_on` fields to reference
`S-JSON-EXTRACT-UDF-001` explicitly at story materialization time.

**Regression risk:** LOW for the UDF implementation (new code). MEDIUM for the plan-gate
(requires walking existing query plans — integration tests needed per SAP-3 spec-arm
reachability requirement).

**Blast radius:** `prism-query` crate only for the UDF. Plan-gate may touch `engine.rs`
validation path — full `just check` required.

---

## Part 2 — Story Breakdown

### Story Definitions

| Story ID | Title | Issues Covered | tdd_mode | Wave | Points |
|----------|-------|----------------|----------|------|--------|
| `S-MCP-TOOL-GATE-001` | Gate 40 operations stubs behind default-off Cargo feature | 1, 2 | strict | W1 | 3 |
| `S-MCP-ENVELOPE-DESCRIBE-001` | Fix prism_describe total_results + add missing virtual field descriptors | 3, 5 | strict | W2 | 3 |
| `S-MCP-NULL-ENCODING-001` | Fix null/list null encoding in build_column_array and map_record | 7 | strict | W2 | 3 |
| `S-DESCRIBE-EXAMPLE-DEDUP-001` | Fix build_example_with_note to exclude class_uid from aggregate column selection | 4 | strict | W2-arch | 2 |
| `S-QUERY-TRUE-TOTAL-001` | Thread upstream sensor total_count through pagination pipeline to total_available | 6 | strict | W2-arch | 8 |
| `S-CLAROTY-OCSF-REMEDIATION-001` | OCSF correctness fixes in claroty.sensor.toml + DTU alignment (issues 8-13) | 8-13 | strict | W3 | 8 |
| `S-JSON-EXTRACT-UDF-001` | Minimal json_extract_string ScalarUDF with literal-key plan gate (new BC+ADR+VP) | 20 | strict | W3 | 5 |
| `S-BETA3-RELEASE-001` | Bundle all beta.3 fixes, promote develop→main, create BOHICA-LABS beta.3 release | 14, 15 | facade | W4 | 2 |
| `S-ONBOARDING-DOCS-001` | SETUP.md §9 base_url + placeholder + attestation runbook + docs sweep | 16-19 | facade | W5 | 2 |

**Total new stories: 9**

### Dependency Graph

```
S-MCP-TOOL-GATE-001          (no deps)   → can start immediately
S-MCP-ENVELOPE-DESCRIBE-001  (no deps)   → can start immediately
S-MCP-NULL-ENCODING-001      (no deps)   → can start immediately
S-DESCRIBE-EXAMPLE-DEDUP-001 (no deps)   → can start immediately
S-QUERY-TRUE-TOTAL-001       (needs spec amendment authored first — F2 gate)
S-CLAROTY-OCSF-REMEDIATION-001 (no code deps; F2 spec amendments first for issues 9/10)
S-JSON-EXTRACT-UDF-001       (needs BC-2.11.025 + ADR-066 authored first — F2 gate)
S-ONBOARDING-DOCS-001        (no deps; can run any time)
S-BETA3-RELEASE-001          (depends on ALL W1+W2+W2-arch+W3 passing CI and merging)
```

### Parallel Execution Batches

**Batch 0 (Spec Pre-work — architect + product-owner, before TDD starts on gated stories):**
- Author `BC-2.11.025` + `ADR-066` + `VP-162` (for S-JSON-EXTRACT-UDF-001)
- Amend `BC-2.11.001` + `ADR-060` §D8 (for S-QUERY-TRUE-TOTAL-001)
- Amend `ADR-058` §K5 (for issues 9/10 adjudication)
- These MUST be at spec-gate-approved status before TDD dispatches for gated stories

**Batch 1 (All parallel — no inter-story deps within batch):**
- `S-MCP-TOOL-GATE-001`
- `S-MCP-ENVELOPE-DESCRIBE-001`
- `S-MCP-NULL-ENCODING-001`
- `S-DESCRIBE-EXAMPLE-DEDUP-001`
- `S-QUERY-TRUE-TOTAL-001` (starts after Batch 0 spec pre-work)
- `S-CLAROTY-OCSF-REMEDIATION-001` (starts after Batch 0 spec pre-work for issues 9/10)
- `S-JSON-EXTRACT-UDF-001` (starts after Batch 0 spec pre-work)
- `S-ONBOARDING-DOCS-001`

**Batch 2 (serial — depends on all Batch 1 merged to develop):**
- `S-BETA3-RELEASE-001`

Stories that MUST serialize (cannot run in parallel):
- `S-CLAROTY-OCSF-REMEDIATION-001` and `S-MCP-NULL-ENCODING-001` both touch the sensor
  adapter/pipeline; risk of merge conflicts in `spec_driven_adapter.rs` — serialize them
  or use separate worktrees with care. RECOMMENDATION: run in separate worktrees; merge
  `S-MCP-NULL-ENCODING-001` first (lower risk), then `S-CLAROTY-OCSF-REMEDIATION-001`.

Stories that SHOULD serialize for PR flow cleanliness:
- `S-QUERY-TRUE-TOTAL-001` touches `engine.rs`, `materialization.rs`, `fanout.rs`,
  `spec_driven_adapter.rs`. All other stories avoid these files except
  `S-MCP-NULL-ENCODING-001` (which touches `spec_driven_adapter.rs`). Merge
  `S-MCP-NULL-ENCODING-001` before `S-QUERY-TRUE-TOTAL-001` to avoid conflict.

---

## Part 3 — Detailed Story Specifications

### S-MCP-TOOL-GATE-001: Tool Gating

**Scope:** Issues 1 + 2.

**Spec changes (AMENDMENTS, no new BCs):**
- BC-2.10.017 §Postconditions: replace "registered in tools/list (visible to clients) but invoke
  the fast-fail handler" with "NOT registered when `operations` feature is absent (default);
  registered and fast-failing ONLY when `operations` feature is enabled"
- BC-2.10.011: update `not_registered_tools` semantics

**Red Gate tests (RG-GATE-001 .. RG-GATE-003):**
- RG-GATE-001: `tools/list` returns exactly 14 tools when `operations` feature absent
- RG-GATE-002: `list_capabilities.not_registered_tools` is empty when `operations` feature absent
- RG-GATE-003: Any of the previously-stubbed tool names returns MCP "method not found" (not
  -32003) when `operations` feature absent

**First task:** rmcp `#[cfg]` compile-spike — verify method omission before writing any tests.

**Live re-validation:** NO (no sensor/data behavior change).

---

### S-MCP-ENVELOPE-DESCRIBE-001: Describe Envelope + Virtual Fields

**Scope:** Issues 3 + 5.

**Spec changes (AMENDMENTS):**
- BC-2.10.012: add EC for `total_results` == `tables.len()`; add `_client`, `_source_table`,
  `_source_type` to synthesized columns (OQ-003 extension)
- ADR-058 §G: extend OQ-003 enumeration to five synthesized columns

**Red Gate tests (RG-DESC-001 .. RG-DESC-004):**
- RG-DESC-001: `prism_describe` response has `total_results == 3` when client has 3 tables
- RG-DESC-002: `prism_describe` columns array for any table includes `_client`, `_source_table`, `_source_type`
- RG-DESC-003: Wire-shape JSON assertion (SID-2): serialize describe response, assert `total_results` in JSON
- RG-DESC-004: Existing EC-10-025 example_query tests remain GREEN

**Live re-validation:** NO (describe output cosmetic; envelope wire-shape verified by test).

---

### S-MCP-NULL-ENCODING-001: Null/List Encoding

**Scope:** Issue 7.

**Spec changes:** BC-2.11.001 EC-11-079 review — confirm or add EC for null-not-literal-string
and empty-array-not-["null"] invariants.

**Red Gate tests (RG-NULL-001 .. RG-NULL-004):**
- RG-NULL-001: String column with JSON `null` → Arrow null cell (not literal "null" string);
  wire-level JSON assertion: key absent or `null` not `"null"`
- RG-NULL-002: List column with all-null elements → `[]` not `["null"]` in raw_extensions
- RG-NULL-003: SAP-2 parity — claroty DTU returns null for optional string field, TOML string
  column produces null cell not "null" string
- RG-NULL-004: Existing non-null tests still pass (no regression)

**Live re-validation:** NO (DTU covers this with RG-NULL-003).

---

### S-DESCRIBE-EXAMPLE-DEDUP-001: Example Query Dedup

**Scope:** Issue 4.

**Spec changes:** BC-2.10.012 EC-10-025 — add: "synthesized metadata columns (`class_uid`,
`_sensor`, `_client`, `_source_table`, `_source_type`) MUST NOT be used as aggregate targets
in auto-generated example queries."

**Red Gate tests (RG-EX-001 .. RG-EX-003):**
- RG-EX-001: `build_example_with_note("claroty_alerts", cols_with_only_class_uid_as_int)` →
  query does NOT contain `class_uid` in GROUP BY
- RG-EX-002: For a table with a real domain Integer column (e.g., `devices_count`), the
  aggregate example uses `devices_count` not `class_uid`
- RG-EX-003: `build_example_with_note` for a table with no non-excluded numerics falls through
  to the datetime count-recent form

**Live re-validation:** NO.

---

### S-QUERY-TRUE-TOTAL-001: True Upstream Total

**Scope:** Issue 6.

**Pre-condition:** BC-2.11.001 amendment and ADR-060 §D8.11 MUST be at spec-gate-approved
status before TDD begins.

**New TOML field:** `total_count_path: Option<String>` in `[tables]` config.

**Red Gate tests (RG-TOT-001 .. RG-TOT-006):**
- RG-TOT-001: Claroty devices table with `total_count_path = "total_count"`, 1200 records,
  LIMIT 25 → `total_available == 1200` not `25`
- RG-TOT-002: Sensor without `total_count_path`, LIMIT 25, 1200 records → `total_available`
  lower-bound behavior unchanged (returns 25 or whatever the early-stop reports)
- RG-TOT-003: Multi-sensor fan-out with mixed sensors → `total_available = max(upstream totals)`
- RG-TOT-004: LIMIT exactly matches upstream total → `is_truncated: false`, `total_available == rows`
- RG-TOT-005: Wire-shape assertion — `total_available` in serialized JSON equals upstream total
- RG-TOT-006: Engine Step 6 formula: `total_available = upstream_total.unwrap_or(total_rows)`;
  `is_truncated = (returned_rows < total_available)`

**Spec amendments required:**
- BC-2.11.001: `total_available` semantics update
- ADR-060 §D8.11: new subsection documenting the propagation chain and `total_count_path`
- `interface-definitions.md`: update `total_available` JSON Schema description

**Live re-validation:** YES (Claroty xDome) — verify `total_available` matches the actual
device/alert counts reported in the xDome UI.

---

### S-CLAROTY-OCSF-REMEDIATION-001: OCSF Correctness Fixes

**Scope:** Issues 8, 9, 10, 11, 12, 13.

**Pre-condition:** ADR-058 §K5 amendment for issues 9/10 MUST be at spec-gate-approved status.

This story is large (8pts) and covers 6 separate TOML + DTU alignment issues. Story-writer
should evaluate at materialization time whether to split issues 8/12/11 (TOML-only) from
issues 13 (DTU + TOML) from issues 9/10 (ADR-058 perimeter amendment + TOML).

**Red Gate tests (RG-OCSF-001 .. RG-OCSF-010):**
- RG-OCSF-001: SAP-2 — `alerts.id` integer coercion: DTU emits integer, TOML string → Arrow string cell
- RG-OCSF-002: `devices.retired` in raw_extensions (not in Tier-1 status_code Arrow column)
- RG-OCSF-003: `devices.is_online` in raw_extensions as bool
- RG-OCSF-004: Every Claroty table with a time column: Arrow `time` column present and non-null for live DTU records
- RG-OCSF-005: `device_vulnerability_relations.device_uid` queryable as a top-level column (not raw_extensions)
- RG-OCSF-006: SAP-2 — `ClarotyAlert.severity_id` emitted on DTU wire; TOML column present
- RG-OCSF-007: `alerts.severity_id` column non-null in query output for test seed data
- RG-OCSF-008: Wire-shape assertion: no `"null"` string for status_code in devices output
- RG-OCSF-009: `finding_info_uid` column in alerts is String type (not integer) in Arrow schema
- RG-OCSF-010: `is_truncated` / `total_available` unchanged by OCSF mapping fixes (regression)

**Issues 10, 11, 13 live-tenant re-validation REQUIRED** after merge.

**DTU parity checkpoint (SAP-2 mandatory):** For EVERY column added or removed in TOML,
read `crates/prism-dtu-claroty/src/types.rs` and the corresponding route handler BEFORE
writing any TOML change. Confirm the field exists in the struct AND is emitted on the wire.

---

### S-JSON-EXTRACT-UDF-001: Minimal json_extract_string UDF

**Scope:** Issue 20.

**Pre-condition:** BC-2.11.025, ADR-066, VP-162 MUST be at spec-gate-approved status.

**New artifacts authored in Batch 0:**
- `BC-2.11.025-json-extract-string-scalar-udf.md` (product-owner)
- `ADR-066-json-extract-scalar-udf.md` (architect)
- `VP-162-json-extract-string-null-safety.md` (architect)

**Red Gate tests (RG-JEX-001 .. RG-JEX-008):**
- RG-JEX-001: `json_extract_string('{"key":"val"}', 'key')` → `"val"` (happy path)
- RG-JEX-002: `json_extract_string('{"key":null}', 'key')` → SQL NULL (null-safe)
- RG-JEX-003: `json_extract_string('{}', 'missing')` → SQL NULL (missing key)
- RG-JEX-004: `json_extract_string(NULL, 'key')` → SQL NULL (null input column)
- RG-JEX-005: Non-object JSON (`"string"`) → SQL NULL (type-safe)
- RG-JEX-006: Key > 256 bytes → `E-QUERY-045` at plan time
- RG-JEX-007: Dynamic key arg (`json_extract_string(col, other_col)`) → `E-QUERY-045` at plan time (SAP-3 reachability: must test from PQL surface)
- RG-JEX-008: Pipe mode: `FROM t | select json_extract_string(raw_col, 'event') as event_type` → executes and returns value; SAP-3: test from `prism_query` public API

**VP-162 Kani proof:** added to P0 queue. Proof harness covers pure extraction function
(no DataFusion context, no Arrow).

**Mandate anchor (TD-VSDD-097 Dim-3):** Every `MUST` in BC-2.11.025 must cite RG-JEX-NNN
in the corresponding test requirement. Story-writer must verify this at materialization.

**Reconciliation (D-2521):** `S-JSON-EXTRACT-TYPED-001.depends_on` and
`S-JSON-EXTRACT-NESTED-001.depends_on` both point to `S-JSON-EXTRACT-UDF-001` (this story).
No rename needed; the IDs match.

**Live re-validation:** NO (pure logic; DTU covers it).

---

### S-BETA3-RELEASE-001: Beta.3 Release Bundle

**Scope:** Issues 14, 15.

**Pre-condition:** ALL of W1+W2+W2-arch+W3 stories merged to develop, CI green.

**Scope:**
1. Verify `RELEASE_PROMOTE_TOKEN` PAT re-scoped for BOHICA-LABS (human action pre-requisite)
2. Develop→main promotion via PR
3. Create tag `v1.0.0-beta.3` → triggers `release-tag.yml` → generates SLSA attestation
4. Verify `gh attestation verify --repo BOHICA-LABS/prism` succeeds on beta.3 artifacts
5. Update RELEASING.md to document beta.2 attestation non-verifiability

> **ERRATA (2026-09-17 state-manager D-2550):** Scope item 2 ("Develop→main promotion via PR") is imprecise and contradicts `RELEASING.md §1 Pre-Release Exception`. Pre-release beta tags MUST use `release-tag.yml` and NEVER touch `main`. The authoritative release mechanism is `gh workflow run release-tag.yml --ref develop -f tag=v1.0.0-beta.3`. The `release-promote.yml` (develop→main promotion) workflow is reserved for stable tags only (X.Y.Z without pre-release suffix). `RELEASING.md §1` and `RELEASING.md §4` are the Source-of-Truth per project precedence rules; this delta-analysis narrative is superseded on this point. `S-BETA3-RELEASE-001 §Prerequisites` and `§Architecture Compliance Rules` capture the correct mechanism.

**No new BC/ADR/VP.** Facade story.

**Live re-validation:** YES — after beta.3 release, run full live-tenant validation per
`.factory/ops/live-tenant-validation-runbook.md` against BOHICA-LABS beta.3 binary.

---

### S-ONBOARDING-DOCS-001: Onboarding Docs Sweep

**Scope:** Issues 16, 17, 18, 19.

**Scope items:**
1. `docs/SETUP.md` §9: replace `my-client` with `<YOUR-CLIENT-ID>` + E-SPEC-022 note
2. `docs/SETUP.md` §9: correct `base_url` example to canonical xDome API endpoint
3. Attestation runbook: add download-to-CWD step before `gh attestation verify`
4. Docs sweep: grep for `drbothen` and `beta.2` residual references in all `docs/` and
   `.factory/ops/` runbooks; update to BOHICA-LABS / beta.3

**No spec amendments needed.** Facade story. Can run in parallel with any story in Batch 1.

---

## Part 4 — Summary & Human Decision Points

### Story Count and Classification

| Metric | Value |
|--------|-------|
| New beta.3 stories | 9 |
| Strict TDD stories | 7 (all W1/W2/W2-arch/W3) |
| Facade stories | 2 (W4 release, W5 docs) |
| Stories needing live tenant re-validation | S-CLAROTY-OCSF-REMEDIATION-001, S-BETA3-RELEASE-001 (issues 10, 11, 13 within OCSF story; and all post-beta.3 validation) |
| Stories needing spec pre-work (Batch 0) | S-JSON-EXTRACT-UDF-001, S-QUERY-TRUE-TOTAL-001, S-CLAROTY-OCSF-REMEDIATION-001 (ADR-058 amendment) |
| Previously recorded fast-follow stubs | 3 (D-2521: S-JSON-EXTRACT-TYPED-001, S-JSON-EXTRACT-NESTED-001, S-SPEC-OVERLAY-RELOCATION-001) |
| **Total story count including fast-follows** | **12** |

### Deferred to Fast-Follow (post-beta.3)

These items were EXPLICITLY scoped out of beta.3 in D-2520 and are already in STORY-INDEX:

| Item | Story ID | Rationale |
|------|----------|-----------|
| Typed JSON accessors (int/float/bool) | S-JSON-EXTRACT-TYPED-001 | Requires type dispatch in UDF layer; beta.3 scope = string only |
| Nested JSONPath access | S-JSON-EXTRACT-NESTED-001 | Injection surface requires bounded path parser; beta.3 = literal-key only |
| Armis overlay relocation | S-SPEC-OVERLAY-RELOCATION-001 | E-SPEC-022 UX debt, low severity |
| `is_online → status` boolean-to-enum coercion | (to be created as fast-follow stub) | Requires new spec-engine feature; ADR-058 §K5 deferral anchor required |

### Items Requiring Human Decision at Spec Gate

The following items require human approval before spec-gate can proceed:

1. **S-QUERY-TRUE-TOTAL-001 scope boundary:** The `total_count_path` TOML field changes
   the public sensor spec format. Confirm: (a) backward-compatible (omitting `total_count_path`
   uses lower-bound behavior, no breaking change); (b) ADR-060 frozen-perimeter amendment
   authorized for beta.3 scope.

2. **ADR-058 §K5 amendment authority:** ADR-058 is a frozen perimeter artifact. The
   `devices.retired → status_code` change (issue 9) and the `is_online` deferral recording
   (issue 10) amend ADR-058 §K5. Confirm this amendment is authorized as a beta.3 remediation
   action.

3. **S-CLAROTY-OCSF-REMEDIATION-001 story split decision:** 8 points covering 6 issues is
   at the upper bound of single-story scope. Architect recommends the story-writer evaluate at
   materialization whether issues 9+10 (ADR-058 perimeter) should be a separate story from
   issues 8+11+12+13 (TOML-only). Human preference on splitting affects wave sequencing.

4. **`S-OCSF-FIDELITY-CLAROTY-DEVICES-STATUS-001` fast-follow stub creation:** The
   adjudication for issue 10 defers boolean-to-enum coercion to a fast-follow story. This story
   does not yet exist in STORY-INDEX. Should state-manager create the stub at the same time as
   the beta.3 story set? Recommend: yes, create alongside D-2521 analogues.

5. **`E-QUERY-045` error code reservation:** The json_extract_string plan-gate requires a new
   error code in `prd-supplements/error-taxonomy.md`. Confirm product-owner reserves this code
   during BC-2.11.025 authoring.

6. **RELEASE_PROMOTE_TOKEN PAT re-scope:** This is a GitHub admin action required before
   S-BETA3-RELEASE-001 can dispatch the release workflow. Confirm human will action this before
   release story starts.

### Items That DO NOT Require Human Decision (architect-adjudicated in-scope)

Per the production-grade default and architect mandate scope in the task instructions:

- **Issue 9/10 adjudication:** `retired` demoted to raw_extensions; `is_online` deferred to
  fast-follow with ADR-058 §K5 anchor. This is the correct engineering decision given the
  type mismatch evidence and the absence of a spec-engine boolean-to-enum coercion feature.
  No human decision needed.

- **Issue 8 (finding_info_uid string vs integer):** OCSF v1.7.0 `finding_info.uid` is
  `String_t`. The `column_type = "string"` declaration is CORRECT. The fix is verifying/
  implementing the integer→string coercion path at the spec-engine boundary (EC-016-013-004).
  No type change to TOML needed; code investigation + test coverage needed.

- **ADR-066 technology decision:** synchronous `serde_json` for the UDF (not arrow-rs JSON
  reader). Rationale: arrow-rs JSON reader is for bulk columnar deserialization; single-key
  extraction per row is better served by `serde_json::from_str` + `get(key)`. This is an
  in-scope architectural decision.

---

*Artifact path: `.factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md`*
*Status: UNCOMMITTED — awaiting state-manager burst*
