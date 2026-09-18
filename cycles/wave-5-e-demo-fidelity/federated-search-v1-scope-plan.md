---
document_type: architecture-design-and-scope-plan
title: "Federated Search v1 Scope Plan — TRUE CROSS-SENSOR FEDERATION (Claroty xDome + CrowdStrike)"
version: "1.0"
created: 2026-09-18
authors: [architect]
status: draft-pending-human-approval
traces_to: ".factory/specs/architecture/ARCH-INDEX.md"
scope: v1-scope-amendment — PENDING HUMAN RATIFICATION of D-2264 amendment
cycle: wave-5-e-demo-fidelity
inputs:
  - ".factory/cycles/wave-5-e-demo-fidelity/federated-search-analysis.md"
  - ".factory/STATE.md"
  - "crates/prism-sensors/specs/crowdstrike.sensor.toml"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
  - "crates/prism-query/src/ast.rs"
  - "crates/prism-query/src/materialization.rs"
  - "crates/prism-sensors/src/fanout.rs"
input-hash: "ca3e89c"
---

# Federated Search v1 Scope Plan
## TRUE CROSS-SENSOR FEDERATION — Claroty xDome + CrowdStrike

**Status:** DESIGN + SCOPE-PLAN for human approval. No ADRs, BCs, or stories are ratified
herein. All story names and effort estimates are proposals. D-2264 governing decision
(Claroty-only v1) is UNCHANGED until human ratification.

**Litmus test (from D-2569):** A SOC analyst or LLM agent asks for "all vulns" —
`FROM VULNERABILITIES` — and gets ONE unified result set unioned across BOTH Claroty xDome
and CrowdStrike, OCSF-normalized, deduped, with `_sensor`/`_client` provenance.

---

## 1. PROPOSED REVISED v1 OBJECTIVE

**Proposed redline to D-2264 (2026-08-21 "Claroty-only" governing decision):**

| | Current D-2264 | Proposed Amendment |
|---|---|---|
| **Sensor scope** | Claroty xDome only | Claroty xDome + CrowdStrike Falcon (federated) |
| **Query model** | Named-table (`FROM claroty_vulnerabilities`) | Concept-addressable (`FROM VULNERABILITIES`) + named-table continues to work |
| **Result model** | Single-sensor, multi-tenant fan-out | Cross-sensor, multi-tenant fan-out with OCSF-unified result set |
| **Live validation bar** | Single Claroty xDome tenant (real API) | DUAL TENANT — live Claroty xDome + live CrowdStrike (real APIs; AD-017 opaque credentials) |
| **Provenance** | `_sensor`/`_client` virtual fields (BC-2.11.012) — already supported | Same, now carrying both sensor labels in same result set |
| **Dedup** | N/A (single sensor) | Cross-sensor dedup by OCSF canonical identifier (`finding_info_uid` / CVE-ID) |

**Framing:** This amendment moves the v1 release gate from "Claroty works end-to-end" to
"Claroty + CrowdStrike federate correctly under a concept-level query, validated live."

**CRITICAL PREREQUISITE FINDING (discovered during this analysis — see Section 3):**
CrowdStrike Falcon does NOT currently have a `vulnerability_finding` table in
`crowdstrike.sensor.toml` and the `prism-dtu-crowdstrike` crate has no Spotlight route.
CrowdStrike Spotlight is the CrowdStrike vulnerability surface. Before the "all vulns"
litmus test can run with CrowdStrike as the second sensor, a new `crowdstrike_vulnerabilities`
table (CrowdStrike Spotlight) must be added to the TOML and a DTU Spotlight route must be
implemented. This is **net-new work** that is NOT in the current beta.3 batch and must be
scoped explicitly.

**Alternative "all vulns" path (lower risk):** If Spotlight work is deferred, the
`FROM VULNERABILITIES` concept query routes only to Claroty (only sensor with
`ocsf_class = "vulnerability_finding"`). This satisfies the routing registry + composite
execution architecture but does NOT satisfy the "cross-sensor" bar. It is a step, not the
destination. The human must decide whether this is acceptable for v1 or whether Spotlight
is required.

**Alternative "all detections" path (no new DTU work):** Both Claroty (`claroty_alerts`)
and CrowdStrike (`crowdstrike_detections`) already declare `ocsf_class = "detection_finding"`.
A `FROM DETECTIONS` concept query would immediately federate two sensors without any new
DTU route. This can serve as the initial "federation proof of concept" with less risk. The
human must decide whether DETECTIONS or VULNERABILITIES is the v1 litmus test.

---

## 2. DESIGN — THREE STACKED LAYERS

### Layer 1: Concept-Addressable Querying

**Goal:** `FROM VULNERABILITIES` (or `FROM DETECTIONS`, etc.) resolves at query-planning time
to the set of concrete sensor tables that declare the matching OCSF class.

#### 2.1.1 OCSF-Class Routing Registry

**Current (verified):**
- `TableSpec.ocsf_class: String` field exists in `prism_spec_engine::spec_parser::TableSpec`
  (declared at spec parse time; holds values like `"vulnerability_finding"`,
  `"detection_finding"`, `"device"`).
- No reverse index exists: there is no function in `prism_spec_engine` or `prism_query`
  that accepts an OCSF class string and returns the set of `(SensorId, table_name)` pairs
  that declare it.
