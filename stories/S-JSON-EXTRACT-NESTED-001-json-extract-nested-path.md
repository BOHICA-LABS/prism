---
document_type: story
story_id: S-JSON-EXTRACT-NESTED-001
title: "PrismQL JSON nested-path access — bounded JSONPath key traversal in json_extract_* (post-beta.3 fast-follow)"
level: "L4"
wave: TBD
epic_id: EPIC-OCSF-ROUTING
priority: P2
status: draft
# BC status: pending PO authorship — behavioral_contracts is empty. Per S-7.01, this story
# MUST remain draft until a product-owner authors and anchors BCs with canonical IDs matching
# BC-\d+\.\d{2}\.\d{3}. No BC covers bounded JSONPath nested access yet.
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
# traces_to: BC layer pending PO authorship. Security posture anchors (CLAUDE.md §agent-harness
# + plan-gate design from D-2520) are primary authority at materialization.
points: 8
# points: 8 estimated — requires a bounded JSONPath parser (new concern not in S-JSON-EXTRACT-UDF-001)
# plus the plan-gate extension. The algorithm is well-understood but adds a new parsing
# surface that must be hardened against path injection and resource exhaustion. Typed-accessor
# variants (S-JSON-EXTRACT-TYPED-001) should ship first to de-risk typed return plumbing.
estimated_days: 3
tdd_mode: strict
subsystems: [SS-11]
# Subsystem anchor justification:
#   SS-11 (Query Engine) owns this story's scope: the nested-path parser and plan-gate
#     live in `prism-query` (engine.rs, ast.rs, sql_parser.rs). SS-11 governs PrismQL
#     parser, AST, and query execution per ARCH-INDEX. The agent-facing query language
#     surface is owned by SS-11 for prompt-injection defense purposes.
# NOTE: D-2549 Finding-5 correction — SS-01 was a stale label; SS-11 is the
#   canonical Query Engine subsystem per ARCH-INDEX Subsystem Registry.
target_module: prism-query
crates_touched: [prism-query]
behavioral_contracts: []
# BC status: pending PO authorship (S-7.01 gate — behavioral_contracts: [] blocks status=ready)
# Traceability parents: S-JSON-EXTRACT-UDF-001, D-2520 design record, CLAUDE.md §agent-harness.
# SECURITY GATE: PO BC authorship MUST include a postcondition bounding path depth/length and
# rejecting non-literal paths to close the path-injection surface before status=ready.
verification_properties: []
holdout_scenarios: []
# holdout_scenarios: PO authors 2–4 hidden SINGLE-USE scenarios at remove-uncertainty time.
# Story-level holdout gate is BLOCKING before demo/push (human-approved 2026-07-13).
# Security reviewer MUST be included in the holdout design given the path-injection surface.
depends_on: [S-JSON-EXTRACT-UDF-001]
# depends_on justification:
#   S-JSON-EXTRACT-UDF-001: the minimal top-level-key accessor must be fully delivered before
#     nested path support is added. The nested-path variant extends the same plan-gate + key-cap
#     mechanism. Delivering nested access before the simpler single-key case would invert the
#     risk profile: the bounded JSONPath parser is a more complex security surface and should
#     be introduced incrementally.
blocks: []
acceptance_criteria_count: 0
# acceptance_criteria_count: 0 — draft stub; ACs to be authored when PO writes BCs
red_gate_tests: 0
# red_gate_tests: 0 — draft stub; RG tests to be authored when ACs exist
risk: HIGH
# Risk justification: nested-path access is an agent-facing query language feature on a
# security-sensitive surface (CLAUDE.md §agent-harness). The path string is provided by the
# LLM agent and traverses a user-controlled JSON blob. Two distinct attack surfaces:
#   1. Path injection: if the path parser is permissive, a crafted path could traverse
#      unexpected structure or trigger panics in the serde_json traversal.
#   2. Resource exhaustion: an unbounded path depth or length could cause excessive stack
#      depth or allocation in the plan-gate path parser before the query executes.
# Both must be addressed via plan-gate bounding (max path depth + max path byte length)
# and literal-path-only restriction (column references rejected), mirroring the perimeter-
# violation/plan-shape gate pattern and the beta.3 256-byte literal-key gate.
assumption_validations: []
risk_mitigations: []
---

# S-JSON-EXTRACT-NESTED-001: PrismQL JSON Nested-Path Access

> **DRAFT STUB — POST-BETA.3 FAST-FOLLOW.** Full ACs, Red Gate list, and BC layer
> are authored at story-materialization time (remove-uncertainty pass + PO BC authorship).
> Human-directed deferral: concrete dependency on minimal `json_extract_string` accessor
> (S-JSON-EXTRACT-UDF-001) landing in beta.3 W3 first.
> Status MUST remain `draft` until `behavioral_contracts:` is populated per S-7.01.

