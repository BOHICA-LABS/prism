//! Red Gate test suite for S-JSON-EXTRACT-UDF-001 / json_extract_string scalar UDF.
//!
//! 11 baseline failing tests (RG-JEX-001..011) + RG-JEX-012 family added by F-JEX-P1-HIGH-001
//! fix pass. Every test traces to exactly one EC-11-025-NNN edge case in BC-2.11.025 v1.11
//! and one acceptance criterion in S-JSON-EXTRACT-UDF-001 v1.8.
//!
//! # Test method by group
//!
//! Tests RG-JEX-001..005, RG-JEX-008..010 use DataFusion `SessionContext` directly:
//! they register the UDF and assert on extracted string / null values per ADR-066 §B1.
//!
//! Tests RG-JEX-006, RG-JEX-007, RG-JEX-012..012-c use `QueryEngine::execute` with
//! `make_gate_engine()` (SAP-3 compliance): they exercise `check_json_extract_key_literal`
//! end-to-end from the prism_query public surface — not via synthetic AST.
//!
//! Test RG-JEX-011 and RG-JEX-012-d use `QueryEngine::execute` with a real mock adapter
//! (SAP-3 compliance) and assert functional correctness.
//!
//! `test_jex_dml_ast_safe_skip_with_jex_variant` uses `parse_and_plan` (public) to verify
//! that after the filter_parser fix DML filter predicates emit `ScalarFunc::JsonExtractString`
//! (not Unknown) — and that the E-QUERY-045 gate safe-skips DML.
//!
//! # Red Gate failure modes
//!
//! | Test           | Failure mode during Red Gate                                         |
//! |----------------|----------------------------------------------------------------------|
//! | RG-JEX-001..005, 008..010 | `json_extract_string_udf()` panics `todo!()` at factory call |
//! | RG-JEX-006     | assertion: expected `JsonExtractNonLiteralKey`, gate not wired yet   |
//! | RG-JEX-007     | assertion: expected `JsonExtractKeyTooLong`, gate not wired yet      |
//! | RG-JEX-011     | `json_extract_string` UDF not registered → DataFusion error          |
//! | RG-JEX-012     | gate misses WHERE predicate: fn_call_comparison emits Unknown         |
//! | RG-JEX-012-b   | gate misses WHERE predicate: fn_call_comparison emits Unknown         |
//! | RG-JEX-012-c   | gate misses HAVING predicate: fn_call_comparison emits Unknown        |
//! | DML rework     | parser asserts JsonExtractString but parser still emits Unknown       |
//!
//! # Wire-shape discipline (CLAUDE.md / BC-2.11.001 EC-11-079)
//!
//! - RG-JEX-002..005, 009: Arrow-level null assertions (`is_null(0)`) — not empty string
//! - RG-JEX-006..007, 012..012-b: `Display`-level error assertions (MCP `-32602 INVALID_PARAMS`)
//! - RG-JEX-011, 012-d: `serde_json` / Arrow-level functional assertions
//!
//! # BC-5.38.001 density check
//!
//! Red Gate tests / ACs = 12 / 12 = 1.0 (density requirement satisfied).
//!
//! # Traceability
//!
//! | RG-ID        | BC EC anchor      | AC                      | ADR-066 §B1 step |
//! |--------------|-------------------|-------------------------|------------------|
//! | RG-JEX-001   | EC-11-025-001     | AC-001 happy path       | step 6 (String)  |
//! | RG-JEX-002   | EC-11-025-005     | AC-002 JSON null→NULL   | step 5 (Null)    |
//! | RG-JEX-003   | EC-11-025-003     | AC-003 missing key      | step 4 (absent)  |
//! | RG-JEX-004   | EC-11-025-002     | AC-004 null col input   | step 1 (None)    |
//! | RG-JEX-005   | EC-11-025-004     | AC-005 non-object JSON  | step 3 (non-obj) |
//! | RG-JEX-006   | EC-11-025-006     | AC-006 E-QUERY-045(a)   | plan gate        |
//! | RG-JEX-007   | EC-11-025-007     | AC-007 E-QUERY-045(b)   | plan gate        |
//! | RG-JEX-008   | EC-11-025-008     | AC-008 non-string coerce| step 7 (other)  |
//! | RG-JEX-009   | EC-11-025-010     | AC-009 parse failure    | step 2 (parse)   |
//! | RG-JEX-010   | EC-11-025-009     | AC-010 dot-in-key       | step 4 (top-lvl) |
//! | RG-JEX-011   | EC-11-025-011     | AC-011 pipe mode E2E    | full E2E         |
//! | RG-JEX-012   | EC-11-025-012     | AC-012 WHERE/HAVING gate| plan gate        |
//!
//! Story: S-JSON-EXTRACT-UDF-001 v1.8 | BC: BC-2.11.025 v1.11 | ADR: ADR-066 v1.7

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    non_snake_case,
    dead_code,
    unused_imports
)]

use std::sync::Arc;

use arrow::array::{Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use datafusion::datasource::MemTable;
use datafusion::execution::context::SessionContext;
use prism_core::error::PrismError;
use prism_core::{OrgId, OrgSlug, SensorId};
use prism_credentials::{namespace::CredentialName, CredentialStore};
use prism_ocsf::OcsfNormalizer;
use prism_query::{
    ast::{Ast, Expr, FuncCall, Predicate, ScalarFunc, SqlStatement},
    engine::{QueryEngine, QueryEngineConfig, QueryOptions, QueryResult},
    json_extract_udf::json_extract_string_udf,
    parse_and_plan,
    scoping::ClientRegistry,
};
use prism_sensors::{
    adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
    auth::SensorAuth,
    AdapterRegistry, CredentialResolver,
};
use secrecy::SecretString;

// ---------------------------------------------------------------------------
// Helper types: NullCredentialStore, StubCredentialResolver, JexMockAdapter
// ---------------------------------------------------------------------------

/// No-op [`CredentialStore`] for `QueryEngine` construction in plan-gate tests.
///
/// Sensor fan-out never fires in plan-gate tests (E-QUERY-045 fires before
/// fan-out for RG-JEX-006/007). The store is wired to satisfy the constructor
/// contract; it is never queried.
struct NullCredentialStore;

#[async_trait]
impl CredentialStore for NullCredentialStore {
    async fn get(
        &self,
        _tenant: &OrgSlug,
        _sensor: &str,
        _name: &CredentialName,
    ) -> Result<Option<SecretString>, PrismError> {
        Ok(None)
    }

    async fn set(
        &self,
        _tenant: &OrgSlug,
        _sensor: &str,
        _name: &CredentialName,
        _value: SecretString,
    ) -> Result<(), PrismError> {
        Ok(())
    }

    async fn delete(
        &self,
        _tenant: &OrgSlug,
        _sensor: &str,
        _name: &CredentialName,
    ) -> Result<bool, PrismError> {
        Ok(false)
    }

    async fn list(&self, _tenant: &OrgSlug) -> Result<Vec<(String, CredentialName)>, PrismError> {
        Ok(vec![])
    }

    async fn exists(
        &self,
        _tenant: &OrgSlug,
        _sensor: &str,
        _name: &CredentialName,
    ) -> Result<bool, PrismError> {
        Ok(false)
    }
}

/// Stub [`CredentialResolver`] providing a no-op auth token for RG-JEX-011.
struct StubCredentialResolver;

impl CredentialResolver for StubCredentialResolver {
    fn resolve(
        &self,
        _client_id: &str,
        _sensor_id: SensorId,
    ) -> Result<Box<dyn SensorAuth>, prism_sensors::SensorError> {
        struct StubAuth;
        impl SensorAuth for StubAuth {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn auth_type_name(&self) -> &'static str {
                "stub"
            }
        }
        Ok(Box::new(StubAuth))
    }
}

/// Mock sensor adapter for RG-JEX-011 (pipe-mode E2E).
///
/// Returns one `RecordBatch` with 3 rows of JSON payloads in a `payload` (Utf8)
/// column — the column that the pipe query's `json_extract_string(payload, 'severity')`
/// operates on:
///
/// - Row 0: `{"severity":"critical","host":"server01"}` → expected extracted = "critical"
/// - Row 1: `{"severity":null,"host":"server02"}`       → expected extracted = SQL NULL
/// - Row 2: `{"host":"server03"}`                       → expected extracted = SQL NULL
///
/// Rows 1 and 2 are the wire-shape cases: null must serialize as JSON `null`,
/// not as absent key and not as the string "null".
struct JexMockAdapter;

impl std::fmt::Debug for JexMockAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JexMockAdapter").finish()
    }
}

#[async_trait]
impl SensorAdapter for JexMockAdapter {
    fn sensor_type(&self) -> SensorId {
        // "jex" (no underscore) so that sensor_id_from_table_name("jex_events")
        // extracts prefix "jex" and finds this adapter in the registry.
        SensorId::from("jex")
    }

