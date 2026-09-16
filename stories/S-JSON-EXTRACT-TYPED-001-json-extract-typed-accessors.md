---
document_type: story
story_id: S-JSON-EXTRACT-TYPED-001
title: "PrismQL JSON typed extract accessors — json_extract_int / json_extract_float / json_extract_bool (post-beta.3 fast-follow)"
level: "L4"
wave: TBD
epic_id: EPIC-OCSF-ROUTING
priority: P2
status: draft
# BC status: pending PO authorship — behavioral_contracts is empty. Per S-7.01, this story
# MUST remain draft until a product-owner authors and anchors BCs with canonical IDs matching
# BC-\d+\.\d{2}\.\d{3}. No BC covers json_extract_int/float/bool typed return contracts yet.
producer: story-writer
timestamp: "2026-09-15T00:00:00Z"
version: "0.1"
modified: "2026-09-15"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - ".factory/stories/S-JSON-EXTRACT-UDF-001-json-extract-scalar-udf.md"
input-hash: "pending-recompute"
# input-hash: pending — to be computed from S-JSON-EXTRACT-UDF-001 after beta.3 lands
traces_to: []
# traces_to: BC layer pending PO authorship. Parent contracts once authored will
# reference the typed-return postconditions and literal-key invariants.
points: 5
# points: 5 estimated — three ScalarUDFs (int/float/bool variants) following the
# established json_extract_string pattern. Less than string variant (already proven
# pattern) but more than a trivial copy due to numeric/boolean ScalarValue mapping
# and plan-gate extension.
estimated_days: 2
tdd_mode: strict
subsystems: [SS-01]
# Subsystem anchor justification:
#   SS-01 (Query Engine) owns this story's scope: the UDFs are registered in
#     `prism-query engine build_session_context` (same site as json_extract_string)
#     and the plan-gate extension lives in `prism-query::engine`. SS-01 governs
#     PrismQL parser, AST, and query execution per ARCH-INDEX.
target_module: prism-query
crates_touched: [prism-query]
behavioral_contracts: []
# BC status: pending PO authorship (S-7.01 gate — behavioral_contracts: [] blocks status=ready)
# Traceability parent: S-JSON-EXTRACT-UDF-001 and D-2520 design record.
# PO must author typed-return + literal-key invariant BCs before dispatch.
verification_properties: []
holdout_scenarios: []
# holdout_scenarios: PO authors 2–4 hidden SINGLE-USE scenarios at remove-uncertainty time.
# Story-level holdout gate is BLOCKING before demo/push (human-approved 2026-07-13).
depends_on: [S-JSON-EXTRACT-UDF-001]
# depends_on justification:
#   S-JSON-EXTRACT-UDF-001: the minimal json_extract_string ScalarUDF must be fully
#     delivered (merged in beta.3 W3) before typed variants are added. The typed variants
#     follow the exact registration path, plan-gate structure, and 256-byte key cap
#     established by the string accessor. Parallel delivery would require coordinating
#     two PRs touching the same engine::build_session_context registration site.
blocks: []
acceptance_criteria_count: 0
# acceptance_criteria_count: 0 — draft stub; ACs to be authored when PO writes BCs
red_gate_tests: 0
# red_gate_tests: 0 — draft stub; RG tests to be authored when ACs exist
risk: MEDIUM
# Risk justification: wrong ScalarValue type mapping (e.g., Float64 instead of Float32)
# causes silent query plan mismatch. Plan-gate must enforce literal-key invariant for all
# three new UDFs identically to the string variant to preserve the agent-harness
# prompt-injection posture (CLAUDE.md §agent-harness).
assumption_validations: []
risk_mitigations: []
---

# S-JSON-EXTRACT-TYPED-001: PrismQL JSON Typed Extract Accessors

> **DRAFT STUB — POST-BETA.3 FAST-FOLLOW.** Full ACs, Red Gate list, and BC layer
> are authored at story-materialization time (remove-uncertainty pass + PO BC authorship).
> Human-directed deferral: concrete dependency = the minimal `json_extract_string`
> accessor (S-JSON-EXTRACT-UDF-001) must ship in beta.3 W3 before this story is dispatched.
> Status MUST remain `draft` until `behavioral_contracts:` is populated per S-7.01.

## Authority