> **SECURITY NOTE:** This story introduces a path-injection and resource-exhaustion surface
> on the agent-facing PrismQL query language. The plan-gate MUST bound path depth, path byte
> length, and reject non-literal paths before this story reaches status=ready.
> A security-reviewer pass is REQUIRED in addition to the standard LOCAL adversarial cascade.

## Authority

**D-2520** (beta.3 live-test triage / W3 remediation cycle) is the source decision record.
The minimal `json_extract_string` ScalarUDF restricts access to top-level string keys only
(no path separators beyond the literal key). This story extends that to nested access via
a bounded JSONPath-style syntax (e.g., `$.metadata.severity`).

**CLAUDE.md §agent-harness** is the governing security authority. PrismQL is consumed by
LLM agents whose prompt text feeds directly into query strings — the path argument to
`json_extract_*` is agent-controlled input and must be treated as an untrusted surface.

## Objective

Extend `json_extract_string` (and, once S-JSON-EXTRACT-TYPED-001 ships, the typed variants)
to accept dot-separated nested key paths (e.g., `json_extract_string(raw_extensions, '$.a.b.c')`)
so that analysts and LLM agents can access nested JSON fields within `raw_extensions` blobs
without promoting every nested field to a first-class TOML column.

## Rationale

The beta.3 top-level-key accessor covers the majority of flat `raw_extensions` access patterns.
However, several sensor schemas (e.g., Claroty `device_details.location`, Armis
`network_interfaces[0].mac`) store semantically important fields at nested depth. Without
nested access, every such field requires a dedicated TOML column or a LIKE-based raw blob
scan, both of which have worse semantics than a typed nested path accessor.

## Source

Origin: D-2520 live-test triage / beta.3 remediation cycle (W3). Human-directed deferral
2026-09-15: concrete dependency on minimal accessor landing first. Nested access is a
separate security surface and must not be co-delivered with the minimal accessor.

## Dependency

`S-JSON-EXTRACT-UDF-001` must be merged (beta.3 W3) before this story is dispatched.
`S-JSON-EXTRACT-TYPED-001` should ship before this story so the typed return plumbing is
established before the path parser is introduced (de-risks the delivery sequence).

## Scope (to be refined by PO at materialization)

- **Path syntax:** bounded dot-path string literals only. Syntax: `$.a`, `$.a.b`, `$.a.b.c`.
  No array indexing (`[N]`) in this story. No wildcard operators. No filter expressions.
  Non-literal paths (column references, function calls) MUST be rejected by the plan-gate.
- **Plan-gate bounds** (to be anchored to BC postconditions at materialization):
  - Max path depth: TBD (3–5 levels recommended; adversary to validate).
  - Max path byte length: 256 bytes (mirrors the beta.3 literal-key gate).
  - Non-literal path argument: rejected with structured `E-QUERY` error code.
  - Invalid path syntax (missing `$.` prefix, empty segment): rejected with structured error.
- **Path parser:** a minimal recursive-descent parser over the literal path string, bounded
  at the plan-gate level before the ScalarUDF body executes. The parser MUST NOT use `eval`
  or `serde_json::Pointer::from_str` without bounding validation.
- **UDF signatures affected:** `json_extract_string` extended. Typed variants
  (`json_extract_int`, `json_extract_float`, `json_extract_bool`) from S-JSON-EXTRACT-TYPED-001
  also extended in the same story if that story has shipped.
- **Error taxonomy:** new `E-QUERY` codes for path-syntax rejection and depth-exceeded — to
  be anchored to BC postconditions. Must NOT invent codes outside the error taxonomy.

## Architecture Anchors

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| ScalarFunc enum (path arg) | `prism-query/src/ast.rs` | Pure |
| Path parser + plan-gate | `prism-query/src/engine.rs` | Pure |
| Parser recognition | `prism-query/src/sql_parser.rs` | Pure |
| Emitter arms | `prism-query/src/pipe_sql_emitter.rs` | Pure |
| Security gate tests | `prism-query/src/tests/` | Pure (unit) |

## Previous Story Intelligence

N/A — first story in this nested-access track. Predecessors S-JSON-EXTRACT-UDF-001 and
S-JSON-EXTRACT-TYPED-001 implementation notes and decisions will be reviewed at materialization.

## Architecture Compliance Rules

(To be extracted from architecture artifacts at materialization.)

- Forbidden dependencies: `prism-query` MUST NOT gain a dependency on `prism-sensors` or
  `prism-spec-engine`.
- Plan-gate is MANDATORY before UDF body execution. No path traversal without a clean gate.
- Literal-path-only restriction: the plan-gate must reject any non-`Expr::Literal` second
  argument, matching the beta.3 pattern.
