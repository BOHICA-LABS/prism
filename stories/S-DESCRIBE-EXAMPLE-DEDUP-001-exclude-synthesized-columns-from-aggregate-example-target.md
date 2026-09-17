---
document_type: story
story_id: S-DESCRIBE-EXAMPLE-DEDUP-001
title: "Fix build_example_with_note: exclude synthesized metadata columns from aggregate query target selection"
wave: 2
epic_id: E-BETA3-REMEDIATION
version: "1.1"
status: ready
producer: story-writer
phase: 3
priority: P0
points: 2
tdd_mode: strict
target_module: prism-mcp
subsystems: ["SS-10"]
# Subsystem anchor justification:
#   SS-10 (MCP Server) owns crates/prism-mcp/src/tools/prism_describe.rs,
#   which is the sole file changed by this story. build_example_with_note is a
#   pure helper function private to the prism-mcp crate. No other subsystem is crossed.
crates_touched: [prism-mcp]
estimated_days: 0.25
depends_on: []
# depends_on anchor justification:
#   No hard product-story dependencies. The delta analysis marks this story as
#   "(no deps) → can start immediately" (beta3-remediation-delta-analysis.md §Dependency Graph).
#   Soft merge-ordering note: S-MCP-ENVELOPE-DESCRIBE-001 (W2) adds three new String-typed
#   synthesized ColumnDescriptors (_client, _source_table, _source_type) to build_ocsf_column_descriptors.
#   Those are String-typed and do NOT affect the Integer/Float agg_col find() that this story fixes.
#   Both stories are safe to implement in parallel worktrees; there is no compile-time dependency.
blocks:
  - S-BETA3-RELEASE-001
# blocks anchor justification:
#   S-BETA3-RELEASE-001: this story is in Wave 2 and must merge to develop before the
#   beta.3 release bundle assembles (delta-analysis §Batch 2: depends on all W1+W2+W2-arch merged).
risk: LOW
# Risk justification:
#   Single pure function change inside prism_describe.rs — build_example_with_note.
#   One-line agg_col predicate change (add exclusion check). No cross-crate callers.
#   No behavioral regression for tables that have real domain Integer/Float columns.
#   No change to safety_envelope.rs, server.rs, or any other crate.
behavioral_contracts:
  - BC-2.10.012
# BC status: AMENDMENT ACTIVE — BC-2.10.012 v1.11 landed 2026-09-17.
#   EC-10-033 (synthesized-column exclusion for Tier-2 agg_col) is present in the BC
#   §Auto-generated example queries and §Edge Cases. Bidirectional AC↔BC traces are
#   complete: AC-001..003 cite EC-10-033; EC-10-033 in BC anchors RG-DEDUP-001..003.
#   Status remains draft pending holdout scenario authoring and product-owner promotion.
verification_properties: []
assumption_validations: []
risk_mitigations: []
---

# S-DESCRIBE-EXAMPLE-DEDUP-001: Fix build_example_with_note Aggregate Column Selection — Exclude Synthesized Metadata Columns

## Authority

**beta3-remediation-delta-analysis.md §Issue 4 + §S-DESCRIBE-EXAMPLE-DEDUP-001** is the
authoritative design specification for this story. Read Part 1 §Issue 4 and Part 3
§S-DESCRIBE-EXAMPLE-DEDUP-001 in full before implementing.
Path: `.factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md`

**BC-2.10.012 v1.11** (prism_describe Schema Discovery Tool) governs example query
generation. §Auto-generated example queries defines the 4-tier priority ladder.
EC-10-033 (synthesized-column exclusion for Tier-2 `agg_col`) is active in v1.11.
Path: `.factory/specs/behavioral-contracts/BC-2.10.012-prism-describe-schema-discovery-tool.md`

**Code site:** `crates/prism-mcp/src/tools/prism_describe.rs` — function
`build_example_with_note` (pub, line ~722), `agg_col` selection block (lines ~789–797),
inline test module `build_example_query_tests` (lines ~802+).

---

## Problem Statement

`build_example_with_note` in `prism_describe.rs` selects the aggregate GROUP BY column for
the Tier 2 example query using an unfiltered `find()` over all Integer and Float columns:

```rust
let agg_col = columns
    .iter()
    .find(|c| matches!(c.col_type, ColumnType::Integer | ColumnType::Float));
```