**D-2520** (beta.3 live-test triage / W3 remediation cycle) is the source decision
record. The minimal `json_extract_string` ScalarUDF delivers top-level string key access
with a nullable VARCHAR return, a literal-key plan-gate, and a 256-byte key cap. This
story extends that pattern to numeric and boolean fields.

**S-JSON-EXTRACT-UDF-001** is the blocking predecessor. Its `ScalarFunc::JsonExtractString`
in `prism-query::ast`, `sql_parser.rs` recognition, `pipe_sql_emitter.rs` emission, and
`engine.rs` ScalarUDF registration define the template this story follows.

## Objective

Add `json_extract_int(json_col, 'key') -> BIGINT`,
`json_extract_float(json_col, 'key') -> FLOAT64`, and
`json_extract_bool(json_col, 'key') -> BOOLEAN` ScalarUDFs to the PrismQL query engine so
that numeric and boolean fields within `raw_extensions` blobs (e.g., `cvss_v3_score`,
`devices_count`, `is_online`) can be compared with **correct semantic ordering** rather than
lexicographic string ordering.

## Rationale

The beta.3 minimal accessor (`json_extract_string`) returns only VARCHAR. Ordered comparisons
on numeric fields via string accessors are **incorrect**: `"9" > "10"` under string ordering,
so `WHERE json_extract_string(raw_extensions, 'cvss_v3_score') > '8.5'` misfires on
double-digit scores. Typed variants close this semantic gap without promoting every numeric
field to a first-class TOML column. Boolean semantics are similarly unrepresentable via
string comparison (`"true" != true` in DataFusion filter contexts).

## Source

Origin: D-2520 live-test triage / beta.3 remediation cycle (W3). Human-directed deferral
2026-09-15: concrete dependency on minimal json_extract_string accessor landing in beta.3.

## Dependency

`S-JSON-EXTRACT-UDF-001` must be merged (beta.3 W3) before this story is dispatched.

## Scope (to be refined by PO at materialization)

- New AST variants: `ScalarFunc::JsonExtractInt`, `ScalarFunc::JsonExtractFloat`,
  `ScalarFunc::JsonExtractBool` in `prism-query::ast` — mirrors existing
  `ScalarFunc::JsonExtractString` variant structure.
- Grammar recognition in `sql_parser.rs` for the three new function names.
- `pipe_sql_emitter.rs` emission arms for all three variants.
- ScalarUDF registration in `engine.rs::build_session_context` for all three, using the
  synchronous `serde_json` extraction path established by the string variant.
- Plan-gate extension in `engine.rs`: literal-key check + 256-byte key cap applied to
  all three new UDFs identically to the string variant.
- Return types: `DataType::Int64` for `json_extract_int`, `DataType::Float64` for
  `json_extract_float`, `DataType::Boolean` for `json_extract_bool`. All nullable.
- Error taxonomy: new error codes in `E-QUERY` namespace if extraction encounters a
  type mismatch (value present but wrong JSON type) — to be anchored to a BC postcondition
  at materialization. Must NOT invent codes outside the taxonomy (CLAUDE.md §Conventions).
- No push-down. All three variants evaluate post-fetch, identical to the string variant.

## Architecture Anchors

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| ScalarFunc enum | `prism-query/src/ast.rs` | Pure (data type) |
| Parser recognition | `prism-query/src/sql_parser.rs` | Pure |
| Plan-gate enforcement | `prism-query/src/engine.rs` | Pure |
| UDF registration | `prism-query/src/engine.rs::build_session_context` | Effectful (session build) |
| Emitter arms | `prism-query/src/pipe_sql_emitter.rs` | Pure |

## Previous Story Intelligence

N/A — first story in this fast-follow track. Predecessor is S-JSON-EXTRACT-UDF-001; its
implementation notes and decisions will be reviewed at materialization time.

## Architecture Compliance Rules

(To be extracted from `architecture/module-decomposition.md` and relevant ADRs at
materialization time.)

- Forbidden dependencies: `prism-query` MUST NOT gain a dependency on `prism-sensors` or
  `prism-spec-engine` (existing perimeter rule, CLAUDE.md §Conventions).
- All new ScalarUDFs MUST follow the literal-key-only plan-gate pattern from D-2520 §design.
- `#[non_exhaustive]` discipline: any new public enum variants added to the AST must follow
  the `#[non_exhaustive]` policy (CLAUDE.md §Conventions).

