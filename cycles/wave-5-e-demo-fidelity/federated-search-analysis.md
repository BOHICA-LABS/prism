---
document_type: architecture-gap-analysis
title: "Federated Search Gap Analysis — What would it take for prism to be a TRUE federated search engine?"
version: "1.0"
created: 2026-09-18
authors: [architect]
status: draft
traces_to: ".factory/specs/architecture/ARCH-INDEX.md"
scope: post-v1 / human scoping decision input
cycle: wave-5-e-demo-fidelity
---

# Federated Search Gap Analysis

**Litmus test:** "If I just wanted to look at all vulns, how would I do that?" — address a
CONCEPT (vulnerabilities) and have prism fan out to every sensor that can serve it,
OCSF-normalize, and return one unified result set WITHOUT naming a specific sensor or table.

**Status:** ANALYSIS ONLY — no ADR, story, code, or spec amendment produced here. Feeds a
human scoping decision.

---

## A. Executive Summary

**Prism is NOT a truly federated search engine today.** It is a multi-tenant, single-sensor
fan-out engine with OCSF normalization scaffolding in progress. The query engine correctly
fans out a query against a single named table (e.g., `claroty_vulnerabilities`) to ALL
configured org tenants of that sensor — but it does NOT automatically discover and fan out
to MULTIPLE DIFFERENT SENSORS based on a concept or OCSF class. The grammar already defines
five composite virtual sources (`EVENTS`, `ALERTS`, `DEVICES`, `ASSETS`, `SESSIONS`) with
documented sensor mappings (prismql-grammar.md §11.2), and the AST correctly classifies them
(`SourceRefKind::Composite`). However, the execution engine does not implement these composite
sources — a composite-source query fails at runtime with `E-QUERY-036` (UnknownSourceTable).
`VULNERABILITIES` does not exist as a composite source at all.

The three top gaps standing between today's prism and true federated search are:
1. **No concept-level / OCSF-class-addressable virtual sources** (`VULNERABILITIES` is
   unrecognized; existing composite sources are grammar-only, not executed).
2. **No capability-based fan-out routing** — there is no registry mapping OCSF class to the
   set of sensor tables that produce it, so the engine cannot auto-discover which sensors to
   query for "vulnerability_finding".
3. **No schema unification layer across heterogeneous sensor shapes** for a federated result
   set (column names differ; the per-sensor ADR-058 OCSF normalization is in flight for the
   v1 Claroty scope, but cross-sensor schema reconciliation is unimplemented and unspecced).

---

## B. Current State — Verified Query Flow, Federation Model, OCSF Catalog, Capability Routing

### B.1 Query Flow (end-to-end, verified in code)

1. **Parse:** `prism_query::filter_parser::PrismQlParser::parse` processes the query string
   and emits an `Ast` value (defined in `prism_query::ast`). All three modes (Filter, SQL,
   Pipe) share this path.

2. **Source classification:** Every source reference in the query is classified into one of
   four `SourceRefKind` variants at parse time via `SourceRefKind::classify`:
   - `External { sensor, table }` — dot-notation e.g. `crowdstrike.detections` (rejected
     downstream per EC-11-067; production queries use underscore form)
   - `Custom` — underscore-form e.g. `crowdstrike_detections`
   - `Internal(InternalTable)` — `prism_*` storage tables
   - `Composite(CompositeSource)` — virtual multi-sensor sources: `EVENTS`, `ALERTS`,
     `DEVICES`, `ASSETS`, `SESSIONS` (case-insensitive)

3. **Plan gates (pre-fan-out):** `execute_inner` runs four sequential gates before any
   sensor fetch — E-QUERY-037 (table availability), E-QUERY-038 (column availability),
   E-QUERY-039 (enrich UDF availability), E-QUERY-045 (json_extract literal key). The table
   availability gate in `table_registry::TableRegistry::check_availability_gate` SKIPS
   composite sources without error (pass-through). Column and type gates also skip composite
   sources (return `Ok(())` without schema lookup — "fail-open").

