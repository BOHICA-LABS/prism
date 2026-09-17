//! Red Gate test suite for S-JSON-EXTRACT-UDF-001 / json_extract_string scalar UDF.
//!
//! 11 failing tests — RG-JEX-001..RG-JEX-011 — all MUST FAIL against the current
//! `todo!()` stubs before any production logic is implemented (Red Gate requirement,
//! BC-5.38.001). Every test traces to exactly one EC-11-025-NNN edge case in
//! BC-2.11.025 v1.8 and one acceptance criterion in S-JSON-EXTRACT-UDF-001 v1.2.
//!
//! # Test method by group
//!
//! Tests RG-JEX-001..005, RG-JEX-008..010 use DataFusion `SessionContext` directly:
//! they register the UDF (which panics with `todo!()` at the `json_extract_string_udf()`
//! factory call) and assert on extracted string / null values per ADR-066 §B1.
//!
//! Tests RG-JEX-006 and RG-JEX-007 use `QueryEngine::execute` (SAP-3 compliance):
//! they exercise the `check_json_extract_key_literal` E-QUERY-045 plan gate
//! end-to-end from the prism_query public surface — not via a synthetic AST.
//!
//! Test RG-JEX-011 uses `QueryEngine::execute` with a mock adapter (SAP-3 compliance)
//! and asserts wire-level null serialization per BC-2.11.001 EC-11-079 (null-not-absent).
//!
//! # Red Gate failure modes
//!
//! | Test           | Failure mode during Red Gate                                         |
//! |----------------|----------------------------------------------------------------------|
//! | RG-JEX-001..005, 008..010 | `json_extract_string_udf()` panics `todo!()` at factory call |
//! | RG-JEX-006     | assertion: expected `JsonExtractNonLiteralKey`, gate not wired yet   |
//! | RG-JEX-007     | assertion: expected `JsonExtractKeyTooLong`, gate not wired yet      |
//! | RG-JEX-011     | `json_extract_string` UDF not registered → DataFusion error          |
//!
//! # Wire-shape discipline (CLAUDE.md / BC-2.11.001 EC-11-079)
//!
//! - RG-JEX-002..005, 009: Arrow-level null assertions (`is_null(0)`) — not empty string
//! - RG-JEX-006..007: `Display`-level error assertions (MCP `-32602 INVALID_PARAMS` source)
//! - RG-JEX-011: `serde_json` null-not-absent assertion on serialized pipe-mode output
//!
//! # BC-5.38.001 density check
//!
//! Red Gate tests / ACs = 11 / 11 = 1.0 (density requirement satisfied).
//!
//! # Traceability
//!
//! | RG-ID       | BC EC anchor      | AC                     | ADR-066 §B1 step |
//! |-------------|-------------------|------------------------|------------------|
//! | RG-JEX-001  | EC-11-025-001     | AC-001 happy path      | step 6 (String)  |
//! | RG-JEX-002  | EC-11-025-005     | AC-002 JSON null→NULL  | step 5 (Null)    |
//! | RG-JEX-003  | EC-11-025-003     | AC-003 missing key     | step 4 (absent)  |
//! | RG-JEX-004  | EC-11-025-002     | AC-004 null col input  | step 1 (None)    |
//! | RG-JEX-005  | EC-11-025-004     | AC-005 non-object JSON | step 3 (non-obj) |
//! | RG-JEX-006  | EC-11-025-006     | AC-006 E-QUERY-045(a)  | plan gate        |
//! | RG-JEX-007  | EC-11-025-007     | AC-007 E-QUERY-045(b)  | plan gate        |
//! | RG-JEX-008  | EC-11-025-008     | AC-008 non-string coerce | step 7 (other) |
//! | RG-JEX-009  | EC-11-025-010     | AC-009 parse failure   | step 2 (parse)   |
//! | RG-JEX-010  | EC-11-025-009     | AC-010 dot-in-key      | step 4 (top-lvl) |
//! | RG-JEX-011  | EC-11-025-011     | AC-011 pipe mode E2E   | full E2E         |
//!
//! Story: S-JSON-EXTRACT-UDF-001 v1.2 | BC: BC-2.11.025 v1.8 | ADR: ADR-066 v1.6

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
use arrow_json;
use async_trait::async_trait;
use datafusion::datasource::MemTable;
use datafusion::execution::context::SessionContext;
use prism_core::error::PrismError;
use prism_core::{OrgId, OrgSlug, SensorId};
use prism_credentials::{namespace::CredentialName, CredentialStore};
use prism_ocsf::OcsfNormalizer;
use prism_query::{
    engine::{QueryEngine, QueryEngineConfig, QueryOptions, QueryResult},
    json_extract_udf::json_extract_string_udf,
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
/// BC-2.11.025 EC-11-025-007 / AC-007 — ADR-066 §B3 + §B2 (MAX_KEY_BYTES = 256, CWE-400).
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
                "RG-JEX-007: max_len must be 256 (ADR-066 §B2 / BC-2.11.025 EC-11-025-007). Got: {max_len}"
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

    // Wire-shape assertion (BC-2.11.001 EC-11-079 null-not-absent / SID-2):
    // Serialize the full result batch through the REAL production serializer —
    // arrow_json::writer::WriterBuilder::new().with_explicit_nulls(true) —
    // which is the exact path prism-mcp server.rs uses for every query response.
    //
    // GENUINELY FAILABLE: arrow_json::WriterBuilder::new() uses explicit_nulls=false
    // by DEFAULT, which OMITS null-valued keys entirely from the serialized row object
    // (producing `{}` instead of `{"extracted":null}`). The contains_key assertion
    // below FAILS under that default, catching the historical [C3]/[H20] defect class
    // (live-audit 2026-07-13, CLAUDE.md wire-shape discipline). The prior hand-built
    // serde_json::json!({..if is_null..}) form was tautological — it constructed the
    // assertion target from the same is_null check being asserted on (OBS-2).
    let mut wire_buf: Vec<u8> = Vec::new();
    let mut wire_writer = arrow_json::writer::WriterBuilder::new()
        .with_explicit_nulls(true)
        .build::<_, arrow_json::writer::JsonArray>(&mut wire_buf);
    wire_writer
        .write(batch)
        .expect("RG-JEX-011 wire: arrow_json write must not fail");
    wire_writer
        .finish()
        .expect("RG-JEX-011 wire: arrow_json finish must not fail");
    let wire_rows: Vec<serde_json::Value> = serde_json::from_slice(&wire_buf)
        .expect("RG-JEX-011 wire: arrow_json output must parse as JSON array of row objects");

    // Row 1: JSON null at 'severity' key → SQL NULL → must serialize as JSON null, NOT absent.
    assert!(
        wire_rows[1]
            .as_object()
            .expect("RG-JEX-011 wire: row 1 must be a JSON object")
            .contains_key("extracted"),
        "RG-JEX-011 wire-shape (EC-11-079 null-not-absent): row 1 'extracted' key must be \
         PRESENT in the serialized wire output. This assertion FAILS when WriterBuilder uses \
         explicit_nulls=false (the arrow_json default) — the defect that caused [C3]/[H20] \
         in the live-audit. Got wire row 1: {:?}",
        wire_rows[1]
    );
    assert!(
        wire_rows[1]["extracted"].is_null(),
        "RG-JEX-011 wire-shape (EC-11-079): row 1 (JSON null at 'severity') must serialize \
         as JSON null, not as the string \"null\" or any other value. Got: {:?}",
        wire_rows[1]["extracted"]
    );

    // Row 2: 'severity' key absent in source JSON → SQL NULL → must serialize as JSON null, NOT absent.
    assert!(
        wire_rows[2]
            .as_object()
            .expect("RG-JEX-011 wire: row 2 must be a JSON object")
            .contains_key("extracted"),
        "RG-JEX-011 wire-shape (EC-11-079 null-not-absent): row 2 'extracted' key must be \
         PRESENT in the serialized wire output (null-not-absent). Got wire row 2: {:?}",
        wire_rows[2]
    );
    assert!(
        wire_rows[2]["extracted"].is_null(),
        "RG-JEX-011 wire-shape (EC-11-079): row 2 (absent 'severity' key → SQL NULL) must \
         serialize as JSON null. Got: {:?}",
        wire_rows[2]["extracted"]
    );
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

    // Wire-shape null-not-absent (BC-2.11.001 EC-11-079):
    // Serialize through the real production arrow_json path (prism-mcp server.rs).
    // GENUINELY FAILABLE: WriterBuilder::new() without .with_explicit_nulls(true)
    // (the arrow_json default) omits null keys, causing contains_key to return false.
    let mut wire_buf: Vec<u8> = Vec::new();
    let mut wire_writer = arrow_json::writer::WriterBuilder::new()
        .with_explicit_nulls(true)
        .build::<_, arrow_json::writer::JsonArray>(&mut wire_buf);
    wire_writer
        .write(batch)
        .expect("F-4-A wire: arrow_json write must not fail");
    wire_writer
        .finish()
        .expect("F-4-A wire: arrow_json finish must not fail");
    let wire_rows: Vec<serde_json::Value> = serde_json::from_slice(&wire_buf)
        .expect("F-4-A wire: arrow_json output must parse as JSON array");
    assert!(
        wire_rows[0]
            .as_object()
            .expect("F-4-A wire: row must be a JSON object")
            .contains_key("extracted"),
        "F-4-A wire-shape (EC-11-079 null-not-absent): 'extracted' key must be PRESENT \
         in the serialized wire output. Fails when WriterBuilder uses explicit_nulls=false \
         (arrow_json default). Got wire row: {:?}",
        wire_rows[0]
    );
    assert!(
        wire_rows[0]["extracted"].is_null(),
        "F-4-A wire-shape (EC-11-079): non-object arm must serialize as JSON null. Got: {:?}",
        wire_rows[0]["extracted"]
    );
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

    // Wire-shape null-not-absent (BC-2.11.001 EC-11-079):
    // Serialize through the real production arrow_json path (prism-mcp server.rs).
    // GENUINELY FAILABLE: WriterBuilder::new() without .with_explicit_nulls(true)
    // (the arrow_json default) omits null keys, causing contains_key to return false.
    let mut wire_buf: Vec<u8> = Vec::new();
    let mut wire_writer = arrow_json::writer::WriterBuilder::new()
        .with_explicit_nulls(true)
        .build::<_, arrow_json::writer::JsonArray>(&mut wire_buf);
    wire_writer
        .write(batch)
        .expect("F-4-C wire: arrow_json write must not fail");
    wire_writer
        .finish()
        .expect("F-4-C wire: arrow_json finish must not fail");
    let wire_rows: Vec<serde_json::Value> = serde_json::from_slice(&wire_buf)
        .expect("F-4-C wire: arrow_json output must parse as JSON array");
    assert!(
        wire_rows[0]
            .as_object()
            .expect("F-4-C wire: row must be a JSON object")
            .contains_key("extracted"),
        "F-4-C wire-shape (EC-11-079 null-not-absent): 'extracted' key must be PRESENT \
         in the serialized wire output. Fails when WriterBuilder uses explicit_nulls=false \
         (arrow_json default). Got wire row: {:?}",
        wire_rows[0]
    );
    assert!(
        wire_rows[0]["extracted"].is_null(),
        "F-4-C wire-shape (EC-11-079): parse-failure arm must serialize as JSON null. Got: {:?}",
        wire_rows[0]["extracted"]
    );
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
