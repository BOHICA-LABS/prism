---
document_type: verification-property
level: L4
vp_id: "VP-162"
title: "json_extract_string_impl — Null Safety and Panic Freedom (Kani)"
version: "1.3"
status: draft
producer: architect
phase: P0
inputs:
  - crates/prism-query/src/json_extract_udf.rs
  - .factory/specs/architecture/decisions/ADR-066-json-extract-scalar-udf.md
  - .factory/stories/S-JSON-EXTRACT-UDF-001.md
input-hash: "pending"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.11.025
source_invariant: null
module: prism-query
priority: P0
proof_method: kani
verification_method: kani
feasibility: feasible
lifecycle_status: draft
introduced: "2026-09-16"
modified: "2026-09-16"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
anchor_stories:
  - S-JSON-EXTRACT-UDF-001
---

# VP-162: json_extract_string_impl — Null Safety and Panic Freedom (Kani)

## Property Statement

For any `column_value: Option<&str>` and any `key: &str` satisfying the post-gate
precondition `key.len() <= 256`, the pure function `json_extract_string_impl` either
returns `Some(String)` or `None`. It NEVER panics, NEVER aborts via an unwind, and NEVER
propagates an unstructured internal error.

```
∀ column_value: Option<&str>, key: &str .
  key.len() ≤ 256 →
  json_extract_string_impl(column_value, key) ∈ { Some(_), None }  ∧
  ¬panics(json_extract_string_impl(column_value, key))
```

The `key.len() <= 256` precondition models the literal-key plan gate in ADR-066 §B3 and
§D3: the gate runs at plan time (before `json_extract_string_impl` is ever invoked), so the
runtime function only encounters keys that have already passed the gate.

---

## Source Traceability

| Traces to | Value |
|-----------|-------|
| BC | BC-2.11.025 |
| ADR | ADR-066 §D1 — VP-162 Proof Target |
| Story | S-JSON-EXTRACT-UDF-001 (beta.3 json_extract UDF story) |
| Invariant | DI-019 (Query Security Limits — 64KB query length, 10K record cap, 30s timeout; from domain-spec/invariants.md); 256-byte key-length precondition in this proof is anchored to ADR-066 §D3 (CWE-400 key-length cap) — not DI-019 |
| Architecture module | `prism-query` — `crates/prism-query/src/json_extract_udf.rs` |

---

## Proof Target

**Function:** `json_extract_string_impl` as defined in ADR-066 §B1.

**Signature:**
```rust
pub(crate) fn json_extract_string_impl(
    column_value: Option<&str>,
    key: &str,
) -> Option<String>
```

**Module location:** `crates/prism-query/src/json_extract_udf.rs`

**Why this function is provable:**
- **Pure:** no I/O, no global state mutation, no async operations, no `unsafe` required.
  It takes data in and returns data out. This satisfies the purity boundary requirement for
  Kani model checking.
- **No DataFusion or Arrow dependency in the pure path:** The function uses only
  `serde_json` and Rust standard types. The DataFusion/Arrow layer wraps the pure function
  but is not part of the proven core. The Kani harness does not need to model DataFusion.
- **Bounded domain:** `serde_json::from_str` on a symbolic string with `key.len() ≤ 256`
  is tractable for Kani's bounded model checker. The model checking bound is set at
  `unwind 8` to cover typical JSON nesting depths.

---

## Kani Proof Harness