4. **Source name extraction:** `materialization::extract_source_names` (shallow) collects the
   raw source string from each AST position and passes it to `resolve_source_refs`. For a
   `Pipe` or `Sql` query with source `EVENTS`, this pushes the string `"EVENTS"` into the
   `source_names` vector.

5. **Fan-out resolution:** `materialization::resolve_source_refs` iterates `source_names`.
   For each name it calls `sensor_id_from_table_name`, which extracts the underscore-prefix
   as the sensor ID. `"EVENTS"` has no underscore separator, so the whole string becomes the
   prefix. After lowercasing, `SensorId::try_from_str("events")` either succeeds (creating a
   SensorId "events") or fails (returning E-QUERY-036 immediately). In either case, no sensor
   named "events" is registered in the `AdapterRegistry`, so the guard
   `!adapter_registry.is_sensor_registered(&sensor_id)` fires and returns
   `Err(PrismError::UnknownSourceTable(...))` — `E-QUERY-036`.

   **Conclusion:** Composite-source queries (EVENTS, ALERTS, DEVICES, ASSETS) fail at
   execution time with E-QUERY-036. SESSIONS is specified as returning E-QUERY-015 (the
   grammar doc §11.2) but E-QUERY-015 is NOT implemented in the codebase — it would also
   fall through to E-QUERY-036.

6. **Actual fan-out (for named tables):** For a valid registered sensor table name (e.g.,
   `claroty_vulnerabilities`), `resolve_source_refs` calls
   `adapter_registry.get_all_for_sensor(&sensor_id)` and produces one `FanOutTarget` per
   `(OrgId, SensorId)` pair. The fan-out semaphore (`MAX_FANOUT_CONCURRENCY = 10` in
   `prism_sensors::fanout`) limits concurrency. `prism_sensors::fanout::fan_out` spawns
   one Tokio task per target and collects successes and partial failures
   (`FanOutResult` per BC-2.01.010).

7. **Bare-filter fan-out:** A filter query with no explicit source (`filter.source.raw.is_empty()`)
   triggers a special path in `run_materialization_pipeline` (Step 3b) that iterates ALL
   registered sensors in the adapter registry and synthesizes one FanOutTarget per
   `(OrgId, SensorId)`. This is the ONLY cross-sensor fan-out path that exists today. It is
   scoped to bare-filter mode and fans out by sensor, not by OCSF class.

### B.2 Fan-Out Model (the crux)

**Today's fan-out is "one table, many tenants"**, not "one concept, many sensors."

`resolve_source_refs` accepts a concrete table name like `claroty_vulnerabilities` and fans
it out to all tenants (OrgIds) that have Claroty configured. It does NOT accept an OCSF class
name like `vulnerability_finding` and route it to all sensor tables that declare that class.
The bare-filter path provides "all sensors" fan-out but with no schema filtering — every
registered sensor receives the same bare predicate, which is only useful if the predicate
fields exist on every target table (fragile across heterogeneous schemas).

**Fan-out architecture (BC-2.01.002):** The `MAX_FANOUT_CONCURRENCY` constant in
`prism_sensors::fanout` caps concurrent sensor fetches per query at 10. This is per-query
fan-out within one sensor type across tenants, not across sensor types.

### B.3 OCSF Catalog

Each sensor TOML spec (`prism-sensors/specs/*.sensor.toml`) declares `ocsf_class` per
`[[tables]]` block. The current registered `ocsf_class` values across production sensors are:

| Sensor | Table | ocsf_class |
|--------|-------|-----------|
| claroty | alerts | `detection_finding` |
| claroty | devices | `device` (pending correction to `inventory_info` per ADR-058 §K2) |
| claroty | vulnerabilities | `vulnerability_finding` |
| claroty | audit_logs | `audit_activity` (pending correction to `entity_management` per ADR-058 §K1) |
| claroty | device_alert_relations | `detection_finding` |
| crowdstrike | detections | `detection_finding` |
| crowdstrike | hosts | `device` |
| crowdstrike | incidents | `incident_finding` |
| armis | devices | `device` |
| armis | alerts | `detection_finding` |
| cyberint | alerts | `detection_finding` |
| cyberint | incidents | `incident_finding` |

