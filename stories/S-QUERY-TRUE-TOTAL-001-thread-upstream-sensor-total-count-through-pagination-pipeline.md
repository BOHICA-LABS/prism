---
document_type: story
story_id: S-QUERY-TRUE-TOTAL-001
title: "Thread upstream sensor total_count through pagination pipeline to total_available"
wave: 2
epic_id: E-BETA3-REMEDIATION
version: "1.2"
status: ready
producer: story-writer
phase: 3
priority: P0
points: 8
tdd_mode: strict
target_module: prism-query
subsystems: ["SS-11", "SS-07"]
# Subsystem anchor justifications:
#   SS-11 (Query Engine) owns crates/prism-query/src/engine.rs and
#   crates/prism-query/src/materialization.rs — the engine Step 6
#   total_available computation and MaterializationOutput struct both
#   live within SS-11's module boundary. The gated formula (RG-QTT-003)
#   is an SS-11 behavioral change.
#
#   SS-07 (Sensor Adapters) owns crates/prism-sensors/src/adapter.rs
#   (FetchOutput), crates/prism-sensors/src/fanout.rs (FanOutResult),
#   crates/prism-sensors/src/pagination.rs (PaginationCursor), and
#   crates/prism-bin/src/spec_driven_adapter.rs (fetch() plumbing).
#   The new upstream_total: Option<usize> field propagates through all
#   SS-07 structs before reaching SS-11 engine Step 6.
#
#   SS-10 (MCP Server) is touched only by the wire-shape test for
#   RG-QTT-004 — not a structural subsystem change, so SS-10 is
#   not listed.
#
#   SS-05 (Spec Engine) is touched for the TableSpec.total_count_path
#   TOML field addition (crates/prism-spec-engine/src/spec_parser.rs).
#   Listed here as implicit via SS-07's dependency on it.
crates_touched:
  - prism-spec-engine
  - prism-sensors
  - prism-bin
  - prism-query
  - prism-mcp
estimated_days: 2.0
depends_on: []
# depends_on anchor justification:
#   No hard compile-time product-story dependencies. Spec pre-work
#   (BC-2.11.001 amendment and ADR-060 §D8.11 addition) is complete
#   per Batch-0 gate (D-2541/checkpoint; BC-2.11.001 v1.38 frozen,
#   ADR-060 v1.28 frozen). Story is unblocked for test-writer dispatch.
#
#   Soft merge-ordering: S-MCP-NULL-ENCODING-001 SHOULD be merged
#   before this story to reduce conflict risk on spec_driven_adapter.rs
#   (both stories touch that file). This is merge-ordering advice,
#   NOT a compile-time dependency — both stories can proceed in parallel
#   in separate worktrees.
blocks:
  - S-BETA3-RELEASE-001
# blocks anchor justification:
#   S-BETA3-RELEASE-001: all W2-arch stories must merge to develop
#   before the beta.3 release bundle dispatches. S-QUERY-TRUE-TOTAL-001
#   is the primary W2-arch story (8 pts; Claroty live-test fix for
#   total_available showing 25 instead of 1,200+ actual devices).
risk: HIGH
# Risk justification:
#   FetchOutput::new currently has ~46 call sites (46 `::new` call sites
#   + 3 FetchOutput struct literals) across prism-mcp/tests (5 files, 8
#   sites), prism-bin/tests (1 file, 4 sites), prism-query/tests (5
#   files, 27 sites — execute_integration_tests.rs alone has 22),
#   prism-query/src/ (materialization.rs production site + inline test
#   module 2 sites), prism-sensors/tests (3 files, 3 sites), and
#   prism-mcp/src/server.rs + prism-bin/src (2 production sites).
#   Three FetchOutput struct literals in prism-sensors/src/fanout.rs
#   and prism-sensors/src/tests/ also need updating. Adding
#   upstream_total: Option<usize> as a new constructor parameter (or
#   struct field with a builder extension) requires a sibling-sweep of
#   all construction sites (TD-VSDD-060). Confirmed count per D-1110
#   grep (`rg 'FetchOutput::new\(' crates/ --type rust | grep -v '///'`). Full `just check` required
#   after struct changes. ADR-060 is a frozen perimeter artifact —
#   amendments required strict adversarial review per BC-5.39.001;
#   those passes are complete at v1.28.
behavioral_contracts:
  - BC-2.11.001
# BC status: ACTIVE v1.38 (frozen per D-2541/checkpoint).
#   BC-2.11.001 v1.38 contains EC-11-095 with 11 MUSTs (MUST-1..11)
#   anchored to S-QUERY-TRUE-TOTAL-001 AC-001..011 + RG-QTT-001..011.
#   Spec-First Gate S-7.01 satisfied: behavioral_contracts is non-empty
#   with canonical BC-S.SS.NNN pattern. Story may be dispatched to
#   test-writer once spec-gate pre-work is confirmed active.
verification_properties: []
assumption_validations: []
risk_mitigations: []
---

# S-QUERY-TRUE-TOTAL-001: Thread Upstream Sensor Total Count Through Pagination Pipeline to total_available

## Authority

**beta3-remediation-delta-analysis.md §Issue 6 + §S-QUERY-TRUE-TOTAL-001** is the authoritative
problem description and design decision for this story. Read Part 1 §Issue 6 and Part 3
§S-QUERY-TRUE-TOTAL-001 in full before implementing.
Path: `.factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md`

**BC-2.11.001 v1.38** (query MCP Tool — §Postconditions Dual-limit-semantics Override;
EC-11-095 true upstream sensor total via `total_count_path`) governs the full behavioral
contract for this story. The 11 MUSTs in EC-11-095 are the authoritative acceptance criteria.
Path: `.factory/specs/behavioral-contracts/BC-2.11.001-query-mcp-tool.md`

**ADR-060 v1.28 §D8.11** (Limit-Aware Early-Stop Pagination — upstream-total propagation
chain) defines the complete design: §D8.11.1 (`total_count_path` TOML field), §D8.11.2
(PaginationCursor elevation), §D8.11.3 (four new struct fields), §D8.11.4 (fan-out
max() aggregation), §D8.11.5 (engine Step 6 gated formula), §D8.11.6 (mandate anchors
RG-QTT-001..011), §D8.11.7 (backward compatibility), §D8.11.8 (scope boundary).
Path: `.factory/specs/architecture/decisions/ADR-060-limit-aware-early-stop-pagination.md`

> NOTE: BC-2.11.001 v1.38 and ADR-060 v1.28 are both FROZEN per D-2541/checkpoint (Batch-0
> spec gate: BC-5.39.001 3-CLEAN passed). These spec files MUST NOT be amended by the
> implementer — any spec discrepancy routes to product-owner/architect via the orchestrator.

---

## Problem Statement

During the beta.2 Monroe demo live-test against the Claroty xDome tenant (jea-readapi),
`query(query="FROM claroty_devices", limit=25)` returned:

```json
{
  "is_truncated": true,
  "total_available": 25,
  "returned_results": 25
}
```

An analyst seeing `total_available: 25` with `is_truncated: true` cannot judge query
completeness — the values imply only 25 records exist (equal to limit), not that there
are actually 1,200+ devices in the tenant. The analyst cannot know whether to narrow the
query or whether the full dataset is represented.

### Root Cause

`QueryResult.total_available` currently reports `total_rows` — the number of rows the engine
saw after all sensor fetching, before the LIMIT cap. When LIMIT=25 and the sensor returned
a full page (early-stop fires), `total_rows == 25 == limit`, so `total_available` is 25.

`PaginationCursor.total_count` in `crates/prism-sensors/src/pagination.rs` already captures
the upstream API total from the `total_count` field in each response page. For a Claroty
xDome tenant with 1,200 devices, this field would be `1200`. However, this value is:
1. Never propagated to `FetchOutput`, `FanOutResult`, `MaterializationOutput`, or engine Step 6.
2. Used ONLY to control pagination halting (`self.offset >= self.total_count`), discarded
   for reporting purposes.

### Fix Design (ADR-060 §D8.11)

1. Add `total_count_path: Option<String>` to `TableSpec` in `spec_parser.rs`. For Claroty,
   set `total_count_path = "total"` on the `devices` and `alerts` tables (the Claroty
   API envelope key is `"total"`, not `"total_count"`; confirmed per DTU routes and
   BC-2.11.001 v1.38 EC-11-095 canonical test vector).
