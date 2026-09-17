//! `json_extract_string` DataFusion ScalarUDF registration (ADR-066 §E).
//!
//! Provides the `json_extract_string(column, 'key')` scalar UDF that extracts a
//! top-level string value from an Arrow `Utf8` JSON column by a **literal** key.
//!
//! # Dead-Path Closure (ADR-066 §H)
//!
//! Three sites in the query codebase reference `json_extract_string`:
//! - `ast.rs`: `ScalarFunc::JsonExtractString` variant
//! - `sql_parser.rs`: `json_extract_string(col, 'key')` parsed into `ScalarFunc::JsonExtractString`
//! - `pipe_sql_emitter.rs`: `ScalarFunc::JsonExtractString` lowered to DataFusion SQL
//!
//! Without this module, DataFusion receives an unregistered function call and
//! returns an opaque runtime error (not structured under E-QUERY-NNN). This module
//! registers the UDF so those AST sites resolve to real execution.
//!
//! # Architecture Compliance
//! - UDF registered under exact name `"json_extract_string"` with types `(Utf8, Utf8)` → `Utf8`
//!   nullable and `Volatility::Immutable` (ADR-066 §E; BC-2.11.025 postcondition §Registration).
//! - `json_extract_string_impl` is a pure function with no DataFusion/Arrow types in its
//!   signature — required for the VP-162 Kani proof harness (ADR-066 §D1).
//! - `invoke_with_args` implements DataFusion 53.1 `ScalarUDFImpl` interface;
//!   `invoke_batch` (deprecated at DataFusion 46.0) is NEVER used.
//! - No `async` code — UDF is synchronous per ADR-066 §B1 + `Volatility::Immutable`.
//! - No `unsafe` blocks.
//!
//! Story: S-JSON-EXTRACT-UDF-001

use std::any::Any;
use std::hash::{Hash, Hasher};

use datafusion::arrow::datatypes::DataType;
use datafusion::error::Result as DataFusionResult;
use datafusion::logical_expr::{
    ColumnarValue, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl, Signature, TypeSignature,
    Volatility,
};

// ---------------------------------------------------------------------------
// JsonExtractStringUdf — ScalarUDFImpl wrapper
// ---------------------------------------------------------------------------

/// DataFusion scalar UDF implementation for `json_extract_string(column, 'key')`.
///
/// Registered at `QueryEngine::new` via `ctx.register_udf(json_extract_string_udf())`.
/// Extracts the top-level string value at `key` from a JSON `Utf8` Arrow column.
///
/// Input signature: `(Utf8, Utf8)` — first arg is the JSON column, second is the
/// literal key (validated at plan time by `check_json_extract_key_literal`).
/// Output type: `Utf8` (nullable). Volatility: `Immutable`.
///
/// Per-row logic is delegated to `json_extract_string_impl` (pure function, VP-162
/// Kani proof target). `invoke_with_args` iterates the Arrow `StringArray` and
/// calls `json_extract_string_impl` per row.
///
/// `#[non_exhaustive]`: forward-compat per CLAUDE.md §Conventions.
///
/// ADR-066 §B1 + §E; BC-2.11.025 postcondition §Registration.
/// Story: S-JSON-EXTRACT-UDF-001.
#[non_exhaustive]
#[derive(Debug)]
pub struct JsonExtractStringUdf {
    /// DataFusion function signature: `(Utf8, Utf8)` → `Utf8` (nullable),
    /// `Volatility::Immutable` per ADR-066 §E.
    signature: Signature,
}

impl JsonExtractStringUdf {
    /// Construct a new `JsonExtractStringUdf` with the canonical ADR-066 §E signature.
    ///
    /// WIRING-EXEMPT: simple constructor wiring the correct DataFusion signature.
    /// No behavioral logic — the real work happens in `invoke_with_args`.
    pub fn new() -> Self {
        Self {
            signature: Signature::new(
                TypeSignature::Exact(vec![DataType::Utf8, DataType::Utf8]),
                Volatility::Immutable,
            ),
        }
    }
}

// WIRING-EXEMPT: Default delegates to new() — required by clippy::new_without_default.
impl Default for JsonExtractStringUdf {
    fn default() -> Self {
        Self::new()
    }
}

// ScalarUDFImpl requires DynEq + DynHash (auto-impl'd for types that implement Eq + Hash + Any).
// We key equality and hashing on the UDF name, which is globally unique within a
// SessionContext (DataFusion enforces uniqueness at registration time).
// Follows the same pattern as InfusionAsyncUdf in infusion_udf.rs.

impl PartialEq for JsonExtractStringUdf {
    /// WIRING-EXEMPT: keyed on name per DynEq contract (1 line, no logic).
    fn eq(&self, _other: &Self) -> bool {
        // All instances have the same name "json_extract_string" — they are equal.
        true
    }
}

impl Eq for JsonExtractStringUdf {}

impl Hash for JsonExtractStringUdf {
    /// WIRING-EXEMPT: keyed on name per DynHash contract (1 line, no logic).
    fn hash<H: Hasher>(&self, state: &mut H) {
        "json_extract_string".hash(state);
    }
}

impl ScalarUDFImpl for JsonExtractStringUdf {
    /// WIRING-EXEMPT: required by `ScalarUDFImpl`; delegates to `self` (1 line, no logic).
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// WIRING-EXEMPT: returns the canonical UDF name per ADR-066 §E (1 line, no logic).
    fn name(&self) -> &str {
        "json_extract_string"
    }