The `ocsf_class` field is stored on each `prism_spec_engine::spec_parser::TableSpec` but is
used only for two purposes: (a) injecting the correct `class_uid` integer column into the
normalized Arrow `RecordBatch` via `prism_bin::spec_driven_adapter::pipeline_result_to_record_batch`
and (b) column naming when `ocsf_column_naming = true` (ADR-058). There is NO reverse index
from OCSF class string → set of sensor tables. No routing based on OCSF class exists.

**Only Claroty has `vulnerability_finding`.** NVD is an enrichment data source (not a
queryable sensor table; registered in `prism-dtu-demo-server` as a global enrichment
instance, not via a sensor TOML spec). No other sensor exposes a vulnerability inventory
table.

### B.4 Capability Routing

There is no capability routing registry mapping OCSF concepts to sensor tables. The word
"routing" in the current codebase refers exclusively to:

- **ADR-058 "OCSF field-name routing"** — the per-column naming convention that maps
  `ocsf_field` paths to underscore-flattened Arrow column names (e.g.,
  `finding_info.uid` → `finding_info_uid`). This is schema normalization, not query routing.
- **`prism_sensors::table_dispatch::route_table_query`** — routes a query to either a live
  API fetch or an event buffer scan based on `TableType` (PointInTime vs EventStream). This
  is per-table dispatch, not cross-sensor routing.
- **Story `S-ADR058-OCSF-ROUTING-001`** — the Stage 2 implementation of ADR-058, which
  adds the OCSF column-naming pipeline and spec-load collision validation. Scoped to
  per-sensor OCSF field-name plumbing, not concept-level cross-sensor routing.

---

## C. The "All Vulns" Worked Example

### C.1 What Happens Today

**Scenario:** User asks "show me all vulnerabilities" — no specific sensor named.

**Natural query attempt 1 — concept-level source (unimplemented):**
```
SELECT * FROM VULNERABILITIES
```
**Result:** `SourceRefKind::classify("VULNERABILITIES")` returns `SourceRefKind::Custom`
("VULNERABILITIES" is not in the composite source keyword set). The table availability gate
(`check_table_availability`) then looks up "VULNERABILITIES" in the `TableRegistry` — not
found → `E-QUERY-037 TableNotAvailable`. Error returned. No data fetched.

**Natural query attempt 2 — existing composite source (grammar-defined, not executed):**
```
SELECT * FROM EVENTS
```
**Result:** `SourceRefKind::classify("EVENTS")` → `Composite(CompositeSource::Events)`.
Table availability gate skips (pass-through for Composite). In `extract_source_names`, raw
string "EVENTS" is pushed to `source_names`. In `resolve_source_refs`,
`sensor_id_from_table_name("EVENTS")` extracts prefix "events" (no underscore, whole string
as prefix). `adapter_registry.is_sensor_registered("events")` → false →
`E-QUERY-036 UnknownSourceTable`. Error returned. No data fetched.

**What the user must type today (v1, Claroty-only scope):**
```sql
SELECT * FROM claroty_vulnerabilities
```
This WORKS for a single Claroty tenant in the v1 scope. The query resolves to
`sensor_id = "claroty"`, fans out to all Claroty-configured OrgIds, and returns a
`RecordBatch` with OCSF-normalized `vulnerability_finding` fields (when
`ocsf_column_naming = true` on the claroty spec, which it is — `claroty.sensor.toml`
`ocsf_column_naming = true`).

**Post-v1 multi-sensor scenario (if CrowdStrike Spotlight or NVD were added as sensor tables):**
The user would have to write:
```sql
SELECT * FROM claroty_vulnerabilities
UNION ALL
SELECT * FROM nvd_cves
```
But PrismQL does not implement `UNION ALL`. The analyst would have to issue two separate
queries and manually reconcile the results. There is no single-query path.