- The `TableRegistry` in `prism_query::table_registry` indexes tables by `(SensorId, table_name)`
  for availability gate checks but has no `by_ocsf_class` query path.

**Proposed:**
A new `OcsfClassRegistry` component (in `prism_spec_engine` or `prism_query::table_registry`,
to be decided by ADR) that:
1. Is populated at spec-load time by iterating all loaded `SensorSpec` → `TableSpec` entries.
2. Builds a `HashMap<String, Vec<(SensorId, String)>>` mapping OCSF class string →
   `[(sensor_id, table_name)]` — the reverse index.
3. Exposes a query method `fn tables_for_ocsf_class(&self, class: &str) -> &[(SensorId, String)]`.
4. Is updated on sensor hot-reload (when a new sensor spec is added at runtime).

**Concept-alias enum:**
A new `ConceptAlias` enum (in `prism_query::ast` alongside `CompositeSource`) maps
human-readable concept keywords to OCSF class strings:

| Alias keyword | OCSF class string | Current registered tables |
|---|---|---|
| `VULNERABILITIES` | `vulnerability_finding` | `claroty_vulnerabilities` (+ proposed `crowdstrike_vulnerabilities`) |
| `DETECTIONS` | `detection_finding` | `claroty_alerts`, `crowdstrike_detections` |
| `INCIDENTS` | `incident_finding` | `crowdstrike_incidents` (retirement-pending per TOML) |
| `DEVICES` | `inventory_info` / `device` | `claroty_devices`, `crowdstrike_devices`, `armis_devices` |

Note: `DEVICES` maps to multiple OCSF classes (Claroty uses `inventory_info` per ADR-058 §K2
pending correction; CrowdStrike uses `device`). The v1 scope should restrict concept aliases
to unambiguous single-class mappings; `VULNERABILITIES` and `DETECTIONS` qualify.

#### 2.1.2 Grammar Extension

**Current:** `SourceRefKind::classify` in `prism_query::ast` recognizes five composite
keywords (`EVENTS`, `ALERTS`, `DEVICES`, `ASSETS`, `SESSIONS`). `VULNERABILITIES` is not
recognized — it falls through to `SourceRefKind::Custom` → E-QUERY-037 (TableNotAvailable).

**Proposed:** Two options:
- **Option A:** Add `VULNERABILITIES` and `DETECTIONS` to the existing `CompositeSource` enum
  as new variants. Minimal grammar change; reuses the existing recognition path.
- **Option B:** Add a new `SourceRefKind::ConceptAlias(ConceptAlias)` variant recognized
  before the composite/custom classification. Cleaner separation from the "structural"
  composite sources (EVENTS, ALERTS, etc.) which are undefined cross-sensor groupings.

Option B is preferred: the existing composite sources have no execution path and carry
legacy design debt; adding VULNERABILITIES and DETECTIONS alongside them conflates a clean
new design with an unimplemented stub. ADR needed to ratify this choice.

#### 2.1.3 Composite-Source Execution in `resolve_source_refs`

**Current (verified):** `resolve_source_refs` in `prism_query::materialization` passes
composite-source raw strings (e.g. `"EVENTS"`) to `sensor_id_from_table_name`, which fails
because `"events"` is not a registered sensor prefix → E-QUERY-036. No expansion path exists.

**Proposed:** Before the `sensor_id_from_table_name` call, add a new branch:
1. Check if the source name matches a `ConceptAlias` keyword (or existing `Composite` variant
   that has an OCSF class mapping).
2. If yes, call `OcsfClassRegistry::tables_for_ocsf_class(ocsf_class)` to obtain the
   concrete `(sensor_id, table_name)` pairs.
3. For each pair, run the existing `adapter_registry.get_all_for_sensor(&sensor_id)` fan-out
   to get per-org targets (same logic as single-sensor path).
4. Synthesize `FanOutTarget` entries for each `(OrgId, SensorId, table_name)` combination.
5. Union the per-concept targets into the output Vec<FanOutTarget>.

This is an additive change to `resolve_source_refs` — the existing single-sensor code path
is untouched for named-table queries.

**Plan-gate impact:** The table availability gate (`check_table_availability`) already
skip-passes composite sources. It will need to either (a) continue to skip concept aliases,
or (b) validate that at least one registered table exists for the concept. Option (b) is
preferred for production-grade error messaging (if no sensor is configured for
`vulnerability_finding`, return a clear error rather than an empty result). ADR needed.

---

### Layer 2: Cross-Sensor Fan-Out

**Goal:** Extend fan-out from "many tenants of ONE sensor" to "many DIFFERENT sensors for
one concept."

#### 2.2.1 Current Fan-Out Model (verified)

`prism_sensors::fanout::fan_out` (BC-2.01.002) accepts a `Vec<FanOutTarget>` where each
target is a `(OrgId, SensorId, table_name, query_params, ...)` tuple. It:
1. Spawns one Tokio task per target.
2. Caps concurrency via `MAX_FANOUT_CONCURRENCY = 10` (constant in `prism_sensors::fanout`).
3. Collects successes and partial failures into a `FanOutResult` (per BC-2.01.010).

The fan-out machinery is **sensor-type-agnostic** — it does not care whether all targets are
the same sensor type. It routes each target to `dispatch_by_table_type`, which calls the
per-target adapter. This means **the fan-out layer itself requires no changes for cross-sensor
federation** — once `resolve_source_refs` produces multi-sensor `FanOutTarget` entries, the
fan-out will execute them correctly.

