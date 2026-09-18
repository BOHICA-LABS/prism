# PR-LEVEL Review — PR #296 (cycle 4)

**Story:** S-JSON-EXTRACT-UDF-001 — `json_extract_string` ScalarUDF + literal-key plan gate (E-QUERY-045)
**PR:** https://github.com/BOHICA-LABS/prism/pull/296
**Branch:** `feature/S-JSON-EXTRACT-UDF-001` → `develop`
**Reviewed SHA:** `3c7650fca622c15fb81e6f9c21b6737491780a73` (verified via `gh pr view 296 --json headRefOid`)
**Merge base:** `561d8baccc670525bccf920615cd6ea305b1410b`
**Diff:** 24 files, +4812 / −16
**Reviewer:** pr-reviewer (fresh-eyes; diff + description + test evidence only)

---

## VERDICT

**REQUEST_CHANGES**

`covered_sha: 3c7650fca622c15fb81e6f9c21b6737491780a73`

**CLEAN (strict): no**
**CLEAN (PR-merge): no**

**5 BLOCKING, 10 SUGGESTION, 8 NIT.**

---

## Summary

Cycle 3's blocking findings are **genuinely closed, not paper-fixed.** I mutation-verified
six of the seven gate walk arms that cycle 3 added tests for — GROUP BY, ORDER BY, JOIN ON,
`Predicate::InSubquery`, `Expr::InSubquery`, and `PipeStage::Where`. Each one, when neutered,
produces a named failing test (`test_jex_rg014`…`rg019`). That is a real closure of
B-C3-1 and deserves saying plainly.

The gate's structural design is also sound, and I verified its most important safety claim
independently rather than taking the comments at their word: `check_jex_in_expr`'s "leaf"
classification is **correct**. I read every AST type the walk declines to descend into —
`Expr::In { field: FieldPath, values: Vec<Literal> }`, `Predicate::{StringOp, Regex, In,
Between, Cidr, Has, Missing, IsNull, Wildcard}`, `StatsStage.aggregates: Vec<StatFunction>`
(whose `AggFunc` variants carry only `FieldPath`/float), `JoinStage.on: JoinCondition`,
`EnrichStage`, `SortExpr`, `FieldsStage`, `VirtualField`, `FromClause` — and none of them can
carry an `Expr` child. The exhaustive-destructure compile guards are the right mechanism and
they are applied honestly.

What blocks merge is one **live security-control bypass** plus an **evidence layer that
cannot be reconciled with the code it attests to**.

The bypass: the E-QUERY-045 gate is defeated by **uppercasing the function name** in any
predicate position. `sql_parser.rs` maps function names case-insensitively
(`name.to_lowercase()`); the cycle-1 T-09a fix in `filter_parser.rs` matches
case-**sensitively** (`func_name.as_str()`), 40 lines below a `RESERVED_KEYWORDS` check that
correctly uses `eq_ignore_ascii_case`. I proved end-to-end that `WHERE
JSON_EXTRACT_STRING(col, other_col)` and a 300-byte uppercase key both sail past the gate,
and that DataFusion then resolves the uppercase name to the registered UDF and executes.
F-JEX-P1-HIGH-001 is closed for one spelling of the function, not for the function.

The evidence: three of the six JSON artifacts are genuine, high-quality wire captures with
verified 256/257-byte boundaries. But the AC-001..AC-010 artifact misstates the inputs and
expected outputs of 7 of the 10 tests it documents, under an "**Observed behavior:**"
heading, and quotes an assertion (`extracted_col.value(0) == "literal_val"`) that appears
nowhere in the workspace. A separate block presents hand-typed text as `cargo nextest`
console output.

---

## Verification performed

Beyond reading the diff, I executed these checks against `3c7650fca`:

| Check | Method | Result |
|---|---|---|
| Gate leaf-classification soundness | read every declined AST type in `ast.rs` | **sound** — no declined type can carry an `Expr` |
| Cycle-3 gate-arm tests load-bearing | 6 source mutations + full `prism-query` suite | **all 6 killed** by a named test (TD-VSDD-059 PASS) |
| Case-sensitivity bypass | temporary E2E probe via `QueryEngine::execute` | **BYPASS CONFIRMED** (BLOCKING-1) |
| DataFusion resolves uppercase → registered UDF | temporary probe via `ctx.sql()` | **confirmed** — executes; 300-byte key runs with no cap |
| Nested-call recursion arm | source mutation + full suite | **survives** — zero coverage (BLOCKING-5) |
| Doubled period in MCP text | parsed the 3 genuine wire captures | **confirmed** in all 3 (BLOCKING-2) |
| `literal_val` in evidence | `grep -rn` over `crates/**/*.rs` | **0 occurrences** (BLOCKING-3) |
| JEX test suite | `cargo nextest run -p prism-query -E 'test(jex)'` | 40/40 PASS (PR says 33) |
| MCP wire test | `cargo nextest run -p prism-mcp` | 1/1 PASS |
| SAP-1 tracing catalog | `git diff … \| grep '^+.*event_type'` | **N/A** — no new emissions |
| SAP-2 DTU↔TOML parity | no sensor TOML in diff | **N/A** |
| SID-1 ignored tests | `grep '^+.*#\[ignore'` over diff | **none added** |
| `unwrap`/`expect` in new prod code | grep `json_extract_udf.rs` | **none** |
| `#[non_exhaustive]` gate | `scripts/check-non-exhaustive.sh` | **PASS 99/99** |
| Error taxonomy registration | `error-taxonomy.md` line 269 | messages match code **byte-for-byte** |
| `engine.rs` decomposition anchor | `tech-debt-register.md` line 248 | **valid** — TD-DECOMP-EPIC-001, anchor `S-DECOMP-ENGINE-A/B` |