- `#[non_exhaustive]` discipline: any new public enum variants in the AST must carry
  `#[non_exhaustive]` per workspace convention.

## Library & Framework Requirements

(Version pins to be confirmed from `architecture/dependency-graph.md` at materialization.)

- `serde_json`: workspace pin. The nested access uses `serde_json::Value::pointer` or
  equivalent; the plan-gate bounds depth BEFORE the JSON pointer traversal.
- DataFusion: workspace pin. No independent version pinning.
- No new crate dependencies should be required for the bounded path parser.

## File Structure Requirements

| File | Action |
|------|--------|
| `crates/prism-query/src/ast.rs` | Extend `ScalarFunc` to carry path-type discriminant (top-level vs. nested) or accept path string in existing fields — exact approach at materialization |
| `crates/prism-query/src/sql_parser.rs` | Update parser to accept dot-path strings for `json_extract_*` |
| `crates/prism-query/src/pipe_sql_emitter.rs` | Update emission arms to handle path argument |
| `crates/prism-query/src/engine.rs` | Add bounded path parser + plan-gate; update UDF body to use JSON pointer traversal |
| `crates/prism-query/src/tests/` | Red Gate tests (security + correctness; enumerated at materialization) |
| `CHANGELOG.md` | Add [Unreleased] > Added row for nested JSON path access |

## Token Budget Estimate

| Item | Tokens |
|------|--------|
| This story spec | ~3 000 |
| S-JSON-EXTRACT-UDF-001 + S-JSON-EXTRACT-TYPED-001 predecessors | ~5 000 |
| `prism-query/src/engine.rs` (relevant sections) | ~6 000 |
| `prism-query/src/ast.rs`, `sql_parser.rs`, `pipe_sql_emitter.rs` | ~8 000 |
| Error taxonomy supplement | ~1 000 |
| Behavioral contracts (pending authorship) | ~3 000 |
| Test files (security + correctness) | ~5 000 |
| **Total estimate** | **~31 000** |

Estimate is within the 20-30% window cap. If security review expands scope, the story may
need to split the path-parser-hardening from the correctness delivery.

## Acceptance Criteria

*N/A — draft stub. ACs to be authored when PO writes BCs (per S-7.01). Placeholder scope:*

- *AC-001 (placeholder): `json_extract_string(raw_extensions, '$.a.b')` traverses nested
  JSON and returns the string value at depth 2 — or NULL if absent.*
- *AC-002 (placeholder): Plan-gate rejects path depth exceeding the configured max with a
  structured `E-QUERY-NNN` error (code to be anchored to BC at materialization).*
- *AC-003 (placeholder): Plan-gate rejects paths longer than 256 bytes.*
- *AC-004 (placeholder): Plan-gate rejects non-literal path arguments (column reference,
  arithmetic expression) with a structured error.*
- *AC-005 (placeholder): Invalid path syntax (missing `$.` prefix, empty segment, `..`
  double-dot) rejected by the plan-gate — not silently ignored.*

## Red Gate Tests

*N/A — draft stub. RG list to be enumerated at materialization (SAC-1). MUST include
at minimum one injection-attempt test and one resource-bound test.*

## Edge Cases

*N/A — draft stub. To be elaborated at materialization. Critical edge cases include:*

- *Path is `$.` (root reference only) — undefined behavior; must reject.*
- *Path segment contains a dot (e.g., `$.a\.b` — escaped dot) — reject; not supported.*
- *JSON value at path is an object or array — return NULL (not error); matches nullable contract.*
- *Non-literal path (agent provides a column reference) — plan-gate reject; primary injection defense.*

## Tasks

(Enumerated at materialization; red-before-green ordering per SID-1.)

1. Remove-uncertainty pass: confirm serde_json JSON pointer behavior + DataFusion Expr
   literal pattern matching API.
2. Security review scoping: confirm plan-gate bounds with security-reviewer.
3. PO authors and anchors BCs (status gate; MUST include path-injection postcondition).
4. Author Red Gate failing tests (SAC-1; include injection + resource-bound tests).
5. Implement bounded path parser + plan-gate.
6. Implement nested traversal in UDF body.
7. Green all Red Gate tests.
8. LOCAL adversarial 3-CLEAN cascade (BC-5.39.001).
9. Security reviewer pass (required for this story due to HIGH risk).
10. Story-level holdout gate.
11. Add CHANGELOG entry under [Unreleased] > Added.
12. Push + PR.

## Changelog

| Version | Date | Change |
|---------|------|--------|
| v0.1 | 2026-09-15 | Initial draft stub registered (D-2521 post-beta.3 fast-follow batch). |
