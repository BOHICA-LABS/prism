//! VP-162: `json_extract_string_impl` null-safety property (Kani formal verification).
//!
//! **Property:** `json_extract_string_impl` never panics and satisfies the
//! null-safety contract:
//! - Invariant 1: no panic on any `Option<&str>` × `&str` input (panic-free)
//! - Invariant 2: `None` input always produces `None` output (null-propagating)
//!
//! ## Proof structure
//!
//! Two Kani harnesses, each covering one invariant:
//! - `vp162_json_extract_string_null_safety` — exercises the full function with
//!   symbolic input strings; verifies no panic occurs across all bounded inputs.
//! - `vp162_b_none_input_is_none_output` — targeted proof that `None` input
//!   always yields `None` output regardless of key value.
//!
//! Both harnesses are `#[cfg(kani)]` gated — zero effect on test/release builds.
//! Phase 5 (formal-verify) dispatches `cargo kani -p prism-query` to execute them.
//!
//! ## Harness file convention
//!
//! This file is authored in Phase 3 (T-06) alongside the `json_extract_string_impl`
//! implementation so the proof is ready for Phase 5 dispatch without a separate
//! story. The harness must compile under `cargo kani -p prism-query` before merge.
//!
//! VP-162 v1.4; ADR-066 §D1; BC-2.11.025 postcondition §Null column (AC-004).
//! Story: S-JSON-EXTRACT-UDF-001 T-06.

// Implementer note: VP-162 §Kani Proof Harness specifies
// `use super::super::json_extract_udf::json_extract_string_impl;` inside
// `mod vp162_proofs`. From a file at `src/proofs/<name>.rs`, `super::super`
// resolves to `crate::proofs`, not the crate root — `json_extract_udf` lives
// at the crate root. Using the absolute `crate::` path instead, matching the
// VP-014 / VP-015 import convention in this module.

#[cfg(kani)]
mod vp162_proofs {
    use crate::json_extract_udf::json_extract_string_impl;

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
        assert!(
            result.is_none(),
            "None column input must produce None output"
        );
    }
}