### C.2 Target State (True Federated Vulnerability Search)

```sql
SELECT * FROM VULNERABILITIES WHERE severity_id >= 7
```
**Expected behavior:** prism looks up which sensor tables have `ocsf_class = "vulnerability_finding"`,
fans out to all of them across all configured tenants, OCSF-normalizes each result set,
unifies schemas, deduplicates by `finding_info.uid` or CVE ID, and returns a single
`RecordBatch` with provenance columns (`_sensor`, `_client`).

This requires every component in Section F below.

---

## D. Definition of "Truly Federated Search" for Prism

A federated search engine for prism satisfies these seven properties:

1. **Concept/class-addressable querying.** The analyst addresses a security domain concept
   (`VULNERABILITIES`, `ALERTS`, `DETECTIONS`) without naming a sensor. The engine resolves
   this to the set of sensor tables that declare the corresponding OCSF class.

2. **Automatic capability-based fan-out.** A capability registry maps each OCSF class
   (and/or concept alias) to the set of `(sensor_id, table_name)` pairs currently registered
   and configured for the requesting org. The engine fans out automatically to all entries in
   that set, not just to the one table explicitly named.

3. **Schema unification.** Results from heterogeneous sensor tables with the same OCSF class
   but different column sets are reconciled into a unified schema before DataFusion processes
   them. This requires either (a) a unified OCSF-canonical projection that all sensors coerce
   to, or (b) a schema-merge step that aligns Arrow field names, fills nulls for absent fields,
   and aggregates vendor-specific fields into `raw_extensions`.

4. **Predicate pushdown per sensor.** Each sensor translates the WHERE predicate into its
   native query parameters before making the API call. Today's ADR-033 / ADR-058 pushdown
   handles time windows and OCSF-named fields for single-sensor queries; federated pushdown
   must handle the case where the same predicate translates differently per sensor (different
   field names, pagination protocols, filter APIs).

5. **Dedup / entity resolution.** The same vulnerability (same CVE-ID, same device) may
   appear in multiple sensor feeds. A federated query must offer optional deduplication by
   canonical identifier (CVE ID, OCSF `finding_info.uid`) across the unified result set.

6. **Federated pagination.** A single `PaginationCursor` (BC-2.11.001) that encodes
   per-sensor page state so `get_next_page` advances all sensors in concert, not just one.

7. **Result provenance.** Virtual fields `_sensor` and `_client` (BC-2.11.012) must be
   preserved in the unified result set so the analyst can see which sensor and tenant each
   row came from. This already works for single-sensor queries; it must survive schema
   unification.

8. **Partial-failure semantics.** If one sensor API is down or returns an error, results from
   the other sensors are still returned with a `sensor_errors` diagnostic array
   (BC-2.01.010). This already works for single-sensor fan-out; it must generalize to
   cross-sensor fan-out.

---

## E. Gap Analysis