All temporary probe files were deleted and `engine.rs` restored; the worktree is clean at
`3c7650fca` (`git status --porcelain` empty).

---

# BLOCKING

### BLOCKING-1 — E-QUERY-045 is bypassable by uppercasing the function name in WHERE / HAVING / pipe-where / filter positions (security-control bypass; F-JEX-P1-HIGH-001 only partially closed)

**Location:** `crates/prism-query/src/filter_parser.rs`, `build_predicate_parser` →
`fn_call_comparison` `.validate(...)` closure (the T-09a fix added by this PR).

```rust
let scalar_func = match func_name.as_str() {
    "json_extract_string" => ScalarFunc::JsonExtractString,
    _ => ScalarFunc::Unknown(func_name),
};
```

`sql_parser.rs` `known_scalar` performs the same mapping **case-insensitively**:

```rust
let func = match name.to_lowercase().as_str() {
    "json_extract_string" => ScalarFunc::JsonExtractString,
    ...
```

So any spelling other than all-lowercase produces `ScalarFunc::Unknown` in predicate
position, which `check_jex_in_expr` handles with the generic
`Expr::FuncCall(FuncCall::Scalar { args, .. })` arm — recursing into arguments but performing
**no key validation at all**. The 21-entry `RESERVED_KEYWORDS` check 40 lines above this
match uses `eq_ignore_ascii_case`, so the module already establishes that identifiers here
are case-insensitive; this new match is the outlier.

**Empirically proven** (temporary probe through `QueryEngine::execute`, since deleted):

| Query | Result |
|---|---|
| `… WHERE json_extract_string(raw_data, severity_col) = 'x'` | `E-QUERY-045: … requires a literal string key …` ✅ |
| `… WHERE JSON_EXTRACT_STRING(raw_data, severity_col) = 'x'` | **no error — gate bypassed** ❌ |
| `… WHERE Json_Extract_String(raw_data, severity_col) = 'x'` | **no error — gate bypassed** ❌ |
| `… WHERE JSON_EXTRACT_STRING(raw_data, '<300 bytes>') = 'x'` | **no error — CWE-400 cap bypassed** ❌ |
| `FROM t \| where JSON_EXTRACT_STRING(raw_data, severity_col) = 'x'` | **no error — gate bypassed** ❌ |
| `SELECT JSON_EXTRACT_STRING(raw_data, severity_col) FROM t` | `E-QUERY-045` ✅ (sql_parser lowercases) |
| `SELECT JSON_EXTRACT_STRING(raw_data, '<300 bytes>') FROM t` | `E-QUERY-045: key is 300 bytes …` ✅ |

The SELECT-vs-WHERE split in the last three rows is the asymmetry made visible: the same
uppercase spelling is gated in projection position and ungated in predicate position.

**This is not a theoretical bypass.** A second probe against a `SessionContext` with the UDF
registered and a real `MemTable` confirmed the downstream behavior:

- `SELECT JSON_EXTRACT_STRING(...)`, `Json_Extract_String(...)` and lowercase **all resolve
  to the registered UDF and return rows** — DataFusion function resolution is
  case-insensitive, and the UDF is registered as lowercase `json_extract_string`.
- A non-literal key that reaches `invoke_with_args` produces
  `Execution error: json_extract_string: key argument must be a Utf8 scalar, got Utf8` —
  an opaque, non-taxonomy DataFusion error. That is precisely the defect class this story
  exists to eliminate (Issue 20 / beta.2 Monroe demo dead path). The message is also
  self-contradictory ("must be a Utf8 scalar, got Utf8").
- A 300-byte literal key that reaches `invoke_with_args` **executes normally** — there is no
  runtime cap, so the CWE-400 control is fully defeated on this path (see SUGGESTION-10).

**Contract impact:**

- **AC-012 is not satisfied as written.** The story states: *"The rejection path is identical
  to SELECT position: same error codes, same plan-time gate, same verbatim messages from
  ADR-066 §F."* It is demonstrably not identical.
