---
document_type: story
story_id: S-MCP-NULL-ENCODING-001
title: "Fix null/list null encoding in build_column_array and map_record"
wave: 2
epic_id: E-BETA3-REMEDIATION
version: "1.2"
status: ready
producer: story-writer
phase: 3
priority: P0
points: 3
tdd_mode: strict
target_module: prism-bin
subsystems:
  - SS-01
  - SS-10
  - SS-16
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns this story's scope because `prism-spec-engine` is listed
#     under SS-01 per ARCH-INDEX (SS-01 row: "prism-sensors, prism-spec-engine, prism-dtu-*").
#     `column_mapping.rs` in `prism-spec-engine` is a direct fix target.
#   SS-10 (MCP Interface) owns this story's scope because `prism-bin` is listed under SS-10
#     per ARCH-INDEX (SS-10 row: "prism-mcp, prism-bin (planned — S-WAVE5-PREP-01)").
#     `spec_driven_adapter.rs` in `prism-bin` is the primary fix target (`build_column_array`).
#   SS-16 (Spec Engine) owns this story's scope because BC-2.16.003 (the governing behavioral
#     contract) is assigned to SS-16 per ARCH-INDEX, and `prism-spec-engine/column_mapping.rs`
#     is owned by SS-16. Same subsystem assignment used by S-ADR058-OCSF-COERCION-001,
#     the closest predecessor story for these same crates.
crates_touched:
  - prism-bin
  - prism-spec-engine
estimated_days: 0.5
depends_on:
  - S-MCP-TOOL-GATE-001
# depends_on anchor justification:
#   S-MCP-TOOL-GATE-001: delta analysis recommends merging the lowest-blast-radius story first
#   to reduce merge-conflict risk on spec_driven_adapter.rs for Wave-2 stories.
#   Both stories touch crates/prism-bin/; serializing them avoids conflicts.
blocks:
  - S-BETA3-RELEASE-001
  - S-CLAROTY-OCSF-REMEDIATION-001
# blocks anchor justifications:
#   S-BETA3-RELEASE-001: W2 must merge before the beta.3 release bundle assembles.
#   S-CLAROTY-OCSF-REMEDIATION-001: both stories touch spec_driven_adapter.rs; merge this
#     story first (lower blast radius, no struct changes) per delta-analysis Part 2 note:
#     "RECOMMENDATION: run in separate worktrees; merge S-MCP-NULL-ENCODING-001 first."
risk: MEDIUM
# Risk justification:
#   MEDIUM — changing null-vs-string behavior may affect existing tests that assert on literal
#   "null" strings. Grep all test fixture JSON for literal "null" string values before fixing;
#   update tests to assert Arrow null cells per delta-analysis §Issue 7 Regression Risk note.
behavioral_contracts:
  - BC-2.11.001
  - BC-2.16.003
# BC status: AMENDMENTS ACTIVE (D-2544).
#
# BC-2.16.003 v1.32 amendments now active:
#   EC-016-013-006 AMENDED: Path A `build_column_array` returns `None` (Arrow null cell) for
#     `Value::Null` string input — NOT the literal string "null". Unambiguous as of v1.32.
#   EC-016-013-041 NEW: `Value::Null` elements in `Value::Array` inputs for string columns
#     are omitted from the compact JSON-list string output (not serialized as "null").
#     An all-null array produces "[]". (EC-016-013-042 was the alternative candidate ID;
#     EC-016-013-041 was assigned per D-2544.)
#
# BC-2.11.001 EC-11-079 (null-not-absent, active, v1.37):
#   This contract governs the DOWNSTREAM serialization layer (Arrow null cell → JSON null key
#   in MCP wire output). EC-11-079 is not the upstream fix target, but the story's fixes feed
#   it: once `build_column_array` produces Arrow null cells (not literal "null" strings),
#   EC-11-079's `WriterBuilder.with_explicit_nulls(true)` ensures those cells serialize as
#   JSON `null` (not absent keys). The wire-shape assertions in this story's ACs verify the
#   end-to-end chain: sensor-null → Arrow-null → JSON-null-not-absent.
#
# Spec-First Gate S-7.01 satisfied; story is unblocked for test-writer dispatch.
verification_properties: []
assumption_validations: []
risk_mitigations: []
---

# S-MCP-NULL-ENCODING-001: Fix null/list null encoding in build_column_array and map_record

## Authority

**beta3-remediation-delta-analysis.md §Issue 7 + §S-MCP-NULL-ENCODING-001** is the
authoritative design decision for this story. Read Part 1 §Issue 7 and Part 3
§S-MCP-NULL-ENCODING-001 in full before implementing.
Path: `.factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md`