For Claroty tables that have no domain-data numeric columns, the first (and often only)
Integer column in the schema array is `class_uid` — the synthesized OCSF class identifier
appended by `build_ocsf_column_descriptors` (OQ-003). For example:
- `claroty_alerts` has `class_uid` (Integer, synthesized) but no earlier domain Integer columns
- `claroty_devices` has `class_uid` (Integer, synthesized) but no earlier domain Integer columns
- `claroty_audit_logs` (same pattern)

Result: `build_example_with_note` produces identical GROUP BY queries across these tables:
`SELECT class_uid, COUNT(*) FROM <table> GROUP BY class_uid ORDER BY COUNT(*) DESC LIMIT 10`

This is a semantically meaningless example — `class_uid` is a constant (value 2004 for
detection findings, 5001 for inventory info) for all rows in a given Claroty table. Grouping
by it always returns a single row with a count equal to the total row count. The LLM agent
receives identical, non-instructive examples that do not demonstrate real sensor data
differentiation.

**Root cause:** The `agg_col` predicate does not exclude synthesized/virtual metadata columns
(`class_uid`, `_sensor`, `_client`, `_source_table`, `_source_type`). These columns are
infrastructure columns appended by `build_ocsf_column_descriptors`, NOT domain-data columns.
They should never be the GROUP BY target of an example query.

**Fix (delta-analysis §Issue 4):** Add a const exclusion set and filter the `agg_col` find():

```rust
const EXAMPLE_EXCLUDED_COLS: &[&str] = &[
    "class_uid",
    "_sensor",
    "_client",
    "_source_table",
    "_source_type",
];

let agg_col = columns
    .iter()
    .find(|c| {
        matches!(c.col_type, ColumnType::Integer | ColumnType::Float)
            && !EXAMPLE_EXCLUDED_COLS.contains(&c.name.as_str())
    });
```

When every Integer/Float column is excluded (e.g., `class_uid` is the only numeric column),
`agg_col` is `None` and the function falls through to Tier 3 (count-recent, if a Datetime
column exists) or Tier 4 (column-free fallback `SELECT * LIMIT 25`).

**No new BC required.** BC-2.10.012 amendment to v1.11 sufficient per delta analysis.

---

## Narrative

As an LLM agent calling `prism_describe` to discover a Claroty client's schema, I want
each table's `example_query` to demonstrate a meaningful domain-data query using real sensor
columns, so that I can write effective PrismQL queries rather than receiving identical
`GROUP BY class_uid` queries that carry no signal about the table's actual content.

---

## Behavioral Contracts

| BC | Title | Version at Authoring | Scope in This Story |
|----|-------|---------------------|---------------------|
| BC-2.10.012 | `` `prism_describe` Schema Discovery Tool (L2) `` | v1.11 | §Auto-generated example queries Tier 2 aggregate template — EC-10-033 synthesized-column exclusion: `class_uid`, `_sensor`, `_client`, `_source_table`, `_source_type` MUST NOT be selected as `agg_col`; fallthrough to Tier 3/4 when all Integer/Float columns are excluded. |

---

## Acceptance Criteria

### AC-001 — `class_uid` is excluded from aggregate target selection when it is the only Integer column

When `build_example_with_note` is called for a table whose only Integer or Float column is
`class_uid` (a synthesized metadata column), the returned `example_query` MUST NOT contain
`class_uid` as a GROUP BY field. The query MUST NOT take the Tier 2 aggregate form
(`SELECT class_uid, COUNT(*) FROM ... GROUP BY class_uid ...`). Instead, the function
falls through to Tier 3 or Tier 4 depending on whether a Datetime column is present.

(traces to BC-2.10.012 §Auto-generated example queries Tier 2 aggregate template,
EC-10-033: "synthesized metadata columns MUST NOT be used as `agg_col`";
delta-analysis §Issue 4 canonical fix specification)

### AC-002 — Real domain Integer column is preferred over synthesized column even when `class_uid` appears earlier in the array

When `build_example_with_note` is called for a table that has `class_uid` (Integer,
synthesized) positioned before a real domain Integer column (e.g., `devices_count`) in
the `columns` array, the `EXAMPLE_EXCLUDED_COLS` exclusion MUST cause `class_uid` to be
skipped, and `devices_count` MUST be selected as the `agg_col`. The returned `example_query`
MUST reference `devices_count` in the GROUP BY form, not `class_uid`.

This assertion must hold with `class_uid` explicitly placed FIRST in the input column
array (ordering-independent exclusion is required).