- The PR description's security claim is false as stated: *"All four predicate positions
  (WHERE/HAVING/pipe-where/SELECT) now consistently route `json_extract_string` to
  `ScalarFunc::JsonExtractString`."*
- `error-taxonomy.md` line 269 asserts **"No fan-out occurs"** for both sub-cases. On the
  uppercase path the query proceeds past the gate to client resolution and sensor fan-out.
- The in-code comment claiming *"parity with sql_parser.rs `known_scalar`"* is inaccurate in
  exactly the dimension that matters.

**Zero test coverage of the gap:** `grep -c "JSON_EXTRACT_STRING("` over
`tests/test_json_extract_udf.rs` returns **0**. No test in the story uses any spelling but
all-lowercase. HAVING is tested only in lowercase (`test_jex_rg012_c`) and shares the same
`fn_call_comparison` code path, so it carries the identical bypass. Filter mode
(`Ast::Filter`) likewise routes through `build_predicate_parser`.

**Suggested fix** — compute the lowercase key before the move, mirroring `sql_parser.rs`:

```rust
let scalar_func = match func_name.to_ascii_lowercase().as_str() {
    "json_extract_string" => ScalarFunc::JsonExtractString,
    _ => ScalarFunc::Unknown(func_name),
};
```

Add E2E tests for `JSON_EXTRACT_STRING` and `Json_Extract_String` in WHERE, HAVING, and
pipe-`| where`, covering both E-QUERY-045(a) and (b). **Scope note:** I checked the other
five `known_scalar` names (`subnet_contains`, `time_window`, `ioc_match`, `mitre_tactic`,
`severity_label`) — none is a registered UDF (`register_udf` appears only for
`json_extract_string` and the infusion UDFs), so the comment's blast-radius argument holds
for them and the fix can stay narrow.

---

### BLOCKING-2 — Doubled period ships in the agent-facing MCP `content[].text` for both E-QUERY-045 variants (SID-2 [H8b] recurrence)

**Composition site:** `crates/prism-mcp/src/error_mapping.rs` ~line 2553:

```rust
let content_text = format!("ERROR: [{}] - {}. {}", fields.category, fields.message, fields.suggestion);
```

The template appends `". "`, and both new `#[error(...)]` messages in
`crates/prism-core/src/error.rs` already end in `.`. I confirmed this in the PR's own
**genuine** wire captures rather than by inference:

```
docs/demo-evidence/S-JSON-EXTRACT-UDF-001/AC-006-non-literal-key-rejection.json:34
  "ERROR: [validation] - E-QUERY-045: … Dynamic key expressions are not supported.. Provide a string literal …"

docs/demo-evidence/S-JSON-EXTRACT-UDF-001/AC-007-key-length-boundary.json:34
  "ERROR: [validation] - E-QUERY-045: json_extract_string key is 257 bytes, which exceeds the 256-byte maximum (CWE-400).. Reduce the json_extract_string key …"

docs/demo-evidence/S-JSON-EXTRACT-UDF-001/AC-012-where-predicate-gate.json:34
  (same doubled period as AC-006)
```

This is the exact defect class SID-2 was codified to prevent — the rule's origin is the
[H8b] doubled `"see audit log. See audit log for details."` escape. Only 14 of 151
`PrismError` Display messages end in a period, so these two variants join a 9% minority that
trips this template; the project convention is to omit the trailing period.

It ships **because** the SID-2 guard was scoped too narrowly. The single composed-output
test, `test_S_JSON_EXTRACT_UDF_001_e_query_045a_sid2_no_example_duplication_in_content_text`,
asserts only `content_text.matches("json_extract_string(col, 'key_name')").count() == 1`. It
discharges SID-2 rule 2 (no duplicated phrase) for variant (a) and nothing else. **SID-2 rule
1 — "assert on the FULL composed string as emitted" — is not discharged for either variant**,
and variant (b) has no composed-output test at all. A full-string `assert_eq!` would have
caught this immediately.

Variant (b) additionally states the same fact twice: message *"exceeds the 256-byte
maximum"*, suggestion *"Reduce … to 256 UTF-8 bytes or fewer"* — `"256"` appears three times
across the pair.

This is user-visible malformed output on the primary LLM-agent-facing surface, with a
one-line fix. Per CLAUDE.md's production-grade default it is a blocker, not an advisory.

**Suggested fix:** drop the trailing `.` from the two suggestion strings (or strip it in the
composition), then add a full-composed `assert_eq!` for **both** variants so the guard is
rule-1 compliant.

---

### BLOCKING-3 — AC-001..AC-010 demo evidence misstates the tests it attests to (7 of 10 ACs), and quotes an assertion that does not exist