**BC-2.16.003** (column-to-ocsf-mapping) — EC-016-013-006 (Value::Null pass-through for
string columns) and §Invariants ("NULL vs absent" rule) govern the upstream ingestion behavior
in `build_column_array` and `map_record`. This story implements amendments to these two
sub-clauses.
Path: `.factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md`

**BC-2.11.001** — EC-11-079 (row-shape null-not-absent wire-shape) governs the downstream
MCP wire serialization. The fix in this story feeds EC-11-079: once `build_column_array`
produces Arrow null cells, `WriterBuilder.with_explicit_nulls(true)` (already shipped per
DEFECT-MCP-ROWSHAPE-NULLS-001) serializes them as JSON `null` (not absent keys). This story
verifies the end-to-end chain at the wire level.
Path: `.factory/specs/behavioral-contracts/BC-2.11.001-query-mcp-tool.md`

**ADR-058** (OCSF column routing) §H (coercion rules) and §I2 (raw_extensions aggregation)
provide the design context for Path A (`build_column_array`) null handling.
Path: `.factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md`

**SAP-2 obligation**: For any Claroty table column that is a nullable string, the DTU route
emission site in `crates/prism-dtu-claroty/src/routes/` MUST emit `null` (not the string
`"null"`) for absent or null optional fields. The implementer MUST read the DTU emission site
— not just the struct definition — before writing the SAP-2 parity test (AC-003/RG-NULL-003).
Reference: CLAUDE.md §SAP-2 probe rule 6 (emission-site authority).

> NOTE: BC-2.16.003 v1.32 amendments are now active per D-2544 — EC-016-013-006 (amended)
> and EC-016-013-041 (new). Spec-First Gate S-7.01 satisfied; story is unblocked for
> test-writer dispatch.

---

## Problem Statement

Two distinct sub-bugs observed in the beta.2 Monroe demo live-test log against a real Claroty
xDome tenant (jea-readapi), both rooted in `build_column_array` in `spec_driven_adapter.rs`:

**Issue 7a — String columns: JSON `null` → literal string `"null"` instead of Arrow null:**

> **D-1110 remove-uncertainty finding (2026-09-16):** Issue 7a is ALREADY FIXED on the
> current develop branch. The `ColumnType::String` arm of `build_column_array` has
> `serde_json::Value::Null => None` as its first explicit match arm (added in commit
> `fff6e28ba feat(enrichment): ENRICH-1/2/3/4-B integration`, June 23, 2026). This was
> present before the beta.2 Monroe demo; the demo was run against a build that predated
> this commit. The code behavior now matches BC-2.16.003 EC-016-013-006 (amended).
>
> Consequence for this story: RG-NULL-001 is a LOCK-IN REGRESSION GUARD (GREEN immediately),
> not a Red Gate failing test. Phase B (T-B01) must VERIFY the existing fix, not implement
> a new one. Only Issue 7b (array element null filtering) requires an implementation fix.
> Red Gate density: 3 genuinely failing tests (RG-NULL-002, RG-NULL-003, RG-NULL-004).

When a sensor API returns a JSON `null` for an optional string field (e.g., `source_ip: null`),
`build_column_array` in the beta.2 release build called `Value::Null.to_string()` (via the
wildcard arm in the String branch), which produced the string `"null"`. The Arrow `Utf8`
column contained the four-character string "null" instead of an Arrow null cell.

**This behavior is fixed on develop.** The `ColumnType::String` branch now has
`serde_json::Value::Null => None` before the wildcard arm. The downstream effects below
describe what the beta.2 release exhibited; they do NOT describe current develop behavior.

**Beta.2 downstream effects (historical, fixed on develop):**
- `SELECT * FROM claroty_alerts WHERE source_ip IS NULL` returned 0 rows (the column had
  the string "null", not an Arrow null cell — the predicate never fired)
- LLM agents reading serialized row JSON saw `"source_ip": "null"` (string) instead of
  `"source_ip": null` (JSON null) — the agent could not distinguish a null field from a field
  whose value happened to be the string "null"
- BC-2.16.003 EC-016-013-006 was violated: the contract says `Value::Null` is "placed in
  OCSF field" (implying null cell), not "stringified and placed as the string 'null'"

**Issue 7b — List columns: `Value::Null` elements in arrays → `["null"]` in raw_extensions:**