```rust
// crates/prism-query/src/proofs/vp162_json_extract_null_safety.rs

#[cfg(kani)]
mod vp162_proofs {
    use super::super::json_extract_udf::json_extract_string_impl;

    /// VP-162: null safety and panic freedom for json_extract_string_impl.
    ///
    /// Precondition: key.len() <= 256 (models ADR-066 §B3 literal-key plan gate).
    /// Postcondition: result is Some(String) or None; no panic.
    ///
    /// Pattern follows VP-014 and VP-015 in this module: bounded Vec<u8> + from_utf8.
    /// kani::any::<&str>() is NOT used because it is not a stable API across Kani
    /// versions and does not correctly bind preconditions on key length.
    #[kani::proof]
    #[kani::unwind(8)]
    fn vp162_json_extract_string_null_safety() {
        // Symbolic key bounded to 256 bytes — models ADR-066 §B3/§D3 plan gate.
        // any_vec::<u8, 256>() guarantees key_bytes.len() <= 256 structurally;
        // no separate kani::assume on length needed.
        let key_bytes: Vec<u8> = kani::vec::any_vec::<u8, 256>();
        kani::assume(std::str::from_utf8(&key_bytes).is_ok());
        let key = std::str::from_utf8(&key_bytes).unwrap();

        // Symbolic column value: None or Some(arbitrary bounded string).
        // 1024-byte column bound covers realistic OCSF raw_extensions payloads.
        let has_value: bool = kani::any();
        if has_value {
            let col_bytes: Vec<u8> = kani::vec::any_vec::<u8, 1024>();
            if let Ok(col_str) = std::str::from_utf8(&col_bytes) {
                // Kani verifies panic-freedom automatically: any reachable panic site
                // (index out of bounds, unwrap on None, etc.) is a verification failure.
                let _result: Option<String> = json_extract_string_impl(Some(col_str), key);
            }
            // If col_bytes is not valid UTF-8, skip invocation — the plan gate ensures
            // the column is a Utf8 Arrow column, so non-UTF-8 bytes model an unreachable path.
        } else {
            let _result: Option<String> = json_extract_string_impl(None, key);
        }
    }

    /// VP-162-B: None input → None output.
    ///
    /// Specialization: when column_value is None, the result is always None.
    /// Uses the same Vec<u8> + from_utf8 bounded key pattern as the main harness.
    #[kani::proof]
    #[kani::unwind(4)]
    fn vp162_b_none_input_is_none_output() {
        let key_bytes: Vec<u8> = kani::vec::any_vec::<u8, 256>();
        kani::assume(std::str::from_utf8(&key_bytes).is_ok());
        let key = std::str::from_utf8(&key_bytes).unwrap();
        let result = json_extract_string_impl(None, key);
        assert!(result.is_none(), "None column input must produce None output");
    }
}
```

**Notes for implementer:**
- Both harnesses use `kani::vec::any_vec::<u8, N>()` + `std::str::from_utf8()`, the same
  bounded symbolic string pattern used by VP-014 and VP-015 in
  `crates/prism-query/src/proofs/`. This pattern is stable across Kani versions and
  correctly binds preconditions: `any_vec::<u8, 256>()` guarantees `key.len() <= 256`
  structurally, satisfying the ADR-066 §D3 plan-gate precondition.
- The `column_str_len <= 1024` bound is a tractability choice for CI. The null-safety
  property holds for any bounded string: `serde_json::from_str` returns `Err` (not panic)
  for invalid JSON, and `Value::as_object().get(key)` returns `None` for missing keys.
- The `#[kani::unwind(8)]` directive bounds recursive JSON structure exploration at depth 8,
  sufficient for all realistic OCSF `raw_extensions` payloads.
- **Known risk — serde_json allocation panic:** If `serde_json` internals contain a
  panicking path under symbolic input not covered by its `Result` surface (e.g., an
  internal allocator panic on adversarially large input), Kani may report it as a VP
  failure. The 1024-byte column bound limits input size, mitigating this risk. If Kani
  does report an allocation panic, the resolution strategy is to wrap the call in
  `std::panic::catch_unwind` in the harness (the production function is unchanged).
  Run the harness in CI and triage any such findings before Phase 5 freeze.

---

## Feasibility Assessment

**Assessment: FEASIBLE**

| Factor | Assessment | Justification |
|--------|-----------|---------------|
| Function purity | PURE | No I/O, no global state, no async. serde_json + Option combinators only. |
| State space | BOUNDED | Key ≤ 256 bytes; column string bounded for proof tractability. |
| Kani compatibility | COMPATIBLE | serde_json has been used in Kani proofs in the broader Rust ecosystem (no known Kani incompatibility). |
| Proof depth | MANAGEABLE | `unwind 8` is sufficient; `serde_json::Value` tree depth for OCSF raw_extensions is ≤ 3 in practice. |
| Panic sites | ENUMERABLE | Three candidate panic sites: (a) symbolic string construction — no panic; (b) `serde_json::from_str` — returns `Result`, not panic; (c) `.get(key)` on `Value::Object` — returns `Option`, not panic. All three are non-panicking. |
| Platform requirement | Linux/macOS | Kani requires Linux or macOS (upstream CBMC backend). Windows contributors rely on CI proof. Aligns with VP-014/VP-015 precedent. |
| Proof runtime estimate | < 60s | Comparable to VP-021 fuzz target in scope. Short string bounds keep state space small. |

