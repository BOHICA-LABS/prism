# Demo Evidence Report — S-MCP-TOOL-GATE-001

**Story:** Gate 40 operations stubs behind default-off Cargo feature to eliminate -32003 catalog pollution
**Feature branch HEAD:** `6a0986ace`
**Recorded:** 2026-09-18
**Method:** Real MCP stdio JSON-RPC (genuine binary output; no fabrication)
**Build:** `cargo build -p prism-bin --bin prism` (default, operations OFF); also `--features prism-mcp/operations` for AC-005 contrast

---

## Coverage Summary

| AC | Description | Status | Artifact |
|----|-------------|--------|---------|
| AC-001 | `tools/list` returns exactly 14 tools (default build) | PASS | `capture-default-tools_list.json` |
| AC-002 | `list_capabilities.not_registered_tools == []` (default build) | PASS | `capture-default-list_capabilities.json` |
| AC-003 | Invoking `get_diagnostics` returns `-32602` NOT `-32003` (default build) | PASS | `capture-default-get_diagnostics.json` |
| AC-004 | All 14 LIVE_TOOLS remain registered and non-gated | PASS | `capture-default-tools_list.json` |
| AC-005 | Contrast: ops build returns 54 tools + `-32003` for `get_diagnostics` | PASS | `capture-operations-tools_list.json`, `capture-operations-get_diagnostics.json` |

All artifacts are genuine captured wire output from real MCP stdio JSON-RPC sessions.

---

## AC-001: `tools/list` Returns Exactly 14 Tools (Default Build)

**Acceptance criterion:** When compiled WITHOUT the `operations` feature (default), `tools/list` returns exactly 14 tools in `result.tools`.

**Wire assertion:** `result.tools.len() == 14` on the serialized JSON response.

**Artifact:** `capture-default-tools_list.json`

**Key extracted fields from wire response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      "<14 entries — see artifact for full list>"
    ]
  }
}
```

**Verified 14 tool names (from wire):**
- `add_sensor_spec`, `check_sensor_health`, `confirm_action`, `create_alias`, `delete_alias`,
  `explain_alias`, `explain_query`, `list_aliases`, `list_capabilities`, `list_sensor_specs`,
  `prism_describe`, `query`, `reload_config`, `validate_config`

**Result:** PASS — `result.tools.len() == 14` confirmed on wire.

---

## AC-002: `list_capabilities.not_registered_tools` is Empty (Default Build)

**Acceptance criterion:** When compiled without `operations`, `list_capabilities(client_id: "demo-client")` returns `not_registered_tools: []` in the wire response.

**Wire assertion:** `structuredContent.results.not_registered_tools == []` in serialized JSON.

**Artifact:** `capture-default-list_capabilities.json`

**Key extracted fields from wire response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "structuredContent": {
      "results": {
        "not_registered_tools": [],
        "client_id": "demo-client",
        "client_registered": false,
        "capabilities": {}
      }
    }
  }
}
```

**Result:** PASS — `structuredContent.results.not_registered_tools == []` confirmed on wire.

---

## AC-003: Invoking `get_diagnostics` Returns `-32602` NOT `-32003` (Default Build)

**Acceptance criterion:** When compiled without `operations`, calling `tools/call` with `name: "get_diagnostics"` returns MCP error code `-32602` (InvalidParams, message `"tool not found"`). MUST NOT return `-32003`.

**Wire assertion:** `error.code == -32602` and `error.message == "tool not found"` on the serialized JSON error envelope.

**Artifact:** `capture-default-get_diagnostics.json`

