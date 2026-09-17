//! Serialized-wire null-not-absent assertion for json_extract_string via the real MCP query tool.
//!
//! This test covers BC-2.11.025 EC-11-025-011 AC-011 wire-shape requirement:
//! SQL NULL produced by `json_extract_string` must serialize as JSON `null` (null-NOT-absent)
//! in the MCP query tool response envelope — exercised through the real `server.query()` path
//! which uses `arrow_json::WriterBuilder::new().with_explicit_nulls(true)`.
//!
//! # Why this test lives in prism-mcp (not prism-query)
//!
//! ADR-066 §B2 and S-JSON-EXTRACT-UDF-001 §Forbidden Dependencies ban `arrow-json` from
//! prism-query's dependency graph. The real serializer lives in `prism-mcp::server`
//! (`handle_query_tool` in server.rs). This test exercises that serializer end-to-end by
//! calling `server.query()` directly — making the wire assertion genuinely failable:
//! if `explicit_nulls=false` (the arrow_json DEFAULT) were used, SQL NULL cells would be
//! ABSENT from the serialized row objects and `contains_key("extracted")` would fail,
//! catching the [C3]/[H20] / DEFECT-MCP-ROWSHAPE-NULLS-001 defect class.
//!
//! # Traceability
//!
//! | Test | BC clause | Defect class guarded |
//! |------|-----------|---------------------|
//! | test_BC_2_11_025_json_extract_string_null_row_wire_null_not_absent | EC-11-025-011 AC-011 + EC-11-079 | arrow_json explicit_nulls=false ([C3]/[H20]) |
//!
//! Story: S-JSON-EXTRACT-UDF-001 v1.5 | BC: BC-2.11.025 v1.9 | ADR: ADR-066 v1.6

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use arrow::{
        array::StringArray,
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use async_trait::async_trait;
    use prism_core::{OrgId, OrgSlug, SensorId};
    use prism_credentials::InMemoryCredentialStore;
    use prism_mcp::server::{PrismServer, QueryToolParams};
    use prism_query::{
        engine::{QueryEngine, QueryEngineConfig},
        scoping::ClientRegistry,
        table_registry::TableRegistry,
    };
    use prism_sensors::{
        adapter::FetchOutput, AdapterRegistry, CredentialResolver,
        QueryParams as SensorQueryParams, SensorAdapter, SensorAuth, SensorError,
        SensorSpec as SensorAdapterSpec,
    };
    use prism_spec_engine::{
        overlay::{OverlayLoader, ResolvedSensorSpec, ResolvedSpecKey, SensorInstanceOverlay},
        spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
    };
    use rmcp::handler::server::wrapper::Parameters;

    // =========================================================================
    // Stub types (mirrors bc_2_11_001_null_row_shape_test.rs pattern)
    // =========================================================================

    struct StubAuth;
    impl SensorAuth for StubAuth {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn auth_type_name(&self) -> &'static str {
            "stub"
        }
    }

    /// Credential resolver that always succeeds (returns StubAuth).
    ///
    /// Required so fan_out() reaches the adapter boundary rather than short-circuiting
    /// with a CredentialNotFound error. Pattern mirrors AlwaysSucceedsCreds in
    /// bc_2_11_001_null_row_shape_test.rs (SID-1 compliance).
    struct AlwaysSucceedsCreds;
    impl CredentialResolver for AlwaysSucceedsCreds {
        fn resolve(
            &self,
            _client_id: &str,
            _sensor_id: SensorId,
        ) -> Result<Box<dyn SensorAuth>, SensorError> {
            Ok(Box::new(StubAuth))
        }
    }

    /// Mock adapter returning 3 rows of JSON payloads in a `payload` (Utf8, nullable) column:
    ///
    /// - Row 0: `{"severity":"critical","host":"server01"}` → json_extract_string → "critical"
    /// - Row 1: `{"severity":null,"host":"server02"}`       → json_extract_string → SQL NULL
    ///                                                          (JSON null at key)
    /// - Row 2: `{"host":"server03"}`                       → json_extract_string → SQL NULL
    ///                                                          (key absent)
    ///
    /// Rows 1 and 2 are the null-producing arms: after serialization through
    /// arrow_json::WriterBuilder::with_explicit_nulls(true) the `extracted` key must be
    /// PRESENT with a JSON null value. With explicit_nulls=false (the arrow_json DEFAULT),
    /// the key would be ABSENT — the [C3]/[H20] defect class.
    struct JexMcpAdapter {
        sensor_id: SensorId,
    }

    impl std::fmt::Debug for JexMcpAdapter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("JexMcpAdapter").finish()
        }
    }

    #[async_trait]
    impl SensorAdapter for JexMcpAdapter {
        fn sensor_type(&self) -> SensorId {
            self.sensor_id.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "jex-mcp-wire-stub"
        }

        async fn fetch(
            &self,
            _spec: &SensorAdapterSpec,
            _params: &SensorQueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<FetchOutput, SensorError> {
            let schema = Arc::new(Schema::new(vec![Field::new(
                "payload",
                DataType::Utf8,
                true,
            )]));
            let payloads: Vec<Option<&str>> = vec![
                Some(r#"{"severity":"critical","host":"server01"}"#), // row 0: happy path
                Some(r#"{"severity":null,"host":"server02"}"#),       // row 1: JSON null at key
                Some(r#"{"host":"server03"}"#),                       // row 2: key absent
            ];
            let arr = StringArray::from(payloads);
            let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
                .expect("JexMcpAdapter: RecordBatch construction must succeed");
            Ok(FetchOutput::new(vec![batch], false, false))
        }
    }

    // =========================================================================
    // Fixture helpers (structurally identical to bc_2_11_001 make_resolved)
    // =========================================================================

    fn make_resolved(
        sensor_id: &str,
        table_name: &str,
        columns: Vec<ColumnSpec>,
        org: &str,
    ) -> (ResolvedSpecKey, ResolvedSensorSpec) {
        let spec = SensorSpec::new(
            sensor_id,
            format!("{sensor_id} sensor"),
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                table_name,
                "security_finding",
                columns,
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );
        let overlay_toml =
            format!("extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@{org}\"");
        let overlay: SensorInstanceOverlay =
            toml::from_str(&overlay_toml).expect("overlay TOML must parse");
        let org_slug = OrgSlug::new(org);
        let resolved =
            OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug.clone());
        let sensor_id_typed = SensorId::new(sensor_id);
        let key: ResolvedSpecKey = (org_slug, sensor_id_typed);
        (key, resolved)
    }

    /// Build a `PrismServer` wired with `JexMcpAdapter` for the `jex_events` table.
    ///
    /// sensor_id="jex" + table_name="events" → DataFusion table name "jex_events"
    /// (formed by TableRegistry as "{sensor_id}_{table_name}").
    ///
    /// The `payload: String` column spec satisfies the E-QUERY-038 column gate for the
    /// query `SELECT json_extract_string(payload, 'severity') AS extracted FROM jex_events`.
    ///
    /// Structurally identical to `make_server_with_returning_null_adapter` in
    /// `bc_2_11_001_null_row_shape_test.rs`.
    fn make_jex_mcp_server() -> PrismServer {
        use prism_core::column::ColumnType;

        let sensor_id_str = "jex";
        let table_name = "events"; // → DataFusion table "jex_events"
        let org = "acme";

        let columns = vec![ColumnSpec::new("payload", ColumnType::String, None, vec![])];

        // Deterministic OrgId (sentinel byte: 0x4a = 'J' for "Json extract wire")
        let org_id = OrgId::from_uuid(uuid::Uuid::from_bytes([
            0x01, 0x9f, 0x3a, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x4a,
        ]));

        let (key, resolved) = make_resolved(sensor_id_str, table_name, columns, org);
        let mut resolved_map = HashMap::new();
        resolved_map.insert(key, resolved.clone());

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&resolved.spec)
            .expect("register_sensor must not fail in fixture");

        let sensor_id_typed = SensorId::new(sensor_id_str);
        let jex_adapter: Arc<dyn SensorAdapter> = Arc::new(JexMcpAdapter {
            sensor_id: sensor_id_typed,
        });
        let mut adapter_registry = AdapterRegistry::new();
        adapter_registry.register(org_id, jex_adapter);

        let mut engine = QueryEngine::new_with_cache_config(
            Arc::new(adapter_registry),
            Arc::new(InMemoryCredentialStore::new()),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            prism_query::cache::CacheConfig::default(),
        );
        engine = engine.with_credential_resolver(Arc::new(AlwaysSucceedsCreds));
        engine = engine.with_resolved_spec_map(Arc::new(resolved_map));
        engine = engine.with_table_registry(registry);

        PrismServer::new().with_query_engine(Arc::new(engine))
    }

    fn query_params(sql: &str) -> QueryToolParams {
        serde_json::from_str(&serde_json::json!({"query": sql}).to_string())
            .expect("QueryToolParams JSON must deserialize")
    }

    fn envelope_json(result: rmcp::model::CallToolResult) -> serde_json::Value {
        result
            .structured_content
            .expect("query must return structured_content (not an error path)")
    }

    // =========================================================================
    // Test
    // =========================================================================

    /// BC-2.11.025 EC-11-025-011 AC-011 wire-shape: json_extract_string SQL NULL rows
    /// serialize as JSON null (null-NOT-absent) in the MCP query tool response envelope.
    ///
    /// Full test path:
    ///   server.query() → QueryEngine::execute → DataFusion UDF →
    ///   arrow_json::WriterBuilder::with_explicit_nulls(true) → serialized row JSON
    ///
    /// GENUINELY FAILABLE (BC-2.11.001 EC-11-079 null-not-absent):
    ///   `arrow_json::WriterBuilder::new()` uses `explicit_nulls=false` by DEFAULT, which
    ///   OMITS null-valued keys from the serialized row object (producing `{}` instead of
    ///   `{"extracted":null}`). The `contains_key("extracted")` assertion FAILS under that
    ///   default, catching the [C3]/[H20] / DEFECT-MCP-ROWSHAPE-NULLS-001 defect class
    ///   (live-audit 2026-07-13, CLAUDE.md wire-shape discipline).
    ///
    /// The prism-query tests (test_jex_rg011*, test_jex_f4_*) cover the Arrow-level
    /// is_null assertions. This test covers the serialized-wire null-NOT-absent guarantee
    /// at the prism-mcp production serializer boundary.
    ///
    /// BC: BC-2.11.025 EC-11-025-011 | Story: S-JSON-EXTRACT-UDF-001 AC-011
    #[tokio::test]
    async fn test_BC_2_11_025_json_extract_string_null_row_wire_null_not_absent() {
        let server = make_jex_mcp_server();
        let result = server
            .query(Parameters(query_params(
                "SELECT json_extract_string(payload, 'severity') AS extracted FROM jex_events",
            )))
            .await
            .expect("server.query must return Ok (MCP tool calls always return Ok)");

        let v = envelope_json(result);
        let rows = v["results"]["rows"]
            .as_array()
            .expect("results.rows must be a JSON array");

        assert_eq!(
            rows.len(),
            3,
            "BC-2.11.025 AC-011 wire: JexMcpAdapter returns 3 rows; got {}",
            rows.len()
        );

        // Row 0: {"severity":"critical"} → json_extract_string → "critical" (sanity check)
        assert_eq!(
            rows[0]["extracted"], "critical",
            "BC-2.11.025 AC-011 wire: row 0 'extracted' must be \"critical\""
        );

        // Row 1: {"severity":null} → json_extract_string → SQL NULL →
        // serialized wire: "extracted" key PRESENT with JSON null (null-NOT-absent).
        //
        // GENUINELY FAILABLE: with explicit_nulls=false (arrow_json DEFAULT), the
        // "extracted" key is ABSENT from the row object; get("extracted") returns None
        // and this assertion fails, catching the [C3]/[H20] defect class.
        assert!(
            rows[1].get("extracted").is_some(),
            "BC-2.11.025 AC-011 wire (EC-11-079 null-not-absent): row 1 (JSON null at \
             'severity' key → SQL NULL) must have 'extracted' key PRESENT in the MCP \
             response envelope. FAILS when arrow_json::WriterBuilder uses \
             explicit_nulls=false (the default). Got row 1: {:?}",
            rows[1]
        );
        assert!(
            rows[1]["extracted"].is_null(),
            "BC-2.11.025 AC-011 wire: row 1 'extracted' must be JSON null (not the string \
             \"null\" or any other value). Got: {:?}",
            rows[1]["extracted"]
        );

        // Row 2: {"host":"server03"} (no 'severity' key) → json_extract_string → SQL NULL →
        // serialized wire: "extracted" key PRESENT with JSON null (null-NOT-absent).
        assert!(
            rows[2].get("extracted").is_some(),
            "BC-2.11.025 AC-011 wire (EC-11-079 null-not-absent): row 2 (absent 'severity' \
             key → SQL NULL) must have 'extracted' key PRESENT in the MCP response envelope. \
             FAILS when arrow_json::WriterBuilder uses explicit_nulls=false (the default). \
             Got row 2: {:?}",
            rows[2]
        );
        assert!(
            rows[2]["extracted"].is_null(),
            "BC-2.11.025 AC-011 wire: row 2 'extracted' must be JSON null. Got: {:?}",
            rows[2]["extracted"]
        );
    }
}