| Property | Status | Evidence (symbol/BC/ADR anchor) |
|----------|--------|--------------------------------|
| Concept/class-addressable source (`FROM VULNERABILITIES`) | **ABSENT** | `SourceRefKind::classify` has no `Vulnerabilities` case; "VULNERABILITIES" → `Custom` → E-QUERY-037 |
| Composite source GRAMMAR support (`EVENTS`, `ALERTS`, etc.) | **PARTIAL** (grammar only) | `SourceRefKind::Composite` variant exists; prismql-grammar.md §11.2 documents sensor mappings; SESSIONS error `E-QUERY-015` undeclared in code |
| Composite source EXECUTION | **ABSENT** | `resolve_source_refs` receives raw "EVENTS" string → `sensor_id_from_table_name` → "events" → not registered → E-QUERY-036; no composite→table expansion path exists |
| OCSF class → sensor table routing registry | **ABSENT** | `TableSpec.ocsf_class` stored per-table but no reverse index; `prism_spec_engine` has no `by_ocsf_class` query; no routing registry struct exists |
| Automatic cross-sensor fan-out (concept-driven) | **ABSENT** | `resolve_source_refs` fans out one sensor ID to all tenants; bare-filter Step 3b fans out all sensors with NO schema filtering; neither path routes by OCSF class |
| Per-sensor OCSF schema normalization (single sensor) | **PARTIAL** (in flight) | ADR-058 `ocsf_column_naming = true` on claroty spec; `ocsf_field_to_arrow_name` in `prism_spec_engine::column_mapping`; `S-ADR058-OCSF-ROUTING-001` still in flight |
| Cross-sensor schema unification (federated result set) | **ABSENT** | No schema-merge step; DataFusion session contexts are single-source; Arrow schemas per sensor have heterogeneous field sets |
| Predicate pushdown per sensor (single sensor, time-window) | **PARTIAL** | ADR-033 time-window extraction pre-fan-out; `prism_query::pushdown::PushDownPlan` per `FanOutTarget`; OCSF field-name pushdown pending `S-ADR058-OCSF-ROUTING-001` |
| Predicate pushdown across heterogeneous sensors | **ABSENT** | Pushdown currently assumes field names are identical across all fan-out targets of one sensor; different OCSF-column names per sensor are not translated |
| Dedup / entity resolution across sensors | **ABSENT** | No dedup by CVE-ID, OCSF `finding_info.uid`, or device identity across sensor result sets |
| Federated pagination (multi-sensor cursor) | **ABSENT** | `PaginationCursor` (BC-2.11.001) is per-table-per-sensor; no multi-sensor cursor encoding or `get_next_page` orchestration across sensor fan-out |
| Result provenance (`_sensor`, `_client`) | **SUPPORTED** | Virtual fields injected post-materialization per BC-2.11.012; survives single-sensor fan-out |
| Partial-failure semantics | **SUPPORTED** | `FanOutResult::errors` per BC-2.01.010; `sensor_errors` propagated to `QueryResult` |

---

## F. What Needs to Happen — Concrete Architectural Changes

Sequenced by dependency:

### F.1 [Prerequisite] Complete ADR-058 OCSF normalization (in flight — v1 scope)

`S-ADR058-OCSF-ROUTING-001` (in flight, parked worktree) delivers the per-sensor OCSF
column-naming pipeline for Claroty. This is required before F.2 because schema unification
builds on a stable single-sensor OCSF schema.

**Affected components:** `prism_spec_engine::column_mapping`, `prism_bin::spec_driven_adapter`,
`prism_query::table_registry`, `claroty.sensor.toml` KF corrections.

### F.2 [Foundation] OCSF class → sensor table routing registry

A new component (or extension of `prism_spec_engine` or `prism_query::table_registry`) that
builds a reverse index: `ocsf_class_string → Vec<(SensorId, table_name)>`. Populated at
spec-load time from each `TableSpec.ocsf_class`. Consulted at query planning when the AST
source kind is `Composite` or a new `OcsfClass` variant.

**New data:** An enum or constant mapping from concept aliases (`VULNERABILITIES`,
`DETECTIONS`, `INCIDENTS`) to OCSF class strings (`"vulnerability_finding"`,
`"detection_finding"`, `"incident_finding"`). This is a new first-class concept in the
spec engine.

**Key decisions needed (human/product-owner):**
- Which concept aliases are v2-scope vs. v3-scope?
- Should `VULNERABILITIES` alias EXACTLY `vulnerability_finding`, or also include
  `detection_finding` tables that carry CVE references?
- How is the registry updated when new sensors are onboarded? (Hot-reload semantics.)

### F.3 [Core] Composite source execution in `resolve_source_refs`

Today `resolve_source_refs` skips nothing for composite sources — it blindly processes their
raw string as a sensor ID prefix and fails. The fix: before the `sensor_id_from_table_name`
call, check if the source_name matches a known composite keyword or OCSF class. If it does,
expand it via the F.2 registry into a list of concrete `(sensor_id, table_name)` pairs and
synthesize `FanOutTarget` entries for each registered pair.

