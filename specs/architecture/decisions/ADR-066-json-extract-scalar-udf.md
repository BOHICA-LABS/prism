---
document_type: adr
adr_id: "ADR-066"
title: "json_extract_string Scalar UDF — Synchronous serde_json Single-Key Extraction with Literal-Key Plan Gate"
status: ACCEPTED
date: "2026-09-16"
modified: "2026-09-16"
version: "1.3"
producer: architect
subsystems_affected: [SS-11]
supersedes: []
superseded_by: null
amends: null
anchor_stories:
  - S-JSON-EXTRACT-UDF-001   # beta.3 minimal json_extract_string story; §Authority of this ADR
  - S-JSON-EXTRACT-TYPED-001  # post-beta.3 fast-follow: typed variants (int/float/bool); depends_on S-JSON-EXTRACT-UDF-001
  - S-JSON-EXTRACT-NESTED-001 # post-beta.3 fast-follow: bounded JSONPath nested paths; depends_on S-JSON-EXTRACT-UDF-001
related_adrs: [ADR-028, ADR-060]
related_bcs: [BC-2.11.001, BC-2.11.025]
locked_decisions: []
wiring_deferred_to: null
inputs:
  - crates/prism-query/src/ast.rs
  - crates/prism-query/src/sql_parser.rs
  - crates/prism-query/src/pipe_sql_emitter.rs
  - crates/prism-query/src/engine.rs
  - .factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md
  - .factory/STATE.md D-2522
input-hash: "pending"
---

# ADR-066: json_extract_string Scalar UDF — Synchronous serde_json Single-Key Extraction with Literal-Key Plan Gate

## Status

ACCEPTED v1.0 (2026-09-16) — D-2522 beta.3 spec-gate approved. Closes the latent
dead-path defect: `ScalarFunc::JsonExtractString` existed in the AST, SQL parser, and
pipe SQL emitter since an earlier wave, but no `ScalarUDF` named `json_extract_string`
was registered with the DataFusion `SessionContext`. Any invocation caused a
DataFusion-internal runtime error unstructured under the prism error taxonomy.

---

## §A Context

### §A1 Dead-Path Defect

Three code sites reference `json_extract_string` but the UDF was never registered:

1. `crates/prism-query/src/ast.rs` — `ScalarFunc::JsonExtractString` variant exists in the
   AST's scalar function enum.
2. `crates/prism-query/src/sql_parser.rs` — `json_extract_string(col, 'key')` SQL syntax
   is parsed into `ScalarFunc::JsonExtractString`.
3. `crates/prism-query/src/pipe_sql_emitter.rs` — `ScalarFunc::JsonExtractString` is
   lowered to DataFusion SQL `json_extract_string(col, key)`.

Without a registered `ScalarUDF`, DataFusion receives an unregistered function call and
returns a plan error. In SQL mode this produces a DataFusion-internal error message not
structured under `E-QUERY-NNN`. In pipe mode (PQL → SQL via pipe_sql_emitter) the same
unregistered-function error surfaces. Both failure modes bypass the prism error taxonomy
and are opaque to LLM agents consuming the MCP output.

Additionally: no literal-key plan gate exists. A non-literal key expression
(`json_extract_string(col, other_column)`) is accepted at parse time and passed to
DataFusion, where it would be evaluated with a runtime-resolved key. This bypasses:
- The 256-byte key length cap (CWE-400)
- The literal-key-only injection prevention contract (§B3)

### §A2 Why a UDF Is the Correct Fix

DataFusion's `FunctionRegistry` requires that every SQL function identifier that appears in
a query plan is registered before execution. The three existing AST/parser/emitter sites
are correct: they correctly identify and emit the function call. The missing piece is the
`ScalarUDF` registration in `engine.rs` at `SessionContext` construction time.

Fixing the dead-path defect by REMOVING the parser and emitter arms would break the
PrismQL grammar promise (the function is documented as available). The correct fix is to
implement and register the UDF.

### §A3 Scope Constraints for Beta.3

Beta.3 scope is deliberately minimal:
- **Literal key only** — dynamic keys rejected at plan time (§B3, §D2)
- **Top-level key only** — no nested JSONPath in beta.3 (§D4)
- **String return type only** — no typed extraction (int/float/bool) in beta.3 (§D5)
- **No sensor push-down** — UDF executes post-fetch in the DataFusion layer only (§D6)
- **Synchronous execution** — no async, pure function (§B1)