**What changes:**
- `resolve_source_refs` produces targets from multiple sensor types (see Layer 1).
- Each `FanOutTarget` carries its own `sensor_id` and `table_name`, which the adapter
  registry uses to dispatch to the correct sensor adapter.

#### 2.2.2 Concurrency Accounting

`MAX_FANOUT_CONCURRENCY = 10` applies per-query. A concept query against 2 sensors × 3
tenants each = 6 targets, well within the 10-target cap. No change to the semaphore value
is needed for the initial 2-sensor v1 scope. The behavior is correct as-is per BC-2.01.002.

#### 2.2.3 Partial Failure Semantics (BC-2.01.010)

`FanOutResult::errors` already propagates per-target failures. For cross-sensor queries:
- If Claroty returns data and CrowdStrike API is down, the result includes Claroty data +
  `sensor_errors` entry for CrowdStrike. This is existing behavior, no change required.
- The `sensor_errors` field in `QueryResult` surfaces to the MCP caller already.

**Only the label "sensor" becomes accurate** — previously all errors were from one sensor
type; now `sensor_errors` may contain entries from different sensor types. No code change
needed; the field is already per-`(OrgId, SensorId)`.

---

### Layer 3: Cross-Sensor Schema Unification, Dedup, Pagination, Provenance

**Goal:** Merge heterogeneous Arrow RecordBatches from multiple sensor types into a single
coherent result set.

#### 2.3.1 Schema Unification

**Current:** `pipeline_result_to_record_batch` in `prism_bin::spec_driven_adapter` produces
an Arrow `RecordBatch` with field names determined by each sensor's TOML column declarations
(or OCSF-flattened names when `ocsf_column_naming = true`). Two sensors declaring the same
OCSF class but with `ocsf_column_naming` set differently produce different Arrow schemas.

**Current status:**
- Claroty: `ocsf_column_naming = true` (ADR-058 enabled)
- CrowdStrike: `ocsf_column_naming` NOT set in `crowdstrike.sensor.toml` (no field present)
- This means Claroty outputs OCSF-flattened column names; CrowdStrike outputs native column
  names (`detection_id`, `severity`, `tactic`, etc.)