**Grammar coverage:** The existing five composite sources in `SourceRefKind::Composite` need
execution wired. The new `VULNERABILITIES` (and future) concepts need both grammar and
execution. The grammar change is additive (new `CompositeSource` variant or a new
`SourceRefKind::OcsfClass` variant).

### F.4 [Schema] Cross-sensor schema unification layer

When `resolve_source_refs` produces targets from multiple sensor types, the DataFusion
session context registration step (`run_materialization_pipeline` Step 4 / Arrow table
registration) must reconcile schemas. Options:

- **Option A: Canonical OCSF projection.** Each sensor's result is projected to the
  canonical set of Arrow fields for its OCSF class (class_uid, the declared Tier-1 fields,
  raw_extensions for Tier-2). All sensors for a given OCSF class share the same projected
  schema. Schema mismatches become null fields.
- **Option B: Schema-merge.** Arrow `MergedSchemaAdapter` pattern — discover the superset
  of fields across all sensors and fill nulls.

Option A aligns with the existing ADR-058 OCSF column-naming work and is the preferred path.
It requires that each sensor's `pipeline_result_to_record_batch` in `prism_bin::spec_driven_adapter`
project to the canonical schema for the declared `ocsf_class`, not just the sensor's own
column set.

### F.5 [Pushdown] Heterogeneous predicate translation

ADR-033's time-window extraction and ADR-058's OCSF field-name mapping must be extended to
handle the case where the same logical predicate (`WHERE severity_id >= 7`) maps to different
API parameter names per sensor. The `PushDownPlan` struct (in `prism_query::pushdown`) must
carry per-sensor translated parameters, not a single plan applied to all targets.

This is the highest-complexity component. The `severity_id` OCSF field maps to
`adjusted_vulnerability_score_level` in Claroty, a different field in a hypothetical NVD
adapter, etc. A predicate-translation layer per sensor is needed, likely declared in the
sensor TOML spec as a pushdown hint alongside the `ocsf_field` annotation.

### F.6 [Pagination] Multi-sensor federated cursor

`PaginationCursor` (BC-2.11.001) today encodes `(sensor_id, table_name, offset/token)`. For
federated pagination, it must encode a map of per-sensor page state. The `get_next_page`
MCP tool must advance all sensors in the map and merge results. This is a new BC and a new
cursor encoding.

### F.7 [Dedup] Optional entity resolution

A post-materialization dedup stage (analogous to `PipeStage::Dedup` but keyed on canonical
OCSF identifiers: `finding_info.uid`, CVE-ID, or device `uid`). The analyst can opt in with
`| dedup finding_info_uid` today for single-sensor queries. Cross-sensor dedup requires
schema unification (F.4) to have unified `finding_info_uid` field names first.

---

## G. Candidate Epic / Story Outline (High-Level, for Later Materialization)

**Do NOT create stories from this section.** These are scoping inputs for the human.

### Epic: E-FEDERATED-SEARCH (post-v1)

**Wave F1 — Routing Registry Foundation** (depends on `S-ADR058-OCSF-ROUTING-001` closed)
- `S-OCSF-CLASS-REGISTRY-001` — Build reverse index `ocsf_class → [(sensor_id, table)]` in
  spec-engine; expose query API
- `S-CONCEPT-ALIAS-REGISTRY-001` — Define concept-alias enum (`VULNERABILITIES`,
  `DETECTIONS`, `INCIDENTS`, `DEVICES`) mapping to OCSF classes; grammar extension for
  new composite sources

**Wave F2 — Composite Source Execution**
- `S-COMPOSITE-SOURCE-EXEC-001` — Wire existing composite source keywords (EVENTS, ALERTS,
  DEVICES, ASSETS) through `resolve_source_refs`; fix E-QUERY-036 for these sources
- `S-VULNERABILITY-SOURCE-001` — Add `VULNERABILITIES` composite source; grammar + execution

**Wave F3 — Schema Unification**
- `S-CROSS-SENSOR-SCHEMA-001` — Canonical OCSF projection per class; Option A schema unification
  across heterogeneous sensor result sets; integration with DataFusion session context
  registration