    /// WIRING-EXEMPT: returns stored signature field (1 line, no logic).
    fn signature(&self) -> &Signature {
        &self.signature
    }

    /// WIRING-EXEMPT: returns `DataType::Utf8` (nullable) per ADR-066 §E (1 line, no logic).
    fn return_type(&self, _arg_types: &[DataType]) -> DataFusionResult<DataType> {
        Ok(DataType::Utf8)
    }

    /// Per-row JSON extraction — the behavioral core (AC-001..AC-010).
    ///
    /// Iterates the Arrow `StringArray` for the JSON column argument and calls
    /// `json_extract_string_impl` per row. Returns a `StringArray` with extracted
    /// values or SQL NULL for each row where extraction returns `None`.
    ///
    /// DataFusion 53.1 `ScalarUDFImpl` interface: `invoke_batch` (deprecated at
    /// DataFusion 46.0) is NEVER used — this method is the sole execution path.
    ///
    /// BC-5.38.005 self-check: "If I include this real implementation, will the test
    /// for this function pass trivially without any implementer work?" — YES. This
    /// method is the behavioral core that RG-JEX-001..010 test. MUST remain `todo!()`.
    fn invoke_with_args(&self, _args: ScalarFunctionArgs) -> DataFusionResult<ColumnarValue> {
        todo!(
            "json_extract_string UDF invoke_with_args not yet implemented — \
             S-JSON-EXTRACT-UDF-001 T-05 (json_extract_string_impl) + T-07 (registration)"
        )
    }
}

// ---------------------------------------------------------------------------
// Factory constructor
// ---------------------------------------------------------------------------

/// Construct the `json_extract_string` DataFusion `ScalarUDF` for registration.
///
/// Returns a `ScalarUDF` wrapping `JsonExtractStringUdf`. Wire into the
/// `SessionContext` at engine construction:
/// ```ignore
/// ctx.register_udf(json_extract_string_udf());
/// ```
///
/// The UDF is registered under name `"json_extract_string"` with input types
/// `(Utf8, Utf8)` → `Utf8` (nullable) and `Volatility::Immutable` per ADR-066 §E.
///
/// BC-5.38.005 self-check: "If I include this real implementation, will the test
/// for this function pass trivially without any implementer work?" — YES (RG-JEX-001
/// requires the UDF to be registered AND execute; implementing the factory without
/// the registration call in `engine.rs` is still incomplete, but returning a
/// non-todo factory would partially satisfy the test once T-07 wires it in).
/// MUST remain `todo!()` to prevent premature partial green.
///
/// S-JSON-EXTRACT-UDF-001 AC-001; ADR-066 §E; BC-2.11.025 postcondition §Registration.
pub fn json_extract_string_udf() -> ScalarUDF {
    todo!(
        "json_extract_string_udf factory not yet implemented — \
         S-JSON-EXTRACT-UDF-001 T-07 (register_udf wiring in engine.rs)"
    )
}

// ---------------------------------------------------------------------------
// Pure extraction function — VP-162 Kani proof target
// ---------------------------------------------------------------------------

/// Pure JSON string extraction — the Kani-provable core (VP-162).
// The `dead_code` lint fires at stub stage because `invoke_with_args` is `todo!()`.
// This function will be called from `invoke_with_args` (S-JSON-EXTRACT-UDF-001 T-05)
// and from the VP-162 Kani proof harness (T-06). The allow is a stub-stage exception.
#[allow(dead_code)]
///
/// Extracts the value at `key` from the JSON object string in `json_col`.
///
/// ## Null-safety contract (VP-162 invariant 2)
///
/// `None` input always produces `None` output. No unwrap, no panic, no I/O.
///
/// ## Return value semantics (ADR-066 §B1 steps 1–7)
///
/// Returns `None` for:
/// - `None` input (`json_col` is an Arrow null, AC-004)
/// - `serde_json::from_str` parse failure — invalid JSON (AC-009)
/// - JSON value is not an object (array, string, number, boolean, null at root, AC-005)
/// - Key absent from the JSON object (AC-003)
/// - `serde_json::Value::Null` at the key — JSON null is treated as absent (AC-002)
///
/// Returns `Some(s.clone())` for `Value::String(s)` at key (AC-001).
/// Returns `Some(value.to_string())` for non-string, non-null values (AC-008).
///
/// Dot-in-key literal is treated as a top-level key name, NOT nested JSONPath
/// (AC-010; ADR-066 §D4; nested path is `S-JSON-EXTRACT-NESTED-001` post-beta.3).
///
/// ## Architecture constraint (ADR-066 §D1 + VP-162 §Proof Target)
///
/// MUST have no DataFusion or Arrow types in its signature. The Kani proof harness
/// (`proofs/vp162_json_extract_null_safety.rs`) calls this function directly.
///
/// BC-5.38.005 self-check: "If I include this real implementation, will the test
/// for this function pass trivially without any implementer work?" — YES (most
/// RG-JEX tests depend on this pure function working correctly). MUST be `todo!()`.
///
/// S-JSON-EXTRACT-UDF-001 AC-001..AC-010; ADR-066 §B1; VP-162 invariant 1 + 2.
pub(crate) fn json_extract_string_impl(_json_col: Option<&str>, _key: &str) -> Option<String> {
    todo!(
        "json_extract_string_impl pure function not yet implemented — \
         S-JSON-EXTRACT-UDF-001 T-05 (ADR-066 §B1 steps 1–7)"
    )
}