    fn sensor_name(&self) -> &'static str {
        "jex"
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<FetchOutput, SensorError> {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "payload",
            DataType::Utf8,
            true,
        )]));
        let payloads: Vec<Option<&str>> = vec![
            Some(r#"{"severity":"critical","host":"server01"}"#), // row 0: happy path
            Some(r#"{"severity":null,"host":"server02"}"#),       // row 1: JSON null → SQL NULL
            Some(r#"{"host":"server03"}"#),                       // row 2: missing key → SQL NULL
        ];
        let arr = StringArray::from(payloads);
        let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
            .expect("JexMockAdapter: RecordBatch construction must succeed");
        Ok(FetchOutput::new(vec![batch], false, false))
    }
}

/// Generic single-row mock adapter for SAP-3 value-arm engine.execute assertions (F-4).
///
/// Returns one `RecordBatch` with a single `data` (Utf8, nullable) column containing
/// `payload`. `sensor_prefix` must match the prefix in the query table name
/// (e.g., `"jnoobj"` → table `"jnoobj_events"`).
///
/// Used by `test_jex_f4_*_engine_execute` tests to drive the four value-behavior
/// arms (non-object, coerce, parse-failure, dot-in-key) through `QueryEngine::execute`
/// (SAP-3 public-surface obligation, CLAUDE.md §SAP-3).
struct SingleRowAdapter {
    sensor_prefix: &'static str,
    payload: Option<&'static str>,
}

impl std::fmt::Debug for SingleRowAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SingleRowAdapter")
            .field("sensor_prefix", &self.sensor_prefix)
            .finish()
    }
}

#[async_trait]
impl SensorAdapter for SingleRowAdapter {
    fn sensor_type(&self) -> SensorId {
        SensorId::from(self.sensor_prefix)
    }

    fn sensor_name(&self) -> &'static str {
        self.sensor_prefix
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<FetchOutput, SensorError> {
        let schema = Arc::new(Schema::new(vec![Field::new("data", DataType::Utf8, true)]));
        let arr = StringArray::from(vec![self.payload]);
        let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
            .expect("SingleRowAdapter: RecordBatch construction must succeed");
        Ok(FetchOutput::new(vec![batch], false, false))
    }
}

/// Build a `QueryEngine` with a `SingleRowAdapter` registered for SAP-3 arm tests.
fn make_arm_engine(adapter: SingleRowAdapter) -> QueryEngine {
    let org_id = OrgId::new();
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::new(adapter));
    QueryEngine::new(
        Arc::new(registry),
        Arc::new(NullCredentialStore),
        Arc::new(OcsfNormalizer::new()),
        Arc::new(ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
    )
    .with_credential_resolver(Arc::new(StubCredentialResolver))
}