2. In `spec_driven_adapter.rs::fetch()`, extract `PaginationCursor.total_count` (which the
   pagination loop already populates) and propagate it as `FetchOutput.upstream_total:
   Option<usize>`.
3. Add `upstream_total: Option<usize>` to `FetchOutput`, `FanOutResult`,
   `MaterializationOutput` (four new struct fields total, per §D8.11.3).
4. In `fan_out()`, OR-aggregate `upstream_total` as `max()` across sensors (§D8.11.4).
5. In engine Step 6, apply the **gated formula** (§D8.11.5):
   ```
   total_available = if any_early_stopped {
       upstream_total.map(|n| n.max(total_rows)).unwrap_or(total_rows)
   } else {
       total_rows
   };
   ```
   Gate reason: `any_early_stopped = false` means the result set is complete —
   `total_rows` IS the correct total. Only apply upstream total when the sensor stopped
   early (pre-LIMIT data exists).

### Scope Boundary

This story covers the QUERY-ENGINE true-total (the `total_available` field in query
results). It does NOT overlap `S-MCP-ENVELOPE-DESCRIBE-001` which fixes `total_results`
in the `prism_describe` envelope (a separate counter, §Issue 3/5).

---

## Narrative

As an LLM analyst agent using `prism_query` to understand dataset size, I want
`total_available` to report the true upstream sensor count when the sensor reports it,
so that I can correctly judge query completeness and decide whether to narrow my query
or increase `limit` without confusing `total_available == limit` with "no more data."

---

## Behavioral Contracts

| BC | Title | Version at Authoring | Scope in This Story |
|----|-------|---------------------|---------------------|
| BC-2.11.001 | `query` MCP Tool Accepts Scoping + PrismQL Query String | v1.38 | §Postconditions EC-11-095: true upstream sensor total via `total_count_path`; gated formula for `total_available`; 11 MUSTs (MUST-1..11) anchored to AC-001..011 + RG-QTT-001..011 |

---

## Acceptance Criteria

### AC-001 — `FetchOutput.upstream_total` populated from `PaginationCursor.total_count`

When a sensor TOML declares `total_count_path` (e.g., `total_count_path = "total"`
for Claroty xDome — the Claroty API response envelope uses the key `"total"`, not
`"total_count"`; confirmed per BC-2.11.001 v1.38 EC-11-095 canonical test vector and
ADR-060 §D8.11.1 example), `spec_driven_adapter.rs::fetch()` MUST populate `FetchOutput.upstream_total`
from `PaginationCursor.total_count` (the value set by `pagination.rs::advance()` from the
API response JSON key at `total_count_path`). When `total_count_path` is absent from TOML,
`FetchOutput.upstream_total` MUST be `None`.

(traces to BC-2.11.001 EC-11-095 MUST-1: `FetchOutput.upstream_total` populated from
`PaginationCursor.total_count` when TOML declares `total_count_path`; ADR-060 §D8.11.2
+ §D8.11.3)

### AC-002 — `FanOutResult.upstream_total = max()` of non-`None` sensor totals

When `fan_out()` OR-aggregates `FetchOutput` results from multiple sensor targets, the
resulting `FanOutResult.upstream_total` MUST be the `max()` of all non-`None`
`FetchOutput.upstream_total` values. When no sensor reports an upstream total
(`all upstream_total == None`), `FanOutResult.upstream_total` MUST be `None`.

(traces to BC-2.11.001 EC-11-095 MUST-2: `FanOutResult.upstream_total = max()` of
non-`None` sensor totals; ADR-060 §D8.11.4)

### AC-003 — `total_available` computed via gated formula when `any_early_stopped = true`

Engine Step 6 MUST compute `total_available` using the gated formula per ADR-060 §D8.11.5:
```
total_available = if any_early_stopped {
    upstream_total.map(|n| n.max(total_rows)).unwrap_or(total_rows)
} else {
    total_rows
}
```
When `any_early_stopped = true` and `upstream_total = Some(N)` and `N >= total_rows`,
`total_available = N`. When `any_early_stopped = false`, `total_available = total_rows`
regardless of `upstream_total` (gate closed — result set is complete; upstream count
does not apply).

(traces to BC-2.11.001 EC-11-095 MUST-3: `total_available` via GATED formula; ADR-060
§D8.11.5)

### AC-004 — MCP wire-level `total_available` reflects sensor upstream total for early-stopped queries

For a single sensor with `total_count_path` declared: when the sensor returns exactly
`limit` rows on a full page (`any_early_stopped = true`) and reports an upstream total
N >= total_rows, the MCP JSON response MUST carry `"total_available": N`. The assertion
MUST be on the serialized JSON wire bytes (SID-2 wire-shape discipline), not only on
pre-serialization Rust struct fields.

Example: Claroty devices with LIMIT=25, 1,200 devices total → `"total_available": 1200`
in the JSON response, NOT `"total_available": 25`.

(traces to BC-2.11.001 EC-11-095 MUST-4: single-sensor w/ `total_count_path`: MCP
`total_available = max(upstream_total, total_rows)` when early-stopped and
`upstream_total >= total_rows`; EC-11-095 canonical test vector)

### AC-005 — Multi-sensor fan-out: `total_available` uses max of sensor upstream totals

For a query spanning two sensors both with `total_count_path` declared (sensor A reports
total=1200, sensor B reports total=500), when both trigger early-stop:
`FanOutResult.upstream_total = max(1200, 500) = 1200`, therefore
`total_available = max(1200, total_rows) = 1200`.

(traces to BC-2.11.001 EC-11-095 MUST-5: multi-sensor both w/ `total_count_path`:
`total_available = max(A_total, B_total)`; ADR-060 §D8.11.4)

### AC-006 — When `total_count_path` absent from TOML: lower-bound behavior unchanged

For a sensor whose TOML declares NO `total_count_path`: `FetchOutput.upstream_total =
None`. The gated formula yields `upstream_total.map(...).unwrap_or(total_rows) = total_rows`
when `any_early_stopped = true`, and `total_available = total_rows` when
`any_early_stopped = false`. In both cases, `total_available` equals `total_rows`
(pre-beta.3 behavior preserved with no regression).

(traces to BC-2.11.001 EC-11-095 MUST-6: `total_count_path` absent:
`upstream_total = None`; `total_available = total_rows` (lower-bound unchanged);
ADR-060 §D8.11.7)

### AC-007 — Early-stop gate: upstream total override only fires when `any_early_stopped = true`

When `any_early_stopped = false` (result set is complete, all pages fetched), engine Step
6 MUST set `total_available = total_rows` EVEN IF `upstream_total = Some(N)` is present.
The upstream total is irrelevant for a complete result set. This prevents
`total_available` from overstating when no truncation occurred.

(traces to BC-2.11.001 EC-11-095 MUST-7: early-stop gate: `any_early_stopped = false`
→ `total_available = total_rows` even when `upstream_total = Some(N)`; ADR-060 §D8.11.5
gate condition)

### AC-008 — Hard invariant: `total_available >= total_rows` in all cases

For any combination of `any_early_stopped`, `upstream_total`, and `total_rows`, the MCP
response MUST satisfy `total_available >= total_rows`. The `.max(total_rows)` in the
gated formula enforces this even when `upstream_total < total_rows` (under-report from
a sensor reporting a stale or incorrect count). No plan, query mode, or sensor configuration
may produce `total_available < total_rows`.

(traces to BC-2.11.001 EC-11-095 MUST-8: hard invariant `total_available >= total_rows`
in all cases including sensor under-report; ADR-060 §D8.11.5 `.max(total_rows)` clause)

### AC-009 — Aggregating plans (GROUP BY/HAVING): `total_available = total_rows`

For a query with GROUP BY or HAVING, `total_available` MUST equal `total_rows`
(the post-aggregation row count) EVEN WHEN the sensor TOML has `total_count_path`
declared and `upstream_total = Some(N)`. The upstream item count predates aggregation
and does not apply to the number of output aggregate buckets. `any_early_stopped` is
the gate guard: for aggregating plans, `any_early_stopped` should be `false` since all
sensor rows must be fetched before aggregation. Engine Step 6 does NOT need special
aggregation detection — the gate (`any_early_stopped = false`) naturally produces
`total_available = total_rows` for complete aggregations.

