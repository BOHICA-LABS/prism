// File-level gate: this entire test file tests the behavior when `operations` is ABSENT.
// When built with `--all-features` (e.g. `just check`) the `operations` feature is
// enabled and these tests must be excluded — they assert catalog.len()==14 which is only
// true without the feature. `#![cfg(not(feature = "operations"))]` provides that
// exclusion. The operations-on behavior is covered by the existing tests in
// `mcp_infrastructure.rs` and `server.rs` that carry `#[cfg(feature = "operations")]`.
#![cfg(not(feature = "operations"))]

/// Red Gate tests for BC-2.10.017 — INV-OPERATIONS-FEATURE-GATE
///
/// Story: S-MCP-TOOL-GATE-001 (gate operations stubs behind default-off Cargo feature)
///
/// Four failing tests that prove the `operations` feature gate is NOT yet in effect.
/// All four tests MUST FAIL against the current (un-gated) code and MUST PASS after
/// the implementer adds the `operations` feature (AC-001..AC-005).
///
/// BC-5.38.001 density: 4 RG tests / 5 ACs = 0.80 (satisfies ≥ 0.50 threshold).
///
/// Wire-shape assertion discipline (CLAUDE.md §Wire-shape assertion discipline):
/// every test that touches an MCP-visible surface includes at least one assertion on
/// the serialized JSON bytes the LLM agent would consume.
///
/// TD-VSDD-091 compliance: comments cite BC section anchors and function names only —
/// no `server.rs:NNN` line-number cites.
use std::sync::Arc;
use std::time::Duration;

