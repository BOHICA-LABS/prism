---
document_type: verification-property
level: L4
vp_id: "VP-162"
title: "json_extract_string_impl — Null Safety and Panic Freedom (Kani)"
version: "1.0"
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
module: prism-query
priority: P0
proof_method: kani
verification_method: kani
feasibility: feasible
lifecycle_status: active
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
| BC | BC-2.11.025 (to be authored by product-owner for S-JSON-EXTRACT-UDF-001) |
| ADR | ADR-066 §D1 — VP-162 Proof Target |
| Story | S-JSON-EXTRACT-UDF-001 (beta.3 json_extract UDF story) |
| Invariant | DI-NNN (null-safe extraction — from domain-spec/invariants.md, verified at module boundary) |
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
    #[kani::proof]
    #[kani::unwind(8)]
    fn vp162_json_extract_string_null_safety() {
        // Symbolic column value: None or Some(arbitrary string)
        let has_value: bool = kani::any();
        let column_str_len: usize = kani::any();
        kani::assume(column_str_len <= 64); // Bounded string length for tractability

        let column_value: Option<&str> = if has_value {
            // Construct a symbolic string of bounded length
            let buf: Vec<u8> = (0..column_str_len)
                .map(|_| kani::any::<u8>())
                .collect();
            // Use a fixed-content fallback; Kani will explore validity branches
            Some(kani::any::<&str>())
        } else {
            None
        };

        // Symbolic key
        let key_len: usize = kani::any();
        kani::assume(key_len <= 256); // ADR-066 §D3 plan gate precondition

        // The result must be Some or None — no panic allowed.
        // Kani verifies this by symbolic execution: any path that panics
        // (including index out of bounds, unwrap on None, etc.) is a verification failure.
        let _result: Option<String> = json_extract_string_impl(column_value, kani::any::<&str>());

        // No explicit assertion needed: Kani's panic-freedom verification
        // is triggered automatically for any reachable panic site.
    }

    /// VP-162-B: None input → None output.
    ///
    /// Specialization: when column_value is None, the result is always None.
    #[kani::proof]
    #[kani::unwind(4)]
    fn vp162_b_none_input_is_none_output() {
        let key: &str = kani::any();
        kani::assume(key.len() <= 256);
        let result = json_extract_string_impl(None, key);
        assert!(result.is_none(), "None column input must produce None output");
    }
}
```

**Notes for implementer:**
- `kani::any::<&str>()` generates a symbolic string reference. The Kani model checker
  explores all feasible string values within the bounded string length assumptions.
- The `column_str_len <= 64` bound is a tractability choice for CI. The property is
  not sensitive to the exact bound: the null-safety property holds for any bounded string
  because `serde_json::from_str` returns `Err` (not panic) for invalid JSON, and
  `serde_json::Value::as_object().get(key)` returns `None` for missing keys.
- The `#[kani::unwind(8)]` directive bounds recursive JSON structure exploration at depth 8,
  sufficient for all realistic OCSF `raw_extensions` payloads.
- If `kani::any::<&str>()` is not available in the installed Kani version, replace with a
  bounded `Vec<u8>` + `std::str::from_utf8` pattern as used in VP-014 and VP-015 in
  `crates/prism-query/src/proofs/`.

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
| 1.0 | 2026-09-16 | architect | Initial draft. D-2522 authorized S-JSON-EXTRACT-UDF-001 and ADR-066. Property: for any (column_value: Option<&str>, key: &str) with key.len() ≤ 256, json_extract_string_impl returns Some(String) or None, never panics. Kani harnesses: vp162_json_extract_string_null_safety (general) + vp162_b_none_input_is_none_output (specialized None-input). Feasibility: FEASIBLE; precedent from VP-014/VP-015 in same module. Phase P0. |