**Files:** `docs/demo-evidence/S-JSON-EXTRACT-UDF-001/AC-001-010-functional-unit-tests.json`
(lines 27–101) and `docs/demo-evidence/S-JSON-EXTRACT-UDF-001/evidence-report.md`
(lines 42–171).

Ground truth is `crates/prism-query/tests/test_json_extract_udf.rs`:

| AC | Artifact claims | Actual test |
|---|---|---|
| AC-002 | `{"status":null}` / key `status` | `{"severity":null}` / key `severity` |
| AC-003 | `{"severity":"high"}` / key `missing_key` | `{"other_field":"value"}` / key `severity` |
| AC-005 | key `key` | key `severity` |
| AC-008 array | `{"tags":["a","b"]}`/`tags` → `["a","b"]` | `{"items":[1,2]}`/`items` → `[1,2]` |
| AC-008 object | report `{"nested":{"k":"v"}}`/`nested`; JSON artifact `{"nested":"value"}` | `{"meta":{"k":"v"}}`/`meta` → `{"k":"v"}` |
| AC-009 | `{not_json}` / key `key` | `"not valid json at all !!"` / key `severity` |
| AC-010 | expected `"literal_val"` | expected `"dotted"` |

Two details make this blocking rather than cosmetic:

1. **The artifacts disagree with each other**, not just with the code — for AC-008 object,
   `evidence-report.md` and the JSON artifact give two different wrong answers, and the JSON
   artifact's version is internally incoherent (extracting key `nested` from
   `{"nested":"value"}` yields `value`, not an object).

2. **A quoted assertion is fabricated.** `evidence-report.md` line 170 reads
   **`Key assertion: extracted_col.value(0) == "literal_val"`**. I verified
   `grep -rn "literal_val" --include="*.rs" crates/` → the only two hits are
   `vp013_cycle_detection.rs` ("literal_value", unrelated) and
   `test_timestamp_literal_valid_iso8601…` (substring of a test name). The string
   `literal_val` occurs **zero** times as a value anywhere in the Rust sources. The actual
   AC-010 test at `tests/test_json_extract_udf.rs:1076` uses
   `{"a.b":"dotted","a":{"b":"nested"}}` and asserts `"dotted"` — on a binding named `col`,
   not `extracted_col`. The `extracted_col` binding is used only by
   `test_jex_rg011`, which tells you these "assertions" were paraphrased and then presented
   as quoted source.

All of this sits under the heading **"Observed behavior:"**. The underlying behaviors are
genuinely covered — all 40 tests pass, I ran them — so this is a defect in the **attestation
layer**, not the implementation. But per the demo-evidence gate, an artifact that cannot be
reconciled with the code it attests to is not evidence. A reviewer cross-checking AC-010
against the source finds a different value and has no way to tell which is right.

**Suggested fix:** regenerate from captured `cargo nextest run … --no-capture` output rather
than hand-authoring, or relabel the file honestly as a paraphrased index and remove the
"Observed behavior" / "Key assertion" framing and the invented strings.

---

### BLOCKING-4 — `evidence-report.md` presents hand-typed text as captured console output, and falsely attributes MCP envelope assertions to prism-query tests

Two distinct false claims in the same file.

**(a) Fabricated console block — lines 229–236:**

```
cargo nextest run -p prism-query -E 'test(test_jex)' --no-fail-fast
33 tests run: 33 passed, 0 failed
```

I ran that exact command at `3c7650fca`: **`40 tests run: 40 passed`**. Two independent tells
that this was typed rather than pasted: nextest's summary format is
`N passed, M skipped` and never emits `0 failed`; and the count is stale by exactly the 7
tests cycle 3 added (`test_jex_rg014`…`rg020`), which are also missing from the report's
33-row table. `AC-001-010-functional-unit-tests.json` lines 10–15 carry the same defect
(`"total_tests": 12` for filter `test(test_jex_rg00)`; actual is 15).

**(b) False wire-assertion attribution — line 188:** the report claims RG-JEX-006, RG-JEX-007,
RG-JEX-012 and RG-JEX-013 *"assert on the MCP `structuredContent.error` envelope."* They do
not, and cannot: those four tests live in `crates/prism-query/tests/test_json_extract_udf.rs`,
and **prism-query has no dependency on prism-mcp** (`crates/prism-query/Cargo.toml` mentions
it only in comments). `test_jex_rg006_non_literal_key_rejected_e_query_045_a` asserts on
`format!("{}", PrismError::JsonExtractNonLiteralKey)` — a `Display` string, pre-serialization,
with no composition and no envelope. That test's own inline comment
(`// Wire-shape assertion (SID-2 composed-output + MCP surface):`) is the same paper claim in
miniature (TD-VSDD-059 shape).

Compounding it: the PR's genuinely strongest evidence —
`crates/prism-mcp/tests/bc_2_11_025_jex_wire_null_test.rs`, 332 new lines asserting
serialized-JSON null-not-absent and guarding the `explicit_nulls` [C3]/[H20] defect class — is
**not cited in any evidence artifact.** The real wire test is omitted while a false substitute
is claimed in its place.