(traces to BC-2.10.012 §Auto-generated example queries Tier 2 aggregate template —
"The first such [non-excluded] column in the schema array is used as `<field>`";
EC-10-033 (active); delta-analysis §Issue 4 const exclusion set)

### AC-003 — When all Integer/Float columns are excluded, fallthrough to Tier 3 (count-recent) or Tier 4 (column-free)

When `build_example_with_note` is called for a table where every Integer/Float column in
the schema array is in `EXAMPLE_EXCLUDED_COLS` (e.g., `class_uid` is the sole numeric), and
no `severity` String column is present, the function MUST fall through past Tier 2:

- If a Datetime column exists (e.g., `detected_time`): the query MUST use the Tier 3
  count-recent form — `SELECT COUNT(*) FROM <table> WHERE <datetime_col> > NOW() - INTERVAL '1h'`
- If no Datetime column exists: the query MUST use the Tier 4 column-free fallback —
  `SELECT * FROM <table> LIMIT 25`

In both cases the returned query MUST NOT contain `GROUP BY` or any reference to `class_uid`
or other excluded column names.

(traces to BC-2.10.012 §Auto-generated example queries Tier 3 count-recent postcondition:
"used when Tiers 1 and 2 did NOT fire AND a Datetime-typed column exists";
Tier 4 column-free postcondition: "used when Tiers 1/2/3 all did NOT fire";
EC-10-033 (active); delta-analysis §Issue 4)

---

## Red Gate Test List (SAC-1 — BC-5.38.001)

All three failing tests reside in the EXISTING inline `#[cfg(test)] mod build_example_query_tests`
block inside `crates/prism-mcp/src/tools/prism_describe.rs`. No external test fixture is
needed — `build_example_with_note` is a pure function that can be driven directly.

- **RG-DEDUP-001**: `test_BC_2_10_012_class_uid_excluded_from_agg_when_only_synthesized_integer`

  Setup: `columns = vec![col("detected_time", ColumnType::Datetime), col("class_uid", ColumnType::Integer)]`
  (No `severity` String column; the only Integer is the synthesized `class_uid`.)

  Assert:
  1. `!q.contains("class_uid")` — excluded column MUST NOT appear in query
  2. `!q.contains("GROUP BY")` — Tier 2 aggregate MUST NOT fire when only excluded integers exist
  3. `q.contains("detected_time")` — Tier 3 count-recent MUST fire using the Datetime column
  4. `q.contains("NOW()")` — confirms count-recent Tier 3 template

  Currently FAILS: `agg_col` find() picks `class_uid` (first Integer in array), produces
  `SELECT class_uid, COUNT(*) FROM claroty_alerts GROUP BY class_uid ORDER BY COUNT(*) DESC LIMIT 10`.
  Covers AC-001, AC-003 (Tier 3 path).

- **RG-DEDUP-002**: `test_BC_2_10_012_domain_integer_selected_over_class_uid_when_class_uid_is_first`

  Setup: `columns = vec![col("class_uid", ColumnType::Integer), col("devices_count", ColumnType::Integer)]`
  (`class_uid` is FIRST in the array — hardest case for exclusion logic.)

  Assert:
  1. `q.contains("devices_count")` — domain column MUST be selected as GROUP BY target
  2. `!q.contains("class_uid")` — excluded column MUST NOT appear in query
  3. `q.contains("GROUP BY devices_count")` — exact Tier 2 form with domain column

  Currently FAILS: `find()` picks `class_uid` (first Integer, no exclusion), produces
  `SELECT class_uid, COUNT(*) FROM ... GROUP BY class_uid ...`.
  Covers AC-002.

- **RG-DEDUP-003**: `test_BC_2_10_012_all_integers_excluded_and_no_datetime_falls_to_column_free_fallback`

  Setup: `columns = vec![col("class_uid", ColumnType::Integer), col("asset_id", ColumnType::String)]`
  (No Datetime column; only excluded Integer; no severity String column.)

  Assert:
  1. `q == "SELECT * FROM claroty_no_datetime_table LIMIT 25"` — exact Tier 4 column-free fallback
  2. `!q.contains("GROUP BY")` — Tier 2 MUST NOT fire
  3. `!q.contains("class_uid")` — excluded column MUST NOT appear
  4. `!q.contains("NOW()")` — Tier 3 count-recent MUST NOT fire (no Datetime)

  Currently FAILS: `find()` picks `class_uid`, produces aggregate query instead of fallback.
  Covers AC-003 (Tier 4 path).