---

## §B Decision

### §B1 — UDF Implementation: Synchronous serde_json Single-Key Extraction

**Decision:** `json_extract_string` is implemented as a synchronous `ScalarUDF` using
`serde_json` for single-key extraction per row. The pure extraction function signature is:

```rust
fn json_extract_string_impl(column_value: Option<&str>, key: &str) -> Option<String>
```

The function:
1. Returns `None` if `column_value` is `None` (null input column → SQL NULL)
2. Parses the string as JSON via `serde_json::from_str::<serde_json::Value>`. On parse
   failure returns `None` (non-JSON input → SQL NULL)
3. If the parsed value is not a `serde_json::Value::Object`, returns `None` (non-object
   JSON → SQL NULL)
4. Calls `.get(key)` on the object. If the key is absent, returns `None` (missing key → SQL NULL)
5. If the key maps to `serde_json::Value::Null`, returns `None` (JSON null value → SQL NULL)
6. If the key maps to a `serde_json::Value::String(s)`, returns `Some(s.clone())`
7. For non-string values (Number, Bool, Array, Object), calls `.to_string()` (JSON
   re-serialization) and returns `Some(...)` to preserve the raw value for downstream use.
   This is intentional: callers can always cast or discard; silent `None` for non-string
   values would produce invisible data loss.

**Return type:** `DataType::Utf8` nullable. DataFusion represents SQL NULL via Arrow null
cells in the output column; `None` from the implementation maps to a null cell.

**Registration site:** `crates/prism-query/src/engine.rs` at `SessionContext` construction,
before any user query is planned. The UDF is registered as
`ctx.register_udf(json_extract_string_udf())` where `json_extract_string_udf()` returns
the assembled `ScalarUDF`.

**Module:** The extraction logic lives in a new file
`crates/prism-query/src/json_extract_udf.rs` with a public `json_extract_string_udf()`
factory function and a `pub(crate) fn json_extract_string_impl(...)` pure function.
The pure function is the VP-162 proof target (§D1).

### §B2 — Why serde_json (Not arrow-rs JSON Reader)

Two approaches were evaluated:

**Option A — serde_json (CHOSEN):** Parse the input string per row with
`serde_json::from_str`, extract the key.

- Single-key extraction per row is a scalar operation. The input is an Arrow `Utf8` column
  where each cell is a JSON-encoded string produced by the `raw_extensions` aggregation
  path. The extraction operates one string per row.
- `serde_json` is already in the dependency tree via `prism-core` and multiple other crates
  in the workspace. No new dependency is added.
- The API surface for single-key extraction is minimal: `Value::as_object().get(key)`.
  The code is straightforward to audit for correctness and for VP-162 Kani proof.
- Failure modes are explicit (`Result`/`Option`); no partial-parse ambiguity.

**Option B — arrow-rs JSON reader (REJECTED):** Deserialize the entire column batch with
the `arrow-json` reader, create a new column batch, project the key.

- `arrow-json` is designed for bulk columnar deserialization of newline-delimited JSON
  (NDJSON). Its API is batch-oriented: it reads entire streams or arrays, not individual
  cells. Adapting it for per-row scalar extraction introduces unnecessary complexity.
- The `explicit_nulls` default behavior of `arrow-json` (which already caused the
  DEFECT-MCP-ROWSHAPE-NULLS-001 null-not-absent defect in beta.2) is a source of subtle
  correctness risk when used in an unfamiliar context.
- No performance benefit for LIMIT-bounded result sets (typical sizes: 1–100 rows after
  early-stop pagination per ADR-060 §D8).

### §B3 — Literal-Key Plan Gate (E-QUERY-045)

**Decision:** Any `json_extract_string` call where the second argument is NOT a
`Expr::Literal(Literal::String(_))` at plan time is rejected with error `E-QUERY-045`
before DataFusion execution.

**Rationale:** Without this gate:
- A dynamic key (`json_extract_string(col, other_col)`) causes the UDF to be invoked with
  a runtime-resolved key value. The 256-byte cap (§D3) cannot be enforced at runtime
  without adding overhead to every invocation and creating a path-dependent failure mode.