Both claims were **introduced by the cycle-2 fix labelled "N-e: accurate wire-shape
labeling"** — a remediation that added a new inaccuracy while closing the old one. That
pattern is worth flagging to the fix-burst explicitly.

---

### BLOCKING-5 — The nested-call recursion arm of the E-QUERY-045 gate has zero test coverage workspace-wide (unguarded security arm; residual of cycle-3 B-C3-1)

**Location:** `crates/prism-query/src/engine.rs`, `check_jex_in_expr`, the
`ScalarFunc::JsonExtractString` arm's valid-literal-key branch:

```rust
// Valid literal key — still recurse into all args to catch
// nested json_extract_string calls (e.g., nested extraction).
for arg in args {
    check_jex_in_expr(arg)?;
}
```

**Mutation-verified:** I deleted this loop and ran the **full** `prism-query` suite —
**1792 tests run, 1792 passed, 0 failed.** Not one test kills it. `grep` confirms no query or
AST anywhere in the story's tests contains two `json_extract_string` occurrences (114 total
occurrences, zero nested).

**To be precise about severity — this is a coverage gap, not a live bypass.** I probed the
unmutated code and the production behavior is correct:

| Query | Result |
|---|---|
| `SELECT json_extract_string(json_extract_string(raw_data, bad_col), 'ok') FROM t` | `E-QUERY-045(a)` ✅ |
| `SELECT json_extract_string(json_extract_string(raw_data, '<300B>'), 'ok') FROM t` | `E-QUERY-045(b)` ✅ |

The loop is load-bearing and works — it is the only thing stopping an outer call with a valid
literal key from shielding an inner call with a non-literal or oversized key. What is missing
is the regression guard. Any future refactor can delete it and CI stays green, silently
reopening a CWE-400 / dynamic-key bypass.

This is exactly the class cycle 3's B-C3-1 addressed ("gate arms deletable with the suite
green"). Cycle 3 closed six walk arms and left this sub-arm open, so the closure is
incomplete. Given the fix-burst is already open for BLOCKING-1 and the remedy is two tests,
it should close here.

**Suggested fix:** two E2E tests through `make_gate_engine()` + `engine.execute` — inner
non-literal key, and inner 257-byte key — using the two queries above, asserting the exact
`PrismError` variants.

*(Nested calls in WHERE position return `E-QUERY-001` parse error — the filter-mode grammar
does not accept a function call as a function argument. SELECT position is the reachable one.)*

---

# SUGGESTION

### SUGGESTION-1 — Third `PrismError` dispatch site not swept (TD-VSDD-060)
`crates/prism-mcp/src/error_mapping.rs` contains **three** dispatch sites. The PR correctly
adds explicit arms to two — `map_prism_error` (~line 447; arms at ~467 and ~479, both
`INVALID_PARAMS`, ahead of the `_ => INTERNAL_ERROR` catch-all at ~487) and
`prism_error_to_structured_call_result` (arms at ~2428 and ~2457, `category: "validation"`,
ahead of the `upstream_error` catch-all at ~2479). The third, `map_prism_error_to_structured`
(~line 2589), was **not swept**: its catch-all at ~2615 is
`_ => ("E-QUERY-001".to_string(), format!("{err}"))`, so both new variants would emit
`code: "E-QUERY-001"` alongside `message: "E-QUERY-045: …"` — a self-contradicting envelope.
Mitigating: it has no in-repo callers (only a test-helper reference in
`crates/prism-mcp/tests/mcp_infrastructure.rs`). It is `pub`, so it is reachable API.

### SUGGESTION-2 — SID-2 rule 1 undischarged; variant (b) has no composed or wire test
See BLOCKING-2 for the consequence. Add a full-string `assert_eq!` on
`extract_content_text_from_result()` output for both variants, and mirror both
`…_e_query_045a_sid2_no_example_duplication_in_content_text` and
`…_e_query_045a_wire_level_serialized_json` for `JsonExtractKeyTooLong`.

### SUGGESTION-3 — AC-004's engine-path companion test was omitted, inconsistently with the PR's own remediation rationale
`test_jex_rg004_null_column_input_returns_sql_null` drives a bare
`SessionContext.sql()` via `make_udf_ctx()` — it never touches `QueryEngine`. The PR's own
F-4 block states that arms "currently only exercised via bare `SessionContext.sql()`" need
`QueryEngine::execute` companions, and adds four (non-object, coerce, parse-fail,
dot-in-key) — but **not** the Arrow-null-column arm. The fixture was built for it:
`SingleRowAdapter` declares `payload: Option<&'static str>` and does
`StringArray::from(vec![self.payload])`, yet `grep -c "payload: None"` = **0**; all four
constructions pass `Some(...)`. The compensating proof the story names for AC-004
(`vp162_b_none_input_is_none_output`) is `#[cfg(kani)]`-gated and does not run in `just check`
or the normal CI test lane. Net: VP-162 invariant 2 from the product surface is asserted by no
executing test. One ~35-line test with `payload: None` closes it.