use prism_mcp::{ListCapabilitiesParams, PrismServer};
use prism_query::{
    cache::SensorResponseCache, invalidation::CacheInvalidator, write_dispatch::NullAuditWriter,
    write_pipeline::WriteExecutor,
};
use prism_security::{confirmation_token::ConfirmationTokenStore, FeatureFlagEvaluator};
use prism_sensors::registry::AdapterRegistry;
use prism_spec_engine::write_endpoint::WriteEndpointRegistry;
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolRequestParams, ClientInfo, ListToolsResult},
    ClientHandler, ServiceExt,
};
use tokio::time::timeout;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The 14 canonical live tools (BC-2.10.017 §Postconditions INV-OPERATIONS-FEATURE-GATE,
/// `LIVE_TOOLS` const in `PrismServer`).
const EXPECTED_LIVE_TOOLS: &[&str] = &[
    "query",
    "explain_query",
    "create_alias",
    "list_aliases",
    "delete_alias",
    "explain_alias",
    "confirm_action",
    "reload_config",
    "add_sensor_spec",
    "list_sensor_specs",
    "validate_config",
    "list_capabilities",
    "prism_describe",
    "check_sensor_health",
];

/// Minimal `PrismServer` with a `WriteExecutor` wired (required so that
/// `list_capabilities` returns `Ok` rather than `Err(Internal)`).
///
/// The endpoint registry is empty — no sensor write endpoints — which is sufficient
/// to let `list_capabilities` succeed: it reads `NOT_YET_AVAILABLE_TOOLS` from the
/// const and returns it via the `not_registered_tools` field regardless of which
/// endpoints are registered.
fn server_with_write_executor_for_gate_tests() -> PrismServer {
    use std::collections::BTreeMap;

    let endpoint_registry = Arc::new(WriteEndpointRegistry::new());

    // Empty FeatureFlagEvaluator: no client capabilities defined, deny-by-default.
    let org_registry = Arc::new(prism_core::OrgRegistry::new());
    org_registry
        .register(
            prism_core::OrgSlug::new("test-client").expect("valid slug"),
            prism_core::ids::OrgId::new(),
        )
        .expect("test-client registration must not conflict");

    let feature_flags = Arc::new(FeatureFlagEvaluator::new(
        BTreeMap::new(),
        Arc::clone(&org_registry),
    ));
    let confirmation_store = Arc::new(ConfirmationTokenStore::new());
    let audit_writer = Arc::new(NullAuditWriter);
    let adapter_registry = Arc::new(AdapterRegistry::new());
    let cache = Arc::new(SensorResponseCache::with_defaults());
    let cache_invalidator = Arc::new(CacheInvalidator::new(cache));

    let write_executor = Arc::new(WriteExecutor::new(
        feature_flags,
        confirmation_store,
        audit_writer,
        adapter_registry,
        endpoint_registry,
        cache_invalidator,
    ));

    PrismServer::new()
        .with_write_executor(write_executor)
        .with_org_registry(org_registry)
}

// ---------------------------------------------------------------------------
// RG-GATE-001: tool catalog length == 14 without `operations` feature
// ---------------------------------------------------------------------------

/// RG-GATE-001 — BC-2.10.017 §Postconditions INV-OPERATIONS-FEATURE-GATE:
/// `production_tool_catalog()` MUST return exactly 14 tools when the `operations`
/// Cargo feature is absent.
///
/// Failure reason in current (un-gated) code: `NOT_YET_AVAILABLE_TOOLS` stubs are
/// registered alongside `LIVE_TOOLS`, so `production_tool_catalog()` currently returns
/// 54 tools. This test fails with an assertion error until the gating is implemented.
///
/// Wire-shape coverage: also serializes `ListToolsResult` to JSON and asserts the
/// `"tools"` array length at the wire level, mirroring what the LLM agent reads.
#[test]
fn test_BC_2_10_017_tools_list_returns_14_tools_without_operations_feature() {
    let catalog = PrismServer::production_tool_catalog();

    // Struct-level assertion (pre-serialization).
    assert_eq!(
        catalog.len(),
        14,
        "RG-GATE-001 BC-2.10.017 INV-OPERATIONS-FEATURE-GATE: \
         production_tool_catalog() MUST return 14 tools when `operations` feature is absent; \
         got {} — currently fails because NOT_YET_AVAILABLE_TOOLS stubs are still registered \
         (implementation pending: add #[cfg(feature = \"operations\")] gate around stub registration)",
        catalog.len()
    );

    // Wire-shape assertion (post-serialization) — asserts the JSON bytes the LLM consumes.
    let list_result = ListToolsResult::with_all_items(catalog);
    let json = serde_json::to_value(&list_result).expect("ListToolsResult must serialize to JSON");
    let tools_arr = json["tools"]
        .as_array()
        .expect("RG-GATE-001 wire-shape: 'tools' must be a JSON array");
    assert_eq!(
        tools_arr.len(),
        14,
        "RG-GATE-001 wire-shape BC-2.10.017: serialized tools array must have 14 elements; \
         got {} — verifies the LLM-visible wire shape, not just the Rust Vec",
        tools_arr.len()
    );
}

// ---------------------------------------------------------------------------
// RG-GATE-002: list_capabilities returns empty not_registered_tools
// ---------------------------------------------------------------------------

/// RG-GATE-002 — BC-2.10.017 §Postconditions INV-OPERATIONS-FEATURE-GATE:
/// `list_capabilities` MUST return `not_registered_tools == []` when the `operations`
/// feature is absent (the NOT_YET_AVAILABLE_TOOLS const must be `&[]`).
///
/// Failure reason in current (un-gated) code: `NOT_YET_AVAILABLE_TOOLS` still holds
/// all 40 ops stubs, so the handler sets `not_registered_tools = NOT_YET_AVAILABLE_TOOLS`
/// and returns a 40-element array.  This test fails with an assertion error until the
/// gating is implemented.
///
/// Wire-shape coverage: navigates `structuredContent → results → not_registered_tools`
/// in the serialized JSON — the exact bytes sent over the MCP wire.
#[tokio::test]
async fn test_BC_2_10_017_list_capabilities_not_registered_tools_empty_without_operations_feature()
{
    let server = server_with_write_executor_for_gate_tests();

    let call_result = server
        .list_capabilities(Parameters(ListCapabilitiesParams::for_client(
            "test-client",
        )))
        .await
        .expect(
            "RG-GATE-002: list_capabilities must return Ok (WriteExecutor is wired; \
             test setup failure if Err here)",
        );

    // Navigate structured_content → results → not_registered_tools.
    let sc = call_result
        .structured_content
        .as_ref()
        .expect("RG-GATE-002: structured_content must be present on list_capabilities result");
    let body = sc.get("results").unwrap_or(sc);
    let not_registered = body.get("not_registered_tools").expect(
        "RG-GATE-002: 'not_registered_tools' key must exist in list_capabilities response \
             (BC-2.10.011 AC-011 contract)",
    );
    let arr = not_registered
        .as_array()
        .expect("RG-GATE-002: 'not_registered_tools' must be a JSON array");

    assert!(
        arr.is_empty(),
        "RG-GATE-002 BC-2.10.017 INV-OPERATIONS-FEATURE-GATE: \
         not_registered_tools MUST be [] when `operations` feature is absent; \
         got {} entries: {:?} — currently fails because NOT_YET_AVAILABLE_TOOLS still holds \
         40 ops-stub names (implementation pending)",
        arr.len(),
        arr
    );

    // Wire-shape assertion — serialize the full CallToolResult and re-navigate at the
    // wire level to confirm the JSON bytes the LLM agent receives are also correct.
    let json = serde_json::to_value(&call_result).expect("CallToolResult must serialize to JSON");
    let wire_sc = json
        .get("structuredContent")
        .expect("RG-GATE-002 wire-shape: 'structuredContent' key must be present in JSON");
    let wire_body = wire_sc.get("results").unwrap_or(wire_sc);
    let wire_arr = wire_body["not_registered_tools"].as_array().expect(
        "RG-GATE-002 wire-shape: 'not_registered_tools' must be a JSON array \
             in the serialized MCP response",
    );
    assert!(
        wire_arr.is_empty(),
        "RG-GATE-002 wire-shape BC-2.10.017: serialized not_registered_tools must be []; \
         got {} entries — verifies the LLM-visible wire bytes, not just the Rust struct",
        wire_arr.len()
    );
}

// ---------------------------------------------------------------------------
// RG-GATE-003: calling an ops tool returns -32602 InvalidParams, not -32003
// ---------------------------------------------------------------------------

/// RG-GATE-003 — BC-2.10.017 §Postconditions INV-OPERATIONS-FEATURE-GATE:
/// invoking an ops stub via the MCP wire (`tools/call` for `get_diagnostics`) MUST
/// return JSON-RPC error code -32602 (InvalidParams, "tool not found") when `operations`
/// is absent — NOT -32003 (NotImplemented) as the current stub returns.
///
/// After the `operations` gate is implemented, `get_diagnostics` will not be registered
/// at all, and the rmcp router returns -32602 ("tool not found") for any unregistered
/// tool name.  The NOT_IMPLEMENTED (-32003) stub path is unreachable in the gated build.
///
/// Failure reason in current (un-gated) code: `get_diagnostics` IS registered; its
/// handler calls `not_yet_available_msg("schedule management")` and returns -32003.
/// This test fails on the wire-shape assertion until the gating is implemented.
///
/// Wire-shape coverage: serializes the `ErrorData` and asserts `code` and `message`
/// fields at the wire level (SAP-3 + SID-2 wire-shape discipline).
#[tokio::test]
async fn test_BC_2_10_017_ops_tool_invocation_returns_invalid_params_without_operations_feature() {
    use rmcp::ServiceError;

    // DummyClientHandler: no-op client sufficient to complete the MCP handshake.
    #[derive(Debug, Clone, Default)]
    struct DummyClientHandler;
    impl ClientHandler for DummyClientHandler {
        fn get_info(&self) -> ClientInfo {
            ClientInfo::default()
        }
    }

    let (server_transport, client_transport) = tokio::io::duplex(4096);

    // Spawn PrismServer on the server-side duplex stream.
    // The server task MUST call `.waiting()` after `.serve()` returns; otherwise the
    // peer is dropped before the client can complete the MCP handshake (the client
    // sends `initialize`, server responds, but without `.waiting()` the server drops
    // the transport before the client's `initialized` notification is received →
    // BrokenPipe). `.waiting()` keeps the server alive to handle tool calls.
    let _server_handle = tokio::spawn(async move {
        if let Ok(peer) = PrismServer::new().serve(server_transport).await {
            let _ = peer.waiting().await;
        }
    });

    // Connect the client — completes the MCP handshake.
    let client = DummyClientHandler::default()
        .serve(client_transport)
        .await
        .expect(
            "RG-GATE-003: DummyClientHandler::serve must complete the MCP handshake \
             (test infrastructure failure if this panics)",
        );

    // Call `get_diagnostics` — an ops stub — via the real JSON-RPC wire.
    let result = timeout(
        Duration::from_secs(5),
        client.call_tool(CallToolRequestParams::new("get_diagnostics")),
    )
    .await
    .expect("RG-GATE-003: call_tool must return within 5s (no hang)");

    // The call MUST return an error (the tool is gated / not found).
    let err_data = match result {
        Err(ServiceError::McpError(err)) => err,
        Ok(_) => panic!(
            "RG-GATE-003 BC-2.10.017: call_tool(get_diagnostics) must return Err when \
             `operations` is absent; got Ok — tool should not be available"
        ),
        Err(other) => panic!(
            "RG-GATE-003: unexpected ServiceError variant (not McpError): {:?}",
            other
        ),
    };

    // Wire-shape assertion — serialize ErrorData and assert at the wire level.
    let err_json = serde_json::to_value(&err_data).expect("ErrorData must serialize to JSON");

    // Must NOT return -32003 (NOT_IMPLEMENTED) — that is the pre-gate stub behaviour.
    assert_ne!(
        err_json["code"].as_i64().unwrap_or(0),
        -32003,
        "RG-GATE-003 BC-2.10.017: error code MUST NOT be -32003 (NOT_IMPLEMENTED); \
         currently returns -32003 via not_yet_available_msg() — this must become -32602 \
         after the operations gate is implemented and get_diagnostics is unregistered"
    );

    // Must NOT return -32601 (METHOD_NOT_FOUND — a different protocol error).
    assert_ne!(
        err_json["code"].as_i64().unwrap_or(0),
        -32601,
        "RG-GATE-003 BC-2.10.017: error code must not be -32601 (METHOD_NOT_FOUND); \
         expected -32602 (INVALID_PARAMS / tool-not-found)"
    );

    // MUST return -32602 (INVALID_PARAMS, message: "tool not found") — the rmcp
    // router's response for any tool name not in the registered set.
    assert_eq!(
        err_json["code"].as_i64().unwrap_or(0),
        -32602,
        "RG-GATE-003 wire-shape BC-2.10.017 INV-OPERATIONS-FEATURE-GATE: \
         error code MUST be -32602 (InvalidParams / 'tool not found') when `operations` \
         is absent; got code {} — currently fails because the stub returns -32003 \
         (implementation pending: gate get_diagnostics behind #[cfg(feature = \"operations\")])",
        err_json["code"].as_i64().unwrap_or(0)
    );

    assert_eq!(
        err_json["message"].as_str().unwrap_or(""),
        "tool not found",
        "RG-GATE-003 wire-shape BC-2.10.017: error message MUST be 'tool not found' \
         (rmcp router canonical message for unregistered tool); \
         got {:?}",
        err_json["message"].as_str()
    );
}

// ---------------------------------------------------------------------------
// RG-GATE-004: all 14 LIVE_TOOLS present; no extras
// ---------------------------------------------------------------------------

/// RG-GATE-004 — BC-2.10.017 §Postconditions INV-OPERATIONS-FEATURE-GATE:
/// `production_tool_catalog()` MUST contain EXACTLY the 14 `LIVE_TOOLS` names and
/// no others when `operations` is absent.
///
/// This test verifies the positive set (all 14 present) AND the negative set
/// (no ops stubs leaked in), together establishing that `NOT_YET_AVAILABLE_TOOLS`
/// is effectively empty (observable indirectly through the catalog length).
///
/// Failure reason in current (un-gated) code: catalog currently holds 54 tools.
/// The `assert_eq!(catalog.len(), 14, ...)` assertion will fail first, proving
/// that NOT_YET_AVAILABLE_TOOLS is still non-empty.
///
/// Wire-shape coverage: serializes `ListToolsResult` and re-checks the `"tools"` array
/// at the wire level.
#[test]
fn test_BC_2_10_017_live_tools_all_present_without_operations_feature() {
    let catalog = PrismServer::production_tool_catalog();
    let catalog_names: Vec<&str> = catalog.iter().map(|t| t.name.as_ref()).collect();

    // --- Positive assertion: every expected live tool is in the catalog ---
    for &name in EXPECTED_LIVE_TOOLS {
        assert!(
            catalog_names.contains(&name),
            "RG-GATE-004 BC-2.10.017 INV-OPERATIONS-FEATURE-GATE: \
             LIVE_TOOLS name '{}' must be present in production_tool_catalog(); \
             catalog contains: {:?}",
            name,
            catalog_names
        );
    }

    // --- Negative assertion: no extra tools (ops stubs) are present ---
    assert_eq!(
        catalog.len(),
        14,
        "RG-GATE-004 BC-2.10.017 INV-OPERATIONS-FEATURE-GATE: \
         production_tool_catalog() MUST have exactly 14 entries (only LIVE_TOOLS) \
         when `operations` feature is absent; got {} entries — currently fails because \
         NOT_YET_AVAILABLE_TOOLS stubs are still registered \
         (implementation pending: gate the 40 ops stubs behind #[cfg(feature = \"operations\")])",
        catalog.len()
    );

    // Wire-shape assertion: re-confirm at the JSON level.
    let list_result = ListToolsResult::with_all_items(catalog);
    let json = serde_json::to_value(&list_result).expect("ListToolsResult must serialize to JSON");
    let tools_arr = json["tools"]
        .as_array()
        .expect("RG-GATE-004 wire-shape: 'tools' must be a JSON array");

    assert_eq!(
        tools_arr.len(),
        14,
        "RG-GATE-004 wire-shape BC-2.10.017: serialized tools JSON array must have \
         14 elements; got {}",
        tools_arr.len()
    );

    // Verify each expected tool name appears exactly once in the wire JSON.
    let wire_names: Vec<&str> = tools_arr
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();

    for &name in EXPECTED_LIVE_TOOLS {
        let count = wire_names.iter().filter(|&&n| n == name).count();
        assert_eq!(
            count, 1,
            "RG-GATE-004 wire-shape BC-2.10.017: LIVE_TOOLS name '{}' must appear \
             exactly once in the serialized tools array; found {} occurrences",
            name, count
        );
    }
}

// ---------------------------------------------------------------------------
// OBS-1 coverage-symmetry: list_capabilities via end-to-end client round-trip
// ---------------------------------------------------------------------------

/// OBS-1 / SAP-3 end-to-end reachability — BC-2.10.017 §Postconditions INV-OPERATIONS-FEATURE-GATE:
/// `list_capabilities` over a real MCP client duplex round-trip MUST return
/// `not_registered_tools == []` when the `operations` feature is absent.
///
/// RG-GATE-002 covers `list_capabilities.not_registered_tools` via a direct handler call.
/// This test adds the matching SAP-3 end-to-end coverage: it drives PrismServer from the
/// public MCP wire surface (JSON-RPC `tools/call` → `list_capabilities`) rather than from
/// an internal Rust handler invocation, proving that the empty `not_registered_tools` result
/// is reachable end-to-end and not just asserted on an internal struct.
///
/// The server is wired with a `WriteExecutor` (same as RG-GATE-002) because
/// `list_capabilities` returns `Err(Internal)` without one.
///
/// Wire-shape assertion discipline: serializes `CallToolResult` to JSON and asserts the
/// `structuredContent → results → not_registered_tools` path at the serialized-bytes level —
/// the exact envelope the LLM agent reads.
///
/// TD-VSDD-091 compliance: no `file.rs:NNN` line-number cites.
#[tokio::test]
async fn test_BC_2_10_017_list_capabilities_not_registered_tools_empty_via_end_to_end_client_roundtrip(
) {
    // DummyClientHandler: minimal no-op client to complete the MCP handshake.
    #[derive(Debug, Clone, Default)]
    struct DummyClientHandler;
    impl ClientHandler for DummyClientHandler {
        fn get_info(&self) -> ClientInfo {
            ClientInfo::default()
        }
    }

    let (server_transport, client_transport) = tokio::io::duplex(4096);

    // Spawn PrismServer WITH WriteExecutor wired — list_capabilities requires it.
    // `.waiting()` keeps the server alive to handle the tool call (same pattern as
    // RG-GATE-003 and OBS-2).
    let _server_handle = tokio::spawn(async move {
        if let Ok(peer) = server_with_write_executor_for_gate_tests()
            .serve(server_transport)
            .await
        {
            let _ = peer.waiting().await;
        }
    });

    // Complete the MCP handshake via the client side.
    let client = DummyClientHandler::default()
        .serve(client_transport)
        .await
        .expect(
            "test_BC_2_10_017_list_capabilities_not_registered_tools_empty_via_end_to_end_client_roundtrip: \
             DummyClientHandler::serve must complete MCP handshake \
             (test infrastructure failure if this panics)",
        );

    // Build arguments: { "client_id": "test-client" } — matches the org slug registered
    // in server_with_write_executor_for_gate_tests() and mirrors RG-GATE-002's call.
    let mut args: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    args.insert(
        "client_id".to_string(),
        serde_json::Value::String("test-client".to_string()),
    );

    // Call `list_capabilities` via the real JSON-RPC wire.
    let call_result = timeout(
        Duration::from_secs(5),
        client.call_tool(
            CallToolRequestParams::new("list_capabilities").with_arguments(args),
        ),
    )
    .await
    .expect(
        "test_BC_2_10_017_list_capabilities_not_registered_tools_empty_via_end_to_end_client_roundtrip: \
         call_tool must return within 5s (no hang)",
    )
    .expect(
        "test_BC_2_10_017_list_capabilities_not_registered_tools_empty_via_end_to_end_client_roundtrip: \
         list_capabilities wire call must succeed (WriteExecutor is wired)",
    );

    // Wire-shape assertion — serialize CallToolResult and navigate at the JSON-bytes level.
    // Path: structuredContent → results → not_registered_tools
    // (mirrors the wire-shape navigation in RG-GATE-002, but from the round-trip result)
    let json = serde_json::to_value(&call_result).expect("CallToolResult must serialize to JSON");

    let wire_sc = json.get("structuredContent").expect(
        "OBS-1 wire-shape BC-2.10.017: 'structuredContent' key must be present in \
             list_capabilities wire response",
    );

    let wire_body = wire_sc.get("results").unwrap_or(wire_sc);

    let wire_arr = wire_body["not_registered_tools"].as_array().expect(
        "OBS-1 wire-shape BC-2.10.017: 'not_registered_tools' must be a JSON array \
         in the serialized MCP response",
    );

    assert!(
        wire_arr.is_empty(),
        "OBS-1 wire-shape BC-2.10.017 INV-OPERATIONS-FEATURE-GATE: \
         serialized not_registered_tools MUST be [] when `operations` feature is absent; \
         got {} entries — SAP-3 end-to-end reachability proof from the MCP wire surface",
        wire_arr.len()
    );
}

// ---------------------------------------------------------------------------
// OBS-2 defence-in-depth: real MCP client round-trip via duplex transport
// ---------------------------------------------------------------------------

/// OBS-2 / SAP-3 end-to-end reachability — BC-2.10.017 §Postconditions INV-OPERATIONS-FEATURE-GATE:
/// `tools/list` over a real MCP client duplex round-trip MUST return exactly 14 tools when
/// the `operations` feature is absent.
///
/// RG-GATE-001 and RG-GATE-004 cover `production_tool_catalog()` via the internal Rust API.
/// This test is the SAP-3 end-to-end coverage for the same AC: it drives PrismServer from the
/// public MCP wire surface (JSON-RPC `tools/list`) rather than from internal API calls, proving
/// the catalog count is reachable end-to-end and not just asserted on an internal struct.
///
/// Wire-shape assertion discipline: serializes `ListToolsResult` to JSON and asserts the
/// `"tools"` array length at the serialized-bytes level — the exact envelope the LLM agent reads.
///
/// TD-VSDD-091 compliance: no `file.rs:NNN` line-number cites.
#[tokio::test]
async fn test_BC_2_10_017_tools_list_14_via_end_to_end_client_roundtrip() {
    // DummyClientHandler: minimal no-op client to complete the MCP handshake.
    #[derive(Debug, Clone, Default)]
    struct DummyClientHandler;
    impl ClientHandler for DummyClientHandler {
        fn get_info(&self) -> ClientInfo {
            ClientInfo::default()
        }
    }

    let (server_transport, client_transport) = tokio::io::duplex(4096);

    // Spawn PrismServer on the server side. `.waiting()` keeps it alive until the
    // client completes its request (same pattern as RG-GATE-003).
    let _server_handle = tokio::spawn(async move {
        if let Ok(peer) = PrismServer::new().serve(server_transport).await {
            let _ = peer.waiting().await;
        }
    });

    // Complete the MCP handshake via the client side.
    let client = DummyClientHandler::default()
        .serve(client_transport)
        .await
        .expect(
            "test_BC_2_10_017_tools_list_14_via_end_to_end_client_roundtrip: \
             DummyClientHandler::serve must complete MCP handshake \
             (test infrastructure failure if this panics)",
        );

    // Call `tools/list` via the real JSON-RPC wire (RunningService Derefs to Peer).
    let list_result = timeout(Duration::from_secs(5), client.list_tools(None))
        .await
        .expect(
            "test_BC_2_10_017_tools_list_14_via_end_to_end_client_roundtrip: \
         list_tools must return within 5s (no hang)",
        )
        .expect(
            "test_BC_2_10_017_tools_list_14_via_end_to_end_client_roundtrip: \
         list_tools wire call must succeed (no MCP error)",
        );

    // Struct-level assertion: 14 tools returned over the wire.
    assert_eq!(
        list_result.tools.len(),
        14,
        "OBS-2 BC-2.10.017 INV-OPERATIONS-FEATURE-GATE: \
         tools/list wire response must return exactly 14 tools when `operations` is absent; \
         got {} — SAP-3 end-to-end reachability proof from the MCP wire surface",
        list_result.tools.len()
    );

    // Wire-shape assertion — serialize ListToolsResult and assert the `"tools"` array
    // length at the JSON-bytes level (the exact envelope the LLM agent consumes).
    let json = serde_json::to_value(&list_result)
        .expect("test_BC_2_10_017_e2e: ListToolsResult must serialize to JSON");
    let tools_arr = json["tools"]
        .as_array()
        .expect("test_BC_2_10_017_e2e wire-shape: 'tools' must be a JSON array");
    assert_eq!(
        tools_arr.len(),
        14,
        "OBS-2 wire-shape BC-2.10.017: serialized 'tools' JSON array must have 14 elements; \
         got {} — verifies the LLM-visible wire bytes, not just the Rust struct",
        tools_arr.len()
    );

    // Verify all 14 LIVE_TOOLS names are present in the wire JSON.
    let wire_names: Vec<&str> = tools_arr
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();
    for &name in EXPECTED_LIVE_TOOLS {
        assert!(
            wire_names.contains(&name),
            "OBS-2 wire-shape BC-2.10.017: LIVE_TOOLS name '{}' must appear in \
             tools/list wire response; tools present: {:?}",
            name,
            wire_names
        );
    }
}