- A subquery or expression as the key could constitute an injection surface: an LLM agent
  generating `json_extract_string(raw, (SELECT secret_key FROM config))` would execute a
  correlated lookup masked as a scalar extraction.
- The literal-key requirement is the primary injection prevention mechanism for this UDF.

**Gate location:** In the engine's plan validation step (before DataFusion execution), walk
the SQL AST and reject any `json_extract_string(col, expr)` where `expr` is not a string
literal. The gate runs for both SQL mode and pipe mode (which lowered to SQL via
`pipe_sql_emitter`). Error code `E-QUERY-045` (reserved in `prd-supplements/error-taxonomy.md`
for this gate).

**Gate implementation note:** The gate inspects the AST's `ScalarFunc::JsonExtractString`
nodes before the SQL is handed to DataFusion for planning. This is distinct from the
DataFusion `analyze` hook; it is a pre-planning validation in `engine.rs` after PQL
parsing but before `ctx.sql(...)` is called.

**SAP-3 reachability requirement:** The gate MUST be reachable from the public PrismQL
surface (a real PQL query string), not only from a synthetic AST injection. RG-JEX-006
tests this from the `prism_query` public API.

---

## §C Formal Correctness Contract

The `json_extract_string_impl` function (§B1) is the Kani verification target (VP-162, §D1). The
following invariants are the exhaustive formal contract for the pure function:

1. **Null safety:** For any `column_value: Option<&str>` and any `key: &str` with
   `key.len() <= 256`, the function returns `Some(String)` or `None` — it NEVER panics
   and NEVER propagates an unstructured Rust error.
2. **None-in / None-out:** If `column_value = None`, the return is always `None`,
   regardless of `key`.
3. **Bounded key precondition:** The 256-byte key-length precondition is enforced at plan
   time (§B3, §D3). The pure function is only called with keys that have already passed the
   gate; the Kani harness (`vp162_b_none_input_is_none_output`) models the post-gate scenario.

Behavioral properties (step-by-step null-path enumeration, non-string coercion) are covered by
the RG-JEX Red Gate tests (§G) rather than by Kani proof.

---

## §D Scope Boundaries

### §D1 — VP-162 Proof Target

`json_extract_string_impl(column_value: Option<&str>, key: &str) -> Option<String>` is
the VP-162 Kani proof target. It is pure (no I/O, no global state, no async). The
property to prove: for any `column_value` and any `key` with `key.len() <= 256`, the
function either returns `Some(String)` or `None`, never panics, and never propagates an
unstructured internal error.

The 256-byte key length precondition is enforced by the literal-key plan gate (§B3) at
plan time; the pure function itself is called only with keys that have already passed the
gate. The Kani harness models the post-gate scenario: `kani::assume(key.len() <= 256)`.

### §D2 — Literal-Key-Only Scope

Only `json_extract_string(column_expr, 'literal_string')` is valid in beta.3. The key
argument MUST be a string literal in the query text. Non-literal key arguments (column
references, expressions, subqueries) are rejected at plan time with E-QUERY-045.

This constraint is PERMANENT for `json_extract_string`; it is NOT relaxed by
S-JSON-EXTRACT-NESTED-001 (which introduces a separate bounded JSONPath mechanism rather
than relaxing the literal requirement of the base function).

### §D3 — Key Length Cap (CWE-400)

The literal key argument MUST be at most 256 bytes (UTF-8 encoded). A key exceeding 256
bytes is rejected at plan time with E-QUERY-045.

**Rationale (CWE-400):** An unbounded key size enables a Denial-of-Service vector through
quadratic or linear-time JSON object lookups against adversarially crafted input. 256 bytes
is sufficient for all realistic OCSF and Claroty JSON field names (the longest observed is
52 bytes). The cap is enforced at plan time (literal key inspection) rather than at runtime,
so there is zero per-row overhead.

### §D4 — Top-Level Key Only (No Nested JSONPath)

`json_extract_string` in beta.3 extracts only TOP-LEVEL keys. The key string is treated as
a single JSON object key, not a JSONPath expression. Nested paths (e.g., `a.b.c` or `$.a[0]`)
are NOT supported; they are treated as literal key strings and will return SQL NULL for any
JSON object that does not have an exact top-level key matching that string.