(traces to BC-2.11.001 EC-11-095 MUST-9: aggregating plan (GROUP BY/HAVING):
`total_available = total_rows` even with `total_count_path`; ADR-060 §D8.11.5
aggregating-plan clause)

### AC-010 — `is_truncated` formula unchanged by `upstream_total`

The introduction of `upstream_total` MUST NOT change the `is_truncated` computation.
`is_truncated` is driven ONLY by `(total_rows > limit) OR any_early_stopped OR
any_pipeline_truncated`. It is NOT derived from comparing `total_available` to
`returned_results`. The existing engine Step 6 formula for `is_truncated` must remain
exactly: `is_truncated = (total_rows > limit) || any_early_stopped || any_pipeline_truncated`.

(traces to BC-2.11.001 EC-11-095 MUST-10: `is_truncated` formula UNCHANGED by
`upstream_total`; EC-11-092/093/095 is_truncated note)

### AC-011 — Multi-sensor mixed `total_count_path`: partial coverage propagates correctly

For a query spanning two sensors where SENSOR-A declares `total_count_path` (reports
`upstream_total = Some(1200)`) and SENSOR-B does NOT declare `total_count_path`
(produces `FetchOutput.upstream_total = None`):
`FanOutResult.upstream_total = Some(1200)` (max() of Some(1200) and None = Some(1200)).
When `any_early_stopped = true`, `total_available = max(1200, total_rows)`.
When `any_early_stopped = false`, `total_available = total_rows`.

(traces to BC-2.11.001 EC-11-095 MUST-11: multi-sensor mixed `total_count_path`:
`upstream_total = Some(N)` from declaring sensor; used when `any_early_stopped = true`;
ADR-060 §D8.11.4 `filter_map(|fo| fo.upstream_total).max()` aggregation)

---

## Red Gate Test List (SAC-1 — BC-5.38.001)

All 11 failing Red Gate tests reside in a NEW file:
`crates/prism-bin/tests/bc_2_11_001_true_total_tests.rs`

This file exercises the full pipeline from `TableSpec.total_count_path` → adapter →
fan-out → materialization → engine Step 6 → MCP wire output. Following the pattern of
`crates/prism-bin/tests/bc_2_16_002_early_stop_adapter_tests.rs`.

- **RG-QTT-001**: `test_qtt_rg001_fetch_output_upstream_total_from_pagination_cursor`
  Assert: when `TableSpec.total_count_path = Some("total_count")` and the mock API
  response body contains `{"total_count": 1200, "items": [...25 items...]}`,
  `FetchOutput.upstream_total == Some(1200)` after `spec_driven_adapter.fetch()`.
  Currently FAILS: `FetchOutput` has no `upstream_total` field. Covers AC-001.

- **RG-QTT-002**: `test_qtt_rg002_fanout_result_upstream_total_is_max`
  Assert: `fan_out()` aggregates two `FetchOutput` values with
  `upstream_total = Some(1200)` and `upstream_total = Some(500)` →
  `FanOutResult.upstream_total = Some(1200)` (max).
  Also asserts: when both are `None` → `FanOutResult.upstream_total = None`.
  Currently FAILS: `FanOutResult` has no `upstream_total` field. Covers AC-002.

- **RG-QTT-003**: `test_qtt_rg003_total_available_gated_formula_early_stop_true`
  Assert: engine Step 6 with `any_early_stopped = true`, `upstream_total = Some(1200)`,
  `total_rows = 25`, `limit = 25` → `total_available = 1200`.
  Also asserts: with `any_early_stopped = false`, `upstream_total = Some(1200)`,
  `total_rows = 25` → `total_available = 25` (gate closed).
  Currently FAILS: engine uses `total_available = total_rows` unconditionally. Covers AC-003.

- **RG-QTT-004**: `test_qtt_rg004_single_sensor_mcp_total_available_reflects_upstream`
  Assert: wire-level integration test (SID-2). Mock adapter returns `upstream_total =
  Some(1200)` with `any_early_stopped = true`. Call `prism_query` tool end-to-end.
  Serialize response to JSON. Assert `response["query_context"]["total_available"] == 1200`
  on the serialized bytes (not pre-serialization struct). Assert `"is_truncated": true`.
  Currently FAILS: `total_available` is 25 (total_rows). Covers AC-004.

- **RG-QTT-005**: `test_qtt_rg005_multi_sensor_total_available_is_max`
  Assert: two mock adapters — sensor A with `upstream_total = Some(1200)`, sensor B with
  `upstream_total = Some(500)`, both `any_early_stopped = true`. Engine Step 6 →
  `total_available = 1200`. Currently FAILS: no upstream_total propagation. Covers AC-005.

- **RG-QTT-006**: `test_qtt_rg006_absent_total_count_path_falls_back_to_total_rows`
  Assert: `TableSpec.total_count_path = None` → `FetchOutput.upstream_total = None`.
  Engine Step 6 with `any_early_stopped = true`, `upstream_total = None`, `total_rows = 25`
  → `total_available = 25` (lower-bound unchanged). No regression from beta.3 change.
  Currently FAILS due to missing field (compilation error). Covers AC-006.

- **RG-QTT-007**: `test_qtt_rg007_no_early_stop_upstream_total_not_applied`
  Assert: engine Step 6 with `any_early_stopped = false`, `upstream_total = Some(1200)`,
  `total_rows = 800`, `limit = 1000` → `total_available = 800` (gate closed — result
  complete). Gate must NOT apply `upstream_total` when `any_early_stopped = false`.
  Currently FAILS: no gated formula. Covers AC-007.

- **RG-QTT-008**: `test_qtt_rg008_total_available_gte_total_rows_invariant`
  Assert: edge case — `any_early_stopped = true`, `upstream_total = Some(10)` (sensor
  under-reports; `upstream_total < total_rows = 25`) → `total_available = 25` (not 10).
  The `.max(total_rows)` clause enforces the hard invariant. Currently FAILS: no gated
  formula with `.max()`. Covers AC-008.

- **RG-QTT-009**: `test_qtt_rg009_aggregating_plan_total_available_equals_total_rows`
  Assert: engine Step 6 with `any_early_stopped = false` (aggregating plan; all rows
  fetched before GROUP BY), `upstream_total = Some(1200)`, `total_rows = 50` (post-agg
  buckets) → `total_available = 50` (gate closed). The gate (`any_early_stopped = false`)
  naturally handles aggregating plans. Currently FAILS: no gated formula. Covers AC-009.

- **RG-QTT-010**: `test_qtt_rg010_is_truncated_unchanged_by_upstream_total`
  Assert: `is_truncated` formula UNCHANGED. With `any_early_stopped = true`,
  `total_rows = 25`, `limit = 25`, `upstream_total = Some(1200)` →
  `is_truncated = true` (driven by `any_early_stopped = true`, NOT by
  `total_available > returned_results`). With `any_early_stopped = false`,
  `total_rows = 800`, `limit = 1000`, `upstream_total = Some(1200)` →
  `is_truncated = false` (only `total_rows > limit` gate; `any_early_stopped = false`).
  Verifies that adding `upstream_total` does not accidentally change the
  `is_truncated` computation. Currently FAILS: missing fields. Covers AC-010.

- **RG-QTT-011**: `test_qtt_rg011_multi_sensor_mixed_total_count_path_partial_coverage`
  Assert: sensor A has `total_count_path` → `FetchOutput.upstream_total = Some(1200)`;
  sensor B has no `total_count_path` → `FetchOutput.upstream_total = None`.
  `FanOutResult.upstream_total = Some(1200)` (filter_map + max; None values excluded).
  Engine Step 6 with `any_early_stopped = true` → `total_available = 1200`.
  Currently FAILS: no upstream_total field. Covers AC-011.

**BC-5.38.001 density check:** 11 failing Red Gate tests / 11 ACs = **1.00** — satisfies
the ≥ 0.5 threshold.

**Red-then-green task ordering:** RG-QTT-001..011 must all be written and confirmed RED by
test-writer BEFORE implementer begins any implementation tasks. See §Tasks.