**BC-5.38.001 density check:** 3 failing Red Gate tests / 3 ACs = **1.0** — satisfies the
≥ 0.5 threshold.

**Red-then-green task ordering:** RG-DEDUP-001..003 must be written by test-writer and
confirmed RED before implementer begins any implementation tasks. See §Tasks.

---

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| `build_example_with_note` — `agg_col` selection | `crates/prism-mcp/src/tools/prism_describe.rs` | Pure (no I/O; unit-testable directly) |
| `EXAMPLE_EXCLUDED_COLS` const | `crates/prism-mcp/src/tools/prism_describe.rs` | Pure (static const) |
| Red Gate tests | `crates/prism-mcp/src/tools/prism_describe.rs` — `#[cfg(test)] mod build_example_query_tests` | Pure (tests) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Table has `_sensor` (String) — already not an Integer/Float column | Not affected by agg_col exclusion; String columns are already not candidates for Tier 2; no behavioral change |
| EC-002 | Table has `_client`, `_source_table`, `_source_type` (String, added by S-MCP-ENVELOPE-DESCRIBE-001) | Same as EC-001 — String-typed; excluded from Integer/Float agg_col find() even without the const exclusion; exclusion list is defensive |
| EC-003 | Table has severity String column + class_uid Integer — severity fires Tier 1 | Tier 1 (severity-IEQ) fires FIRST per BC-2.10.012 priority ladder; Tier 2 agg_col logic is never reached; no change in behavior |
| EC-004 | Table has Float column `risk_score_normalized` (domain Float) + class_uid Integer | After exclusion: `risk_score_normalized` is selected as agg_col (non-excluded Float); correct behavior |
| EC-005 | Table has zero columns (EC-10-025) | `agg_col` find() returns None (empty slice); unchanged — falls through to Tier 4 column-free fallback `SELECT * LIMIT 25` |
| EC-006 | `EXAMPLE_EXCLUDED_COLS` includes `_sensor` — but `_sensor` is String, not Integer/Float | Exclusion is redundant for String columns but defensive and correct; no false positives in find() because the `matches!(col_type, Integer | Float)` predicate already excludes Strings |

---

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story file | ~4,000 | |
| BC-2.10.012 v1.11 (read §Auto-generated example queries + §Edge Cases only) | ~6,000 | Priority ladder + Tier 2 aggregate template + EC-10-033 exclusion clause are the relevant sections |
| `crates/prism-mcp/src/tools/prism_describe.rs` (full) | ~30,000 | Contains build_example_with_note, existing tests, ColumnDescriptor, and the OQ-003 block |
| beta3-remediation-delta-analysis.md §Issue 4 + §S-DESCRIBE-EXAMPLE-DEDUP-001 | ~2,000 | Reference for fix design and const name |
| **Total estimated** | **~42,000** | Well within one context window |

---

## Tasks

### Red Gate tests (to be written by test-writer BEFORE implementation — SAC-1)

All three tests are added to the EXISTING inline `#[cfg(test)] mod build_example_query_tests`
block in `crates/prism-mcp/src/tools/prism_describe.rs`. Use the same `col(name, type)`
helper defined at line ~807 in that module.

- [ ] **RG-DEDUP-001**: `test_BC_2_10_012_class_uid_excluded_from_agg_when_only_synthesized_integer`
  Set up `columns = vec![col("detected_time", ColumnType::Datetime), col("class_uid", ColumnType::Integer)]`.
  Invoke `build_example_with_note("claroty_alerts", &columns)`.
  Assert: `!q.contains("class_uid")`, `!q.contains("GROUP BY")`, `q.contains("detected_time")`,
  `q.contains("NOW()")`. Must be RED before implementation. Covers AC-001, AC-003 (Tier 3).

- [ ] **RG-DEDUP-002**: `test_BC_2_10_012_domain_integer_selected_over_class_uid_when_class_uid_is_first`
  Set up `columns = vec![col("class_uid", ColumnType::Integer), col("devices_count", ColumnType::Integer)]`.
  (`class_uid` is explicitly FIRST to require exclusion logic, not ordering.)
  Invoke `build_example_with_note("claroty_devices", &columns)`.
  Assert: `q.contains("devices_count")`, `q.contains("GROUP BY devices_count")`,
  `!q.contains("class_uid")`. Must be RED before implementation. Covers AC-002.