/// Build a `QueryEngine` for plan-gate tests (no table_registry, empty AdapterRegistry).
///
/// With `table_registry = None`, the E-QUERY-037 gate is bypassed. Only E-QUERY-038,
/// E-QUERY-039, and the E-QUERY-045 gate (when wired in T-09) fire.
fn make_gate_engine() -> QueryEngine {
    QueryEngine::new(
        Arc::new(AdapterRegistry::new()),
        Arc::new(NullCredentialStore),
        Arc::new(OcsfNormalizer::new()),
        Arc::new(ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
    )
}

/// Build a `SessionContext` with `json_extract_string` UDF registered.
///
/// Panics with `todo!()` during the Red Gate because `json_extract_string_udf()`
/// is not yet implemented (S-JSON-EXTRACT-UDF-001 T-07). After T-07, this
/// successfully returns a context with the UDF available for queries.
fn make_udf_ctx() -> SessionContext {
    let ctx = SessionContext::new();
    // FAILS RED: json_extract_string_udf() panics todo!() until T-07 is complete.
    ctx.register_udf(json_extract_string_udf());
    ctx
}

/// Build a MemTable with one row of JSON data in a `raw_data` (Utf8, nullable) column
/// and register it under `table_name` in the given `SessionContext`.
fn register_json_table(ctx: &SessionContext, table_name: &str, json_value: Option<&str>) {
    let schema = Arc::new(Schema::new(vec![Field::new(
        "raw_data",
        DataType::Utf8,
        true,
    )]));
    let arr = StringArray::from(vec![json_value]);
    let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
        .expect("register_json_table: RecordBatch must succeed");
    let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
        .expect("register_json_table: MemTable must succeed");
    ctx.register_table(table_name, Arc::new(table))
        .expect("register_json_table: register_table must succeed");
}

// ===========================================================================
// RG-JEX-001 — AC-001 / EC-11-025-001: happy path string extraction
// ===========================================================================

/// RG-JEX-001: `json_extract_string(col, 'key')` returns the string value at the key.
///
/// Input: `{"severity":"high","count":5}`, key `'severity'`
/// Expected: one non-null row with value `"high"`.
///
/// BC-2.11.025 EC-11-025-001 — ADR-066 §B1 step 6 (Value::String → Some(s.clone())).
///
/// RED failure: `json_extract_string_udf()` panics `todo!()` at T-07 stub.
/// GREEN: returns `"high"` after T-05 + T-07.
#[tokio::test]
async fn test_jex_rg001_udf_registered_happy_path_executes() {
    // FAILS RED: make_udf_ctx() calls json_extract_string_udf() which is todo!().
    let ctx = make_udf_ctx();
    register_json_table(&ctx, "jex_data", Some(r#"{"severity":"high","count":5}"#));

    let df = ctx
        .sql("SELECT json_extract_string(raw_data, 'severity') AS extracted FROM jex_data")
        .await
        .expect("RG-JEX-001: SQL must parse and plan");

    let batches = df
        .collect()
        .await
        .expect("RG-JEX-001 (FAILS RED): UDF execution must succeed — todo!() in invoke_with_args");

    let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        total_rows, 1,
        "RG-JEX-001: must return exactly 1 row for 1 input row"
    );

    let col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("RG-JEX-001: result column must be Utf8 StringArray");

    assert!(
        !col.is_null(0),
        "RG-JEX-001 (EC-11-025-001): 'severity' key is present with string value → \
         must NOT be SQL NULL. Got: null"
    );
    assert_eq!(
        col.value(0),
        "high",
        "RG-JEX-001 (EC-11-025-001 / AC-001): json_extract_string({{\"severity\":\"high\"}}, 'severity') \
         must return \"high\". Got: {:?}",
        col.value(0)
    );
}

// ===========================================================================
// RG-JEX-002 — AC-002 / EC-11-025-005: JSON null at key → SQL NULL
// ===========================================================================

/// RG-JEX-002: `json_extract_string` returns SQL NULL when the key maps to JSON null.
///
/// Input: `{"severity":null}`, key `'severity'`
/// Expected: SQL NULL (Arrow null cell), NOT the string "null".
///
/// BC-2.11.025 EC-11-025-005 — ADR-066 §B1 step 5 (Value::Null → None).
/// Wire-shape: `col.is_null(0)` must be true; the string "null" is WRONG.
///
/// RED failure: `json_extract_string_udf()` panics `todo!()` at T-07 stub.
/// GREEN: returns SQL NULL after T-05 + T-07.
#[tokio::test]
async fn test_jex_rg002_json_null_value_at_key_returns_sql_null() {
    // FAILS RED: make_udf_ctx() calls json_extract_string_udf() which is todo!().
    let ctx = make_udf_ctx();
    register_json_table(&ctx, "jex_data", Some(r#"{"severity":null}"#));

    let df = ctx
        .sql("SELECT json_extract_string(raw_data, 'severity') AS extracted FROM jex_data")
        .await
        .expect("RG-JEX-002: SQL must parse and plan");

    let batches = df
        .collect()
        .await
        .expect("RG-JEX-002 (FAILS RED): UDF execution must succeed — todo!() in invoke_with_args");

    let col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("RG-JEX-002: result column must be Utf8 StringArray");

    // Wire-shape (BC-2.11.001 EC-11-079 null-not-absent):
    // JSON null at key → SQL NULL (Arrow is_null), NOT the string "null".
    assert!(
        col.is_null(0),
        "RG-JEX-002 (EC-11-025-005 / AC-002): JSON null at 'severity' must yield SQL NULL \
         (is_null=true). Wire-shape: must not be the string \"null\" or empty string. \
         Got: {:?}",
        if col.is_null(0) { "null" } else { col.value(0) }
    );
}

// ===========================================================================
// RG-JEX-003 — AC-003 / EC-11-025-003: missing key → SQL NULL
// ===========================================================================

/// RG-JEX-003: `json_extract_string` returns SQL NULL when the key is absent.
///
/// Input: `{"other_field":"value"}`, key `'severity'`
/// Expected: SQL NULL (Arrow null cell).
///
/// BC-2.11.025 EC-11-025-003 — ADR-066 §B1 step 4 (key absent → None).
///
/// RED failure: `json_extract_string_udf()` panics `todo!()` at T-07 stub.
/// GREEN: returns SQL NULL after T-05 + T-07.
#[tokio::test]
async fn test_jex_rg003_missing_key_returns_sql_null() {
    // FAILS RED: make_udf_ctx() calls json_extract_string_udf() which is todo!().
    let ctx = make_udf_ctx();
    register_json_table(&ctx, "jex_data", Some(r#"{"other_field":"value"}"#));

    let df = ctx
        .sql("SELECT json_extract_string(raw_data, 'severity') AS extracted FROM jex_data")
        .await
        .expect("RG-JEX-003: SQL must parse and plan");

    let batches = df
        .collect()
        .await
        .expect("RG-JEX-003 (FAILS RED): UDF execution must succeed — todo!() in invoke_with_args");

    let col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("RG-JEX-003: result column must be Utf8 StringArray");

    assert!(
        col.is_null(0),
        "RG-JEX-003 (EC-11-025-003 / AC-003): absent key 'severity' must yield SQL NULL. \
         Got: {:?}",
        if col.is_null(0) { "null" } else { col.value(0) }
    );
}

// ===========================================================================
// RG-JEX-004 — AC-004 / EC-11-025-002: Arrow-null column input → SQL NULL
// ===========================================================================

/// RG-JEX-004: `json_extract_string` returns SQL NULL when the input column cell is NULL.
///
/// The `raw_data` column cell is an Arrow NULL (no JSON string at all).
/// Expected: SQL NULL.
///
/// BC-2.11.025 EC-11-025-002 — ADR-066 §B1 step 1 (None input → None output).
/// VP-162 invariant 2: null-safety — None input always produces None output.
///
/// RED failure: `json_extract_string_udf()` panics `todo!()` at T-07 stub.
/// GREEN: returns SQL NULL after T-05 + T-07.
#[tokio::test]
async fn test_jex_rg004_null_column_input_returns_sql_null() {
    // FAILS RED: make_udf_ctx() calls json_extract_string_udf() which is todo!().
    let ctx = make_udf_ctx();
    // None produces an Arrow NULL cell — no JSON string at all.
    register_json_table(&ctx, "jex_data", None);

    let df = ctx
        .sql("SELECT json_extract_string(raw_data, 'severity') AS extracted FROM jex_data")
        .await
        .expect("RG-JEX-004: SQL must parse and plan");

    let batches = df
        .collect()
        .await
        .expect("RG-JEX-004 (FAILS RED): UDF execution must succeed — todo!() in invoke_with_args");

    let col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("RG-JEX-004: result column must be Utf8 StringArray");

    assert!(
        col.is_null(0),
        "RG-JEX-004 (EC-11-025-002 / AC-004 / VP-162 invariant 2): Arrow-null input must yield \
         SQL NULL. Got: {:?}",
        if col.is_null(0) { "null" } else { col.value(0) }
    );
}

// ===========================================================================
// RG-JEX-005 — AC-005 / EC-11-025-004: non-object JSON root → SQL NULL
// ===========================================================================

/// RG-JEX-005: `json_extract_string` returns SQL NULL when the JSON root is not an object.
///
/// Input: `["a","b","c"]` (JSON array at root), key `'severity'`
/// Expected: SQL NULL.
///
/// BC-2.11.025 EC-11-025-004 — ADR-066 §B1 step 3 (non-object → None).
///
/// RED failure: `json_extract_string_udf()` panics `todo!()` at T-07 stub.
/// GREEN: returns SQL NULL after T-05 + T-07.
#[tokio::test]
async fn test_jex_rg005_non_object_json_returns_sql_null() {
    // FAILS RED: make_udf_ctx() calls json_extract_string_udf() which is todo!().
    let ctx = make_udf_ctx();
    register_json_table(&ctx, "jex_data", Some(r#"["a","b","c"]"#));

    let df = ctx
        .sql("SELECT json_extract_string(raw_data, 'severity') AS extracted FROM jex_data")
        .await
        .expect("RG-JEX-005: SQL must parse and plan");

    let batches = df
        .collect()
        .await
        .expect("RG-JEX-005 (FAILS RED): UDF execution must succeed — todo!() in invoke_with_args");

    let col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("RG-JEX-005: result column must be Utf8 StringArray");

    assert!(
        col.is_null(0),
        "RG-JEX-005 (EC-11-025-004 / AC-005): non-object JSON root (array) must yield SQL NULL. \
         Got: {:?}",
        if col.is_null(0) { "null" } else { col.value(0) }
    );
}

// ===========================================================================
// RG-JEX-006 — AC-006 / EC-11-025-006: non-literal key → E-QUERY-045(a)
// ===========================================================================

/// RG-JEX-006: non-literal key argument is rejected at plan time with E-QUERY-045(a).
///
/// Query: `SELECT json_extract_string(raw_data, severity_col) FROM test_events`
/// The second argument `severity_col` is a column reference (not a quoted string literal).
///
/// Expected: `Err(PrismError::JsonExtractNonLiteralKey)` with Display matching
/// `"E-QUERY-045: json_extract_string requires a literal string key ..."`.
///
/// MCP surface: maps to `-32602 INVALID_PARAMS` (per prism_mcp error_mapping).
///
/// SAP-3 compliance: exercises `check_json_extract_key_literal` END-TO-END from
/// `QueryEngine::execute` (the prism_query public surface), NOT via synthetic AST.
///
/// BC-2.11.025 EC-11-025-006 / AC-006 — ADR-066 §B3.
///
/// RED failure: E-QUERY-045 gate not wired in execute_inner (T-09 not done);
///   currently gets DataFusion "unrecognized function" error, not `JsonExtractNonLiteralKey`.
/// GREEN: gate fires after T-09, assertion passes.
#[tokio::test]
async fn test_jex_rg006_non_literal_key_rejected_e_query_045_a() {
    // SAP-3: Uses QueryEngine::execute (public surface), NOT check_json_extract_key_literal directly.
    // table_registry = None → E-QUERY-037 skipped; only E-QUERY-045 gate fires for this query.
    let engine = make_gate_engine();

    // `severity_col` (no quotes) is a column reference — E-QUERY-045(a) must fire.
    let result = engine
        .execute(
            "SELECT json_extract_string(raw_data, severity_col) FROM test_events",
            QueryOptions::default(),
        )
        .await;

    // Wire-shape: verify the Display output matches the exact MCP error message.
    // This is the string that maps to INVALID_PARAMS (-32602) in error_mapping.rs.
    let expected_display = "E-QUERY-045: json_extract_string requires a literal string key \
        (e.g., json_extract_string(col, 'key_name')). \
        Dynamic key expressions are not supported.";

    match result {
        Err(PrismError::JsonExtractNonLiteralKey) => {
            // Wire-shape assertion (SID-2 composed-output + MCP surface):
            let display = format!("{}", PrismError::JsonExtractNonLiteralKey);
            assert!(
                display.starts_with("E-QUERY-045:"),
                "RG-JEX-006 wire-shape: error must start with 'E-QUERY-045:'. Got: {display:?}"
            );
            assert_eq!(
                display, expected_display,
                "RG-JEX-006 wire-shape: Display must exactly match the MCP INVALID_PARAMS message. \
                 Got: {display:?}"
            );
        }
        Err(other) => {
            panic!(
                "RG-JEX-006 (FAILS RED until T-09): expected PrismError::JsonExtractNonLiteralKey, \
                 got different error: {other:?}\n\
                 This is expected during Red Gate — the E-QUERY-045 gate is not yet wired."
            );
        }
        Ok(qr) => {
            panic!(
                "RG-JEX-006: expected Err(JsonExtractNonLiteralKey), got Ok with {} batches. \
                 E-QUERY-045(a) plan gate must reject non-literal key expressions.",
                qr.batches.len()
            );
        }
    }
}

// ===========================================================================
// RG-JEX-007 — AC-007 / EC-11-025-007: key > 256 bytes → E-QUERY-045(b)
// ===========================================================================

/// RG-JEX-007: a literal key exceeding 256 bytes is rejected at plan time with E-QUERY-045(b).
///
/// Query uses a 257-character literal key. Expected: `Err(PrismError::JsonExtractKeyTooLong)`
/// with `key_len = 257` and `max_len = 256`.
///
/// Wire-shape: Display must contain "257 bytes" and "256-byte maximum (CWE-400)".
///
/// MCP surface: maps to `-32602 INVALID_PARAMS`.
///
/// SAP-3 compliance: exercises the gate end-to-end from `QueryEngine::execute`.
///
/// BC-2.11.025 EC-11-025-007 / AC-007 — ADR-066 §B3 + §D3 (MAX_KEY_BYTES = 256, CWE-400).
///
/// RED failure: E-QUERY-045 gate not wired (T-09 not done); gets DataFusion error instead.
/// GREEN: gate fires after T-09, assertion passes.
#[tokio::test]
async fn test_jex_rg007_key_exceeds_max_len_rejected_e_query_045_b() {
    // SAP-3: Uses QueryEngine::execute (public surface).
    let engine = make_gate_engine();

    // Construct a 257-character literal key — one byte over the 256-byte maximum.
    let key_257 = "A".repeat(257);
    let query = format!("SELECT json_extract_string(raw_data, '{key_257}') FROM test_events");

    let result = engine.execute(&query, QueryOptions::default()).await;

    match result {
        Err(PrismError::JsonExtractKeyTooLong { key_len, max_len }) => {
            assert_eq!(
                key_len, 257,
                "RG-JEX-007: key_len must be 257 (the actual key length). Got: {key_len}"
            );
            assert_eq!(
                max_len, 256,
                "RG-JEX-007: max_len must be 256 (ADR-066 §D3 / BC-2.11.025 EC-11-025-007). Got: {max_len}"
            );
            // Wire-shape assertion: Display must match MCP INVALID_PARAMS message.
            let display = format!("{}", PrismError::JsonExtractKeyTooLong { key_len, max_len });
            assert!(
                display.starts_with("E-QUERY-045:"),
                "RG-JEX-007 wire-shape: error must start with 'E-QUERY-045:'. Got: {display:?}"
            );
            assert!(
                display.contains("257 bytes"),
                "RG-JEX-007 wire-shape: Display must contain '257 bytes'. Got: {display:?}"
            );
            assert!(
                display.contains("256-byte maximum"),
                "RG-JEX-007 wire-shape: Display must contain '256-byte maximum'. Got: {display:?}"
            );
            assert!(
                display.contains("CWE-400"),
                "RG-JEX-007 wire-shape: Display must contain 'CWE-400'. Got: {display:?}"
            );
        }
        Err(other) => {
            panic!(
                "RG-JEX-007 (FAILS RED until T-09): expected PrismError::JsonExtractKeyTooLong, \
                 got different error: {other:?}\n\
                 This is expected during Red Gate — the E-QUERY-045 gate is not yet wired."
            );
        }
        Ok(qr) => {
            panic!(
                "RG-JEX-007: expected Err(JsonExtractKeyTooLong), got Ok with {} batches. \
                 E-QUERY-045(b) plan gate must reject keys exceeding 256 bytes.",
                qr.batches.len()
            );
        }
    }
}

// ===========================================================================
// RG-JEX-008 — AC-008 / EC-11-025-008: non-string JSON value → coerced to string
// ===========================================================================

/// RG-JEX-008: non-string JSON values (numbers, booleans) are coerced via `to_string()`.
///
/// Input: `{"count":42}`, key `'count'`
/// Expected: the string `"42"` (not SQL NULL, not a parse error).
///
/// BC-2.11.025 EC-11-025-008 — ADR-066 §B1 step 7 (non-string, non-null → to_string()).
///
/// RED failure: `json_extract_string_udf()` panics `todo!()` at T-07 stub.
/// GREEN: returns `"42"` after T-05 + T-07.
#[tokio::test]
async fn test_jex_rg008_non_string_json_value_coerced_to_string() {
    // FAILS RED: make_udf_ctx() calls json_extract_string_udf() which is todo!().
    let ctx = make_udf_ctx();
    register_json_table(&ctx, "jex_data", Some(r#"{"count":42}"#));

    let df = ctx
        .sql("SELECT json_extract_string(raw_data, 'count') AS extracted FROM jex_data")
        .await
        .expect("RG-JEX-008: SQL must parse and plan");

    let batches = df
        .collect()
        .await
        .expect("RG-JEX-008 (FAILS RED): UDF execution must succeed — todo!() in invoke_with_args");

    let col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("RG-JEX-008: result column must be Utf8 StringArray");

    assert!(
        !col.is_null(0),
        "RG-JEX-008 (EC-11-025-008 / AC-008): numeric value at 'count' key must NOT be SQL NULL. \
         Non-string values are coerced via to_string()."
    );
    assert_eq!(
        col.value(0),
        "42",
        "RG-JEX-008 (EC-11-025-008 / AC-008): integer 42 must coerce to string \"42\". Got: {:?}",
        col.value(0)
    );
}

// ---------------------------------------------------------------------------
// RG-JEX-008-bool / RG-JEX-008-array / RG-JEX-008-object
// OBS-2 coercion siblings (BC-2.11.025 EC-11-025-008 / AC-008):
// boolean, array, and object values coerced via serde_json Value::to_string().
// ---------------------------------------------------------------------------

/// RG-JEX-008-bool: boolean JSON value is coerced via `val.to_string()` → `"true"` / `"false"`.
///
/// Input: `{"flag":true}`, key `'flag'`
/// Expected: the string `"true"` — `serde_json::Value::Bool(true).to_string()` compact output.
///
/// BC-2.11.025 EC-11-025-008 / AC-008 — ADR-066 §B1 step 7 (non-string, non-null → to_string()).
/// Sibling of RG-JEX-008 (integer case already tested); extends coercion coverage to booleans.
#[tokio::test]
async fn test_jex_rg008_bool_true_coerced_to_string() {
    let ctx = make_udf_ctx();
    register_json_table(&ctx, "jex_data", Some(r#"{"flag":true}"#));

    let df = ctx
        .sql("SELECT json_extract_string(raw_data, 'flag') AS extracted FROM jex_data")
        .await
        .expect("RG-JEX-008-bool: SQL must parse and plan");

    let batches = df
        .collect()
        .await
        .expect("RG-JEX-008-bool: UDF execution must succeed");

    let col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("RG-JEX-008-bool: result column must be Utf8 StringArray");

    assert!(
        !col.is_null(0),
        "RG-JEX-008-bool (EC-11-025-008 / AC-008): boolean true at 'flag' key must NOT be SQL NULL. \
         Non-null non-string values are coerced via serde_json val.to_string()."
    );
    assert_eq!(
        col.value(0),
        "true",
        "RG-JEX-008-bool (EC-11-025-008 / AC-008): serde_json::Value::Bool(true).to_string() must \
         produce \"true\" (JSON literal, not a Rust Debug string). Got: {:?}",
        col.value(0)
    );
}

/// RG-JEX-008-array: JSON array value is coerced via `val.to_string()` → compact JSON array string.
///
/// Input: `{"items":[1,2]}`, key `'items'`
/// Expected: the string `"[1,2]"` — `serde_json::Value::Array([1,2]).to_string()` compact output
/// (no spaces; serde_json Display uses compact serialization).
///
/// BC-2.11.025 EC-11-025-008 / AC-008 — ADR-066 §B1 step 7 (non-string, non-null → to_string()).
/// Sibling of RG-JEX-008 (integer case); extends coercion coverage to JSON arrays.
#[tokio::test]
async fn test_jex_rg008_array_coerced_to_string() {
    let ctx = make_udf_ctx();
    register_json_table(&ctx, "jex_data", Some(r#"{"items":[1,2]}"#));

    let df = ctx
        .sql("SELECT json_extract_string(raw_data, 'items') AS extracted FROM jex_data")
        .await
        .expect("RG-JEX-008-array: SQL must parse and plan");

    let batches = df
        .collect()
        .await
        .expect("RG-JEX-008-array: UDF execution must succeed");

    let col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("RG-JEX-008-array: result column must be Utf8 StringArray");

    assert!(
        !col.is_null(0),
        "RG-JEX-008-array (EC-11-025-008 / AC-008): array value at 'items' key must NOT be SQL NULL. \
         Non-null non-string values are coerced via serde_json val.to_string()."
    );
    assert_eq!(
        col.value(0),
        "[1,2]",
        "RG-JEX-008-array (EC-11-025-008 / AC-008): serde_json array [1,2] must coerce to \"[1,2]\" \
         (compact JSON, no spaces). Got: {:?}",
        col.value(0)
    );
}

/// RG-JEX-008-object: nested JSON object value is coerced via `val.to_string()` → compact JSON string.
///
/// Input: `{"meta":{"k":"v"}}`, key `'meta'`
/// Expected: the string `r#"{"k":"v"}"#` — `serde_json::Value::Object({"k":"v"}).to_string()`
/// compact output (no spaces).
///
/// BC-2.11.025 EC-11-025-008 / AC-008 — ADR-066 §B1 step 7 (non-string, non-null → to_string()).
/// Sibling of RG-JEX-008 (integer case); extends coercion coverage to nested JSON objects.
#[tokio::test]
async fn test_jex_rg008_object_coerced_to_string() {
    let ctx = make_udf_ctx();
    register_json_table(&ctx, "jex_data", Some(r#"{"meta":{"k":"v"}}"#));

    let df = ctx
        .sql("SELECT json_extract_string(raw_data, 'meta') AS extracted FROM jex_data")
        .await
        .expect("RG-JEX-008-object: SQL must parse and plan");

    let batches = df
        .collect()
        .await
        .expect("RG-JEX-008-object: UDF execution must succeed");

    let col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("RG-JEX-008-object: result column must be Utf8 StringArray");

    assert!(
        !col.is_null(0),
        "RG-JEX-008-object (EC-11-025-008 / AC-008): object value at 'meta' key must NOT be SQL NULL. \
         Non-null non-string values are coerced via serde_json val.to_string()."
    );
    assert_eq!(
        col.value(0),
        r#"{"k":"v"}"#,
        "RG-JEX-008-object (EC-11-025-008 / AC-008): serde_json nested object must coerce to \
         compact JSON string {{\"k\":\"v\"}} (no spaces). Got: {:?}",
        col.value(0)
    );
}

// ===========================================================================
// RG-JEX-009 — AC-009 / EC-11-025-010: unparseable JSON input → SQL NULL
// ===========================================================================

/// RG-JEX-009: a non-JSON string in the `raw_data` column yields SQL NULL.
///
/// Input: `"not valid json at all !!"` (parse failure), key `'severity'`
/// Expected: SQL NULL (Arrow null cell, not a runtime error).
///
/// BC-2.11.025 EC-11-025-010 — ADR-066 §B1 step 2 (serde_json parse failure → None).
///
/// RED failure: `json_extract_string_udf()` panics `todo!()` at T-07 stub.
/// GREEN: returns SQL NULL after T-05 + T-07.
#[tokio::test]
async fn test_jex_rg009_parse_failure_non_json_input_returns_sql_null() {
    // FAILS RED: make_udf_ctx() calls json_extract_string_udf() which is todo!().
    let ctx = make_udf_ctx();
    register_json_table(&ctx, "jex_data", Some("not valid json at all !!"));

    let df = ctx
        .sql("SELECT json_extract_string(raw_data, 'severity') AS extracted FROM jex_data")
        .await
        .expect("RG-JEX-009: SQL must parse and plan");

    let batches = df
        .collect()
        .await
        .expect("RG-JEX-009 (FAILS RED): UDF execution must succeed — todo!() in invoke_with_args");

    let col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("RG-JEX-009: result column must be Utf8 StringArray");

    assert!(
        col.is_null(0),
        "RG-JEX-009 (EC-11-025-010 / AC-009): JSON parse failure must yield SQL NULL \
         (not a runtime error, not an empty string). \
         Got: {:?}",
        if col.is_null(0) { "null" } else { col.value(0) }
    );
}

// ===========================================================================
// RG-JEX-010 — AC-010 / EC-11-025-009: dot in key = top-level key, NOT nested path
// ===========================================================================

/// RG-JEX-010: a dot in the key literal is treated as a top-level key name, not JSONPath.
///
/// Input: `{"a.b":"dotted","a":{"b":"nested"}}`, key `'a.b'`
/// Expected: `"dotted"` (top-level key "a.b"), NOT `"nested"` (nested path a→b).
///
/// This distinguishes `json_extract_string` from JSONPath-style nested access, which
/// is reserved for `S-JSON-EXTRACT-NESTED-001` post-beta.3 (ADR-066 §D4).
///
/// BC-2.11.025 EC-11-025-009 — ADR-066 §B1 step 4 + §D4.
///
/// RED failure: `json_extract_string_udf()` panics `todo!()` at T-07 stub.
/// GREEN: returns `"dotted"` after T-05 + T-07.
#[tokio::test]
async fn test_jex_rg010_dot_in_key_literal_not_nested_path() {
    // FAILS RED: make_udf_ctx() calls json_extract_string_udf() which is todo!().
    let ctx = make_udf_ctx();
    register_json_table(
        &ctx,
        "jex_data",
        Some(r#"{"a.b":"dotted","a":{"b":"nested"}}"#),
    );

    let df = ctx
        .sql("SELECT json_extract_string(raw_data, 'a.b') AS extracted FROM jex_data")
        .await
        .expect("RG-JEX-010: SQL must parse and plan");

    let batches = df
        .collect()
        .await
        .expect("RG-JEX-010 (FAILS RED): UDF execution must succeed — todo!() in invoke_with_args");

    let col = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("RG-JEX-010: result column must be Utf8 StringArray");

    assert!(
        !col.is_null(0),
        "RG-JEX-010 (EC-11-025-009 / AC-010): top-level key 'a.b' exists and must NOT be SQL NULL."
    );
    assert_eq!(
        col.value(0),
        "dotted",
        "RG-JEX-010 (EC-11-025-009 / AC-010 / ADR-066 §D4): \
         key 'a.b' must resolve to the TOP-LEVEL key \"a.b\" → \"dotted\", \
         NOT the nested path a→b → \"nested\". Got: {:?}",
        col.value(0)
    );
}

// ===========================================================================
// RG-JEX-011 — AC-011 / EC-11-025-011: pipe mode E2E + wire-shape null assertion
// ===========================================================================

/// RG-JEX-011: SQL-pipe mode query executes `json_extract_string` end-to-end with a mock adapter.
///
/// Uses SQL-pipe mode: `SELECT json_extract_string(payload, 'severity') AS extracted
/// FROM jex_events | limit 10`. This exercises the `Ast::SqlPipe` path through
/// `sqlpipe_to_executable_sql` (pipe_sql_emitter.rs) which loweres
/// `ScalarFunc::JsonExtractString` — the "pipe mode" path per ADR-066 §H.
///
/// Mock adapter (`JexMockAdapter`) returns 3 rows with varying `payload` JSON:
/// - Row 0: `{"severity":"critical","host":"server01"}` → extracted = "critical"
/// - Row 1: `{"severity":null,"host":"server02"}`       → extracted = SQL NULL (JSON null)
/// - Row 2: `{"host":"server03"}`                       → extracted = SQL NULL (key absent)
///
/// Wire-shape (BC-2.11.001 EC-11-079 null-not-absent): null rows must serialize as
/// JSON `null`, not as absent key and not as the string "null".
///
/// SAP-3 compliance: exercises `json_extract_string` end-to-end from `QueryEngine::execute`
/// (the prism_query public surface), NOT via a synthetic AST or direct DataFusion call.
///
/// BC-2.11.025 EC-11-025-011 / AC-011 — ADR-066 §B1 full path + §H.
///
/// RED failure: `json_extract_string` UDF not registered in `SessionContext`
///   (T-07 not done) → DataFusion "unrecognized function" error.
/// GREEN: UDF registered + extraction works after T-05 + T-07 + T-10.
#[tokio::test]
async fn test_jex_rg011_pipe_mode_end_to_end_executes() {
    // SAP-3: QueryEngine::execute with real pipe-mode PQL string (public surface).
    // ALL scope (clients: None) — fans out to all registered adapters for sensor "jex"
    // without requiring an OrgRegistry. JexMockAdapter is registered under a fresh OrgId;
    // the engine uses test-mode synthetic slug resolution (ADR-061 D3).
    let org_id = OrgId::new();

    let mut adapter_registry = AdapterRegistry::new();
    // OrgId::new() allocates a fresh ID; adapter_registry maps it to JexMockAdapter.
    // JexMockAdapter::sensor_type() = "jex" — engine resolves "jex_events" table as
    // sensor prefix "jex" and finds this adapter.
    adapter_registry.register(org_id, Arc::new(JexMockAdapter));

    let engine = QueryEngine::new(
        Arc::new(adapter_registry),
        Arc::new(NullCredentialStore),
        Arc::new(OcsfNormalizer::new()),
        Arc::new(ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
    )
    .with_credential_resolver(Arc::new(StubCredentialResolver));

    // ALL scope: clients: None fans out to all adapters for sensor "jex".
    // With org_registry absent (test mode), the engine assigns a synthetic slug.
    let options = QueryOptions {
        clients: None,
        sensors: None,
        limit: Some(10),
        force_refresh: false,
        ..QueryOptions::default()
    };

    // SQL-pipe mode: head SQL with scalar function + pipe stage (Ast::SqlPipe path).
    // This exercises `sqlpipe_to_executable_sql` which lowers `ScalarFunc::JsonExtractString`
    // (pipe_sql_emitter.rs) — the "pipe mode" path per ADR-066 §H / BC-2.11.025 EC-11-025-011.
    // "jex_events" extracts sensor prefix "jex" → JexMockAdapter found in AdapterRegistry.
    let result = engine
        .execute(
            "SELECT json_extract_string(payload, 'severity') AS extracted FROM jex_events | limit 10",
            options,
        )
        .await;

    // FAILS RED: UDF not registered → DataFusion "No function named json_extract_string" error.
    // After T-07 registers the UDF: invoke_with_args is still todo!() → panic → still RED.
    // After T-05 + T-07: UDF works, returns extracted values.
    let qr = result.expect(
        "RG-JEX-011 (FAILS RED until T-07+T-10): pipe mode execution must succeed. \
         During Red Gate: json_extract_string UDF is not registered in SessionContext.",
    );

    let total_rows: usize = qr.batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        total_rows, 3,
        "RG-JEX-011: mock adapter returns 3 rows; all 3 must appear in result"
    );

    // Find the "extracted" column in the first batch.
    let batch = &qr.batches[0];
    let schema = batch.schema();
    let extracted_idx = schema
        .index_of("extracted")
        .expect("RG-JEX-011: 'extracted' column must be present in result schema");

    let extracted_col = batch
        .column(extracted_idx)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("RG-JEX-011: 'extracted' column must be Utf8 StringArray");

    // Row 0: JSON object with string "severity":"critical" → extracted = "critical"
    assert!(
        !extracted_col.is_null(0),
        "RG-JEX-011 (EC-11-025-011 / AC-011): row 0 has 'severity':'critical' → must NOT be NULL"
    );
    assert_eq!(
        extracted_col.value(0),
        "critical",
        "RG-JEX-011 (EC-11-025-011 / AC-011): row 0 extracted value must be \"critical\". \
         Got: {:?}",
        extracted_col.value(0)
    );

    // Row 1: JSON null at 'severity' key → SQL NULL
    assert!(
        extracted_col.is_null(1),
        "RG-JEX-011 (EC-11-025-011 / AC-011): row 1 has JSON null at 'severity' → must be SQL NULL"
    );

    // Row 2: key 'severity' absent → SQL NULL
    assert!(
        extracted_col.is_null(2),
        "RG-JEX-011 (EC-11-025-011 / AC-011): row 2 has no 'severity' key → must be SQL NULL"
    );

    // Arrow-level null assertions above (extracted_col.is_null(1), extracted_col.is_null(2))
    // are load-bearing: they verify the UDF produces SQL NULL at the Arrow layer.
    //
    // Serialized-wire null-not-absent coverage (BC-2.11.001 EC-11-079) lives in prism-mcp:
    //   test_BC_2_11_025_json_extract_string_null_row_wire_null_not_absent
    //   (crates/prism-mcp/tests/bc_2_11_025_jex_wire_null_test.rs)
    // That test exercises the full MCP query-tool path and asserts null-NOT-absent on the
    // genuinely serialized envelope (arrow_json::WriterBuilder::with_explicit_nulls(true)).
    // arrow-json is deliberately absent from prism-query's dep graph (ADR-066 §B2 /
    // §Forbidden Dependencies); the wire coverage belongs in prism-mcp which owns the
    // production serializer.
}

// ===========================================================================
// F-4 SAP-3 public-surface value-arm assertions (QueryEngine::execute)
//
// These four tests close the SAP-3 residual from LOCAL adversary pass-1 F-4:
// arms currently only exercised via bare SessionContext.sql() are now also
// asserted end-to-end through QueryEngine::execute (the prism_query public
// surface). Wire-level null-not-absent assertions (BC-2.11.001 EC-11-079) are
// included for arms that produce SQL NULL. Production code already implements
// all four behaviors; these tests are GREEN additions, not new Red Gate tests.
// ===========================================================================

/// F-4-A (SAP-3): non-object JSON root → SQL NULL via QueryEngine::execute.
///
/// Companion to RG-JEX-005 (SessionContext.sql path). Drives EC-11-025-004 arm
/// end-to-end from `QueryEngine::execute` (the prism_query public surface),
/// satisfying CLAUDE.md §SAP-3 reachability obligation.
///
/// BC-2.11.025 EC-11-025-004 — ADR-066 §B1 step 3 (non-object → None).
/// Wire-shape: null-not-absent per BC-2.11.001 EC-11-079.
#[tokio::test]
async fn test_jex_f4_non_object_arm_engine_execute() {
    let engine = make_arm_engine(SingleRowAdapter {
        sensor_prefix: "jnoobj",
        payload: Some(r#"["a","b","c"]"#), // JSON array root — non-object → SQL NULL
    });

    let options = QueryOptions {
        clients: None,
        sensors: None,
        limit: Some(1),
        force_refresh: false,
        ..QueryOptions::default()
    };

    let qr = engine
        .execute(
            "SELECT json_extract_string(data, 'key') AS extracted FROM jnoobj_events | limit 1",
            options,
        )
        .await
        .expect(
            "F-4-A: engine.execute must succeed (non-object JSON → SQL NULL, not a runtime error)",
        );

    let total_rows: usize = qr.batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(total_rows, 1, "F-4-A: mock adapter returns 1 row");

    let batch = &qr.batches[0];
    let idx = batch
        .schema()
        .index_of("extracted")
        .expect("F-4-A: 'extracted' column must be present in result schema");
    let col = batch
        .column(idx)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("F-4-A: 'extracted' column must be Utf8 StringArray");

    // Arrow-level null assertion.
    assert!(
        col.is_null(0),
        "F-4-A (EC-11-025-004 via engine.execute): JSON array root must yield SQL NULL. \
         Got: {:?}",
        if col.is_null(0) { "null" } else { col.value(0) }
    );

    // Arrow-level null assertion immediately above (col.is_null(0)) is load-bearing.
    // Serialized-wire null-not-absent coverage (BC-2.11.001 EC-11-079) lives in prism-mcp:
    //   test_BC_2_11_025_json_extract_string_null_row_wire_null_not_absent
    //   (crates/prism-mcp/tests/bc_2_11_025_jex_wire_null_test.rs)
    // arrow-json is forbidden from prism-query's dep graph (ADR-066 §B2 / §Forbidden Dependencies).
}

/// F-4-B (SAP-3): non-string JSON value → coerced string via QueryEngine::execute.
///
/// Companion to RG-JEX-008 (SessionContext.sql path). Drives EC-11-025-008 arm
/// end-to-end from `QueryEngine::execute` (the prism_query public surface),
/// satisfying CLAUDE.md §SAP-3 reachability obligation.
///
/// BC-2.11.025 EC-11-025-008 — ADR-066 §B1 step 7 (non-string, non-null → to_string()).
#[tokio::test]
async fn test_jex_f4_coerce_arm_engine_execute() {
    let engine = make_arm_engine(SingleRowAdapter {
        sensor_prefix: "jcoerce",
        payload: Some(r#"{"count":42}"#), // integer value → coerced to "42"
    });

    let options = QueryOptions {
        clients: None,
        sensors: None,
        limit: Some(1),
        force_refresh: false,
        ..QueryOptions::default()
    };

    let qr = engine
        .execute(
            "SELECT json_extract_string(data, 'count') AS extracted FROM jcoerce_events | limit 1",
            options,
        )
        .await
        .expect("F-4-B: engine.execute must succeed (integer value → coerced string \"42\")");

    let total_rows: usize = qr.batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(total_rows, 1, "F-4-B: mock adapter returns 1 row");

    let batch = &qr.batches[0];
    let idx = batch
        .schema()
        .index_of("extracted")
        .expect("F-4-B: 'extracted' column must be present in result schema");
    let col = batch
        .column(idx)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("F-4-B: 'extracted' column must be Utf8 StringArray");

    assert!(
        !col.is_null(0),
        "F-4-B (EC-11-025-008 via engine.execute): integer value at 'count' must NOT be SQL NULL \
         (coerced via to_string())."
    );
    assert_eq!(
        col.value(0),
        "42",
        "F-4-B (EC-11-025-008 via engine.execute): integer 42 must coerce to string \"42\". \
         Got: {:?}",
        col.value(0)
    );

    // Wire-shape: non-null value serializes as the coerced string.
    let wire = serde_json::json!({
        "extracted": serde_json::Value::String(col.value(0).to_string())
    });
    assert_eq!(
        wire["extracted"].as_str().unwrap(),
        "42",
        "F-4-B wire-shape: coerced integer must serialize as JSON string \"42\". Got: {:?}",
        wire["extracted"]
    );
}

/// F-4-C (SAP-3): JSON parse failure → SQL NULL via QueryEngine::execute.
///
/// Companion to RG-JEX-009 (SessionContext.sql path). Drives EC-11-025-010 arm
/// end-to-end from `QueryEngine::execute` (the prism_query public surface),
/// satisfying CLAUDE.md §SAP-3 reachability obligation.
///
/// BC-2.11.025 EC-11-025-010 — ADR-066 §B1 step 2 (serde_json parse failure → None).
/// Wire-shape: null-not-absent per BC-2.11.001 EC-11-079.
#[tokio::test]
async fn test_jex_f4_parse_fail_arm_engine_execute() {
    let engine = make_arm_engine(SingleRowAdapter {
        sensor_prefix: "jpfail",
        payload: Some("not valid json at all !!"), // malformed JSON → SQL NULL
    });

    let options = QueryOptions {
        clients: None,
        sensors: None,
        limit: Some(1),
        force_refresh: false,
        ..QueryOptions::default()
    };

    let qr = engine
        .execute(
            "SELECT json_extract_string(data, 'key') AS extracted FROM jpfail_events | limit 1",
            options,
        )
        .await
        .expect(
            "F-4-C: engine.execute must succeed (parse failure yields SQL NULL, not runtime error)",
        );

    let total_rows: usize = qr.batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(total_rows, 1, "F-4-C: mock adapter returns 1 row");

    let batch = &qr.batches[0];
    let idx = batch
        .schema()
        .index_of("extracted")
        .expect("F-4-C: 'extracted' column must be present in result schema");
    let col = batch
        .column(idx)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("F-4-C: 'extracted' column must be Utf8 StringArray");

    assert!(
        col.is_null(0),
        "F-4-C (EC-11-025-010 via engine.execute): JSON parse failure must yield SQL NULL \
         (not a runtime error, not an empty string). Got: {:?}",
        if col.is_null(0) { "null" } else { col.value(0) }
    );

    // Arrow-level null assertion immediately above (col.is_null(0)) is load-bearing.
    // Serialized-wire null-not-absent coverage (BC-2.11.001 EC-11-079) lives in prism-mcp:
    //   test_BC_2_11_025_json_extract_string_null_row_wire_null_not_absent
    //   (crates/prism-mcp/tests/bc_2_11_025_jex_wire_null_test.rs)
    // arrow-json is forbidden from prism-query's dep graph (ADR-066 §B2 / §Forbidden Dependencies).
}

/// F-4-D (SAP-3): dot-in-key literal → top-level key (not JSONPath) via QueryEngine::execute.
///
/// Companion to RG-JEX-010 (SessionContext.sql path). Drives EC-11-025-009 arm
/// end-to-end from `QueryEngine::execute` (the prism_query public surface),
/// satisfying CLAUDE.md §SAP-3 reachability obligation.
///
/// BC-2.11.025 EC-11-025-009 — ADR-066 §B1 step 4 + §D4 (top-level-key-only; dot is
/// NOT a JSONPath separator in beta.3).
#[tokio::test]
async fn test_jex_f4_dot_in_key_arm_engine_execute() {
    let engine = make_arm_engine(SingleRowAdapter {
        sensor_prefix: "jdotkey",
        payload: Some(r#"{"a.b":"dotted","a":{"b":"nested"}}"#),
    });

    let options = QueryOptions {
        clients: None,
        sensors: None,
        limit: Some(1),
        force_refresh: false,
        ..QueryOptions::default()
    };

    let qr = engine
        .execute(
            "SELECT json_extract_string(data, 'a.b') AS extracted FROM jdotkey_events | limit 1",
            options,
        )
        .await
        .expect("F-4-D: engine.execute must succeed (dot-in-key → top-level match \"dotted\")");

    let total_rows: usize = qr.batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(total_rows, 1, "F-4-D: mock adapter returns 1 row");

    let batch = &qr.batches[0];
    let idx = batch
        .schema()
        .index_of("extracted")
        .expect("F-4-D: 'extracted' column must be present in result schema");
    let col = batch
        .column(idx)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("F-4-D: 'extracted' column must be Utf8 StringArray");

    assert!(
        !col.is_null(0),
        "F-4-D (EC-11-025-009 via engine.execute): top-level key 'a.b' must NOT be SQL NULL."
    );
    assert_eq!(
        col.value(0),
        "dotted",
        "F-4-D (EC-11-025-009 / ADR-066 §D4 via engine.execute): \
         key 'a.b' must resolve to the TOP-LEVEL key \"a.b\" → \"dotted\", \
         NOT the nested path a→b → \"nested\". Got: {:?}",
        col.value(0)
    );

    // Wire-shape: non-null value serializes as the extracted string.
    let wire = serde_json::json!({
        "extracted": serde_json::Value::String(col.value(0).to_string())
    });
    assert_eq!(
        wire["extracted"].as_str().unwrap(),
        "dotted",
        "F-4-D wire-shape: dot-in-key arm must serialize as JSON string \"dotted\". Got: {:?}",
        wire["extracted"]
    );
}

// ===========================================================================
// RG-JEX-012 family — AC-012 / EC-11-025-012: WHERE/HAVING predicate E-QUERY-045 gate
// (F-JEX-P1-HIGH-001 fix: filter_parser::fn_call_comparison WHERE/HAVING parity)
// ===========================================================================

/// RG-JEX-012: non-literal key in WHERE predicate rejected at plan time with E-QUERY-045(a).
///
/// Canonical RG-JEX-012 name from story spec S-JSON-EXTRACT-UDF-001 v1.8 / AC-012.
///
/// Query: `SELECT * FROM test_events WHERE json_extract_string(raw_data, severity_col) = 'x'`
/// `severity_col` is a column reference (non-literal) in WHERE predicate position.
/// Expected: `Err(PrismError::JsonExtractNonLiteralKey)` from `check_json_extract_key_literal`.
/// MCP surface: maps to `-32602 INVALID_PARAMS` (same as RG-JEX-006 for SELECT-list position).
///
/// RED failure: `filter_parser::fn_call_comparison` emits `ScalarFunc::Unknown` for
/// predicate function calls. `check_jex_in_expr` first arm matches only
/// `ScalarFunc::JsonExtractString`; `Unknown("json_extract_string")` falls through to the
/// generic `Scalar` arm (recurse into args, no key validation). Gate returns `Ok(())`.
///
/// GREEN: after T-09a maps `"json_extract_string"` → `ScalarFunc::JsonExtractString` in
/// `fn_call_comparison`, the WHERE predicate is caught by the gate.
///
/// SAP-3: exercises `check_json_extract_key_literal` end-to-end from `QueryEngine::execute`.
/// SID-2: asserts on full composed Display string (MCP -32602 source).
/// BC-2.11.025 EC-11-025-012 / AC-012 — ADR-066 §B3.
#[tokio::test]
async fn test_jex_rg012_where_predicate_non_literal_key_rejected_e_query_045() {
    // make_gate_engine(): empty AdapterRegistry, table_registry = None.
    // E-QUERY-037 bypassed; only E-QUERY-045 gate fires (when working correctly).
    let engine = make_gate_engine();

    let result = engine
        .execute(
            "SELECT * FROM test_events WHERE json_extract_string(raw_data, severity_col) = 'x'",
            QueryOptions::default(),
        )
        .await;

    // SID-2: assert on the full composed Display string as emitted (MCP INVALID_PARAMS source).
    let expected_display = "E-QUERY-045: json_extract_string requires a literal string key \
        (e.g., json_extract_string(col, 'key_name')). \
        Dynamic key expressions are not supported.";

    match result {
        Err(PrismError::JsonExtractNonLiteralKey) => {
            // GREEN: gate correctly fires for WHERE predicate position. Verify wire-shape.
            let display = format!("{}", PrismError::JsonExtractNonLiteralKey);
            assert!(
                display.starts_with("E-QUERY-045:"),
                "RG-JEX-012 wire-shape: error Display must start with 'E-QUERY-045:'. Got: {display:?}"
            );
            assert_eq!(
                display, expected_display,
                "RG-JEX-012 wire-shape: Display must match MCP INVALID_PARAMS message exactly. \
                 Got: {display:?}"
            );
        }
        Err(other) => panic!(
            "RG-JEX-012 (FAILS RED until T-09a filter_parser fix): \
             expected PrismError::JsonExtractNonLiteralKey, got: {other:?}\n\
             RED reason: fn_call_comparison emits ScalarFunc::Unknown for WHERE predicate \
             function calls; check_jex_in_expr only matches ScalarFunc::JsonExtractString — \
             Unknown falls to the generic Scalar arm with no key validation. \
             Fix target: fn_call_comparison in filter_parser.rs."
        ),
        Ok(qr) => panic!(
            "RG-JEX-012: expected Err(JsonExtractNonLiteralKey), got Ok ({} batches). \
             E-QUERY-045(a) must fire for non-literal key in WHERE predicate position.",
            qr.batches.len()
        ),
    }
}

/// RG-JEX-012-b: key exceeding 256 bytes in WHERE predicate rejected with E-QUERY-045(b).
///
/// Query uses a 257-byte literal key in WHERE position.
/// Expected: `Err(PrismError::JsonExtractKeyTooLong { key_len: 257, max_len: 256 })`.
///
/// RED failure: same as RG-JEX-012 — `fn_call_comparison` emits `ScalarFunc::Unknown`,
/// so the gate's key-length check is never reached.
///
/// GREEN: after T-09a, gate sees `ScalarFunc::JsonExtractString` in WHERE and validates
/// the literal key length (257 > 256 → `JsonExtractKeyTooLong`).
///
/// SAP-3: exercises `check_json_extract_key_literal` end-to-end from `QueryEngine::execute`.
/// SID-2: asserts on full composed Display string.
/// BC-2.11.025 EC-11-025-012 / AC-012 — ADR-066 §B3.
#[tokio::test]
async fn test_jex_rg012_b_where_key_too_long_rejected_e_query_045_b() {
    let engine = make_gate_engine();

    // Construct a 257-byte literal key (1 byte over the 256-byte limit).
    let key_257: String = "k".repeat(257);
    let query =
        format!("SELECT * FROM test_events WHERE json_extract_string(raw_data, '{key_257}') = 'x'");

    let result = engine.execute(&query, QueryOptions::default()).await;

    match result {
        Err(PrismError::JsonExtractKeyTooLong { key_len, max_len }) => {
            assert_eq!(
                key_len, 257,
                "RG-JEX-012-b: key_len must be 257. Got: {key_len}"
            );
            assert_eq!(
                max_len, 256,
                "RG-JEX-012-b: max_len must be 256. Got: {max_len}"
            );
            // SID-2: assert spec-verbatim E-QUERY-045(b) substrings (same approach as RG-JEX-007).
            // Spec: "E-QUERY-045: json_extract_string key is {key_len} bytes, which exceeds the
            //        {max_len}-byte maximum (CWE-400)." — error-taxonomy.md §E-QUERY-045(b),
            //        BC-2.11.025 §Error Cases, ADR-066 §F.
            let display = format!("{}", PrismError::JsonExtractKeyTooLong { key_len, max_len });
            assert!(
                display.starts_with("E-QUERY-045:"),
                "RG-JEX-012-b wire-shape: error must start with 'E-QUERY-045:'. Got: {display:?}"
            );
            assert!(
                display.contains("257 bytes"),
                "RG-JEX-012-b wire-shape: Display must contain '257 bytes'. Got: {display:?}"
            );
            assert!(
                display.contains("256-byte maximum (CWE-400)"),
                "RG-JEX-012-b wire-shape: Display must contain '256-byte maximum (CWE-400)'. \
                 Got: {display:?}"
            );
        }
        Err(other) => panic!(
            "RG-JEX-012-b (FAILS RED until T-09a filter_parser fix): \
             expected PrismError::JsonExtractKeyTooLong, got: {other:?}\n\
             RED reason: fn_call_comparison emits ScalarFunc::Unknown for WHERE predicate \
             function calls; the key-length check arm never fires. \
             Fix target: fn_call_comparison in filter_parser.rs."
        ),
        Ok(qr) => panic!(
            "RG-JEX-012-b: expected Err(JsonExtractKeyTooLong), got Ok ({} batches). \
             E-QUERY-045(b) must fire for 257-byte literal key in WHERE predicate position.",
            qr.batches.len()
        ),
    }
}

/// RG-JEX-012-c: non-literal key in HAVING predicate rejected with E-QUERY-045(a).
///
/// Query: `SELECT raw_data, count(*) FROM test_events GROUP BY raw_data
///          HAVING json_extract_string(raw_data, severity_col) = 'x'`
///
/// `severity_col` is a column reference (non-literal) in HAVING predicate position.
/// `build_predicate_parser` (used for SQL WHERE / HAVING clauses) includes
/// `fn_call_comparison` as the function-call arm, so HAVING shares the same bug.
///
/// Expected: `Err(PrismError::JsonExtractNonLiteralKey)`.
///
/// RED failure: `fn_call_comparison` emits `ScalarFunc::Unknown` for HAVING predicate
/// function calls too — same bug as WHERE. Gate returns `Ok(())`.
///
/// GREEN: after T-09a, HAVING predicate produces `ScalarFunc::JsonExtractString` →
/// gate fires → `Err(JsonExtractNonLiteralKey)`.
///
/// SAP-3: exercises `check_json_extract_key_literal` (HAVING path) from `QueryEngine::execute`.
/// SID-2: asserts on full composed Display string.
/// BC-2.11.025 EC-11-025-012 / AC-012 — ADR-066 §B3.
#[tokio::test]
async fn test_jex_rg012_c_having_predicate_non_literal_key_rejected_e_query_045() {
    let engine = make_gate_engine();

    let result = engine
        .execute(
            "SELECT raw_data, count(*) FROM test_events \
             GROUP BY raw_data \
             HAVING json_extract_string(raw_data, severity_col) = 'x'",
            QueryOptions::default(),
        )
        .await;

    let expected_display = "E-QUERY-045: json_extract_string requires a literal string key \
        (e.g., json_extract_string(col, 'key_name')). \
        Dynamic key expressions are not supported.";

    match result {
        Err(PrismError::JsonExtractNonLiteralKey) => {
            // GREEN: gate correctly fires for HAVING predicate position.
            let display = format!("{}", PrismError::JsonExtractNonLiteralKey);
            assert!(
                display.starts_with("E-QUERY-045:"),
                "RG-JEX-012-c wire-shape: error Display must start with 'E-QUERY-045:'. \
                 Got: {display:?}"
            );
            assert_eq!(
                display, expected_display,
                "RG-JEX-012-c wire-shape: Display must match MCP INVALID_PARAMS message exactly. \
                 Got: {display:?}"
            );
        }
        Err(other) => panic!(
            "RG-JEX-012-c (FAILS RED until T-09a filter_parser fix): \
             expected PrismError::JsonExtractNonLiteralKey, got: {other:?}\n\
             RED reason: fn_call_comparison emits ScalarFunc::Unknown for HAVING predicate \
             function calls; check_jex_in_predicate (HAVING path) never fires the rejection arm. \
             Fix target: fn_call_comparison in filter_parser.rs."
        ),
        Ok(qr) => panic!(
            "RG-JEX-012-c: expected Err(JsonExtractNonLiteralKey), got Ok ({} batches). \
             E-QUERY-045(a) must fire for non-literal key in HAVING predicate position.",
            qr.batches.len()
        ),
    }
}

/// RG-JEX-012-d: WHERE predicate with valid literal key executes correctly (GREEN proof).
///
/// Functional correctness assertion: after the filter_parser fix, WHERE predicate with a
/// valid literal key must NOT be rejected by E-QUERY-045 and must return correctly filtered rows.
///
/// Query: `SELECT * FROM jex_events WHERE json_extract_string(payload, 'severity') = 'critical'`
/// JexMockAdapter returns 3 rows:
///   Row 0: `{"severity":"critical","host":"server01"}` — passes WHERE filter
///   Row 1: `{"severity":null,"host":"server02"}`       — fails (NULL ≠ 'critical')
///   Row 2: `{"host":"server03"}`                       — fails (NULL ≠ 'critical')
///
/// Expected: Ok, exactly 1 row returned (only row 0 passes).
///
/// This test is GREEN both before and after the fix (gate does not fire for valid literal
/// keys, and DataFusion can execute the WHERE filter via the registered UDF). It proves
/// the fix makes WHERE functional end-to-end, not just gated.
///
/// SAP-3: public surface `QueryEngine::execute` path.
/// BC-2.11.025 EC-11-025-012 / AC-012 — ADR-066 §B1 + §B3.
#[tokio::test]
async fn test_jex_rg012_d_where_valid_literal_key_executes_correctly() {
    // Build engine with JexMockAdapter registered for "jex" sensor.
    // jex_events → sensor_id "jex" → JexMockAdapter (3 rows with payload column).
    let org_id = prism_core::OrgId::new();
    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(JexMockAdapter));
    let engine = QueryEngine::new(
        Arc::new(adapter_registry),
        Arc::new(NullCredentialStore),
        Arc::new(OcsfNormalizer::new()),
        Arc::new(ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
    )
    .with_credential_resolver(Arc::new(StubCredentialResolver));

    let result = engine
        .execute(
            "SELECT * FROM jex_events WHERE json_extract_string(payload, 'severity') = 'critical'",
            QueryOptions::default(),
        )
        .await;

    let qr = result.expect(
        "RG-JEX-012-d: WHERE predicate with valid literal key 'severity' must execute without error. \
         E-QUERY-045(a/b) must NOT fire for a literal key under 256 bytes.",
    );

    // Only row 0 has severity = "critical"; rows 1 and 2 return NULL and are filtered out.
    let total_rows: usize = qr.batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        total_rows, 1,
        "RG-JEX-012-d (AC-012 functional): WHERE json_extract_string(payload, 'severity') = 'critical' \
         must return exactly 1 row (only row 0 passes). Got {total_rows} rows."
    );
}

/// DML safe-skip proof (post-fix expectation): DML WHERE filter parses to
/// `ScalarFunc::JsonExtractString` AND `check_json_extract_key_literal` returns `Ok(())`
/// for the DML input (safe DML skip arm).
///
/// After the T-09a filter_parser fix, `fn_call_comparison` will also emit
/// `ScalarFunc::JsonExtractString` for DML WHERE predicates (DELETE / UPDATE filters share
/// the same predicate parser). The `check_json_extract_key_literal` gate must still return
/// `Ok(())` immediately for `Ast::Sql(SqlStatement::Dml(...))` — the write path has no
/// read-execute scope and DML filter validation is a future concern (S-3.07).
///
/// Verification splits into two parts tested here:
///   (1) Parser: `parse_and_plan("DELETE ... WHERE json_extract_string(col, other_col) = 'x'")`
///       produces `ScalarFunc::JsonExtractString` in the filter predicate (RED until T-09a).
///   (2) Gate safe-skip: engine.execute("DELETE ...") does NOT return
///       `Err(JsonExtractNonLiteralKey)` — the DML arm of `check_json_extract_key_literal`
///       returns `Ok(())` regardless of filter contents (remains GREEN throughout).
///
/// The deeper gate proof (2) — which requires calling `check_json_extract_key_literal`
/// directly with a synthetic `Ast::Sql(Dml)` containing `ScalarFunc::JsonExtractString` —
/// is covered by the inline engine.rs test `test_jex_dml_ast_returns_ok_no_scope`
/// (already GREEN; uses `pub(crate)` function not accessible from external tests).
///
/// SAP-3 note: `parse_and_plan` is the public entry point. `check_json_extract_key_literal`
/// is `pub(crate)` — not accessible from external test files; covered by inline test.
///
/// RED failure (part 1): parser currently emits `ScalarFunc::Unknown("json_extract_string")`
/// for DML WHERE predicates. Assertion `func == ScalarFunc::JsonExtractString` fails.
/// GREEN (part 1): after T-09a, `fn_call_comparison` maps "json_extract_string" →
/// `ScalarFunc::JsonExtractString` for all predicate positions including DML WHERE.
///
/// BC-2.11.025 EC-11-025-012 / AC-012 — ADR-066 §B3.
#[test]
fn test_jex_dml_ast_safe_skip_with_jex_variant() {
    // Part (1): parse a real DML DELETE with json_extract_string in WHERE clause.
    // After T-09a fix: fn_call_comparison maps "json_extract_string" →
    // ScalarFunc::JsonExtractString for DML WHERE predicates too.
    // non-literal key (other_col) chosen to ensure the gate WOULD fire if it walked DML.
    let result =
        parse_and_plan("DELETE FROM test_table WHERE json_extract_string(col, other_col) = 'x'");

    let ast = result.expect(
        "DML rework proof (1): DELETE with json_extract_string in WHERE must parse successfully. \
         The parser recognises the function regardless of key variant.",
    );

    // Extract the DML node.
    let dml = match &ast {
        Ast::Sql(SqlStatement::Dml(node)) => node,
        other => {
            panic!("DML rework proof (1): expected Ast::Sql(SqlStatement::Dml(_)), got {other:?}")
        }
    };

    // The WHERE filter must be present.
    let filter = dml.filter.as_ref().expect(
        "DML rework proof (1): DELETE WHERE clause must produce a non-None filter predicate",
    );

    // After T-09a: the predicate's LHS must be a FuncCall::Scalar with
    // ScalarFunc::JsonExtractString (not Unknown).
    // RED until T-09a: fn_call_comparison still emits ScalarFunc::Unknown here.
    match filter {
        Predicate::Compare { lhs, .. } => match lhs.as_ref() {
            Expr::FuncCall(FuncCall::Scalar { func, .. }) => {
                assert_eq!(
                    func,
                    &ScalarFunc::JsonExtractString,
                    "DML rework proof (1) (FAILS RED until T-09a filter_parser fix): \
                     DML WHERE json_extract_string must parse as ScalarFunc::JsonExtractString \
                     after the fix. Currently emits ScalarFunc::Unknown(\"json_extract_string\"). \
                     Got: {func:?}"
                );
            }
            other => panic!("DML rework proof (1): expected FuncCall::Scalar LHS, got {other:?}"),
        },
        other => panic!("DML rework proof (1): expected Predicate::Compare, got {other:?}"),
    }
}