---

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| TOML spec `total_count_path` field | `crates/prism-spec-engine/src/spec_parser.rs` — `TableSpec` struct | Pure (data definition) |
| `FetchOutput.upstream_total` field + `FetchOutput::new` | `crates/prism-sensors/src/adapter.rs` | Pure (data definition) |
| `FanOutResult.upstream_total` field + max() aggregation | `crates/prism-sensors/src/fanout.rs` — `fan_out()` | Effectful (async I/O orchestration) |
| `MaterializationOutput.upstream_total` field | `crates/prism-query/src/materialization.rs` | Pure (data struct) |
| Engine Step 6 gated formula | `crates/prism-query/src/engine.rs` — `execute_query_internal()` | Effectful (DataFusion execution) |
| Adapter plumbing (upstream_total propagation) | `crates/prism-bin/src/spec_driven_adapter.rs` — `fetch()` | Effectful (HTTP I/O) |
| Claroty TOML `total_count_path` addition | `crates/prism-sensors/specs/claroty.sensor.toml` | Pure (config data) |
| Red Gate integration tests | `crates/prism-bin/tests/bc_2_11_001_true_total_tests.rs` (NEW) | Pure (tests) |
| FetchOutput::new call sites (TD-VSDD-060 sibling sweep) | 46 `::new` sites + 3 struct literals across `prism-mcp/tests/` (5 files, 8 sites), `prism-bin/tests/` (1 file, 4 sites), `prism-query/tests/` (5 files, 27 sites), `prism-query/src/` (3 sites), `prism-sensors/tests/` (3 files, 3 sites), `prism-mcp/src/server.rs` (1), `prism-bin/src/spec_driven_adapter.rs` (1), and 3 FetchOutput struct literals in `prism-sensors/src/` | Pure (sibling sweep) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `total_count_path` declared but API response body does not contain that key | `upstream_total = None` (graceful degradation per ADR-060 §D8.11.1 Validation row); no error raised; `total_available = total_rows` (lower-bound fallback) |
| EC-002 | `total_count_path` declared but value at that key is a string, not integer | `upstream_total = None` (non-numeric parse failure); graceful degradation; same fallback as EC-001 |
| EC-003 | Sensor reports `total_count = 0` (API bug / empty table) | `upstream_total = Some(0)`, formula: `max(0, total_rows) = total_rows`; hard invariant `total_available >= total_rows` preserved |
| EC-004 | `any_early_stopped = false` with `any_pipeline_truncated = true` (DI-019 cap, no early-stop) | `total_available = total_rows` (gate closed); upstream_total not applied; conservative under-report accepted (per ADR-060 §D8.11.5 OBS-2); `is_truncated = true` via `any_pipeline_truncated` term |
| EC-005 | Multi-sensor query, one sensor succeeds with `upstream_total = Some(1200)`, the other fails with `SensorError::AllTargetsFailed` | `sensor_errors` contains the failing sensor entry; `any_early_stopped` reflects the surviving sensor; `total_available` derived from the surviving sensor's upstream total if `any_early_stopped = true` |
| EC-006 | `total_count_path = "total"` for Claroty + sensor returns `total_count = usize::MAX` on first page (initialization sentinel; `usize::MAX` is the `PaginationCursor` pre-response initialization value, not an API total) | `upstream_total` should NOT be set from `PaginationCursor.total_count = usize::MAX` (sentinel value); implementer must NOT propagate the sentinel — only propagate after the first API response sets a real value (T-B03 sentinel guard: `if raw == usize::MAX { None } else { Some(raw) }`) |
| EC-007 | `FetchOutput::new` call sites — existing stubs that simulate no-truncation | All existing `FetchOutput::new(batches, false, false)` calls must be updated to `FetchOutput::new(batches, false, false, None)` (4th param = `upstream_total = None`); there are 46 `::new` call sites + 3 struct literals per TD-VSDD-060 sibling sweep (see T-A02) |
| EC-008 | Engine Step 6 scheduled-path (response cache hit — no fresh fan-out) | `upstream_total` not available on cache-hit path; `total_available = total_rows` (same as pre-beta.3); no regression |

---

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story file | ~15,000 | |
| BC-2.11.001 v1.38 — EC-11-095 + §Postconditions Dual-limit-semantics (primary AC source) | ~8,000 | Load §Postconditions + §Edge Cases EC-11-095; skip unrelated EC rows |
| ADR-060 v1.28 §D8.11.1..§D8.11.8 (implementation design) | ~12,000 | Load §D8.11 subsections only; skip earlier §D8.1..§D8.10 |
| `crates/prism-spec-engine/src/spec_parser.rs` (TableSpec section) | ~8,000 | Load `pub struct TableSpec` and surrounding context |
| `crates/prism-sensors/src/pagination.rs` (full) | ~6,000 | `PaginationCursor`, `advance()`, `paginate_offset_limit()` |
| `crates/prism-sensors/src/adapter.rs` (FetchOutput section) | ~3,000 | FetchOutput struct + `new()` constructor |
| `crates/prism-sensors/src/fanout.rs` (FanOutResult + fan_out) | ~18,000 | FanOutResult struct, `fan_out()`, `fan_out_with_overlay_map()` |
| `crates/prism-query/src/materialization.rs` (MaterializationOutput section) | ~8,000 | MaterializationOutput struct + relevant context |
| `crates/prism-query/src/engine.rs` (Step 6 section ~lines 1095-1135) | ~5,000 | Step 6 context only; full engine.rs is ~17k lines |
| `crates/prism-bin/src/spec_driven_adapter.rs` (fetch() + FetchOutput::new call site) | ~25,000 | Large file; load fetch() function and FetchOutput construction |
| `crates/prism-sensors/specs/claroty.sensor.toml` (devices + alerts tables) | ~20,000 | Load tables needing `total_count_path = "total"` (Claroty key is `"total"`) |
| FetchOutput::new call sites (sibling sweep, 46 `::new` sites + 3 struct literals across 19 files) | ~15,000 | Batch with grep; update to 4-arg form |
| New test file `bc_2_11_001_true_total_tests.rs` | ~10,000 | 11 test functions |
| Delta analysis §Issue 6 + §S-QUERY-TRUE-TOTAL-001 | ~5,000 | Reference for design rationale |
| **Total estimated (full implementation)** | **~158,000** | Within one context window; implementer should use phase-scoped reads |

**Phase-scoped loading recommendation:** The implementer should read only the files relevant
to each phase rather than all files at once. Phase A (struct changes) needs adapter.rs,
fanout.rs, materialization.rs, and the FetchOutput::new sibling sites. Phase B (adapter
plumbing) needs spec_parser.rs, pagination.rs, spec_driven_adapter.rs. Phase C (engine
Step 6) needs engine.rs lines 1090-1140. Loading the full set simultaneously risks
context pressure.

---

## Tasks

### Red Gate tests (to be written by test-writer BEFORE implementation — SAC-1)

All 11 tests are in the NEW file `crates/prism-bin/tests/bc_2_11_001_true_total_tests.rs`.
All non-trivial function bodies in stubs use `todo!()` until Green Gate phase.
The test file imports `FetchOutput`, `FanOutResult`, `SensorSpec`, `TableSpec` etc. from
the relevant crates. Tests that require mock adapters follow the pattern in
`crates/prism-bin/tests/bc_2_16_002_early_stop_adapter_tests.rs`.

- [ ] **RG-QTT-001**: Write `test_qtt_rg001_fetch_output_upstream_total_from_pagination_cursor`.
  Assert `FetchOutput.upstream_total == Some(1200)` when adapter is called with a mock
  TableSpec carrying `total_count_path = Some("total_count")` and mock API response body
  `{"total_count": 1200, "items": [...]}`. Must FAIL (compilation error — no `upstream_total`
  field) before Phase A struct changes land.

- [ ] **RG-QTT-002**: Write `test_qtt_rg002_fanout_result_upstream_total_is_max`.
  Assert max() aggregation of FetchOutput.upstream_total values across two mock targets.
  Also assert None when all None. Must FAIL before Phase A.