## Library & Framework Requirements

(Version pins to be confirmed from `architecture/dependency-graph.md` at materialization.)

- DataFusion version: inherit from workspace; do NOT pin independently.
- `serde_json`: workspace pin (same as S-JSON-EXTRACT-UDF-001).
- No new crate dependencies should be required.

## File Structure Requirements

(To be fully elaborated at materialization; the following is a pre-materialization sketch.)

| File | Action |
|------|--------|
| `crates/prism-query/src/ast.rs` | Add `JsonExtractInt`, `JsonExtractFloat`, `JsonExtractBool` to `ScalarFunc` enum |
| `crates/prism-query/src/sql_parser.rs` | Add recognition for the three new function names |
| `crates/prism-query/src/pipe_sql_emitter.rs` | Add emission arms for all three variants |
| `crates/prism-query/src/engine.rs` | Register three new ScalarUDFs; extend plan-gate |
| `crates/prism-query/src/tests/` | Red Gate tests (enumerated at materialization) |
| `CHANGELOG.md` | Add [Unreleased] > Added row for typed extract accessors |

## Token Budget Estimate

(Rough pre-materialization; to be confirmed at materialization.)

| Item | Tokens |
|------|--------|
| This story spec | ~2 000 |
| S-JSON-EXTRACT-UDF-001 (predecessor) | ~3 000 |
| `prism-query/src/ast.rs` | ~4 000 |
| `prism-query/src/engine.rs` (relevant sections) | ~6 000 |
| `prism-query/src/sql_parser.rs` | ~4 000 |
| `prism-query/src/pipe_sql_emitter.rs` | ~3 000 |
| Error taxonomy supplement | ~1 000 |
| Behavioral contracts (pending authorship) | ~2 000 |
| Test files | ~3 000 |
| **Total estimate** | **~28 000** |

Estimated context is well within the 20-30% window cap for a focused in-scope delivery.

## Acceptance Criteria

*N/A — draft stub. ACs to be authored when PO writes BCs (per S-7.01). Placeholder scope:*

- *AC-001 (placeholder): `json_extract_int(raw_extensions, 'cvss_v3_score')` returns a
  BIGINT (nullable) — plan-gate rejects non-literal key; integer type enables correct
  numeric comparison.*
- *AC-002 (placeholder): `json_extract_float(raw_extensions, 'score')` returns a FLOAT64
  (nullable) — correct fractional comparison semantics.*
- *AC-003 (placeholder): `json_extract_bool(raw_extensions, 'is_online')` returns a BOOLEAN
  (nullable) — correct boolean predicate without string coercion.*
- *AC-004 (placeholder): All three UDFs inherit the 256-byte literal key cap and
  non-literal-key plan-gate rejection from S-JSON-EXTRACT-UDF-001.*
- *AC-005 (placeholder): Type-mismatch case (value present in JSON but wrong type) returns
  NULL without a panic, consistent with nullable-return contract.*

## Red Gate Tests

*N/A — draft stub. RG list to be enumerated at materialization (SAC-1).*

## Edge Cases

*N/A — draft stub. To be elaborated at materialization. Relevant edge cases:*

- *JSON key exists but value is `null` → return NULL (not error).*
- *JSON key exists but value is wrong type (e.g., string when int expected) → return NULL.*
- *JSON key is a non-literal (column reference or expression) → plan-gate rejects with
  structured error code.*
- *Key length exceeds 256 bytes → plan-gate rejects.*

## Tasks

(Enumerated at materialization; red-before-green ordering per SID-1.)

1. Remove-uncertainty pass: confirm DataFusion ScalarUDF typed return API for Int64/Float64/Boolean.
2. PO authors and anchors BCs (status gate).
3. Author Red Gate failing tests (SAC-1 enumerated RG list).
4. Implement ScalarFunc AST variants, parser recognition, emitter arms, UDF registration, plan-gate.
5. Green all Red Gate tests.
6. LOCAL adversary 3-CLEAN cascade (BC-5.39.001).
7. Story-level holdout gate.
8. Add CHANGELOG entry under [Unreleased] > Added.
9. Push + PR.

## Changelog

| Version | Date | Change |
|---------|------|--------|
| v0.1 | 2026-09-15 | Initial draft stub registered (D-2521 post-beta.3 fast-follow batch). |