**Wire response (exact):**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "error": {
    "code": -32602,
    "message": "tool not found"
  }
}
```

**Result:** PASS — `error.code == -32602`, `error.message == "tool not found"`. Confirmed NOT `-32003` and NOT `-32601`.

Also captured `create_schedule` (another previously-stubbed ops tool) at `capture-default-create_schedule.json`:
- **Wire:** `error.code == -32602`, `error.message == "tool not found"` — same correct behavior.

---

## AC-004: All 14 LIVE_TOOLS Remain Registered (Default Build)

**Acceptance criterion:** The 14 LIVE_TOOLS are registered unconditionally regardless of `operations` feature state. No ops stubs leak into the catalog.

**Evidence:** Same `capture-default-tools_list.json` artifact as AC-001. The wire response confirms:
- `result.tools.len() == 14` (no ops stubs leaked in; ops stubs would push catalog above 14)
- All 14 expected LIVE_TOOL names are present in the wire response (positive set confirmed)
- `prism_describe`, `list_capabilities`, `query`, `check_sensor_health` are all present — BC-2.10.012 protection boundary satisfied

**Result:** PASS — all 14 LIVE_TOOLS registered; catalog count == 14 (no leakage of 40 ops stubs).

---

## AC-005: Contrast — Operations Build Returns 54 Tools + `-32003` for `get_diagnostics`

**Acceptance criterion:** When built with `--features prism-mcp/operations`, `tools/list` shows 54 tools and `get_diagnostics` returns `-32003` (fast-fail preserved; unchanged from pre-feature-gate behavior).

**Build used for contrast:** `cargo build -p prism-bin --bin prism --features prism-mcp/operations`

**AC-005 Part A: tools/list with operations feature**

**Artifact:** `capture-operations-tools_list.json`

**Wire response summary:**
- `result.tools.len() == 54` (14 LIVE_TOOLS + 40 ops stubs)
- All 40 ops tool names present in catalog including `get_diagnostics`, `create_schedule`, etc.

**AC-005 Part B: get_diagnostics with operations feature**

**Artifact:** `capture-operations-get_diagnostics.json`

**Wire response (exact):**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "error": {
    "code": -32003,
    "message": "Feature not yet available: sensor diagnostics — adapter registry empty (GAP-002-A; full sensor adapter dispatch wires in S-5.04-SENSOR-HEALTH-ADAPTER-DISPATCH) (prism-operations not merged)"
  }
}
```

**list_capabilities with operations feature:**

**Artifact:** `capture-operations-list_capabilities.json`

Wire confirms `not_registered_tools` array has 40 entries (all ops stub names).

**AC-005 Contrast Summary:**

| Metric | Default (operations OFF) | Operations ON |
|--------|--------------------------|---------------|
| `tools/list` count | 14 | 54 |
| `not_registered_tools` count | 0 | 40 |
| `get_diagnostics` error code | -32602 ("tool not found") | -32003 (fast-fail) |

**Result:** PASS — gate behavior confirmed. Default build is clean (14 tools, empty not_registered_tools, -32602 for unknown tools). Operations build preserves pre-gate behavior (54 tools, 40 not_registered_tools, -32003 fast-fail).

---

## Artifact Index

| File | Content | Size |
|------|---------|------|
| `capture-default-tools_list.json` | `tools/list` MCP wire response, default build (14 tools) | ~84KB |
| `capture-default-list_capabilities.json` | `list_capabilities` MCP wire response, default build (`not_registered_tools: []`) | ~10KB |
| `capture-default-get_diagnostics.json` | `tools/call get_diagnostics` MCP wire response, default build (error -32602) | ~1KB |
| `capture-default-create_schedule.json` | `tools/call create_schedule` MCP wire response, default build (error -32602) | ~1KB |
| `capture-operations-tools_list.json` | `tools/list` MCP wire response, operations build (54 tools) | ~84KB |
| `capture-operations-list_capabilities.json` | `list_capabilities` MCP wire response, operations build (`not_registered_tools: [40 entries]`) | ~12KB |
| `capture-operations-get_diagnostics.json` | `tools/call get_diagnostics` MCP wire response, operations build (error -32003) | ~1KB |
| `capture-operations-create_schedule.json` | `tools/call create_schedule` MCP wire response, operations build | ~1KB |
| `capture-summary.json` | Extracted key fields from all captures (machine-readable summary) | ~5KB |
| `evidence-report.md` | This file | — |

---

## Fabrication Attestation

All artifacts in this directory are genuine captured wire output from real MCP stdio JSON-RPC sessions against the built `prism` binary at feature branch HEAD `6a0986ace`. No artifact is fabricated, mocked, hand-written, or derived from test harness output. Captured by demo-recorder agent on 2026-09-18.