When a Tier-2 column (one that aggregates into `raw_extensions`) has an API response value
that is an array containing `null` elements (e.g., `tag_list: [null]` or `tag_list: [null, null]`),
the `Value::Array` arm in the String branch of `build_column_array` serializes each element
via `other.to_string()`. `Value::Null.to_string()` → `"null"`, producing `["null"]` in the
compact JSON-list string stored under `raw_extensions`.

**Downstream effects:**
- Agents and analysts querying `raw_extensions` see `{"tag_list": "[\"null\"]"}` — a JSON-list
  string containing the element `"null"` (a string) — instead of `"[]"` (no real elements)
- `json_extract_string(raw_extensions, 'tag_list')` returns `"[\"null\"]"`, which is
  semantically wrong: the column is null/empty, not a list containing the string "null"
- Confuses downstream processing that attempts to parse the compact JSON-list string back
  into a typed list

---

## Narrative

As a PrismQL query engine and MCP tool consumer, I want nullable string columns and
list columns containing null elements to materialize as Arrow null cells and empty
JSON-list strings respectively, so that `IS NULL` predicates, LLM agent wire-shape
inspection, and `raw_extensions` key lookups all behave correctly.

---

## Behavioral Contracts

| BC | Title | Version at Authoring | Scope in This Story |
|----|-------|---------------------|---------------------|
| BC-2.16.003 | Column-to-OCSF Mapping | v1.32 | §Edge Cases EC-016-013-006 (amended): `Value::Null` for string column → Arrow null cell (not literal "null" string); §Invariants null-vs-absent rule; EC-016-013-041 (new): null elements in `Value::Array` arm omitted |
| BC-2.11.001 | Query MCP Tool | v1.37 (active) | EC-11-079: null-not-absent wire-shape — this story's upstream fix feeds the EC-11-079 guarantee at the MCP serialization layer |

**BC-2.16.003 amendment required before test-writer dispatch:**
1. Update EC-016-013-006 to explicitly state Path A returns `None` (Arrow null cell) for
   `Value::Null` string input — not `Some("null")`.
2. Add EC-016-013-041 (or amend EC-016-013-026) to specify that `Value::Null` elements within
   `Value::Array` inputs in the String arm MUST be omitted, not serialized as `"null"`.

---

## Acceptance Criteria

### AC-001 — String column with JSON `null` input materializes as Arrow null cell

When `build_column_array` processes a sensor API response where a `column_type = "string"`
column's value is JSON `null` (`serde_json::Value::Null`), the function returns `None`
(Arrow null cell). It MUST NOT return `Some("null")` (the literal four-character string).

Wire-shape assertion (SID-2, CLAUDE.md wire-shape assertion discipline): serialize the query
result to JSON and assert that the nullable string column's value is JSON `null` (not the
JSON string `"null"`). Concretely: `{"source_ip": null}` is correct; `{"source_ip": "null"}`
is a contract violation.

The `IS NULL` predicate MUST correctly identify null cells: `WHERE source_ip IS NULL` returns
the row; `WHERE source_ip = 'null'` does NOT (no row has the string value "null").

(traces to BC-2.16.003 EC-016-013-006 amended postcondition: Path A `build_column_array`
returns `None` for `Value::Null` string input, NOT `Some("null")`)

### AC-002 — List column with all-null array elements produces `"[]"` not `"[\"null\"]"` in raw_extensions

When `build_column_array` processes a `column_type = "string"` Tier-2 column whose API
response value is `serde_json::Value::Array(arr)` where every element `arr[i]` is
`serde_json::Value::Null`, the function produces the empty compact JSON-list string `"[]"`
— NOT the string `"[\"null\"]"` (JSON-list containing the element "null").

More generally: when serializing the compact JSON-list string for a `Value::Array` input,
`Value::Null` elements MUST be omitted from the output. A mixed array
`[null, "192.168.1.1", null]` produces `"[\"192.168.1.1\"]"` (nulls filtered out).

Wire-shape assertion (SID-2): parse the `raw_extensions` JSON object for a row with a
null-valued list column and assert the key's value is `"[]"`, not `"[\"null\"]"`.

(traces to BC-2.16.003 new EC-016-013-041: `Value::Null` elements in `Value::Array` arm
are omitted, not serialized as the string "null"; consistent with §Invariants null-vs-absent
rule that nulls are not fabricated as string data)

### AC-003 — SAP-2 parity: Claroty DTU emits JSON null for nullable string fields; TOML pipeline materializes Arrow null cell

For a Claroty table with at least one nullable string column (e.g., `claroty_alerts.source_ip`
or equivalent): the DTU route handler in `crates/prism-dtu-claroty/src/routes/` MUST emit
`null` (JSON null) in the wire JSON for absent or null optional string fields — not the string
`"null"`. The spec-driven adapter pipeline (Path A, `build_column_array`) MUST then produce
an Arrow null cell for that column, consistent with AC-001.

