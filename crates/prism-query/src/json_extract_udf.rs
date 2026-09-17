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
use std::sync::Arc;

use datafusion::arrow::array::{Array, StringArray};
use datafusion::arrow::datatypes::DataType;
use datafusion::common::ScalarValue;
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
    /// ADR-066 §B1; BC-2.11.025 postcondition §Execution.
    /// S-JSON-EXTRACT-UDF-001 AC-001..AC-010.
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> DataFusionResult<ColumnarValue> {
        // Step 1: Extract the literal key from args[1].
        // After plan-gate validation (check_json_extract_key_literal), args[1] is always
        // a Scalar::Utf8 — the plan gate rejects non-literal keys (E-QUERY-045).
        // We still handle edge cases defensively (NULL key → all-NULL output).
        let key_opt: Option<String> = match args.args.get(1) {
            Some(ColumnarValue::Scalar(ScalarValue::Utf8(opt))) => opt.clone(),
            Some(ColumnarValue::Scalar(ScalarValue::LargeUtf8(opt))) => opt.clone(),
            Some(ColumnarValue::Scalar(ScalarValue::Null)) | None => None,
            Some(other) => {
                return Err(datafusion::error::DataFusionError::Execution(format!(
                    "json_extract_string: key argument must be a Utf8 scalar, got {:?}",
                    other.data_type()
                )));
            }
        };
        let key = match key_opt {
            Some(k) => k,
            None => {
                // NULL key → all-NULL output (null-propagating; VP-162 invariant 2).
                let nulls: Vec<Option<&str>> = vec![None; args.number_rows];
                return Ok(ColumnarValue::Array(Arc::new(StringArray::from(nulls))));
            }
        };

        // Step 2: Iterate the JSON column (args[0]) and apply per-row extraction.
        let json_col = args.args.first().ok_or_else(|| {
            datafusion::error::DataFusionError::Execution(
                "json_extract_string: expected 2 arguments, got 0".to_string(),
            )
        })?;

        match json_col {
            ColumnarValue::Array(col_arr) => {
                // Column input: iterate every row of the StringArray.
                let str_arr = col_arr
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .ok_or_else(|| {
                        datafusion::error::DataFusionError::Execution(format!(
                            "json_extract_string: JSON column must be Utf8, got {:?}",
                            col_arr.data_type()
                        ))
                    })?;

                let results: Vec<Option<String>> = (0..str_arr.len())
                    .map(|i| {
                        if str_arr.is_null(i) {
                            None // Arrow-null row → SQL NULL (AC-004 / VP-162 invariant 2)
                        } else {
                            json_extract_string_impl(Some(str_arr.value(i)), &key)
                        }
                    })
                    .collect();

                // Convert Vec<Option<String>> → StringArray (nulls become Arrow nulls).
                let out_arr: StringArray = results
                    .iter()
                    .map(|opt| opt.as_deref())
                    .collect::<Vec<Option<&str>>>()
                    .into();

                Ok(ColumnarValue::Array(Arc::new(out_arr)))
            }
            ColumnarValue::Scalar(ScalarValue::Utf8(opt)) => {
                // Scalar input (e.g., constant expression) — single extraction.
                let extracted = json_extract_string_impl(opt.as_deref(), &key);
                Ok(ColumnarValue::Scalar(ScalarValue::Utf8(extracted)))
            }
            ColumnarValue::Scalar(ScalarValue::Null) => {
                // Scalar NULL input → NULL output (AC-004 / VP-162 invariant 2).
                Ok(ColumnarValue::Scalar(ScalarValue::Utf8(None)))
            }
            ColumnarValue::Scalar(other) => {
                Err(datafusion::error::DataFusionError::Execution(format!(
                    "json_extract_string: JSON column must be Utf8 or Null, got {:?}",
                    other.data_type()
                )))
            }
        }
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
    ScalarUDF::from(JsonExtractStringUdf::new())
}

// ---------------------------------------------------------------------------
// Pure extraction function — VP-162 Kani proof target
// ---------------------------------------------------------------------------

/// Pure JSON string extraction — the Kani-provable core (VP-162).
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
/// - `None` input (`json_col` is an Arrow null, AC-004)                  — step 1
/// - `serde_json::from_str` parse failure — invalid JSON (AC-009)        — step 2
/// - JSON value is not an object (array, string, number, etc., AC-005)   — step 3
/// - Key absent from the JSON object (AC-003)                            — step 4
/// - `serde_json::Value::Null` at the key — JSON null = absent (AC-002)  — step 5
///
/// Returns `Some(s.clone())` for `Value::String(s)` at key (AC-001)     — step 6.
/// Returns `Some(value.to_string())` for non-string, non-null (AC-008)   — step 7.
///
/// Dot-in-key literal is treated as a top-level key name, NOT nested JSONPath
/// (AC-010; ADR-066 §D4; nested path reserved for `S-JSON-EXTRACT-NESTED-001`).
///
/// ## Architecture constraint (ADR-066 §D1 + VP-162 §Proof Target)
///
/// MUST have no DataFusion or Arrow types in its signature. The Kani proof harness
/// (`proofs/vp162_json_extract_null_safety.rs`) calls this function directly.
///
/// S-JSON-EXTRACT-UDF-001 AC-001..AC-010; ADR-066 §B1; VP-162 invariant 1 + 2.
pub(crate) fn json_extract_string_impl(json_col: Option<&str>, key: &str) -> Option<String> {
    let json_str = json_col?; // step 1: None input → None output (AC-004; VP-162 invariant 2)
    let value: serde_json::Value = serde_json::from_str(json_str).ok()?; // step 2: parse failure → None (AC-009)
    let obj = value.as_object()?; // step 3: non-object JSON root → None (AC-005)
    let val = obj.get(key)?; // step 4: key absent → None (AC-003)
    if val.is_null() {
        return None; // step 5: JSON null at key → SQL NULL (AC-002)
    }
    if let serde_json::Value::String(s) = val {
        return Some(s.clone()); // step 6: string value → clone (AC-001)
    }
    Some(val.to_string()) // step 7: non-string, non-null → coerce via to_string (AC-008)
}