- [ ] **RG-QTT-003**: Write `test_qtt_rg003_total_available_gated_formula_early_stop_true`.
  Unit-test engine Step 6 logic directly (or via a minimal MaterializationOutput +
  execute_query_internal test). Assert gated formula behavior for both gate-open and
  gate-closed branches. Must FAIL before Phase C.

- [ ] **RG-QTT-004**: Write `test_qtt_rg004_single_sensor_mcp_total_available_reflects_upstream`.
  End-to-end integration test: mock adapter returns `any_early_stopped = true`,
  `upstream_total = Some(1200)`, `total_rows = 25`. Call the MCP query tool. Serialize
  response to JSON. Assert `json["query_context"]["total_available"] == 1200` on serialized
  bytes (SID-2). Must FAIL before full pipeline is wired.

- [ ] **RG-QTT-005**: Write `test_qtt_rg005_multi_sensor_total_available_is_max`.
  Two mock adapters (sensor A: `upstream_total = Some(1200)`, sensor B:
  `upstream_total = Some(500)`, both `any_early_stopped = true`). Assert
  `total_available = 1200` in MCP response. Must FAIL before Phase A.

- [ ] **RG-QTT-006**: Write `test_qtt_rg006_absent_total_count_path_falls_back_to_total_rows`.
  Mock adapter with `TableSpec.total_count_path = None`. Assert `FetchOutput.upstream_total
  = None` and `total_available = total_rows` (25 when limit=25 and early-stopped). Must FAIL
  before Phase A.

- [ ] **RG-QTT-007**: Write `test_qtt_rg007_no_early_stop_upstream_total_not_applied`.
  Mock adapter with `any_early_stopped = false`, `upstream_total = Some(1200)`,
  `total_rows = 800`. Assert `total_available = 800` (gate closed). Must FAIL before Phase C.

- [ ] **RG-QTT-008**: Write `test_qtt_rg008_total_available_gte_total_rows_invariant`.
  Edge case: `any_early_stopped = true`, `upstream_total = Some(10)` (under-report),
  `total_rows = 25`. Assert `total_available = 25` (`.max(total_rows)` floor). Must FAIL
  before Phase C.

- [ ] **RG-QTT-009**: Write `test_qtt_rg009_aggregating_plan_total_available_equals_total_rows`.
  Mock adapter with `any_early_stopped = false` (aggregating plan), `upstream_total =
  Some(1200)`, `total_rows = 50` (post-GROUP BY buckets). Assert `total_available = 50`.
  Must FAIL before Phase C.

- [ ] **RG-QTT-010**: Write `test_qtt_rg010_is_truncated_unchanged_by_upstream_total`.
  Assert `is_truncated` formula is UNCHANGED. Verify: (a) `any_early_stopped = true`,
  `total_rows = 25`, `limit = 25`, `upstream_total = Some(1200)` → `is_truncated = true`;
  (b) `any_early_stopped = false`, `total_rows = 800`, `limit = 1000` → `is_truncated =
  false` regardless of `upstream_total`. Must FAIL before Phases A–C.

- [ ] **RG-QTT-011**: Write `test_qtt_rg011_multi_sensor_mixed_total_count_path_partial_coverage`.
  Sensor A: `upstream_total = Some(1200)`. Sensor B: `upstream_total = None` (no
  `total_count_path`). Assert `FanOutResult.upstream_total = Some(1200)`. Assert
  `total_available = 1200` when `any_early_stopped = true`. Must FAIL before Phase A.

- [ ] **Regression confirmation**: Verify all pre-existing tests in the test runner compile
  (they will fail to compile before Phase A since `FetchOutput::new` signature changes;
  the test-writer documents the compilation failures as the expected RED state).

---

### Implementation tasks (to be executed by implementer AFTER Red Gate — SAC-1)

#### Phase A — Struct changes (FetchOutput, FanOutResult, MaterializationOutput)

Phase A establishes the `upstream_total: Option<usize>` field across all three pipeline
structs. This is the HIGHEST blast-radius phase (TD-VSDD-060 sibling sweep for
FetchOutput::new).

- [ ] **T-A01**: In `crates/prism-sensors/src/adapter.rs`, add `pub upstream_total:
  Option<usize>` field to `FetchOutput` (after `pipeline_truncated`). The struct is
  `#[non_exhaustive]`, so callers outside the crate use `FetchOutput::new()`.

  Add doc comment:
  ```rust
  /// The upstream item count reported by the sensor API when `total_count_path`
  /// is declared in the TOML spec for this table. `None` when `total_count_path`
  /// is absent or the API response does not contain a parseable integer at that key.
  /// Propagates through `FanOutResult.upstream_total → MaterializationOutput.upstream_total`
  /// → engine Step 6 `total_available` gated formula (ADR-060 §D8.11.3/§D8.11.5).
  pub upstream_total: Option<usize>,
  ```

  Update `FetchOutput::new()` to accept a 4th parameter:
  ```rust
  pub fn new(
      batches: Vec<RecordBatch>,
      any_early_stopped: bool,
      pipeline_truncated: bool,
      upstream_total: Option<usize>,
  ) -> Self {
      Self { batches, any_early_stopped, pipeline_truncated, upstream_total }
  }
  ```

- [ ] **T-A02 (TD-VSDD-060 sibling sweep)**: Grep for ALL `FetchOutput::new(` call sites
  workspace-wide: `rg 'FetchOutput::new\(' crates/ --type rust`. Also grep for struct
  literals: `rg 'FetchOutput \{' crates/ --type rust`. Update EVERY call site to pass
  `None` as the 4th argument; add `upstream_total: None` to each struct literal.
  Confirmed call sites per D-1110 grep (46 `::new` code sites + 3 struct literals):
  - `crates/prism-mcp/tests/` — 5 test files, 8 `::new` call sites, pass `None`
    (bc_2_11_001_null_row_shape_test.rs: 2, bc_s_5_04_health_test.rs: 3,
     defect_live_envelope_obs_001_test.rs: 1, defect_t13_audit_ecode_sap3_test.rs: 1,
     query_tool_sensor_errors_test.rs: 1)
  - `crates/prism-bin/tests/` — 1 test file, 4 `::new` call sites, pass `None`
    (`mcp_integration_tests.rs` only; bc_2_16_002_early_stop_adapter_tests.rs
     has only doc comments referencing `::new`, not actual call sites)
  - `crates/prism-query/tests/` — 5 test files, 27 `::new` call sites, pass `None`
    (`execute_integration_tests.rs` alone has 22 call sites; plan_shape_gate_tests.rs: 2;
     pipe_execution_tests.rs: 1; slug_isolation_tests.rs: 1; filter_mode.rs: 1)
  - `crates/prism-query/src/tests/defect_csdevices_empty_memtable_tests.rs` — 2 `::new`
    call sites (inline test module), pass `None`
  - `crates/prism-query/src/materialization.rs` — 1 production `::new` call site, pass `None`
  - `crates/prism-sensors/tests/` — 3 test files, 3 `::new` call sites, pass `None`
    (org_id_binding.rs: 1, cr013_fan_out_org_id_consistency.rs: 1,
     multi_tenant_dtu_routing_integration.rs: 1)
  - `crates/prism-mcp/src/server.rs` — 1 production `::new` call, pass `None`
  - `crates/prism-bin/src/spec_driven_adapter.rs` — 1 production `::new` call (updated in Phase B to pass `Some(upstream_total)`)
  - `crates/prism-sensors/src/fanout.rs` — 1 FetchOutput struct literal (NOT `::new`);
    add `upstream_total: None` (same-crate, `#[non_exhaustive]` does not restrict)
  - `crates/prism-sensors/src/tests/bc_2_01_002.rs` — 1 FetchOutput struct literal;
    add `upstream_total: None`
  - `crates/prism-sensors/src/tests/bc_2_01_010.rs` — 1 FetchOutput struct literal;
    add `upstream_total: None`
  After updating all sites, run `cargo check --workspace` — must exit 0 (compilation only).
  DO NOT run `just check` yet — tests will fail (RED state is expected per TDD).