### SUGGESTION-4 — The two "accept" boundary tests are negative-only and cannot distinguish "gate accepted" from "gate never ran"
`test_jex_rg007_b_key_at_exactly_max_len_accepted` asserts only
`!matches!(result, Err(JsonExtractKeyTooLong))` / `!matches!(… NonLiteralKey)`, and
`test_jex_rg007_c_multibyte_key_at_boundary_accepted` likewise. Because the gate is invoked as
`if let Ok(ast) = PrismQlParser::parse(…) { check_json_extract_key_literal(&ast)?; }`, a parse
failure silently skips it and both tests still pass. They currently do reach the gate — the
matching 257-byte and 258-byte rejection tests use identical query shapes and fire — so this
is durability, not a present false gate. Assert the specific expected terminal outcome.

### SUGGESTION-5 — Two of the MCP wire test's null assertions are tautological in isolation, with an undocumented ordering dependency
In `crates/prism-mcp/tests/bc_2_11_025_jex_wire_null_test.rs`,
`rows[1]["extracted"].is_null()` and `rows[2]["extracted"].is_null()` pass when the key is
**absent**, because `serde_json::Value`'s `Index` impl returns `Value::Null` for a missing
key. They are non-vacuous only because of the `rows[n].get("extracted").is_some()` assertions
immediately above. That ordering is load-bearing and uncommented: anyone "simplifying" away
the `get()` checks silently turns this into a false gate for the exact `explicit_nulls`
defect class it exists to catch. Collapse to
`assert_eq!(rows[1].get("extracted"), Some(&serde_json::Value::Null))`.

### SUGGESTION-6 — Two in-file comments claim prism-mcp wire coverage that does not exist, and the coercion arm has an unasserted wire distinction
`tests/test_json_extract_udf.rs` (F-4-B coerce, ~L1374; F-4-D dot-in-key, ~L1497) both state
that wire coverage "lives in prism-mcp:
`test_BC_2_11_025_json_extract_string_null_row_wire_null_not_absent`". That test only
exercises `{"severity": "critical" | null | absent}` — neither arm appears in it.
Substantively, coercion has an MCP-visible wire distinction nothing asserts: `{"count":42}`
must serialize as the JSON **string** `"42"` (Utf8 Arrow column), not the number `42`. Under
the wire-shape rule that is the required assertion class. Extend the MCP fixture with a
numeric/boolean payload row and correct the two comments.

### SUGGESTION-7 — `AC-011-pipe-mode-udf-registration.json` is a trimmed/reordered transcript labelled as a wire capture
Its `mcp_response.result` keys are `['isError','content']` — `structuredContent` is entirely
absent and the order is inverted versus every genuine capture
(`['content','structuredContent','isError']`); the embedded `content[0].text` also lacks
`normalized_pql`. The production tool always emits `structuredContent`. Its
`unit_test_evidence` block, by contrast, **is** accurate against
`test_jex_rg011_pipe_mode_end_to_end_executes`. Re-capture verbatim or relabel as an
abridged excerpt — the report's Recording Method section claims unqualified "MCP wire
transcripts (JSON-RPC 2.0 over stdio)".

### SUGGESTION-8 — No single test drives pipe mode through the MCP `query` tool (AC-011 is union-covered only)
`test_jex_rg011_pipe_mode_end_to_end_executes` does pipe mode but asserts at the Arrow layer;
the MCP wire test asserts at the wire layer but sends plain SQL with no pipe stage. The union
satisfies AC-011; no single test does. Cheapest close: append `| limit 10` to the MCP test's
query string — the fixture already returns 3 rows and the assertions are position-independent.

### SUGGESTION-9 — Capture harness for the cycle-2 re-recordings is not preserved (reproducibility)
The genuine AC-007/AC-012 captures use `"id": 3` for their second sub-demo, but the committed
`prism_plan_gate_demo.py` hardcodes `id: 2` for every call. Those captures therefore came
from a different, unpreserved harness, so the evidence is not reproducible from the committed
artifacts. Commit the cycle-2 capture script alongside the VHS driver.