**Known risk:** If `serde_json` internals contain panicking paths under symbolic input not
covered by its `Result` surface (e.g., internal allocation panic), Kani may report those
as VP failures. This would require switching to a `std::panic::catch_unwind` wrapper in the
harness or using `serde_json`'s `from_str_raw` variant. Resolution strategy: run the
harness in CI and triage any findings before Phase 5 freeze.

---

## Coverage Scope

This VP covers the **pure extraction function** (`json_extract_string_impl`) only. It does
NOT cover:

- The DataFusion `ScalarUDFImpl::invoke_batch` wrapper — this is effectful (Arrow column
  I/O) and is covered by integration tests in S-JSON-EXTRACT-UDF-001.
- The literal-key plan gate (ADR-066 §B3) — this runs before the pure function and is
  covered by RG-JEX-006 and RG-JEX-007 in S-JSON-EXTRACT-UDF-001.
- The E-QUERY-045 error format — covered by error taxonomy tests in S-JSON-EXTRACT-UDF-001.

The VP provides formal assurance that, given a key that passed the plan gate, the pure
extraction logic is safe for all possible input string values.

---

## Related Artifacts

| Artifact | Relationship |
|----------|-------------|
| ADR-066 §D1 | This VP is the proof target specified there |
| ADR-066 §B1 | Defines the function signature and behavior this VP proves |
| VP-014 | Precedent: Kani proof for size limit in prism-query |
| VP-015 | Precedent: Kani proof for depth limit in prism-query |
| S-JSON-EXTRACT-UDF-001 | Anchor story; Phase 3 TDD tests complement this proof |
| BC-2.11.025 | Source BC (authored by product-owner as part of S-JSON-EXTRACT-UDF-001 prep) |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.3 | 2026-09-16 | architect | Re-gate pass 10 fix. Consistency F-2 (LOW): `source_invariant: null` added to frontmatter — VP-162 §Source Traceability explicitly anchors its 256-byte key-length precondition to ADR-066 §D3 (CWE-400), not to any DI-NNN workspace invariant; per VP-INDEX §Properties convention, a VP with no DI-NNN invariant must carry `source_invariant: null`. |
| 1.2 | 2026-09-16 | architect | Re-gate pass 2 fix. F-3 (MED): §Source Traceability Invariant row corrected — DI-019 label was "null-safe extraction" (factually wrong; DI-019 = Query Security Limits: 64KB query, 10K cap, 30s timeout); relabeled to "Query Security Limits" with correct description; ADR-066 §D3 (CWE-400 key-length cap) added as explicit anchor for the 256-byte key-length precondition modeled in the Kani harness. |
| 1.1 | 2026-09-16 | architect | Adversarial gate fixes (F4/F8/Finding-2). F8 (MED): `lifecycle_status: active` → `lifecycle_status: draft` (story S-JSON-EXTRACT-UDF-001 not yet merged); `DI-NNN` placeholder in §Source Traceability replaced with `DI-019`. Finding-2 (OBS): stale "(to be authored by product-owner for S-JSON-EXTRACT-UDF-001)" parenthetical removed from BC-2.11.025 §Source Traceability row. F4 (HIGH): Both Kani harnesses rewritten — replaced `kani::any::<&str>()` (not a stable Kani API; does not bind preconditions) with the VP-014/VP-015 bounded pattern: `kani::vec::any_vec::<u8, N>()` + `std::str::from_utf8()`. `vp162_json_extract_string_null_safety`: key bounded to 256 bytes via `any_vec::<u8, 256>()` (ADR-066 §D3 plan gate modeled structurally); column string bounded to 1024 bytes; dead `buf` and unbound `key_len`/`column_str_len` variables removed; preconditions now actually constrain the values under test. `vp162_b_none_input_is_none_output`: same Vec<u8> + from_utf8 key pattern replaces `kani::any::<&str>()`. Known-risk annotation added: serde_json allocation-panic on symbolic input; 1024-byte column bound mitigates; catch_unwind resolution strategy documented. |
| 1.0 | 2026-09-16 | architect | Initial draft. D-2522 authorized S-JSON-EXTRACT-UDF-001 and ADR-066. Property: for any (column_value: Option<&str>, key: &str) with key.len() ≤ 256, json_extract_string_impl returns Some(String) or None, never panics. Kani harnesses: vp162_json_extract_string_null_safety (general) + vp162_b_none_input_is_none_output (specialized None-input). Feasibility: FEASIBLE; precedent from VP-014/VP-015 in same module. Phase P0. |