- [ ] **RG-DEDUP-003**: `test_BC_2_10_012_all_integers_excluded_and_no_datetime_falls_to_column_free_fallback`
  Set up `columns = vec![col("class_uid", ColumnType::Integer), col("asset_id", ColumnType::String)]`.
  Invoke `build_example_with_note("claroty_no_datetime_table", &columns)`.
  Assert: `q == "SELECT * FROM claroty_no_datetime_table LIMIT 25"`, `!q.contains("GROUP BY")`,
  `!q.contains("class_uid")`, `!q.contains("NOW()")`. Must be RED before implementation.
  Covers AC-003 (Tier 4 path).

- [ ] **Regression baseline**: Confirm all existing tests in `build_example_query_tests`
  remain GREEN with the new Red Gate tests added (no stub breakage). Specifically verify:
  - `test_crit1_no_datetime_column_produces_column_free_query` (claroty_devices String/Bool only) — still GREEN
  - `test_crit1_datetime_column_named_event_time_used_in_count_recent` — still GREEN
  - `test_crit1_zero_columns_uses_column_free_fallback` — still GREEN

### Implementation tasks (to be executed by implementer after Red Gate — SAC-1)

#### Phase A — Add EXAMPLE_EXCLUDED_COLS const

- [ ] **T-A01**: In `crates/prism-mcp/src/tools/prism_describe.rs`, add the exclusion const
  near the top of the file (in the module-level const block, or just above `build_example_with_note`):
  ```rust
  /// Synthesized metadata columns that MUST NOT be used as aggregate GROUP BY targets
  /// in auto-generated example queries (BC-2.10.012 §Auto-generated example queries Tier 2).
  /// These are infrastructure columns appended by build_ocsf_column_descriptors (OQ-003),
  /// not domain-data columns — grouping by them produces semantically meaningless examples.
  /// S-DESCRIBE-EXAMPLE-DEDUP-001 AC-001..003 / RG-DEDUP-001..003.
  const EXAMPLE_EXCLUDED_COLS: &[&str] = &[
      "class_uid",
      "_sensor",
      "_client",
      "_source_table",
      "_source_type",
  ];
  ```

#### Phase B — Update agg_col selection in build_example_with_note

- [ ] **T-B01**: In `build_example_with_note`, replace the existing `agg_col` find block
  (lines ~789–791 in current HEAD, matching the predicate
  `find(|c| matches!(c.col_type, ColumnType::Integer | ColumnType::Float))`) with:
  ```rust
  // Aggregate variant: pick the first non-excluded Integer/Float domain column.
  // BC-2.10.012 §Auto-generated example queries Tier 2 (S-DESCRIBE-EXAMPLE-DEDUP-001 AC-001..003).
  // EXAMPLE_EXCLUDED_COLS filters synthesized metadata columns so they are never used as
  // GROUP BY targets — they are infrastructure columns, not domain-data (delta-analysis §Issue 4).
  let agg_col = columns
      .iter()
      .find(|c| {
          matches!(c.col_type, ColumnType::Integer | ColumnType::Float)
              && !EXAMPLE_EXCLUDED_COLS.contains(&c.name.as_str())
      });
  ```

- [ ] **T-B02**: Run `cargo nextest run -p prism-mcp -E 'test(test_BC_2_10_012_class_uid_excluded_from_agg)'`
  — must be GREEN after T-B01.

- [ ] **T-B03**: Run `cargo nextest run -p prism-mcp -E 'test(test_BC_2_10_012_domain_integer_selected_over_class_uid)'`
  — must be GREEN after T-B01.

- [ ] **T-B04**: Run `cargo nextest run -p prism-mcp -E 'test(test_BC_2_10_012_all_integers_excluded_and_no_datetime)'`
  — must be GREEN after T-B01.

#### Phase C — Final verification

- [ ] **T-C01**: Run `cargo nextest run -p prism-mcp --no-fail-fast`. All tests must pass
  including RG-DEDUP-001..003 (GREEN) and all pre-existing tests (no regression). Pay special
  attention to the existing `build_example_query_tests` module — all prior tests must remain GREEN.

- [ ] **T-C02**: Run `just check` (full workspace). Must exit 0. Change is confined to one
  function in one crate; no cross-crate compilation breakage expected. Confirm `just clippy`
  passes (the new const and the updated closure must be clippy-clean under workspace settings).