**Wave F4 — Federated Pushdown and Pagination**
- `S-FEDERATED-PUSHDOWN-001` — Per-sensor predicate translation via OCSF-field pushdown hints
- `S-FEDERATED-CURSOR-001` — Multi-sensor `PaginationCursor` encoding; federated `get_next_page`

**Wave F5 — Dedup and Polish**
- `S-CROSS-SENSOR-DEDUP-001` — OCSF-identifier-keyed dedup across federated result set

### v1 vs post-v1 recommendation

**v1 scope (Claroty-only, D-2264 GOVERNING DECISION 2026-08-21):** None of the above is in
scope. The user today types `SELECT * FROM claroty_vulnerabilities`, and for v1 this is the
correct answer. The v1 goal is a fully-working single-sensor end-to-end experience.

**Post-v1 (when CrowdStrike, Armis, Cyberint rejoin scope per D-2443 post-v1 de-scope):**
Wave F1 and F2 are the minimal viable federation step. Wave F3 (schema unification) is
required before federation is query-correct. Waves F4–F5 are high-value but deferrable.

The order of impact for the "all vulns" litmus test:
1. Wave F1 + F2 → analyst can type `FROM VULNERABILITIES`; gets Claroty data only (one sensor)
2. When additional vulnerability sensors are onboarded → gets multi-sensor results
3. Wave F3 → schemas reconcile, no column-name confusion
4. Wave F4 → pushdown works correctly across sensors
5. Wave F5 → optional dedup, cleaner result sets

---

## H. Open Questions for Human / Product Owner

1. **Scope boundary for "VULNERABILITIES":** Should `VULNERABILITIES` map only to tables
   with `ocsf_class = "vulnerability_finding"`, or also to `detection_finding` tables that
   carry CVE-related fields (e.g., Armis device vulnerability metadata in
   `armis_devices.vulnerabilities.first_cve_id`)? The OCSF `device` class carries
   vulnerability sub-objects — these are column sub-fields, not separate tables.

2. **NVD as a query-able sensor vs. enrichment-only:** NVD is currently wired as a global
   enrichment source (for `| enrich nvd(cve_id)`), not as a queryable sensor table. Should
   NVD get a `nvd_cves` sensor TOML spec so `SELECT * FROM nvd_cves` works? This requires a
   DTU clone route and spec, and NVD has different pagination semantics (REST bulk download
   vs. API query).

3. **Schema unification strategy:** Canonical OCSF projection (Option A) vs. schema-merge
   (Option B). Option A requires that all sensors for a given OCSF class expose a consistent
   Tier-1 field set, which may require sensor TOML amendments when new sensors are onboarded.
   Option B is more permissive but produces wider result schemas. Which is preferable for the
   analyst UX?

4. **`UNION ALL` in PrismQL:** Should PrismQL support explicit `UNION ALL` as an interim path
   for analysts who want to query two specific sensor tables before full federation is built?
   This is a grammar + DataFusion change, independent of the routing registry.

5. **ADR-058 completion timeline:** The `S-ADR058-OCSF-ROUTING-001` story is parked (worktree
   exists, pre-TDD). Federation Wave F1 depends on it. What is the current sequencing priority
   — does OCSF routing unblock beta.3 or is it post-beta.3?

6. **Dedup identity for vulnerabilities:** Cross-sensor vulnerability dedup requires a
   canonical identity key. Is `CVE-ID` the primary key, or `(CVE-ID, affected-device-id)`?
   Armis surface vulnerability IDs per device; Claroty surfaces them per asset. The dedup
   key affects how Wave F5 is designed.

7. **Partial-failure semantics for concept queries:** If `FROM VULNERABILITIES` fans out to
   three sensors and two are down, should the response include the one sensor's data with
   `sensor_errors` for the two failures (current BC-2.01.010 behavior), or should a
   majority-failure threshold exist? The current BC-2.01.010 is "at least one success = return
   partial data." Is that the right policy for concept-level queries where the analyst expects
   all sources?