The implementer MUST read the DTU emission site directly (the route handler's `json!` body or
`serde` struct serialization) — NOT just the struct definition — to verify the wire emission
produces JSON null before writing the SAP-2 parity Red Gate test (per CLAUDE.md §SAP-2 probe
rule 6: emission-site authority supersedes struct definition).

Wire-shape assertion: assert on the serialized JSON bytes of a `prism_query` tool response
for a query against the Claroty DTU. Rows with a null optional string column MUST carry the
key with value `null` (not `"null"`). Test uses the prism-bin/prism-mcp test harness against
the Claroty DTU.

(traces to BC-2.16.003 §Invariants null-vs-absent rule: "a column present with `Value::Null`
is placed in its destination... as a JSON null value"; CLAUDE.md §SAP-2 probe obligation)

### AC-004 — Existing non-null string and array tests continue to pass

All existing tests for `build_column_array` and `map_record` that assert on non-null string
values (e.g., string `"192.168.1.1"` for `source_ip`), non-null arrays (e.g., `ip_list`
compact JSON-list string), and numeric columns MUST remain GREEN after the fix.

Specifically: EC-016-013-026 (ENRICH-1 arm for `Value::Array` with non-null elements) must
remain correct — `["192.168.1.1", "10.0.0.1"]` → `"[\"192.168.1.1\",\"10.0.0.1\"]"` is
unchanged. Only the null-element filtering is new behavior.

(traces to BC-2.16.003 EC-016-013-026: the `Value::Array` arm for non-null elements is
unchanged; regression guard)

---

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| `build_column_array` String arm — null fix (Issue 7a) | `crates/prism-bin/src/spec_driven_adapter.rs` | Pure (takes `Value`, returns `Option<String>`) |
| `build_column_array` `Value::Array` arm — null element filter (Issue 7b) | `crates/prism-bin/src/spec_driven_adapter.rs` | Pure |
| `map_record` null handling review | `crates/prism-spec-engine/src/column_mapping.rs` | Pure (Path B — zero live callers per ADR-058 §K5; fix for completeness) |
| Claroty DTU emission site review (SAP-2) | `crates/prism-dtu-claroty/src/routes/alerts.rs` (or applicable route) | Effectful (DTU HTTP route handler) |
| New test file | `crates/prism-bin/tests/bc_2_16_003_null_encoding.rs` | Pure (test assertions) |

**Architecture classification per ADR-058:**
- **Path A** (`build_column_array` in `spec_driven_adapter.rs`) is the **sole live production
  path** per ADR-058 §K5/§A1. All fixes to null handling in Path A are what matters at runtime.
- **Path B** (`map_record` in `column_mapping.rs`) has **zero live production callers** per
  ADR-058 §A1. Fix `map_record` for completeness (per the null-vs-absent §Invariants rule),
  but Path B behavior does not affect the beta.2 live-test defects.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `column_type = "string"`, sensor API returns `null` for optional column (e.g., `source_ip: null`) | Arrow null cell; wire: `"source_ip": null` (JSON null, not `"null"`) |
| EC-002 | `column_type = "string"`, sensor API returns `[null]` for an array (raw list Tier-2 column) | Compact JSON-list string `"[]"` — null elements filtered; NOT `"[\"null\"]"` |
| EC-003 | `column_type = "string"`, sensor API returns `[null, "192.168.1.1", null]` (mixed null + non-null) | Compact JSON-list string `"[\"192.168.1.1\"]"` — null elements omitted |
| EC-004 | `column_type = "string"`, sensor API returns `[]` (empty array) | Compact JSON-list string `"[]"` — existing EC-016-013-026 empty-array behavior unchanged |
| EC-005 | `column_type = "string"`, sensor API returns a non-null string value (regression guard) | Non-null string passed through unchanged; no regression from null-handling fix |
| EC-006 | `column_type = "integer"` or `"boolean"`, sensor API returns `null` | Existing Rule 3 pass-through behavior unchanged per EC-016-013-006 (only String columns are affected by issue 7a) |
| EC-007 | `map_record` Path B: `Value::Null` for a string column | `map_record` places `Value::Null` in the destination as JSON null value per §Invariants null-vs-absent rule; NOT the string "null". Path B fix mirrors Path A fix for completeness. |
| EC-008 | Claroty DTU route emits `null` for optional string field absent from fixture data | DTU JSON emission contains `"optional_field": null` (JSON null); spec-driven pipeline produces Arrow null cell; wire: `"optional_field": null` |

---

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story file | ~5,000 | |
| BC-2.16.003 v1.32 (active — primary contract) | ~25,000 | Large file — §EC table, §Coercion Matrix, §Invariants; EC-016-013-006 amended + EC-016-013-041 new |
| BC-2.11.001 v1.37 (§Postconditions + §Edge Cases EC-11-079 only) | ~8,000 | Read targeted sections only; full file is 44k tokens |
| `spec_driven_adapter.rs` (`build_column_array` function + String arm context) | ~8,000 | Targeted read of the function block; large file overall |
| `column_mapping.rs` (`map_record` function) | ~4,000 | Targeted read |
| `crates/prism-dtu-claroty/src/types.rs` (alert + device structs) | ~3,000 | SAP-2 parity check — struct definitions |
| Claroty DTU route handler (alerts or devices, whichever has nullable strings) | ~3,000 | SAP-2 parity check — emission site |
| New test file `bc_2_16_003_null_encoding.rs` | ~3,000 | 4 Red Gate tests |
| `beta3-remediation-delta-analysis.md` §Issue 7 + §S-MCP-NULL-ENCODING-001 | ~2,000 | Reference |
| ADR-058 §H + §I2 (coercion rules + raw_extensions aggregation) | ~4,000 | Context for Path A design |
| **Total estimated** | **~65,000** | Fits comfortably within one context window |

---

## Tasks

### Red Gate tests (to be written by test-writer BEFORE implementation)

All tests live in `crates/prism-bin/tests/bc_2_16_003_null_encoding.rs` (new file)
unless noted otherwise.

**PREREQUISITE MET (D-2544):** BC-2.16.003 v1.32 amendments are active — EC-016-013-006
(amended) and EC-016-013-041 (new) have reached `status: active`. Spec-First Gate S-7.01
satisfied; test-writer dispatch is unblocked.

- [ ] **RG-NULL-001**: `test_BC_2_16_003_string_column_json_null_materializes_as_arrow_null`
  Assert: `build_column_array` called with `column_type = "string"` and `serde_json::Value::Null`
  as the field value returns `None` (Arrow null cell). Also assert the NEGATIVE: the return
  value is NOT `Some("null")` (the literal string). Wire-shape (SID-2): construct a
  RecordBatch with the Arrow null cell, serialize via `WriterBuilder::with_explicit_nulls(true)`,
  assert the JSON row contains `"column_name": null` (JSON null), NOT `"column_name": "null"`
  (JSON string).
  **Currently PASSES on develop (lock-in regression guard)** — the `ColumnType::String` arm
  already has `serde_json::Value::Null => None` (commit `fff6e28ba`, June 2026). Write this
  test to lock the existing correct behavior; do NOT treat it as driving a code fix. It is
  NOT a Red Gate failing test; do NOT count it toward the Red Gate density check.
  AC-001.

- [ ] **RG-NULL-002**: `test_BC_2_16_003_array_with_all_null_elements_produces_empty_json_list`
  Assert: `build_column_array` called with `column_type = "string"` and
  `serde_json::Value::Array(vec![Value::Null])` returns `Some("[]")` (compact empty
  JSON-list string). Also test with `vec![Value::Null, Value::Null]` → `Some("[]")`.
  Negative assertion: return value MUST NOT be `Some("[\"null\"]")`.
  Currently FAILS (returns `Some("[\"null\"]")`).
  AC-002.

- [ ] **RG-NULL-003**: `test_BC_2_16_003_mixed_null_and_string_elements_filters_nulls`
  Assert: `build_column_array` called with `column_type = "string"` and
  `serde_json::Value::Array(vec![Value::Null, Value::String("192.168.1.1".into()), Value::Null])`
  returns `Some("[\"192.168.1.1\"]")` (only non-null elements retained).
  AC-002 (mixed array sub-case).

- [ ] **RG-NULL-004**: `test_BC_2_16_003_sap2_claroty_nullable_string_null_via_dtu`
  SAP-2 parity test: launch the Claroty DTU (or use a fixture JSON), run a `prism_query` call
  that projects a nullable string column from a Claroty table (e.g., `claroty_alerts.source_ip`
  or equivalent column confirmed nullable in the DTU struct). For rows where the DTU emits
  `null` for the optional field:
  (a) Assert DTU wire emission: the JSON envelope contains `"column_name": null` (JSON null)
      — verified by reading the DTU route handler source directly (SAP-2 probe rule 6)
  (b) Assert Arrow materialization: the Arrow column for that row is a null cell
  (c) Assert MCP wire output: serialized query response JSON contains `"column_name": null`
      (JSON null, not `"null"` string, and not key-absent — EC-11-079)
  Currently FAILS for (b) and (c) if the DTU emits `null` and the pipeline converts it to
  the string "null".
  AC-003.

**Note on non-null regression guard:** the existing test suite for `build_column_array`
in `crates/prism-bin/src/spec_driven_adapter.rs` (inline `#[cfg(test)] mod tests`) includes
tests for non-null string and array values (EC-016-013-026 coverage). These are NOT new Red
Gate tests — they are existing passing tests that must REMAIN GREEN after the fix. The
implementer verifies them as part of T-F01 verification.

**Red Gate density check (BC-5.38.001):** **3 genuinely failing tests** (RG-NULL-002,
RG-NULL-003, RG-NULL-004) before implementation begins. RG-NULL-001 is a lock-in regression
guard (GREEN immediately per D-1110 finding — Issue 7a already fixed on develop). The story
has 4 ACs; AC-004 is a regression guard. Density: 3 failing RG tests / 4 ACs = 0.75 —
satisfies the ≥ 0.5 threshold. All non-trivial function bodies use `todo!()` stubs.

### Implementation tasks (to be executed by implementer AFTER Red Gate)

#### Phase A — Pre-fix grep for literal "null" string in tests

- [ ] **T-A01**: Before writing any fix code, run:
  ```bash
  rg '"null"' crates/ --type rust -l
  ```
  Identify all Rust test files that assert on the literal string `"null"` in JSON or Arrow
  context. List them. These tests may need to be updated after the fix to assert Arrow null
  cells instead. Document the list in the PR description as a regression risk inventory.

#### Phase B — Verify `build_column_array` String arm (Issue 7a already fixed)

- [ ] **T-B01**: **VERIFICATION ONLY — no code change expected.** In
  `crates/prism-bin/src/spec_driven_adapter.rs`, locate the `ColumnType::String` branch of
  `build_column_array`. Verify the first explicit match arm is `serde_json::Value::Null => None`
  (not a wildcard `other => other.to_string()`). This fix was landed in commit `fff6e28ba`
  (June 2026); if it is present, T-B01 is complete with NO code change.

  If — unexpectedly — the `serde_json::Value::Null => None` arm is absent, STOP and report
  to the orchestrator. Do NOT silently add it: verify the issue is real before writing a fix.

  After T-B01 verification, run RG-NULL-001. It MUST be GREEN (it is a lock-in test for
  existing behavior, not a test driving a new fix — per D-1110 remove-uncertainty finding).

#### Phase C — Fix `build_column_array` `Value::Array` arm (Issue 7b)

- [ ] **T-C01**: In `crates/prism-bin/src/spec_driven_adapter.rs`, find the `Value::Array(arr)`
  arm within the `ColumnType::String` branch of `build_column_array`. This arm currently
  serializes all elements via `other.to_string()`, which produces `"null"` for null elements.
  Change the array element serialization to SKIP `Value::Null` elements:

  ```rust
  // Before (approximate — read actual code):
  let joined: String = arr.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",");

  // After (null-element filtering):
  let elements: Vec<String> = arr.iter()
      .filter(|v| !v.is_null())
      .map(|v| /* existing stringification logic */)
      .collect();
  let compact = format!("[{}]", elements.join(","));
  Some(compact)
  ```

  The existing behavior for non-null elements MUST be unchanged (EC-016-013-026 regression
  guard). Empty array `[]` and all-null array both produce `"[]"`.

  After T-C01, RG-NULL-002 and RG-NULL-003 must be GREEN.

#### Phase D — Fix `map_record` Path B (Issue 7a mirror, completeness)

- [ ] **T-D01**: In `crates/prism-spec-engine/src/column_mapping.rs`, review `map_record`
  (Path B) for the same `Value::Null` → string-"null" conversion. Per ADR-058 §K5, Path B
  has zero live production callers — but the §Invariants contract requires null to be placed
  as JSON null value, not the string "null". Apply the symmetric fix to Path B for
  completeness. This is a LOW priority within this story (Path B is not a live defect path).

  If the `map_record` implementation already handles `Value::Null` correctly (places JSON null
  in the destination), document this in the PR description and skip T-D01.

#### Phase E — SAP-2 parity verification

- [ ] **T-E01**: Before writing RG-NULL-004, read the Claroty DTU route handler source for
  the table with nullable string columns. Specifically read the `json!` body or
  `serde` struct for the route that emits optional string fields. Verify the wire emission
  contains `null` (JSON null) not the string `"null"` for absent or null optional fields.

  If the DTU route handler DOES emit `"null"` (string) for absent fields (a separate DTU
  defect), flag it as a SAP-2 finding and report to the orchestrator. Do NOT silently fix
  it in this story — that is a DTU defect in scope for S-CLAROTY-OCSF-REMEDIATION-001.

  If the DTU is clean (emits JSON null), proceed to write RG-NULL-004.

#### Phase F — Regression verification and final checks

- [ ] **T-F01**: Run `cargo test -p prism-bin --no-fail-fast`.
  All tests must pass. RG-NULL-001..004 must be GREEN.
  Existing tests for non-null string and array values (EC-016-013-026 coverage) must remain GREEN.

- [ ] **T-F02**: Run `cargo test -p prism-spec-engine --no-fail-fast`.
  All tests must pass (Path B `map_record` fix, if T-D01 was applied).

- [ ] **T-F03**: Run `just check` (full workspace). Must exit 0.

#### Phase G — CHANGELOG

- [ ] **T-G01** (BEFORE creating the PR): Add a CHANGELOG entry under `[Unreleased] > Fixed`:
  ```markdown
  - Fix null string encoding in spec-driven adapter: sensor API `null` values for optional
    string columns now materialize as Arrow null cells instead of the literal string "null".
    `IS NULL` predicates and wire-shape null assertions now work correctly for nullable string
    columns. Array columns with null elements no longer produce `["null"]` in `raw_extensions`
    JSON blobs. Resolves beta.2 live-test issue 7.
  ```

---

## Previous Story Intelligence

**S-MCP-TOOL-GATE-001** (Wave 1, E-BETA3-REMEDIATION epic):
- Landed the `operations` Cargo feature gate in `prism-mcp`.
- Relevant for merge ordering: merge S-MCP-TOOL-GATE-001 before this story to reduce
  `spec_driven_adapter.rs` merge-conflict risk.
- No code lessons directly applicable — S-MCP-TOOL-GATE-001 is a Cargo feature story;
  this story is a null-handling fix in the sensor adapter pipeline.

**S-ADR058-OCSF-COERCION-001** (Wave prior, same crate scope):
- Fixed `column_type = "string"` + `Value::Object` → null cell + `column_coercion_failure`
  warn (EC-016-013-008, AC-005). This is the CLOSEST predecessor story.
- Lesson: the wildcard arm in the String branch of `build_column_array` was previously the
  source of coercion bugs (it called `other.to_string()` for all non-matched types). That fix
  replaced the wildcard for `Value::Object`. This story must find the remaining `Value::Null`
  case in the same branch.
- Lesson from S-ADR058-OCSF-COERCION-001 adversary cascade: always read the DTU struct AND
  the route handler's `json!` body (SAP-2 probe rule 6) — the struct may carry a field
  that the handler body doesn't emit.
- Lesson: subsystem attribution for prism-bin = SS-10 (ARCH-INDEX SS-10 row: "prism-mcp,
  prism-bin (planned — S-WAVE5-PREP-01)") — confirmed correct per S-ADR058-OCSF-COERCION-001
  v1.6 correction.

---

## Architecture Compliance Rules

1. **Path A is the sole live production path (ADR-058 §A1/§K5).** All runtime behavior at
   issue is in Path A (`build_column_array` in `spec_driven_adapter.rs`). Path B (`map_record`
   in `column_mapping.rs`) has zero live production callers. Fix both for contract completeness,
   but the live defect is exclusively Path A.

2. **Do not change `Value::Array` non-null behavior (EC-016-013-026 protection).** The
   `Value::Array` arm's existing behavior for non-null elements — compact JSON-list string,
   ENRICH-1 wildcard columns, `"[]"` for empty array — is CORRECT and MUST NOT change.
   Only null-element filtering is new. Changing non-null behavior regresses `ip_list`,
   `mac_list`, `network_list`, `vlan_list` columns on Claroty devices.

3. **Read DTU emission site before writing SAP-2 parity test (SAP-2 probe rule 6).** The
   DTU route handler's JSON body is authoritative for wire emission — the `ClarotyAlert`
   struct definition does not prove what the handler emits. A field in the struct absent from
   the `json!` body emits nothing on the wire.

4. **Wire-shape assertions mandatory (CLAUDE.md wire-shape discipline, 2026-07-13).** Every
   test covering the null encoding issue MUST include at least one assertion on serialized
   JSON output — not only pre-serialization Arrow structs. The key null/string distinction
   MUST be asserted at the JSON byte level.

5. **No `--no-verify` hook bypass.** Pre-commit hooks must pass on every commit.

6. **No inline line-number cites in tests or comments (TD-VSDD-091).** Use function names
   and BC section anchors, not `spec_driven_adapter.rs:NNN` line numbers.

---

## Library & Framework Requirements

All versions pinned in workspace `Cargo.toml` — use workspace pins, not standalone versions.

| Dependency | Version | Note |
|-----------|---------|------|
| `serde_json` | workspace pin | `Value::Null`, `Value::Array`, `.is_null()` — stable API |
| `arrow-json` | per-crate pin (`"58"` in `crates/prism-bin/Cargo.toml`; Cargo.lock resolves to 58.2.0) | `WriterBuilder::with_explicit_nulls(true)` — required for wire-shape assertions in tests |
| Rust toolchain | per `rust-toolchain.toml` | Stable channel; edition 2024 |

No new dependencies are introduced by this story.

**Forbidden Dependencies:** `prism-bin` (via `spec_driven_adapter.rs`) MUST NOT gain any
new dependency on `prism-mcp` internal modules. The null-encoding fix is purely within the
Arrow materialization layer — no MCP tool handler dependency.

---

## File Structure Requirements

### Files to CREATE

| File | Purpose |
|------|---------|
| `crates/prism-bin/tests/bc_2_16_003_null_encoding.rs` | Red Gate tests RG-NULL-001..004 (new) |

### Files to MODIFY

| File | Change |
|------|--------|
| `crates/prism-bin/src/spec_driven_adapter.rs` | `build_column_array`: verify `Value::Null` → `None` in String branch (T-B01 verify-only — fix already landed); filter `Value::Null` elements from `Value::Array` arm (T-C01) |
| `crates/prism-spec-engine/src/column_mapping.rs` | `map_record` Path B: mirror null-handling fix (T-D01) — if not already correct |
| `CHANGELOG.md` | Add [Unreleased] > Fixed entry (T-G01) |

### Files NOT to touch

- `crates/prism-mcp/src/server.rs` — out of scope (Wave 1 story S-MCP-TOOL-GATE-001)
- `crates/prism-mcp/src/safety_envelope.rs` — out of scope (Wave 2 story S-MCP-ENVELOPE-DESCRIBE-001)
- `crates/prism-sensors/specs/claroty.sensor.toml` — out of scope (Wave 3 story S-CLAROTY-OCSF-REMEDIATION-001)
- `crates/prism-dtu-claroty/src/types.rs` — read-only for SAP-2 parity check; modifications
  are Wave 3 scope (S-CLAROTY-OCSF-REMEDIATION-001)

---

## Holdout Authoring Note

`behavioral_contracts: [BC-2.11.001, BC-2.16.003]` is non-empty. Per the story-level holdout
gate protocol (D-1715/D-1716, human-approved 2026-07-13), the product-owner must author 2–4
HIDDEN, SINGLE-USE holdout scenarios for this story at story-materialization time (the same
touchpoint as the remove-uncertainty pass). Suggested scenario areas:
- A query projecting a nullable string column against the Claroty DTU: verify wire JSON has
  `null` (not `"null"` string) for null cells
- An `IS NULL` predicate on a nullable string column: verify it returns the row that has a
  null value (not the string "null")
- A `raw_extensions` inspection for a Tier-2 list column with all-null elements: verify `[]`
  not `["null"]`

Holdout scenarios are stored in the holdout directory that test-writer/implementer never read.

---

## History

| Version | Date | Change |
|---------|------|--------|
| 1.2 | 2026-09-16 | D-1110 remove-uncertainty pass: (1) Issue 7a corrected — `Value::Null => None` already present in ColumnType::String arm of `build_column_array` (commit `fff6e28ba`, June 2026); RG-NULL-001 reclassified as lock-in regression guard (GREEN immediately, not Red Gate); T-B01 changed from fix to verify-only; density check updated to 3 failing / 4 ACs = 0.75. (2) `arrow-json` version corrected — per-crate pin `"58"` in Cargo.toml (not "workspace pin 58.2.0"); Cargo.lock resolves to 58.2.0. |
| 1.1 | 2026-09-16 | F3 BC/ADR pin propagation (D-2544): BC-2.16.003 v1.31→v1.32. AMENDMENT PENDING annotation replaced with settled reference to active ECs (EC-016-013-006 amended, EC-016-013-041 new). "EC-016-013-042" alternative ID removed. PREREQUISITE note updated to PREREQUISITE MET. |
| 1.0 | 2026-09-16 | Initial story decomposition |