#### Phase D — CHANGELOG

- [ ] **T-D01** (BEFORE creating the PR): Add a CHANGELOG entry under `[Unreleased] > Fixed`:
  ```markdown
  - Fix `prism_describe` example query generation: synthesized metadata columns
    (`class_uid`, `_sensor`, `_client`, `_source_table`, `_source_type`) are now
    excluded from the Tier 2 aggregate GROUP BY target selection in
    `build_example_with_note`. Previously, Claroty tables without domain-data numeric
    columns produced identical `GROUP BY class_uid` queries across all tables.
    Resolves beta.2 live-test issue 4.
  ```

---

## Previous Story Intelligence

**S-MCP-ENVELOPE-DESCRIBE-001** (W2, `status: ready v1.1`) is the sibling story in this epic
and wave. It adds three new String-typed synthesized `ColumnDescriptor` entries to
`build_ocsf_column_descriptors` (`_client`, `_source_table`, `_source_type`). These are
String-typed and are NOT affected by the Integer/Float `agg_col` find() that this story fixes.
Both stories are safe to implement in parallel worktrees on separate branches.

**Important cross-story invariant (AC-004 of S-MCP-ENVELOPE-DESCRIBE-001):** That story's
AC-004 asserts that adding `_client`, `_source_table`, `_source_type` (String) to the columns
array does NOT change `build_example_with_note` output. The exclusion const added by this
story extends that guarantee to the `class_uid` (Integer) exclusion — both stories collectively
ensure that all five synthesized columns are excluded from example query generation.

**The `claroty_devices` no-datetime test in existing tests** (`test_crit1_no_datetime_column_produces_column_free_query`)
uses `risk_score: ColumnType::String` — NOT Integer. This table has no Integer columns at all
in that test vector, so it currently falls through to the column-free fallback for a different
reason (no Integer/Float columns, not exclusion). After this fix, the same result is reached
via two possible paths for actual Claroty tables: (a) no Integer columns at all, or (b) only
excluded Integer columns. The existing test remains valid.

---

## Architecture Compliance Rules

1. **Do NOT change the Tier 1 (severity-IEQ) branch.** The severity-IEQ Tier 1 fires for ANY
   table with a `severity` String column, before Tier 2. BC-2.10.012 §Auto-generated example
   queries priority ladder: Tier 1 is highest priority and is not affected by this story's
   `agg_col` change. Do not modify the `has_severity` block.

2. **EXAMPLE_EXCLUDED_COLS includes String-typed columns defensively.** `_sensor`, `_client`,
   `_source_table`, `_source_type` are String-typed and would not be selected by the
   `matches!(col_type, Integer | Float)` predicate anyway. They are included in the exclusion
   const for defensive correctness and documentation value — if a future spec change introduces
   an Integer variant of one of these columns, the exclusion would still apply. This is not
   dead code; it is a contract anchor per BC-2.10.012 §Auto-generated example queries.

3. **Do NOT add the exclusion to the Datetime column selection (`datetime_col`).** The count-recent
   Tier 3 uses `find(|c| matches!(c.col_type, ColumnType::Datetime))` which cannot match any of
   the excluded Integer columns. `datetime_col` selection is correct and must not be modified.

4. **ColumnType::String for `_sensor`, `_client`, `_source_table`, `_source_type`.** These are
   the canonical virtual field types per BC-2.11.012. The exclusion const uses string names
   (not types) — name-based exclusion is the correct approach, as the type predicate already
   prevents Strings from reaching Tier 2.

5. **Use `prism_core::column::ColumnType` only.** The retired shadow enum
   `prism_spec_engine::types::ColumnType` MUST NOT be introduced. All ColumnType references in
   `prism_describe.rs` already use the canonical `prism_core::column::ColumnType` (ADR-024,
   CLAUDE.md §Forbidden patterns). No change needed; maintain this convention.

6. **No volatile line-number cites in code comments.** Per TD-VSDD-091: comments added by this
   story must reference function names and BC section anchors (e.g., `build_example_with_note`,
   `BC-2.10.012 §Auto-generated example queries Tier 2`), not `prism_describe.rs:NNN` line numbers.

7. **No `--no-verify` hook bypass.** Per CLAUDE.md non-negotiable git rules. If lefthook
   pre-commit/pre-push fails, fix the root cause.

---

## Library & Framework Requirements