**Deferred to:** `S-JSON-EXTRACT-NESTED-001` (post-beta.3 fast-follow, P2, 8pts). That
story introduces a bounded JSONPath parser with an explicit injection-prevention design.
The dependency chain is `S-JSON-EXTRACT-UDF-001` → `S-JSON-EXTRACT-NESTED-001`.

### §D5 — String Return Type Only (No Typed Variants)

`json_extract_string` returns `Utf8` (nullable string) only. Typed extraction for
numeric, boolean, and other JSON types is deferred.

**Deferred to:** `S-JSON-EXTRACT-TYPED-001` (post-beta.3 fast-follow, P2, 5pts). That
story adds `json_extract_int`, `json_extract_float`, `json_extract_bool` variants, each
with typed DataFusion return types and their own plan gates. The dependency chain is
`S-JSON-EXTRACT-UDF-001` → `S-JSON-EXTRACT-TYPED-001`.

### §D6 — No Sensor Push-Down

`json_extract_string` is a DataFusion-layer UDF. It executes post-fetch on the in-memory
Arrow column produced by the spec-driven adapter. No sensor API receives a JSON-path filter
derived from this UDF. Push-down of JSON extraction predicates to sensor APIs is outside
scope for beta.3 and all defined fast-follow stories.

---

## §E DataFusion Registration Contract

The `ScalarUDF` must satisfy the following DataFusion registration requirements:

| Property | Value |
|----------|-------|
| Function name | `json_extract_string` (exact — matches parser/emitter) |
| Input signature | `(Utf8, Utf8)` — two nullable string arguments |
| Return type | `Utf8` nullable |
| Volatility | `Immutable` — same inputs always produce same output; DataFusion may CSE or cache |
| Implementation type | `ScalarUDFImpl` (DataFusion 53.x API) |

**Constructor pattern:**

```rust
// crates/prism-query/src/json_extract_udf.rs

use datafusion::logical_expr::{ScalarUDF, ScalarUDFImpl, Volatility};
use datafusion::arrow::datatypes::DataType;

pub fn json_extract_string_udf() -> ScalarUDF {
    ScalarUDF::new_from_impl(JsonExtractStringUdf::new())
}

#[derive(Debug)]
struct JsonExtractStringUdf { signature: Signature }

impl JsonExtractStringUdf {
    fn new() -> Self {
        let signature = Signature::exact(
            vec![DataType::Utf8, DataType::Utf8],
            Volatility::Immutable,
        );
        Self { signature }
    }
}

impl ScalarUDFImpl for JsonExtractStringUdf {
    fn name(&self) -> &str { "json_extract_string" }
    fn signature(&self) -> &Signature { &self.signature }
    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Ok(DataType::Utf8)  // nullable handled via Arrow null cell mechanism
    }
    fn invoke_batch(&self, args: &[ColumnarValue], batch_size: usize) -> Result<ColumnarValue> {
        // Delegate to per-row json_extract_string_impl
        // ... (implementer fills in Arrow column iteration)
    }
}
```

The implementer resolves the exact `ScalarUDFImpl` trait method signatures against the
DataFusion version pinned in `Cargo.toml` before authoring the final implementation.

---

## §F Error Taxonomy

`E-QUERY-045` is reserved in `prd-supplements/error-taxonomy.md` for the
`json_extract_string` plan-gate rejection. It covers two sub-cases:

| Sub-case | Trigger | Message |
|----------|---------|---------|
| E-QUERY-045(a) | Non-literal key argument | `"E-QUERY-045: json_extract_string requires a literal string key (e.g., json_extract_string(col, 'key_name')). Dynamic key expressions are not supported."` |
| E-QUERY-045(b) | Key exceeds `{max_len}` bytes | `"E-QUERY-045: json_extract_string key is {key_len} bytes, which exceeds the {max_len}-byte maximum (CWE-400)."` |

Both sub-cases are rejected at plan time, before DataFusion planning. They are surfaced
through the standard prism `E-QUERY-NNN` error response path (structured, not
DataFusion-internal).

---

## §G Mandate Anchors (TD-VSDD-097 Dim-3)

Every MUST in BC-2.11.025 and this ADR is anchored to a specific Red Gate test in
S-JSON-EXTRACT-UDF-001:

| MUST | Story AC / Red Gate |
|------|---------------------|
| UDF registered at engine construction | S-JSON-EXTRACT-UDF-001 RG-JEX-001 (happy path executes) |
| JSON null value → SQL NULL (§B1 step 5) | S-JSON-EXTRACT-UDF-001 RG-JEX-002 |
| Missing key → SQL NULL (§B1 step 4) | S-JSON-EXTRACT-UDF-001 RG-JEX-003 |
| Null column → SQL NULL (§B1 step 1) | S-JSON-EXTRACT-UDF-001 RG-JEX-004 |
| Non-object JSON → SQL NULL (§B1 step 3) | S-JSON-EXTRACT-UDF-001 RG-JEX-005 |
| Non-literal key rejected with E-QUERY-045(a) — reachable from `prism_query` public API (SAP-3) | S-JSON-EXTRACT-UDF-001 RG-JEX-006 |
| Key > 256 bytes rejected with E-QUERY-045(b) | S-JSON-EXTRACT-UDF-001 RG-JEX-007 |
| Non-string value (Number, Bool, Array, Object) → JSON re-serialized string, NOT NULL (§B1 step 7) | S-JSON-EXTRACT-UDF-001 RG-JEX-008 |
| Parse failure — non-JSON column value → SQL NULL (§B1 step 2) | S-JSON-EXTRACT-UDF-001 RG-JEX-009 |
| Dot-in-key treated as literal top-level key (§D4): `'a.b'` matches exact object key `"a.b"`, NOT nested path | S-JSON-EXTRACT-UDF-001 RG-JEX-010 |
| Pipe mode end-to-end: `json_extract_string` in PQL pipe expression executes correctly (SAP-3 public-surface reachability) | S-JSON-EXTRACT-UDF-001 RG-JEX-011 |
| VP-162 Kani proof covers pure function | VP-162 (Phase 5 formal-verify) |

---

## §H Latent Dead-Path Defect Closure

The `ScalarFunc::JsonExtractString` variant in `ast.rs`, the parse arm in `sql_parser.rs`,
and the emit arm in `pipe_sql_emitter.rs` have existed since a prior wave and are correct.
They form the complete AST-to-SQL pipeline for the function call. The only missing piece
is the UDF registration.

This ADR authorizes closing the defect by implementing and registering the UDF (§B1) and
adding the plan gate (§B3), NOT by removing the existing AST/parser/emitter arms.
Removing those arms would be a regression: the PrismQL grammar advertises
`json_extract_string` as available, and the AST/parser/emitter represent correct,
committed behavior.

---

## Rationale

1. **Synchronous serde_json over arrow-rs:** See §B2. serde_json is simpler, already in
   the dependency tree, and correct for per-row scalar extraction. The arrow-rs JSON reader
   is designed for bulk columnar deserialization, not scalar extraction. Performance is
   acceptable for LIMIT-bounded result sets.

2. **Literal-key-only over runtime key resolution:** See §B3. Injection prevention and
   the 256-byte cap cannot be reliably enforced at runtime without per-invocation overhead
   and path-dependent failure modes. A plan-time gate with E-QUERY-045 is the correct
   mechanism.

3. **Top-level key only in beta.3:** JSONPath introduces a bounded parser, a grammar, and
   an injection surface analysis that are out of scope for a focused remediation cycle.
   `S-JSON-EXTRACT-NESTED-001` provides the correct venue for that design with dedicated
   adversarial review.

4. **String return type only in beta.3:** Typed variants require type dispatch, return type
   inference changes to the DataFusion UDF registration, and separate null-safety proofs.
   `S-JSON-EXTRACT-TYPED-001` handles this correctly as a follow-on.

---

## Consequences

### Positive

- The latent dead-path defect is closed: `json_extract_string(col, 'key')` in PrismQL
  now executes correctly in both SQL mode and pipe mode.
- VP-162 Kani proof provides formal null-safety assurance for the pure extraction function.
- E-QUERY-045 provides a structured, taxonomy-compliant error for invalid invocations,
  replacing the opaque DataFusion-internal error.
- The literal-key gate prevents injection via dynamic key expressions.
- No changes to the grammar, parser, AST, or pipe SQL emitter are needed.

### Negative / Trade-offs

- Nested JSONPath access requires a separate query and a `raw_extensions` column + post-extract
  in beta.3 (until `S-JSON-EXTRACT-NESTED-001` ships).