- [ ] **T-A03**: In `crates/prism-sensors/src/fanout.rs`, add `pub upstream_total:
  Option<usize>` field to `FanOutResult`. Update `Default` impl if present, or ensure
  `#[derive(Default)]` initializes to `None`.

  In `fan_out()`, add max() aggregation after collecting `FetchOutput` results:
  ```rust
  let upstream_total: Option<usize> = fetch_outputs
      .iter()
      .filter_map(|fo| fo.upstream_total)
      .max();
  ```
  (Mirrors ADR-060 §D8.11.4 code snippet.)

  `fan_out_with_overlay_map()` contains a FetchOutput struct literal at `fanout.rs:974`
  that must be updated: add `upstream_total: None` (NOT `upstream_total: false` — the
  type is `Option<usize>`, not `bool`; `false` would be a compile error). Same-crate
  struct literals are NOT restricted by `#[non_exhaustive]`. FanOutResult is constructed
  via `FanOutResult::default()` throughout `fan_out()` (lines ~243/287/420); the
  `#[derive(Default)]` impl initializes the new `upstream_total: Option<usize>` field to
  `None` automatically — no manual construction-site updates are required for FanOutResult.
  The only manual update is the max() aggregation added after the loop (see above).

- [ ] **T-A04**: In `crates/prism-query/src/materialization.rs`, add `pub upstream_total:
  Option<usize>` field to `MaterializationOutput`. Update all construction sites within
  the file to include `upstream_total` (typically from `FanOutResult.upstream_total`).

- [ ] **T-A05**: Run `cargo test -p prism-sensors --no-fail-fast` — should compile and
  have any RG-QTT tests in `prism-sensors` scope go RED as expected. Run
  `cargo test -p prism-query --no-fail-fast` — same.

#### Phase B — Adapter plumbing (spec_parser.rs, pagination.rs, spec_driven_adapter.rs)

Phase B wires `total_count_path` from the TOML spec through the adapter to `FetchOutput`.

- [ ] **T-B01**: In `crates/prism-spec-engine/src/spec_parser.rs`, add
  `total_count_path: Option<String>` field to `TableSpec`. Place after the
  `steps` field or similar location. Add doc comment:
  ```rust
  /// Optional top-level JSON key in the API response body that contains the
  /// upstream item count. When set, the adapter reads this key to populate
  /// `FetchOutput.upstream_total`, enabling `total_available` to report the
  /// true upstream total rather than a lower bound.
  ///
  /// MUST be a top-level key only (ADR-060 §D8.11.1/§D8.11.8). Nested paths
  /// (e.g. "meta.total") are FORBIDDEN; nested path support is deferred to
  /// S-JSON-EXTRACT-NESTED-001.
  ///
  /// When absent, `upstream_total = None` and `total_available` falls back to
  /// `total_rows` (pre-beta.3 lower-bound behavior; ADR-060 §D8.11.7).
  ///
  /// Example: `total_count_path = "total"` for Claroty xDome (the Claroty API response
  /// envelope uses key `"total"`, confirmed per BC-2.11.001 v1.38 EC-11-095 canonical
  /// test vector and ADR-060 §D8.11.1 example code).
  pub total_count_path: Option<String>,
  ```
  The `#[serde(default)]` attribute (or `Option<String>` with implicit None default) must
  ensure existing TOML files without this key continue to parse without error.

- [ ] **T-B02**: In `crates/prism-sensors/specs/claroty.sensor.toml`, add
  `total_count_path = "total"` to the `alerts` and `devices` tables. The Claroty xDome
  API response envelope uses `"total"` as the key (confirmed: DTU routes emit
  `{"alerts": [...], "total": N, "page": N}` and `{"devices": [...], "total": N, "page": N}`
  per `crates/prism-dtu-claroty/src/routes/alerts.rs` and `devices.rs`; also confirmed
  in BC-2.11.001 v1.38 EC-11-095 canonical test vector and ADR-060 §D8.11.1 example).
  Place it near the pagination config block for each table. Do NOT add it to tables using
  `PaginationConfig::None` (single-fetch tables, e.g., `org_acl_policies`) since there
  is no pagination loop to accumulate a total for those tables. Use a comment:
  ```toml
  # total_count_path (ADR-060 §D8.11.1): Claroty API returns {"total": N, ...}
  # at the top level (key is "total", NOT "total_count"). This propagates to
  # FetchOutput.upstream_total for true total reporting in query responses
  # (S-QUERY-TRUE-TOTAL-001).
  total_count_path = "total"
  ```
  SAP-2 check: no DTU struct change is required — `total_count_path` is a spec-engine
  extraction config, not a DTU wire-shape field.

- [ ] **T-B03**: In `crates/prism-bin/src/spec_driven_adapter.rs`, in the `fetch()`
  function, after the pagination loop completes, extract `upstream_total` from the
  final state of `PaginationCursor.total_count` (or the accumulated upstream total
  from the last response page) and pass it to `FetchOutput::new(...)` as the 4th arg:
  ```rust
  let upstream_total = if table_spec.total_count_path.is_some() {
      // PaginationCursor.total_count holds the API-reported total after advance()
      // is called on each page. If it is still usize::MAX (sentinel — never set
      // by the API), treat as None (no valid upstream total available).
      let raw = pagination_cursor.total_count;
      if raw == usize::MAX { None } else { Some(raw) }
  } else {
      None
  };
  FetchOutput::new(all_batches, any_early_stopped, pipeline_truncated, upstream_total)
  ```
  NOTE: The sentinel guard (`raw == usize::MAX → None`) is critical per EC-006 in this
  story's edge cases — `usize::MAX` is the initialization value before the first API
  response, not a real upstream total.

  **The exact implementation detail may vary** based on how `spec_driven_adapter.rs`
  structures its pagination loop. The implementer must read `spec_driven_adapter.rs`
  `fetch()` in full before writing T-B03 code. The key constraint from ADR-060 §D8.11.2:
  "Spec-engine and `spec_driven_adapter.rs` call `pagination.rs` routines; the last-page
  `PaginationCursor.total_count` MUST flow via the `FetchOutput.upstream_total` field."

- [ ] **T-B04**: Run `cargo test -p prism-bin -E 'test(rg001)'` and
  `cargo test -p prism-bin -E 'test(rg006)'` — both should now go GREEN (RG-QTT-001 and
  RG-QTT-006 cover the adapter plumbing).

#### Phase C — Engine Step 6 gated formula

Phase C updates engine Step 6 `total_available` to use the gated formula.