All versions pinned in workspace `Cargo.toml` — use workspace pins.

| Dependency | Version | Note |
|-----------|---------|------|
| `prism_core::column::ColumnType` | workspace pin | Use `ColumnType::Integer` / `ColumnType::Float` — canonical variants only (CLAUDE.md §ColumnType canonical naming) |
| Rust toolchain | per `rust-toolchain.toml` | Stable channel; edition 2024 |

No new crate dependencies are introduced by this story.

### Forbidden Dependencies

- `prism_spec_engine::types::ColumnType` — RETIRED shadow enum per ADR-024 (CLAUDE.md §Forbidden
  patterns). The `prism_describe.rs` module already uses `prism_core::column::ColumnType`; do not
  introduce the shadow variant.

---

## File Structure Requirements

### Files to MODIFY

| File | Change | Phase |
|------|--------|-------|
| `crates/prism-mcp/src/tools/prism_describe.rs` | Add `EXAMPLE_EXCLUDED_COLS` const; update `agg_col` find() predicate in `build_example_with_note`; add RG-DEDUP-001..003 tests to `build_example_query_tests` module | Phase A + B + Red Gate |
| `CHANGELOG.md` | Add `[Unreleased] > Fixed` entry (T-D01) | Phase D |

### Files NOT to touch

- `crates/prism-mcp/src/safety_envelope.rs` — out of scope (S-MCP-ENVELOPE-DESCRIBE-001)
- `crates/prism-mcp/tests/mcp_prism_describe.rs` — Red Gate tests go in the INLINE module
  (`#[cfg(test)] mod build_example_query_tests` in `prism_describe.rs`), NOT the external
  integration test file. `build_example_with_note` is a pure function; inline unit tests
  are the correct and faster test mechanism.
- `crates/prism-mcp/src/server.rs` — out of scope
- `crates/prism-mcp/src/tools/operations.rs` — out of scope (S-MCP-TOOL-GATE-001)
- `crates/prism-query/` — out of scope
- `.factory/specs/behavioral-contracts/BC-2.10.012-*` — amendment is product-owner work;
  implementer does NOT modify BCs. The required amendment is documented in the frontmatter
  comment of this story.
- `.factory/specs/architecture/decisions/ADR-058-*` — architect work; implementer does NOT
  modify ADRs.

---

## Holdout Authoring Note

`behavioral_contracts: [BC-2.10.012]` is non-empty. Per the story-level holdout gate protocol
(D-1715/D-1716, human-approved 2026-07-13), the product-owner must author 2–4 HIDDEN,
SINGLE-USE holdout scenarios for this story at story-materialization time (the same touchpoint
as the remove-uncertainty pass). Suggested holdout scenario themes:

- A `prism_describe` call for a Claroty client whose tables have no domain-data Integer columns
  (only `class_uid`); assert that no returned `example_query` contains `GROUP BY class_uid`.
- A `prism_describe` call for a client with one table that has a real domain Integer column
  AND `class_uid`; assert the example uses the domain column.
- A combined scenario verifying that adding the exclusion did not suppress the severity-IEQ
  Tier 1 for a severity-vocabulary table that also has `class_uid`.

Holdout scenarios are stored in the holdout directory that test-writer/implementer never read.

---

## History

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-09-17 | Initial story decomposition — beta.3 S-DESCRIBE-EXAMPLE-DEDUP-001; traces to beta3-remediation-delta-analysis.md §Issue 4; BC-2.10.012 v1.10; 3 ACs, 3 RG tests (density 1.0); spec gap reported: BC-2.10.012 v1.11 amendment pending product-owner. |
| 1.1 | 2026-09-17 | Step 1 pin propagation: BC-2.10.012 v1.10→v1.11 amendment active (EC-10-033 landed); AMENDMENT PENDING frontmatter comment settled; EC-10-033 reference updated from "pending" to active in AC-001..003 traces, §Behavioral Contracts table, §Authority, and §Token Budget. |
| 1.1 | 2026-09-17 | D-1110 remove-uncertainty: corrected function reference `build_column_descriptors_ocsf` → `build_ocsf_column_descriptors` in §Problem Statement, §depends_on comment, §Previous Story Intelligence, and Tasks §T-A01 doc comment (5 occurrences). BC spec defect: BC-2.10.012 v1.11 §Auto-generated example queries Tier 2 EC-10-033 text also uses the wrong function name — routed to product-owner for BC amendment. |