- Typed extraction (e.g., extracting an integer from JSON) requires casting in SQL:
  `CAST(json_extract_string(col, 'count') AS BIGINT)` until `S-JSON-EXTRACT-TYPED-001`
  ships. The CAST may fail at DataFusion level for non-numeric values; callers must handle
  this via `TRY_CAST` or accept DataFusion's cast error.
- The `invoke_batch` implementation iterates Arrow columns per-row (scalar per-cell), which
  may be slower than vectorized approaches for large unLIMITed datasets. For
  LIMIT-bounded (25–100 rows) MCP queries this is irrelevant. For future bulk ETL use
  cases, a vectorized path may be warranted.

---

## Source / Origin

Issue 20 from the beta.2 Monroe live-test (jea-readapi, 2026-09-15, D-2520). The
`json_extract_string` dead-path defect was identified when the analyst issued a PQL query
using the function. DataFusion returned an unregistered-function runtime error unstructured
under the prism error taxonomy. D-2522 (human spec-gate approval, 2026-09-16) authorized
the beta.3 scope constraints: synchronous serde_json, literal-key plan gate, top-level
key only, string return type only, no push-down.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.3 | 2026-09-16 | architect | Re-gate pass 2 fixes. OBS-1 (LOW): `## §C Formal Correctness Contract` section added between `## §B Decision` and `## §D Scope Boundaries` to fill the lettering gap (A,B,C,D,E,F,G,H now complete); §C contains the three invariants VP-162 must prove (null-safety, None-in/None-out, bounded-key precondition) and clarifies that behavioral RG-JEX properties are covered by Red Gate tests not Kani. All §D1..§D6 cross-references remain valid — subsection names unchanged. |
| 1.2 | 2026-09-16 | architect | Re-gate pass 1 fixes. OBS-3 (LOW): `## §C Scope Boundaries` header renamed to `## §D Scope Boundaries` — §D1..§D6 subsections already carried the §D prefix; the §C parent header was the discrepancy, not the subsections. F-6 (MED, POL-24): §F error message strings corrected to be VERBATIM with error-taxonomy v2.84 / BC-2.11.025: (a) now includes inline example `(e.g., json_extract_string(col, 'key_name'))` and uses "Dynamic key expressions are not supported." (b) now uses `{key_len} bytes, which exceeds the {max_len}-byte maximum (CWE-400).` with bytes-count placeholder and CWE reference. |
| 1.1 | 2026-09-16 | architect | Adversarial gate fixes (F2/F7/F10). F2 (HIGH): §G mandate table — RG-JEX-006/007 swap corrected to canonical (RG-JEX-006 = non-literal-key gate; RG-JEX-007 = key>256 gate); §B3 "RG-JEX-007 tests this" → "RG-JEX-006 tests this". F7 (MED): §G expanded from 8 to 12 entries — all distinct MUSTs now have dedicated gates: RG-JEX-008 = non-string-coercion (§B1 step 7, NOT NULL); RG-JEX-009 = parse-failure→NULL (§B1 step 2); RG-JEX-010 = dot-in-key literal treatment (§D4 top-level-key-only, injection boundary); RG-JEX-011 = pipe-mode e2e SAP-3 public-surface reachability. Canonical RG-JEX-001..011 list published for PO to mirror into BC-2.11.025. F10 (LOW): §F error message table — `E-QUERY-045:` prefix added to both message strings; hardcoded `256` replaced with `{max_len}` in E-QUERY-045(b) message and trigger cell. |
| 1.0 | 2026-09-16 | architect | Initial. D-2522 beta.3 spec-gate approved. Closes latent dead-path defect: `ScalarFunc::JsonExtractString` existed in ast.rs/sql_parser.rs/pipe_sql_emitter.rs but no ScalarUDF registered. Defines: §B1 synchronous serde_json extraction; §B2 serde_json over arrow-rs rationale; §B3 literal-key plan gate (E-QUERY-045); §D1 VP-162 proof target; §D2–§D6 scope constraints (literal-key, top-level, string-only, no push-down); §E DataFusion registration contract; §F error taxonomy (E-QUERY-045 sub-cases a/b); §G mandate anchors (TD-VSDD-097 Dim-3: 9 MUST→RG mappings); §H latent defect closure rationale. Deferrals anchored: S-JSON-EXTRACT-TYPED-001 (typed variants) + S-JSON-EXTRACT-NESTED-001 (nested JSONPath). |