**Proposed — Option A (Canonical OCSF Projection):**
Before result set merge, project each sensor's RecordBatch to the canonical Arrow schema for
its OCSF class. The canonical schema is defined by the Tier-1 fields for that class
(as declared in ADR-058's Tier-1 OCSF field taxonomy). Absent columns fill with null. This
produces identical Arrow schemas for all sensors sharing an OCSF class.

**Prerequisites for Option A:**
1. ADR-058 `ocsf_column_naming = true` must be activated for ALL participating sensors, not
   just Claroty. CrowdStrike's `crowdstrike.sensor.toml` needs `ocsf_column_naming = true`
   added and the `ocsf_field` annotations on its columns verified complete. Current
   crowdstrike.sensor.toml already has `ocsf_field` annotations on all declared columns
   (`finding.uid`, `time`, `status`, `severity`, `device.uid`, `attack.tactic.name`,
   `attack.technique.name`). Adding the flag is low-risk but requires a story.
2. A canonical Arrow schema registry per OCSF class. This can be derived from the
   `OcsfClassRegistry` (Layer 1): the union of all Tier-1 OCSF fields declared across all
   sensors for a given class.
3. A `project_to_canonical_schema` step inserted after per-sensor materialization and before
   DataFusion session context registration (`run_materialization_pipeline` Step 4).

**Alternative — Option B (Schema Merge):**
Arrow provides a `SchemaRef` merge; DataFusion supports heterogeneous schemas in a UNION
via null-filling. This is simpler to implement but produces wider, messier schemas for the
analyst. Option A aligns with OCSF's explicit class contract and is the production-grade
default.

#### 2.3.2 Dedup / Entity Resolution

**Current:** No cross-sensor dedup exists. The `| dedup` pipe stage operates on a named
column within a single-sensor result. After schema unification (2.3.1), `finding_info_uid`
will be a unified column name for `vulnerability_finding` class results.

**Proposed:**
After schema unification, insert an optional `DEDUP BY finding_info_uid` post-processing
step in the DataFusion query. This is implementable as a DataFusion window function
(`ROW_NUMBER() OVER (PARTITION BY finding_info_uid ORDER BY time DESC)` → keep first row
per uid). For the vulnerability use case, `finding_info_uid` corresponds to a CVE-ID or
vendor-specific finding identifier mapped via `ocsf_field = "finding.uid"`.

**V1 dedup identity for vulnerabilities:**
- Claroty `claroty_vulnerabilities` maps `vulnerability_id` → `finding.uid`
- CrowdStrike Spotlight (proposed) would need a `spotlight_id` or CVE-ID field mapped
  to `finding.uid` in the TOML.
- True cross-CVE dedup (same CVE on same device across sensors) requires a two-level key:
  `(finding_info_uid, device_uid)`. This is v2 complexity; v1 dedup can be single-key
  `finding_info_uid` with documented limitation.

#### 2.3.3 Federated Pagination

**Current:** `PaginationCursor` (BC-2.11.001) encodes per-table page state for a single
sensor. The cursor is opaque bytes carrying `(sensor_id, table_name, offset/token)`.

**Cross-sensor federation impact:** A concept query returns a merged result set; the cursor
must encode page state for EACH sensor that contributed results. On `get_next_page`, each
sensor must be advanced independently and results re-merged.

**Proposed — v1 scope:**
A federated cursor encodes a `Vec<PerSensorCursorState>` where each entry is
`(sensor_id, table_name, offset/token)`. The `get_next_page` handler unpacks this,
re-invokes fan-out for each sensor from its saved offset, and re-runs schema unification.

**Complexity note:** This requires a BC amendment to BC-2.11.001 (new cursor encoding),
a new story, and a new holdout scenario. This is MEDIUM complexity but non-trivial because
cursor encoding is a serialized wire format — it must remain backward-compatible with
single-sensor cursors. Design decision: single-sensor cursors remain unchanged; only
concept-query cursors use the federated encoding (distinguished by a version byte prefix).

**V1 simplification:** Federated pagination can be deferred to "all results in one page"
(no cursor) for the initial v1 demo if the result count is small. This is acceptable for
a demo validation but NOT production-grade for large vuln inventories. The production-grade
default requires federated cursor support before v1 ships.

#### 2.3.4 Provenance (`_sensor`, `_client`)

**Current (verified):** Virtual fields `_sensor` and `_client` are injected by
`prism_query::materialization` post-fan-out (BC-2.11.012). Each `FanOutTarget` carries
its `sensor_id` and `org_id`, which are stamped as Arrow column values on the result rows.

**Cross-sensor federation:** This already works correctly — each target's rows get their
own `_sensor` value. Schema unification (Layer 3) must preserve these virtual columns.
The canonical OCSF projection step (Option A) must explicitly carry `_sensor` and `_client`
forward as non-OCSF metadata columns. No structural change needed; care required in the
schema unification implementation.

---

## 3. CROWDSTRIKE v1-GRADE READINESS ASSESSMENT

### 3.1 CrowdStrike Spotlight — THE CRITICAL GAP

**Finding: CrowdStrike has NO vulnerability_finding table today.**

Verified from `crates/prism-sensors/specs/crowdstrike.sensor.toml`:
- Table 1: `detections` — `ocsf_class = "detection_finding"` (IMPLEMENTED, DTU route exists)
- Table 2: `devices` — `ocsf_class = "device"` (IMPLEMENTED, DTU route exists)
- Table 3: `incidents` — `ocsf_class = "incident_finding"` (RETIREMENT-PENDING; DTU gap noted in TOML)

**No Spotlight table exists.** `crates/prism-dtu-crowdstrike/src/routes/` contains only:
`detections.rs`, `hosts.rs`, `oauth.rs`, `writes.rs` — NO spotlight route.

**CrowdStrike Spotlight API facts (for ADR):**
CrowdStrike Falcon Spotlight exposes vulnerability data via:
- `GET /spotlight/queries/vulnerabilities/v1` — returns vulnerability IDs
- `GET /spotlight/entities/vulnerabilities/GET/v1` — returns full vulnerability records
This is the same 2-step QueryV2 → GetEntities pattern already used by `detections` and
`devices` in the current TOML. The Spotlight API requires a tenant with the Spotlight
module licensed.

**What's needed to add CrowdStrike as a vulnerability_finding sensor:**
1. New `[[tables]]` block in `crowdstrike.sensor.toml`: `table_name = "vulnerabilities"`,
   `ocsf_class = "vulnerability_finding"`, `ocsf_column_naming = true`, with OCSF-mapped
   columns for Spotlight fields (`cve.uid`, `severity`, `finding.uid`, `status`,
   `device.uid`, `time`).
2. New DTU route in `prism-dtu-crowdstrike/src/routes/spotlight.rs` implementing:
   - `GET /spotlight/queries/vulnerabilities/v1` (query IDs)
   - `GET /spotlight/entities/vulnerabilities/GET/v1` (fetch records)
   with fixtures for testing.
3. DTU `mod.rs` and router registration for the new route.
4. TOML steps declaration for the 2-step pipeline.
5. Holdout scenarios against the live CrowdStrike Spotlight API (AD-017 opaque credentials).

**Effort estimate:** 1 story, 3–5 story points. Analogous to `S-CLAROTY-DEVVULNREL-001` +
`S-CLAROTY-DEVVULNREL-DTU-001` (claroty device-vulnerability-relations) but for CrowdStrike
Spotlight. This is REQUIRED before `FROM VULNERABILITIES` can federate across both sensors.

### 3.2 CrowdStrike ocsf_column_naming Status

`crowdstrike.sensor.toml` does NOT have `ocsf_column_naming = true`. Adding this flag:
- Activates OCSF-flattened column names for all CrowdStrike tables.
- Requires that all `ocsf_field` annotations are correct and complete.
- Current `detections` table has `ocsf_field` annotations on 7 columns; the behaviors_ioc_*
  columns use `source_path` rather than `ocsf_field` (vendor-specific, no OCSF mapping).
- This must be done BEFORE schema unification can work for cross-sensor detection queries.
- This is 1 story, 2–3 points (TOML amendment + test coverage).

### 3.3 CrowdStrike Multi-Region DTU

Story `S-DEMO-CROWDSTRIKE-MULTIREGION-001` (status: from stories directory) addresses
multi-region `base_url` support. This is compatible with and independent of the Spotlight
addition. The DTU and production adapter already support `${env.CROWDSTRIKE_BASE_URL}`
variable substitution per `crowdstrike.sensor.toml` line `base_url = "${env.CROWDSTRIKE_BASE_URL}"`.

### 3.4 Live-Tenant Validation Feasibility

- Human confirms live CrowdStrike tenant access (D-2569).
- AD-017 opaque credential model: `CROWDSTRIKE_CLIENT_ID` + `CROWDSTRIKE_CLIENT_SECRET`
  env vars already referenced in `crowdstrike.sensor.toml` `[[credential_refs]]`. The
  OAuth2 plugin (`crowdstrike-oauth2`) is wired for production use.
- Multi-tenant: CrowdStrike uses per-region base URL; human operator sets `CROWDSTRIKE_BASE_URL`
  per tenant config.
- Holdout scenarios must be authored for live CrowdStrike Spotlight (same holdout-evaluator
  pattern as Claroty live validation). Scenarios: (a) Spotlight query returns N vulnerabilities
  with `_sensor = "crowdstrike"`; (b) dedup by `finding_info_uid` removes duplicates present
  in both sensors; (c) partial failure (Claroty down) returns CrowdStrike data + sensor_error;
  (d) concept query `FROM VULNERABILITIES` returns rows from both sensors in single result set.

---

## 4. "ALL VULNS" END-TO-END TRACE (PROPOSED DESIGN)

**Query:** `SELECT * FROM VULNERABILITIES WHERE severity_id >= 7`

1. **Parse:** `PrismQlParser::parse` processes the query. Source `VULNERABILITIES` is
   classified by `SourceRefKind::classify` → NEW: `SourceRefKind::ConceptAlias(ConceptAlias::Vulnerabilities)`.

2. **Plan gate:** Table availability gate in `table_registry::TableRegistry::check_availability_gate`
   — NEW: for `ConceptAlias`, check that at least one sensor table declares `vulnerability_finding`
   in the `OcsfClassRegistry`. If no table registered → NEW error E-QUERY-NNN
   "NoSensorsForConcept". If tables registered → pass-through.

3. **Column/type gates:** NEW: for concept-alias sources, column gate validates against the
   canonical OCSF Tier-1 field set for `vulnerability_finding` (not any single sensor's schema).
   `severity_id` is a Tier-1 OCSF field for `vulnerability_finding` → passes.

4. **Source name extraction:** `extract_source_names` returns `"VULNERABILITIES"` as raw
   source string. `resolve_source_refs` receives it.

5. **Concept expansion in `resolve_source_refs`:** NEW branch detects `"VULNERABILITIES"` →
   `ConceptAlias::Vulnerabilities` → `OcsfClassRegistry::tables_for_ocsf_class("vulnerability_finding")`
   → `[("claroty", "vulnerabilities"), ("crowdstrike", "vulnerabilities")]`.

6. **Fan-out target synthesis:** For each `(sensor_id, table_name)` pair, call
   `adapter_registry.get_all_for_sensor(&sensor_id)` → one `FanOutTarget` per `(OrgId, SensorId)`.
   For 1 Claroty org + 1 CrowdStrike org: 2 FanOutTargets total.

7. **Predicate pushdown:** NEW: ADR-033 time-window extraction is sensor-agnostic (extracts
   `WHERE time >` / `<` predicates). The `severity_id >= 7` predicate is an OCSF Tier-1 field.
   Per-sensor pushdown translation: Claroty maps `severity_id` to its native filter parameter
   (if declared in TOML pushdown hints); CrowdStrike Spotlight does the same. For v1, field-name
   pushdown can be deferred — the `WHERE severity_id >= 7` applies post-fetch in DataFusion.
   The v1 pushdown scope is time-window only (existing ADR-033 behavior).

8. **Fan-out execution:** `prism_sensors::fanout::fan_out` spawns 2 async tasks (one per
   target). Concurrency cap `MAX_FANOUT_CONCURRENCY = 10` — 2 targets trivially within cap.
   Each task calls the appropriate sensor adapter (Claroty → xDome API; CrowdStrike →
   Spotlight API). Partial failures handled by `FanOutResult` per BC-2.01.010.

9. **Per-sensor OCSF normalization:** Each sensor's raw JSON response is processed by
   `pipeline_result_to_record_batch` in `prism_bin::spec_driven_adapter`. With
   `ocsf_column_naming = true` on both sensors (post Layer 3 prerequisite), both produce
   OCSF-flattened Arrow column names. Claroty outputs `finding_info_uid`, `severity_id`, etc.
   CrowdStrike outputs the same names from its `ocsf_field` annotations.

10. **Schema unification:** NEW `project_to_canonical_schema` step projects both RecordBatches
    to the canonical `vulnerability_finding` Arrow schema (Tier-1 OCSF fields + virtual columns
    `_sensor`, `_client`). Absent columns fill with Arrow null. Both batches now share an
    identical Arrow schema.

11. **DataFusion registration:** Both RecordBatches registered into the DataFusion session
    context as the same logical table (source name `"VULNERABILITIES"`). DataFusion sees a
    single unified table with rows from both sensors.

12. **DataFusion query execution:** `WHERE severity_id >= 7` applied. SELECT * returns all
    matching rows.

13. **Post-DataFusion dedup (optional):** `DEDUP BY finding_info_uid` step removes rows where
    the same CVE appears in both sensor feeds. Opt-in for v1; required before production.

14. **Pagination cursor:** If result exceeds page limit, NEW federated cursor encodes
    `[(claroty, vulnerabilities, offset_A), (crowdstrike, vulnerabilities, offset_B)]`.

15. **Wire shape:** MCP `query` tool response carries rows with `finding_info_uid`, `severity_id`,
    `_sensor` (`"claroty"` or `"crowdstrike"`), `_client` (org ID), and other OCSF Tier-1 fields.
    Wire-shape assertions per CLAUDE.md wire-shape assertion discipline validate the serialized
    JSON output.

---

## 5. REVISED RELEASE GATE (DUAL-LIVE-TENANT VALIDATION)

**Gate replaces:** Current D-2264 gate — "live Claroty xDome validation."

**Proposed v1 dual-tenant gate — ALL must pass:**

| # | Gate | Validation Method |
|---|---|---|
| G1 | `SELECT * FROM claroty_vulnerabilities LIMIT 10` returns rows with OCSF-normalized field names (`finding_info_uid`, `severity_id`) from live Claroty xDome tenant | Holdout evaluator vs real xDome API; wire-level JSON assertion |
| G2 | `SELECT * FROM crowdstrike_vulnerabilities LIMIT 10` returns rows with OCSF-normalized field names from live CrowdStrike Spotlight tenant | Holdout evaluator vs real CrowdStrike API; wire-level JSON assertion |
| G3 | `SELECT * FROM VULNERABILITIES LIMIT 20` returns rows from BOTH sensors in a single result set with `_sensor` provenance correctly labeled | Holdout evaluator; asserts rows present from both sensor labels |
| G4 | `SELECT * FROM VULNERABILITIES LIMIT 20` with one sensor deliberately returning an error — other sensor's rows returned with `sensor_errors` field populated | Holdout evaluator; partial-failure scenario |
| G5 | `SELECT * FROM VULNERABILITIES WHERE severity_id >= 7 LIMIT 10` — `severity_id` filtering post-unification returns correct subset | Holdout evaluator; arithmetic assertion |
| G6 | `SELECT * FROM DETECTIONS LIMIT 20` — CrowdStrike detections + Claroty alerts in single result set (the "free" cross-sensor federation with no new DTU work) | Holdout evaluator; additional confidence gate |
| G7 | SOC analyst natural-language Q&A scenario: LLM agent invokes `query` tool with `FROM VULNERABILITIES`, receives unified result, generates coherent analysis | Human-observed demo; not automated |

**Credentials:** AD-017 opaque — values never transit AI context. Separate CI environment
with real-API holdout scenarios runs under human-managed credential injection (same pattern
as Claroty live validation).

---

## 6. REUSE MAP

### Unchanged (carry over from current architecture):

| Component | Status | Notes |
|---|---|---|
| `prism_sensors::fanout::fan_out` + `MAX_FANOUT_CONCURRENCY` + `FanOutResult` | **UNCHANGED** | Fan-out machinery is sensor-agnostic; multi-sensor targets already work |
| `FanOutTarget` struct | **UNCHANGED** | Already carries `(OrgId, SensorId, table_name, query_params)` |
| BC-2.01.010 partial-failure semantics | **UNCHANGED** | `sensor_errors` propagation already works |
| BC-2.11.012 `_sensor`/`_client` provenance injection | **UNCHANGED** | Virtual field stamping is per-target; works for multi-sensor |
| `prism_spec_engine::spec_parser::TableSpec.ocsf_class` | **UNCHANGED** | Already stores the class string; just needs a reverse index built from it |
| ADR-033 time-window predicate extraction | **UNCHANGED** | Sensor-agnostic pushdown; applies per-target |
| `claroty.sensor.toml` + all Claroty DTU routes | **UNCHANGED** | Claroty work is complete (merged stories) |
| `crowdstrike.sensor.toml` detections/devices tables | **UNCHANGED** | Only the new `vulnerabilities` table is additive |
| `prism-dtu-crowdstrike` detections/hosts routes | **UNCHANGED** | Only a new `spotlight.rs` route is added |
| OAuth2 auth plugin for CrowdStrike | **UNCHANGED** | Already production-wired |

### Modified (extension only — no existing API surface broken):

| Component | Change | Risk |
|---|---|---|
| `prism_query::ast::SourceRefKind` | New variant `ConceptAlias(ConceptAlias)` + new `ConceptAlias` enum | LOW — additive; existing `Composite`, `Custom`, `Internal`, `External` variants unchanged |
| `prism_query::ast::SourceRefKind::classify` | New match arm before existing composite check | LOW — additive branch |
| `prism_query::materialization::resolve_source_refs` | New concept-expansion branch before `sensor_id_from_table_name` | MEDIUM — core hot path; requires thorough test coverage and 3-CLEAN adversarial pass |
| `prism_query::table_registry::TableRegistry` | Add `OcsfClassRegistry` sub-component (populated at spec-load) | LOW — additive data structure |
| `prism_bin::spec_driven_adapter::pipeline_result_to_record_batch` | Add `project_to_canonical_schema` step (new post-materialization stage) | MEDIUM — touches Arrow schema handling; requires SAP-2 verification |
| `crowdstrike.sensor.toml` | Add `ocsf_column_naming = true` + new `[[tables]]` Spotlight block | LOW-MEDIUM — TOML change; full SAP-2 compliance required |

### Net-new (no reuse):

| Component | Description | Risk |
|---|---|---|
| `ConceptAlias` enum | New type in `prism_query::ast` | LOW |
| `OcsfClassRegistry` | New struct + reverse-index builder | LOW |
| `project_to_canonical_schema` | Arrow schema projection step | MEDIUM — Arrow type handling has edge cases |
| Federated `PaginationCursor` encoding | New cursor variant (BC-2.11.001 amendment) | MEDIUM |
| `crowdstrike_vulnerabilities` TOML table | New Spotlight sensor spec | LOW-MEDIUM |
| `prism-dtu-crowdstrike/src/routes/spotlight.rs` | New DTU route for Spotlight API | MEDIUM — new API surface to replicate |
| Canonical OCSF schema registry per class | New data structure in `prism_spec_engine` | LOW |

---

## 7. CANDIDATE EPIC + WAVE/STORY OUTLINE

**Status: PROPOSALS ONLY — no stories created. Names are candidates, not ratified IDs.**

### Epic: E-TRUE-FEDERATION (v1 amendment)

**Hard prerequisite:** Beta.3 Batch-1 (8 remaining stories including S-MCP-TOOL-GATE-001
through S-CLAROTY-OCSF-TOML-001) must COMPLETE before federation work begins. `S-CLAROTY-OCSF-TOML-001`
and `S-CLAROTY-OCSF-STATUS-001` in Batch-1 complete the OCSF normalization foundation that
Layer 3 schema unification depends on.

**Dependency ordering:**

```
[Batch-1 complete (8 stories)]
         ↓
Wave F-0: CrowdStrike OCSF + Spotlight Foundation (unblocks all)
  S-CS-OCSF-NAMING-001        — crowdstrike.sensor.toml ocsf_column_naming=true + field audit
  S-CS-SPOTLIGHT-TOML-001     — crowdstrike_vulnerabilities TOML table (Spotlight API spec)
  S-CS-SPOTLIGHT-DTU-001      — prism-dtu-crowdstrike spotlight route + fixtures
         ↓
Wave F-1: Routing Registry + Grammar
  S-OCSF-CLASS-REGISTRY-001   — OcsfClassRegistry reverse index (spec-engine or table_registry)
  S-CONCEPT-ALIAS-AST-001     — ConceptAlias enum + SourceRefKind variant + classify() extension
         ↓
Wave F-2: Composite Source Execution
  S-CONCEPT-SOURCE-EXEC-001   — resolve_source_refs concept-expansion branch + plan gate error
         ↓
Wave F-3: Schema Unification
  S-OCSF-CANONICAL-SCHEMA-001 — OcsfClassRegistry canonical schema builder + project_to_canonical
  S-CS-OCSF-ACTIVATE-001      — Activate ocsf_column_naming on CrowdStrike + integration tests
         ↓
Wave F-4: Federated Pagination + Dedup
  S-FEDERATED-CURSOR-001      — Federated PaginationCursor encoding (BC-2.11.001 amendment)
  S-CROSS-SENSOR-DEDUP-001    — DEDUP BY finding_info_uid post-unification step
         ↓
Wave F-5: Live Dual-Tenant Validation + Release Gate
  S-LIVE-DUAL-TENANT-001      — Holdout scenarios G1–G7 vs real APIs; release gate documentation
```

**Relation to current beta.3 Batch-1:**
- Batch-1 continues UNAFFECTED in parallel (as directed in D-2569).
- `S-CLAROTY-OCSF-STATUS-001` (story 9/10) and `S-CLAROTY-OCSF-TOML-001` (story 10/10 of
  non-facade stories) complete the Claroty OCSF normalization. These are PREREQUISITES for
  federation schema unification but are already in the batch.
- No federation stories start until Batch-1 closes. No story materialization before
  D-2264 is amended (pending human ratification of this plan).

**Story count estimate: 11 new stories** (Wave F-0: 3, Wave F-1: 2, Wave F-2: 1, Wave F-3: 2,
Wave F-4: 2, Wave F-5: 1). These are distinct from the existing beta.3 stories.

---

## 8. EFFORT / RISK / TIMELINE

### Effort Estimates (story points, same scale as current project)

| Wave | Stories | Points | Complexity |
|---|---|---|---|
| F-0: CrowdStrike Spotlight Foundation | 3 | 8–10 | LOW-MEDIUM (DTU work is well-understood pattern) |
| F-1: Routing Registry + Grammar | 2 | 5–7 | LOW (additive AST + data structure) |
| F-2: Composite Source Execution | 1 | 5–8 | MEDIUM-HIGH (core hot path; adversarial cascade required) |
| F-3: Schema Unification | 2 | 8–10 | HIGH (Arrow schema projection; correctness-critical) |
| F-4: Federated Pagination + Dedup | 2 | 6–8 | MEDIUM (cursor encoding is delicate; dedup is additive) |
| F-5: Live Dual-Tenant Validation | 1 | 3–5 | MEDIUM (holdout authoring + live API access) |
| **Total** | **11** | **35–48** | |

**Comparison to current batch:**
Batch-1 is 10 stories at roughly 25–35 points total (estimated from story points in
`S-MCP-TOOL-GATE-001` 5pt, `S-JSON-EXTRACT-UDF-001` 5pt, etc.). Federation adds
approximately 1.5–1.8× the remaining batch-1 effort on top of the post-beta.3 schedule.

### Impact on v1 Release Date

**Current trajectory (Claroty-only v1):**
Batch-1: ~8 weeks at current pace (2–3 stories/week) → beta.3 closure early Nov 2026
→ v1.0.0 shortly after.

**Proposed trajectory (federation v1):**
Batch-1 first (unchanged): ~8 weeks → beta.3 closure early Nov 2026
Federation waves (sequential): 11 stories at 3–4 stories/wave with 3-CLEAN adversarial
cascades → ~4–6 weeks of additional work → v1.0.0 mid-to-late Dec 2026.

**Timeline impact: approximately 6–10 weeks extension to v1 release date.**

### Risk Register

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| CrowdStrike Spotlight API structure differs from Detections (3rd endpoint, different pagination) | MEDIUM | HIGH | Research Spotlight API docs before story materialization; S-CS-SPOTLIGHT-TOML-001 includes API research sub-task |
| Schema unification Arrow projection has correctness edge cases (null handling, type coercion) | MEDIUM | HIGH | SAP-2 wire-level assertions required; adversarial pass specifically targeting Arrow null semantics |
| CrowdStrike tenant does not have Spotlight module licensed | LOW-MEDIUM | HIGH (blocks live validation G2/G3/G4) | Verify license scope before committing to Spotlight path; if not licensed, fall back to DETECTIONS federation litmus test |
| ocsf_column_naming=true on CrowdStrike introduces column name regressions in existing tests | LOW-MEDIUM | MEDIUM | CrowdStrike tests are DTU-only; wave F-0 includes a regression sweep |
| Federated cursor encoding breaks existing single-sensor pagination | LOW | HIGH | Version-prefix byte distinguishes single-sensor vs federated cursor; backward-compat by design |
| resolve_source_refs hot-path change introduces regression | MEDIUM | HIGH | Full 3-CLEAN adversarial cascade required for S-CONCEPT-SOURCE-EXEC-001; Red Gate tests include E-QUERY-036 non-regression |

---

## 9. OPEN QUESTIONS FOR HUMAN / PRODUCT-OWNER

**Q1 — Litmus test confirmation:** Is `FROM VULNERABILITIES` (cross-sensor) the mandatory
v1 litmus test, or is `FROM DETECTIONS` (no new DTU work; 2 sensors already have
`detection_finding`) an acceptable initial federation proof of concept? If DETECTIONS is
acceptable first, Wave F-0 (Spotlight) can be deferred to a v1.1 cycle, and the v1 release
gate becomes lighter. **This is the highest-leverage decision point in this plan.**

**Q2 — CrowdStrike Spotlight license:** Does the live CrowdStrike tenant have the Spotlight
module licensed and accessible via the `GET /spotlight/queries/vulnerabilities/v1` endpoint?
If not, the vulnerability federation path is blocked and Q1 becomes the only viable path.

**Q3 — Schema unification strategy:** Canonical OCSF projection (Option A, narrower result,
correct OCSF semantics) vs schema-merge (Option B, wider result, easier to implement)?
Option A is the production-grade default; Option B is explicitly NOT recommended by this plan.

**Q4 — Dedup key for vulnerabilities:** `finding_info_uid` alone (CVE-ID or vendor ID) vs
`(finding_info_uid, device_uid)` (same vulnerability on same device)? The former gives
cross-sensor dedup of the same CVE regardless of device. The latter gives exact-instance
dedup. V1 recommendation: `finding_info_uid` alone with a documented limitation note.

**Q5 — Federated pagination scope for v1:** Full federated cursor (production-grade,
required for large vuln inventories) vs "first page only" simplification for the live demo?
The production-grade default requires the full cursor. The plan proposes full cursor in
Wave F-4. If timeline is critical, the cursor story can be the last one before the release
gate, with the live demo capped at a page size the single-request returns.

**Q6 — Beta.3 Batch-1 sequencing:** Does the human want to keep Batch-1 at current pace
(no interruption) and then start federation, or reprioritize within Batch-1 to move
`S-CLAROTY-OCSF-STATUS-001` and `S-CLAROTY-OCSF-TOML-001` earlier (they are the Batch-1
stories that directly unblock federation)? Currently S-CLAROTY-OCSF-STATUS-001 is story 9/10
in the resume sequence.

**Q7 — D-2264 ratification timing:** This plan is a proposal. When does the human want to
ratify (or amend) D-2264? Story materialization and TDD starts on federation are BLOCKED
until ratification. The human can ratify after reviewing this plan, or request further
changes to the design/scope.

**Q8 — DETECTIONS federation as a parallel confidence gate:** Even under the VULNERABILITIES
path, `FROM DETECTIONS` federation (CrowdStrike + Claroty alerts) is a "free" win once Layer
1 and Layer 2 are built (both sensors already have `detection_finding` tables). Should it be
included as Gate G6 in the release criteria? Plan includes it as such (Section 5, G6).

**Q9 — `ocsf_column_naming = true` for all sensors or just the federation participants?**
The plan proposes enabling it for CrowdStrike in Wave F-0. Armis and Cyberint are de-scoped
post-v1 per D-2443. Should they be amended in Wave F-0 as well (small additional work), or
strictly limited to Claroty + CrowdStrike for v1? Recommendation: limit to Claroty + CrowdStrike
for v1; Armis/Cyberint can be swept in a post-v1 sensor re-activation wave.