- [ ] **T-C01**: In `crates/prism-query/src/engine.rs`, in `execute_query_internal()`
  (around line 1095-1135), update Step 6 to use the gated formula:
  ```rust
  // Step 6: Apply tool-level limit truncation + compute total_available.
  //
  // Gated formula (ADR-060 §D8.11.5, S-QUERY-TRUE-TOTAL-001 AC-003):
  // When any_early_stopped = true: upstream_total.map(|n| n.max(total_rows)).unwrap_or(total_rows)
  // When any_early_stopped = false: total_rows (result set complete; upstream count N/A)
  // Hard invariant: total_available >= total_rows (enforced by .max(total_rows)).
  // Aggregating plans: any_early_stopped = false → total_available = total_rows (correct).
  // is_truncated formula UNCHANGED: (total_rows > limit) || any_early_stopped || any_pipeline_truncated.
  let total_available: usize = if output.any_early_stopped {
      output.upstream_total
          .map(|n| n.max(total_rows))
          .unwrap_or(total_rows)
  } else {
      total_rows
  };
  ```
  Replace the existing `total_available: total_rows` assignment with this formula.
  The `is_truncated` line must remain UNCHANGED (BC-2.11.001 EC-11-092/093/095 MUST-10).

  There are TWO `total_available: total_rows` assignment sites in engine.rs (lines ~1133
  and ~1417 per the code grep). The second is the scheduled-path (cache-hit). For the
  cache-hit path, `upstream_total` is not available — leave `total_available: total_rows`
  unchanged on that path (EC-008 in this story's edge cases; no regression).

- [ ] **T-C02**: Thread `MaterializationOutput.upstream_total` through from `FanOutResult`
  in the materialization step. The `materialize()` function (or equivalent in
  `engine.rs`/`materialization.rs`) that converts `FanOutResult → MaterializationOutput`
  must copy `fan_out_result.upstream_total` to `materialization_output.upstream_total`.
  Similarly, the engine that converts `MaterializationOutput → Step 6 inputs` must pass
  `output.upstream_total` to the gated formula.

- [ ] **T-C03**: Run `cargo test -p prism-query --no-fail-fast`. RG-QTT-003, RG-QTT-007,
  RG-QTT-008, RG-QTT-009, RG-QTT-010 should go GREEN. Any failures indicate a plumbing
  gap in the FanOutResult → MaterializationOutput → engine chain.

- [ ] **T-C04**: Run the full RG-QTT suite: `cargo test -p prism-bin -E 'test(rg0)'`.
  All 11 tests must be GREEN before proceeding to final verification.

#### Phase D — Final verification

- [ ] **T-D01**: Run `cargo test -p prism-bin --no-fail-fast`. All 11 RG-QTT tests GREEN.
  All existing `bc_2_16_002_early_stop_adapter_tests.rs` tests remain GREEN (no regression
  on `any_early_stopped` behavior — only `total_available` computation is changed).

- [ ] **T-D02**: Run `cargo test -p prism-mcp --no-fail-fast`. All existing tests GREEN.
  The mock adapter call sites updated in T-A02 must not introduce any semantic regressions.

- [ ] **T-D03**: Run `just check` (full workspace). Must exit 0. The blast-radius for this
  story is HIGH — `just check` is mandatory, not optional, before declaring done.

- [ ] **T-D04**: Verify `cargo clippy -p prism-sensors -p prism-query -p prism-bin --
  -D warnings` exits 0. Confirm new `upstream_total` fields have doc comments. Confirm
  no unused-variable warnings on the new field in any non-test construction path.

#### Phase E — CHANGELOG

- [ ] **T-E01** (BEFORE creating the PR): Add a CHANGELOG entry under `[Unreleased] > Fixed`:
  ```markdown
  - Fix `query` tool `total_available` reporting: when a sensor TOML declares
    `total_count_path` and the sensor stops early (query result truncated by LIMIT),
    `total_available` now reflects the upstream sensor total reported by the API
    (e.g., 1200 actual Claroty devices) rather than the LIMIT value (e.g., 25).
    Implements ADR-060 §D8.11 upstream-total propagation chain. Resolves beta.2
    live-test issue 6 (total_available: 25 when 1,200+ Claroty devices exist).
  ```

---

## Previous Story Intelligence

**S-MCP-ENVELOPE-DESCRIBE-001** (W2, E-BETA3-REMEDIATION) fixes `prism_describe`
`total_results` (the envelope counter, not `total_available` in query results) and adds
missing virtual field descriptors. That story touches `safety_envelope.rs` and
`prism_describe.rs` only — NO overlap with this story's files. Both stories can proceed
in parallel worktrees.

**S-MCP-NULL-ENCODING-001** (W2, E-BETA3-REMEDIATION) also touches
`spec_driven_adapter.rs`. **Soft merge-ordering recommendation:** merge
`S-MCP-NULL-ENCODING-001` before `S-QUERY-TRUE-TOTAL-001` to reduce merge-conflict
risk in `spec_driven_adapter.rs`. If both are implemented in parallel, use careful
worktree discipline.

**S-ENGINE-LIMIT-EARLY-STOP-001** (completed, merged) added `any_early_stopped` and
`any_pipeline_truncated` to `FetchOutput` and the `is_truncated` formula. That PR is the
direct predecessor; this story extends the same chain with `upstream_total`. The patterns
established in that story (e.g., the 3-field `FetchOutput::new` constructor, the
`fan_out` max-aggregation idiom) are the model to follow. Read
`crates/prism-bin/tests/bc_2_16_002_early_stop_adapter_tests.rs` for the test structure.

**FetchOutput::new blast radius**: when `S-ENGINE-LIMIT-EARLY-STOP-001` added
`pipeline_truncated`, the same sibling-sweep was required. At that time there were ~15
call sites; there are now ~46 `::new` call sites + 3 FetchOutput struct literals (per
D-1110 grep). The pattern is well-established. The implementer must grep ALL sites
before declaring the sibling sweep complete (TD-VSDD-060 Dim-1). The primary growth
vs. the previous sweep is in `execute_integration_tests.rs` (22 call sites alone).

**Key lesson from the early-stop cascade**: the `usize::MAX` sentinel initialization of
`PaginationCursor.total_count` caused subtle issues in prior work. This story explicitly
guards against propagating the sentinel value (EC-006, T-B03 sentinel guard). Do not
omit this guard — a `total_available: usize::MAX` in the MCP response would be catastrophic.

---

## Architecture Compliance Rules

1. **`FetchOutput.upstream_total` is `Option<usize>`, NOT `usize`.** `None` is the correct
   representation for "no upstream total declared or parsed." Never use a sentinel value
   (e.g., `0` or `usize::MAX`) as a substitute for `None`. See EC-006.

2. **Gated formula only fires when `any_early_stopped = true`.** The gate is NOT
   `upstream_total.is_some()`. A sensor with `total_count_path` declared but whose query
   completes without early-stopping (all pages fetched) must still produce
   `total_available = total_rows` (complete result). ADR-060 §D8.11.5 is explicit: the
   upstream total is semantically valid only for truncated (early-stopped) results.

3. **Hard invariant `total_available >= total_rows` enforced by `.max(total_rows)` in
   the formula.** The `.map(|n| n.max(total_rows))` clause is NOT optional. Dropping it
   exposes the caller to `total_available < returned_results`, which is incoherent.

4. **`is_truncated` formula MUST remain `(total_rows > limit) || any_early_stopped ||
   any_pipeline_truncated`.** Adding `|| (total_available > returned_results)` to
   `is_truncated` would be WRONG — it would break the case where `any_early_stopped = true`
   and `total_rows == limit` (EC-11-092). The `is_truncated` computation path must not be
   touched by this story. If the implementer believes `is_truncated` needs updating, stop
   and route to product-owner/architect.

5. **`total_count_path` is TOP-LEVEL key only (ADR-060 §D8.11.1/§D8.11.8).** The spec
   field `total_count_path = "meta.total"` (nested path) is FORBIDDEN. The TOML parser
   must accept any string but the adapter must only perform a top-level `.get(key)` lookup
   on the response body root. Nested JSONPath support is deferred to
   `S-JSON-EXTRACT-NESTED-001`. If a `total_count_path` value contains a `.` separator,
   the top-level lookup will fail to find the key → `upstream_total = None` (graceful
   degradation per §D8.11.1 Validation row) — do not add dot-notation splitting logic.

6. **Scheduled-path (cache-hit) `total_available` left as `total_rows`.** The second
   `total_available: total_rows` assignment in `engine.rs` at the cache-hit path is
   intentionally NOT updated by this story. `upstream_total` is not available on a
   cache-hit. EC-008 documents this as accepted behavior. Changing the cache-hit path
   to produce a stale `upstream_total` would be wrong.

7. **No `--no-verify` hook bypass.** Per CLAUDE.md non-negotiable git rules. If a
   pre-commit hook fails, fix the root cause.

8. **Volatile line-number cites forbidden in code comments.** Per TD-VSDD-091/092:
   new code comments must reference function names and ADR section anchors
   (e.g., `ADR-060 §D8.11.5`), not `engine.rs:NNN` line numbers.

9. **`FetchOutput` is `#[non_exhaustive]`.** External crates use `FetchOutput::new()` —
   never struct literal syntax. Internal same-crate code may use struct literals but
   must update all existing internal literals for the new field.

10. **Arc-DI plumbing (ADR-022).** This story does not add new injectable dependencies.
    The `upstream_total` field flows through existing data structs. No new `Arc<dyn ...>`
    is required.

---

## Library & Framework Requirements

All versions pinned in workspace `Cargo.toml` — use workspace pins, not standalone versions.

| Dependency | Version | Note |
|-----------|---------|------|
| `serde_json` | workspace pin | `body.get(total_count_path)` extraction in spec_driven_adapter.rs |
| `arrow` / `arrow-json` | workspace pin | RecordBatch type in FetchOutput; no new usage |
| Rust toolchain | per `rust-toolchain.toml` | Stable channel; edition 2024 |

No new crate dependencies are introduced by this story.

### Forbidden Dependencies

- **`prism_spec_engine::types::ColumnType`** — RETIRED shadow enum per ADR-024. This
  story does not touch column types directly but the sibling-sweep in T-A02 traverses
  test files. If any test file imports the retired enum during the sweep, fix it in-scope
  per CLAUDE.md §Forbidden patterns.
- **`reqwest::Client::new()` without `.timeout()`** — if spec_driven_adapter.rs uses
  a reqwest client, it must already use the 30s timeout. This story must not regress that
  configuration. Do not add new reqwest clients without timeout.

---

## File Structure Requirements

### Files to CREATE

| File | Purpose |
|------|---------|
| `crates/prism-bin/tests/bc_2_11_001_true_total_tests.rs` | 11 Red Gate tests (RG-QTT-001..011) |

### Files to MODIFY

| File | Change | Phase |
|------|--------|-------|
| `crates/prism-sensors/src/adapter.rs` | Add `upstream_total: Option<usize>` to `FetchOutput`; update `FetchOutput::new()` to 4-param | Phase A |
| `crates/prism-sensors/src/fanout.rs` | Add `upstream_total: Option<usize>` to `FanOutResult`; add max() aggregation in `fan_out()` | Phase A |
| `crates/prism-query/src/materialization.rs` | Add `upstream_total: Option<usize>` to `MaterializationOutput` | Phase A |
| `crates/prism-mcp/tests/` (5 files, 8 sites) | Update `FetchOutput::new(...)` to 4-arg form, passing `None` | Phase A (T-A02 sibling sweep) |
| `crates/prism-bin/tests/` (1 file — mcp_integration_tests.rs, 4 sites) | Update `FetchOutput::new(...)` to 4-arg form, passing `None` | Phase A (T-A02 sibling sweep) |
| `crates/prism-query/tests/` (5 files, 27 sites) | Update `FetchOutput::new(...)` to 4-arg form, passing `None` | Phase A (T-A02 sibling sweep) |
| `crates/prism-query/src/tests/defect_csdevices_empty_memtable_tests.rs` (2 sites) | Update `FetchOutput::new(...)` to 4-arg form, passing `None` | Phase A (T-A02 sibling sweep) |
| `crates/prism-query/src/materialization.rs` (1 production site) | Update `FetchOutput::new(...)` to 4-arg form, passing `None` | Phase A (T-A02 sibling sweep) |
| `crates/prism-sensors/tests/` (3 files, 3 sites) | Update `FetchOutput::new(...)` to 4-arg form, passing `None` | Phase A (T-A02 sibling sweep) |
| `crates/prism-sensors/src/fanout.rs` (1 FetchOutput struct literal) | Add `upstream_total: None` field to struct literal | Phase A (T-A02 sibling sweep) |
| `crates/prism-sensors/src/tests/bc_2_01_002.rs` (1 FetchOutput struct literal) | Add `upstream_total: None` field to struct literal | Phase A (T-A02 sibling sweep) |
| `crates/prism-sensors/src/tests/bc_2_01_010.rs` (1 FetchOutput struct literal) | Add `upstream_total: None` field to struct literal | Phase A (T-A02 sibling sweep) |
| `crates/prism-mcp/src/server.rs` (1 site) | Update `FetchOutput::new(...)` to 4-arg form, passing `None` | Phase A (T-A02 sibling sweep) |
| `crates/prism-spec-engine/src/spec_parser.rs` | Add `total_count_path: Option<String>` to `TableSpec` | Phase B |
| `crates/prism-bin/src/spec_driven_adapter.rs` | Extract `PaginationCursor.total_count` → pass as `upstream_total` to `FetchOutput::new(...)` | Phase B |
| `crates/prism-sensors/specs/claroty.sensor.toml` | Add `total_count_path = "total"` to `alerts` and `devices` tables (key is `"total"`, not `"total_count"`) | Phase B |
| `crates/prism-query/src/engine.rs` | Update Step 6 `total_available` to gated formula | Phase C |
| `CHANGELOG.md` | Add [Unreleased] > Fixed entry | Phase E |

### Files NOT to touch

- `.factory/specs/behavioral-contracts/BC-2.11.001-*` — FROZEN at v1.38; implementer must NOT modify
- `.factory/specs/architecture/decisions/ADR-060-*` — FROZEN at v1.28; implementer must NOT modify
- `.factory/specs/architecture/decisions/ADR-058-*` — out of scope (S-CLAROTY-OCSF-REMEDIATION-001)
- `crates/prism-mcp/src/safety_envelope.rs` — out of scope (S-MCP-ENVELOPE-DESCRIBE-001)
- `crates/prism-mcp/src/tools/prism_describe.rs` — out of scope (S-MCP-ENVELOPE-DESCRIBE-001)
- `crates/prism-core/src/virtual_fields.rs` — out of scope; no virtual field changes
- `crates/prism-sensors/src/pagination.rs` — `PaginationCursor.total_count` is already set
  by `pagination.rs::advance()`; the ONLY change needed is CONSUMING it in `spec_driven_adapter.rs`
  (Phase B). Do NOT modify the pagination halting logic or `PaginationCursor` struct itself.

---

## Holdout Authoring Note

`behavioral_contracts: [BC-2.11.001]` is non-empty. Per the story-level holdout gate
protocol (D-1715/D-1716, human-approved 2026-07-13), the product-owner must author 2–4
HIDDEN, SINGLE-USE holdout scenarios for this story at story-materialization time (the
same touchpoint as the remove-uncertainty pass). Suggested holdout scenario themes:
- A `prism_query` call against Claroty devices with LIMIT=25, verifying `total_available`
  in the MCP JSON response reflects the true upstream device count (wire-level assertion)
- A query where `any_early_stopped = false` (complete result), verifying `total_available`
  equals `returned_results` (gate-closed branch does not overclaim)
- A multi-sensor query (Claroty + another sensor) verifying max() aggregation produces
  the correct `total_available`

Holdout scenarios are stored in the holdout directory that test-writer/implementer never read.

---

## History

| Version | Date | Change |
|---------|------|--------|
| 1.2 | 2026-09-17 | Re-pinned ADR-060 v1.27→v1.28 (§D8.11.2 PaginationCursor.total_count type correction; §D8.11.6 unchanged) and BC-2.11.001 v1.37→v1.38 (EC-11-095 total_count notation fix; MUST table + formula unchanged). Version-only pin refresh; no AC/RG/behavioral change. |
| 1.1 | 2026-09-17 | D-1110 remove-uncertainty pass. (1) FetchOutput::new call-site count corrected ~20 → 46 code `::new` sites + 3 struct literals (confirmed by `rg 'FetchOutput::new\(' crates/ --type rust \| grep -v '///'`); updated in frontmatter risk comment, Architecture Mapping table, Token Budget table, T-A02 task (full file-level breakdown added with missing prism-sensors/tests/, prism-query/src/materialization.rs production site, prism-query/src/tests/ inline tests, and 3 FetchOutput struct literals), File Structure Requirements table, and Previous Story Intelligence. (2) Claroty `total_count_path` key corrected `"total_count"` → `"total"` per BC-2.11.001 v1.37 EC-11-095 canonical test vector (line 137 says `total_count_path = "total"`) and ADR-060 §D8.11.1 example (`total_count_path = "total"`); DTU confirmed: `crates/prism-dtu-claroty/src/routes/alerts.rs` and `devices.rs` emit `{"total": N, ...}` not `{"total_count": N}`; corrected in AC-001 parenthetical, T-B01 doc comment example, T-B02 description + TOML comment + TOML value, EC-006. (3) T-A03 compile error fixed: `upstream_total: false` → `upstream_total: None` (`FetchOutput.upstream_total` is `Option<usize>`, not `bool`). Spec defect identified for routing: ADR-060 §D8.11.2 claims `PaginationCursor.total_count: Option<usize>` but actual code (`pagination.rs`) has `pub total_count: usize` with `usize::MAX` sentinel. Story's T-B03 sentinel guard is correct for the actual type. Route to architect. |
| 1.0 | 2026-09-17 | Initial story decomposition from Batch-0 frozen spec (BC-2.11.001 v1.37, ADR-060 v1.27 §D8.11). 11 ACs per BC-2.11.001 EC-11-095 MUST-1..11; 11 Red Gate tests RG-QTT-001..011 per ADR-060 §D8.11.6 mandate anchors. |