### SUGGESTION-10 — `invoke_with_args` does not enforce the 256-byte cap, making the plan gate a single point of failure for CWE-400
`json_extract_udf.rs` documents this explicitly ("enforced here at plan time but NOT in
`invoke_with_args`"). BLOCKING-1 is the concrete demonstration of why that matters: one gap in
the gate walk defeats the control entirely. The key is extracted **once per batch**, not per
row, so a length check there costs nothing measurable and would have contained the uppercase
bypass to a structured error. Recommend adding it as defense-in-depth.

---

# NIT

- **NIT-1 — Stale module-doc header** in `tests/test_json_extract_udf.rs`: L19–21 describes
  `test_jex_dml_ast_safe_skip_with_jex_variant`, which is not in this file (it lives in
  `engine.rs` §`jex_gate_walk_completeness_tests` and uses `parse_sql_dml`, not
  `parse_and_plan`); L3 claims "11 baseline failing tests" for a 35-test file; L4–5 claims
  every test traces to exactly one `EC-11-025-NNN`, false for rg014..rg020; the L26–34 table
  row `DML rework` matches no test name.
- **NIT-2 — ~20 present-tense comments describing code that no longer exists**: "`json_extract_string_udf()`
  … panics `todo!()` … not yet implemented" (L309–314), "the E-QUERY-045 gate (when wired in
  T-09)" (L296), "`invoke_with_args` is still `todo!()` → panic → still RED" (L1180), plus
  per-test "FAILS RED" lines. Red-Gate provenance is a legitimate convention, but the
  `todo!()` claims are now false statements about `json_extract_udf.rs`.
- **NIT-3 — Blanket lint allows mask rot**: `#![allow(… non_snake_case, dead_code,
  unused_imports)]` (L65–72). `non_snake_case` is unnecessary (all test names are snake_case),
  and the other two already hide a defect: **`QueryResult` (L88) is imported and never used**.
- **NIT-4 — Redundant `#[cfg(test)] mod tests` wrapper** in
  `bc_2_11_025_jex_wire_null_test.rs` (L26–27). A no-op for `tests/*.rs` targets, but if the
  file were ever moved into `src/` the single test would silently vanish. No sibling
  integration test in `crates/prism-query/tests/` uses this wrapper.
- **NIT-5 — `Display` asserted on locally constructed errors**, not the returned one:
  `let display = format!("{}", PrismError::JsonExtractNonLiteralKey);` in rg006 (L614),
  rg012 (L1548), rg012-c (L1678). These test the `thiserror` attribute, not the engine.
  Prefer `format!("{returned_err}")`.
- **NIT-6 — `codes::INVALID_PARAMS` asserted only symbolically**; no test pins it to the
  literal `-32602`. The paired `assert_ne!(code, codes::INTERNAL_ERROR)` limits blast radius,
  but a corrupted constant passes all six tests.
- **NIT-7 — `prism_plan_gate_demo.py` robustness**: it prints a *derived* `isError` (true iff
  `structuredContent.error` is non-empty) rather than reading `r.get("isError")`, which is
  present in the response — so the `.gif` displays a computed label as if read from the wire
  (values happen to be correct in all six demos). It also swallows every parse error
  (`except Exception: pass`) and discards stderr, so a crashed or hung binary renders as a
  query with no output rather than a visible failure. The `.tape`'s `Sleep 60s` against
  6 × 10 s timeouts leaves no margin.
- **NIT-8 — Untested minor arms**: `args.get(1) == None` (e.g. `json_extract_string(col)`)
  falls to `_ => Err(JsonExtractNonLiteralKey)`, so a 1-arg call reports "requires a literal
  string key" rather than an arity error — untested and arguably a message-quality issue.
  Also: the cap is measured on the decoded literal with no test for an escaped-character key;
  and `invoke_with_args` accepts `ScalarValue::LargeUtf8` for the *key* but not for the JSON
  *column* (harmless under the `Exact([Utf8, Utf8])` signature, but asymmetric).

---

## Checklist

| # | Item | Verdict |
|---|---|---|
| 1 | **Diff coherence** | **PASS.** All 24 files trace to the story. The `Cargo.lock` rustls 0.23.40→0.23.45 bump (RUSTSEC-2026-0285) is unrelated but justified in the description with a transitive-dependency table; `cargo audit` and `cargo deny` pass. |
| 2 | **Description accuracy** | **FAIL.** Security claim of four-position parity is false (BLOCKING-1). Test count 33 vs actual 40. `engine.rs` stated 17,651; actual 17,672. |
| 3 | **Test coverage** | **FAIL.** Nested-recursion arm has zero coverage (BLOCKING-5); no case-variant coverage at all (BLOCKING-1); AC-004 engine path uncovered (SUGGESTION-3); SID-2 rule 1 undischarged (SUGGESTION-2). |
| 4 | **Demo evidence** | **FAIL.** 3 of 6 JSONs are genuine, high-quality captures with both success and error paths and verified 256/257-byte boundaries. But AC-001..010 misstates 7 of 10 ACs with a fabricated assertion (BLOCKING-3), a hand-typed console block is presented as nextest output (BLOCKING-4), and AC-011 is a trimmed transcript (SUGGESTION-7). **`prism_plan_gate_demo.py` does genuinely spawn the built binary over MCP stdio and parse its real stdout — the `.gif`/`.webm` are NOT fabricated.** |
| 5 | **Commit quality** | **PASS.** Conventional format, story/finding IDs in subjects, no AI attribution. |
| 6 | **Diff size** | **FLAG (accepted).** +4812 exceeds the 500-line threshold, but ~1,300 lines are production and the rest are tests plus demo evidence. The `engine.rs` growth rationale is valid — I verified the TD-DECOMP-EPIC-001 registration at `tech-debt-register.md:248` with anchor `S-DECOMP-ENGINE-A/B`. |
| 7 | **Missing changes** | **FAIL.** AC-012's "rejection path is identical to SELECT position" is not implemented for non-lowercase spellings (BLOCKING-1). |
| 8 | **Dependency status** | **PASS.** S-ADR058-OCSF-ROUTING-001 (PR #241) is merged. |

## Standing probes

| Probe | Result |
|---|---|
| **SAP-1** tracing emission catalog | **N/A** — no new `event_type` emissions in the diff. |
| **SAP-2** DTU↔TOML parity | **N/A** — no sensor TOML touched. |
| **SAP-3** spec-arm reachability | **MOSTLY PASS.** 18 E2E tests through `QueryEngine::execute`, 0 synthetic-AST in the integration file; all four required positions (SELECT / WHERE / HAVING / pipe-`\| where`) have E2E coverage, plus GROUP BY, ORDER BY, JOIN ON and both `InSubquery` arms. The four synthetic-AST tests in `engine.rs` all carry explicit grammar-unreachable rationale (rule 3 satisfied). **Residuals:** nested-call arm (BLOCKING-5) and AC-004 (SUGGESTION-3). |
| **SID-1** no ignored tests | **PASS.** Zero `#[ignore]`, zero feature gates, zero commented-out tests; 40/40 + 1/1 executed. |
| **SID-2** composed-output assertions | **FAIL.** Rule 1 undischarged for both variants; variant (b) has no composed test; the [H8b] doubled-period defect ships (BLOCKING-2). |
| **TD-VSDD-059** paper-fix detection | **MIXED.** Cycle-3's six gate-arm tests are genuinely load-bearing (mutation-verified). But two paper claims remain: the `// Wire-shape assertion (SID-2 composed-output + MCP surface)` comment on RG-JEX-006 (no serialization, no composition) and the false attribution at `evidence-report.md:188` (BLOCKING-4). |
| **Wire-shape discipline** | **PASS with caveats.** `bc_2_11_025_jex_wire_null_test.rs` asserts on genuinely serialized bytes — `structured_content.results.rows` is produced by `arrow_json WriterBuilder::with_explicit_nulls(true)` and re-parsed, so the present-as-null assertions genuinely fail under the `explicit_nulls=false` default. Caveats: SUGGESTION-5 (tautological halves) and SUGGESTION-6 (coercion arm unasserted). |

## CI

No failures. Passing: Clippy, Format, Cargo audit, Cargo deny, Semver, Workspace layout,
non-exhaustive compile-fail (99/99), Perimeter compile-fail, GitGuardian, WASM32, and 8 more.
Pending at review time: `Test (aarch64-apple-darwin)`, `Test (x86_64-pc-windows-msvc)`,
`Test (no-default-features)`, `E2E smoke`, `Fuzz smoke`. `mergeable: MERGEABLE`,
`mergeStateStatus: BLOCKED` (awaiting review).

---

## Required to clear this cycle

1. **BLOCKING-1** — lowercase the function-name match in `filter_parser.rs`; add
   uppercase/mixed-case E2E tests for WHERE, HAVING and pipe-`| where`, covering both
   E-QUERY-045(a) and (b). Correct the false parity claims in the PR description and the
   in-code "parity with sql_parser.rs" comment.
2. **BLOCKING-2** — remove the doubled period; add full-composed-string assertions for both
   variants (discharges SID-2 rule 1).
3. **BLOCKING-3** — regenerate or honestly relabel the AC-001..010 evidence; remove the
   fabricated `literal_val` assertion.
4. **BLOCKING-4** — replace the hand-typed nextest block with real captured output at the
   final HEAD; correct the AC-011 wire attribution and cite
   `crates/prism-mcp/tests/bc_2_11_025_jex_wire_null_test.rs`.
5. **BLOCKING-5** — add two nested-call E2E tests.

Per the frozen-HEAD rule (DRIFT-ORCH-PRLEVEL-PUSH-001), pushing these fixes resets the
PR-LEVEL streak; cycle 5 must re-gate on the new HEAD with a fresh `covered_sha`.

**Note for the fix-burst:** two of the five blockers (BLOCKING-2's narrow SID-2 guard,
BLOCKING-4's false attribution) were *introduced or left open by previous cycles'
remediations*. Worth an explicit re-read of each cycle-4 fix against its own claim before
re-gating.
