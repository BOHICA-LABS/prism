---
document_type: behavioral-contract
level: L3
version: "1.4"
status: active
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-10"
capability: "CAP-034"
lifecycle_status: active
introduced: demo-readiness-2026-06-24
modified: "2026-09-18"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-046-three-mode-correctness-filter-sql-pipe-mode-bridge-error-and-execution-validation.md"
input-hash: "883049c"
traces_to: ["CAP-034"]
extracted_from: null
---

# BC-2.10.017: Not-Yet-Available Tools Fast-Fail — Audit Channel Non-Blocking

## Description

When the `operations` Cargo feature is enabled, tools in the `NOT_YET_AVAILABLE_TOOLS` set MUST return a fast-fail JSON-RPC error response within 1 second of the tool invocation request arriving at the MCP server. When the `operations` feature is absent (the production default), `NOT_YET_AVAILABLE_TOOLS` is an empty slice `&[]` — no stub tools are registered in the MCP catalog and no `-32003` responses are generated; previously-stubbed tool names are simply unknown to the router. The audit channel write (`emit_tool_audit`) MUST NOT be on the blocking path before the fast-fail guard fires (when `operations` is enabled); the guard MUST fire before any audit emission for tools in this set.

## Preconditions

- A `tools/call` request arrives for a tool whose name is in the `NOT_YET_AVAILABLE_TOOLS` set
- The MCP server has an `audit_writer: Option<Arc<dyn AuditWriter>>` wired (or `None` in test construction); there is no `mpsc::Sender<AuditEntry>` on the audit path

## Postconditions

- **When the `operations` Cargo feature is ABSENT (default — production state):** `NOT_YET_AVAILABLE_TOOLS` is the empty slice `&[]`. No stub operations tools are registered in the MCP catalog; `tools/list` reports only the 14 `LIVE_TOOLS`. Invocations of previously-stubbed tool names return the rmcp standard tool-not-found error (`-32602` InvalidParams, message `"tool not found"`) — not the prism-specific `-32003` fast-fail. `emit_tool_audit` is never reached for these invocations because no handler exists for the tool. (S-MCP-TOOL-GATE-001 AC-001/AC-003/AC-005 — RG-GATE-001, RG-GATE-003)

- **When the `operations` Cargo feature is ENABLED (opt-in):** `NOT_YET_AVAILABLE_TOOLS` is populated with the ~40 stub tool names. The following postconditions apply in this state:
  - The tool invocation returns a structured JSON-RPC `-32003` error response within **1 second** of request receipt
  - The response body uses the `not_yet_available_msg` pattern: a structured JSON-RPC error with code `-32003`, message indicating the tool is not yet available, and a `content[].text` with human-readable guidance
  - The NOT_YET_AVAILABLE fast-fail guard fires BEFORE `emit_tool_audit` is called — the handlers for `list_infusions`, `infusion_status`, and `plugin_status` return `Err(not_yet_available_msg(...))` directly WITHOUT ever calling `emit_tool_audit` (Option A per original contract). No audit event is emitted for a not-yet-available tool invocation because no tool execution occurred and nothing needs to be audited.
  - `emit_tool_audit` is an `async fn` that `.await`s `AuditWriter::write_tool_call` on `Option<Arc<dyn AuditWriter>>`. There is no `mpsc::Sender` channel and no `try_send` call on the audit path. The pre-D-1110 hypothesis of using `try_send` as a non-blocking mechanism (Option B) was superseded — the shipped implementation achieves non-blocking behavior for NOT_YET_AVAILABLE tools by never reaching `emit_tool_audit` at all.

## Invariants

- **INV-NOT-YET-AVAILABLE-GUARD-ORDER:** For tools in `NOT_YET_AVAILABLE_TOOLS`, the guard check MUST evaluate before any blocking audit `.await` in the tool dispatch path. The shipped implementation satisfies this invariant by placing the fast-fail `return Err(not_yet_available_msg(...))` at the start of each handler body — `emit_tool_audit` is never reached for these tools.
- **INV-AUDIT-NON-BLOCKING:** (D-1110 reconciliation) The audit path uses `Arc<dyn AuditWriter>::write_tool_call(...).await` — there is no `mpsc::Sender` channel and no `try_send` call. For NOT_YET_AVAILABLE tools specifically, the non-blocking invariant is satisfied by the guard-reorder: `emit_tool_audit` is not called at all, so the async audit write cannot block the fast-fail response regardless of `AuditWriter` implementation.
- The set `NOT_YET_AVAILABLE_TOOLS` is a compile-time constant — not configurable at runtime
- **INV-OPERATIONS-FEATURE-GATE:** When the `operations` Cargo feature is absent (the default), `NOT_YET_AVAILABLE_TOOLS` is the empty slice `&[]` — no stub tools are registered in the MCP catalog, `tools/list` reports only the 14 `LIVE_TOOLS`, and invocations of previously-stubbed tool names return the rmcp standard tool-not-found error (`-32602` InvalidParams, message `"tool not found"`), NOT the prism-specific `-32003` fast-fail. When `operations` is enabled, `NOT_YET_AVAILABLE_TOOLS` is populated with stub tool names; stub tools are registered and visible in `tools/list`; invocations return `-32003`. The 14 `LIVE_TOOLS` MUST remain unconditionally registered in both feature states — the `operations` gate applies exclusively to the operations impl block. Verified by S-MCP-TOOL-GATE-001 AC-001/AC-003/AC-004 (RG-GATE-001, RG-GATE-003, RG-GATE-004).

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| JSON-RPC `-32003` | `operations` feature ENABLED + Tool in `NOT_YET_AVAILABLE_TOOLS` is invoked | Fast-fail within 1s: `{"code": -32003, "message": "Tool '{tool_name}' is not yet available in this release. …"}` |
| JSON-RPC `-32602` | `operations` feature ABSENT (default) + Previously-stubbed tool name (e.g., `get_diagnostics`) is invoked | rmcp standard tool-not-found: `{"code": -32602, "message": "tool not found"}` — the tool was never registered; no stub exists in the MCP catalog; NOT `-32003` |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-017-001 | `list_infusions` called when `AuditWriter::write_tool_call` would block (slow AuditWriter) | Returns `-32003` within 1s; the NOT_YET_AVAILABLE guard fires BEFORE `emit_tool_audit`, so the async audit write is never reached; no audit drop warning (`emit_tool_audit` not called); NO hang |
| EC-10-017-002 | `plugin_status` called with valid `plugin_name` argument | Returns `-32003` fast-fail within 1s; argument value ignored |
| EC-10-017-003 | `infusion_status` called | Returns `-32003` fast-fail within 1s |
| EC-10-017-004 | Concurrent invocations of `list_infusions` (e.g., 5 simultaneous) | All return `-32003` within 1s each; no head-of-line blocking |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

Note: These vectors cover two mutually exclusive compilation states. Rows marked `operations-enabled` require `--features operations`; rows marked `operations-absent` apply to the default production build.

| Input | Compilation State | Expected Output | Category |
|-------|------------------|----------------|----------|
| `tools/call list_infusions` | operations-enabled | JSON-RPC error `-32003` within 1s | happy-path (fast-fail) |
| `tools/call plugin_status {"plugin_name": "crowdstrike-oauth2.prx"}` | operations-enabled | JSON-RPC error `-32003` within 1s | happy-path (fast-fail) |
| `tools/call infusion_status {"infusion_name": "threat_score"}` | operations-enabled | JSON-RPC error `-32003` within 1s | happy-path (fast-fail) |
| `tools/call get_diagnostics` | operations-absent (default) | JSON-RPC error `{"code": -32602, "message": "tool not found"}` — NOT `-32003` | happy-path (feature-gate) |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| (none allocated) | Fast-fail within 1s | integration test (timing assertion) |

## Related BCs

- **BC-2.10.011** (related — list_capabilities: NOT in NOT_YET_AVAILABLE set): `list_capabilities` is always-available; this BC governs a different set of tools
- **BC-2.10.009** (unrelated — prompts): prompt fast-fail is governed by BC-2.10.016; this BC governs tool fast-fail

## Architecture Anchors

- `crates/prism-mcp/src/server.rs` — `emit_tool_audit` async fn: signature `async fn emit_tool_audit(audit_writer: Option<&Arc<dyn AuditWriter>>, tool: &str, client_id: Option<&str>, outcome: &str) -> Result<Option<String>, rmcp::model::ErrorData>`; uses `.await` on `AuditWriter::write_tool_call`; no mpsc/try_send path exists
- `crates/prism-mcp/src/server.rs` — `NOT_YET_AVAILABLE_TOOLS` compile-time constant; `list_infusions`, `infusion_status`, and `plugin_status` handlers each call `Err(not_yet_available_msg(...))` as their first and only statement — `emit_tool_audit` is never reached (INV-NOT-YET-AVAILABLE-GUARD-ORDER satisfied by construction)
- ADR-046 §BLOCKER-004 root-cause (original hypothesis was `emit_tool_audit` blocking `send()`; D-1110 established the shipped fix is Option A guard-reorder)

## Story Anchor

TBD

## VP Anchors

(none allocated; timing test is integration test scope)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| Capability Anchor Justification | CAP-034 ("MCP Server & Transport") per capabilities.md §CAP-034 — this BC governs the tool dispatch middleware in `PrismServer`. CAP-034 describes "Tool dispatch errors surface as structured MCP error responses, never as raw panics" and the middleware layer on every tool dispatch. Not-yet-available tools are a class of conditional tool registration that the MCP server must handle without blocking. |
| L2 Invariants | DI-004 (audit completeness — audit MUST NOT block the fast-fail path; guard-reorder (Option A) satisfies this: `emit_tool_audit` is never reached for NOT_YET_AVAILABLE tools) |
| Priority | P0 |
| Closes findings | BLOCKER-004 (`list_infusions`, `plugin_status`, `infusion_status` hang indefinitely) |
| ADR traces | ADR-046 (root-cause hypothesis for BLOCKER-004) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | S-MCP-TOOL-GATE-001-LOCAL-pass-1-OBS-3 | 2026-09-18 | product-owner | Additive table completeness. §Error Cases: added operations-ABSENT row (`operations` feature absent + previously-stubbed tool name invoked → MCP −32602 `"tool not found"`, NOT −32003). §Canonical Test Vectors: added compilation-state column and operations-absent row (`tools/call get_diagnostics` operations-absent → `{code: -32602, message: "tool not found"}`). Tables now enumerate both compilation states (operations-enabled → −32003; operations-absent/default → −32602), consistent with §Postconditions and INV-OPERATIONS-FEATURE-GATE (already correct at v1.3). No behavioral change — RG-GATE-003 already tests −32602. Pre-existing duplicate v1.1 changelog rows merged into single row. Closes S-MCP-TOOL-GATE-001 LOCAL pass-1 OBS-3. |
| 1.3 | beta3-remediation-BC-2.10.017-v1.3 | 2026-09-16 | product-owner | Factual correction: −32601 (MethodNotFound) → −32602 (InvalidParams, `"tool not found"`). §Postconditions (absent-feature path) and INV-OPERATIONS-FEATURE-GATE updated to reflect rmcp 1.7.0 `handler/server/router/tool.rs` → `ErrorData::invalid_params("tool not found")` and prism `error_mapping.rs` tool-not-found → −32602 (deliberately NOT −32601 per inline comment at `error_mapping.rs:86-91`). Story anchor S-MCP-TOOL-GATE-001 AC-003/RG-GATE-003 now correctly cites −32602. TD-VSDD-097 sibling note: the −32602 fact also appears in S-MCP-TOOL-GATE-001 body (AC-003/RG-GATE-003/T-D01); story-writer is correcting that in parallel — no story edits made here. |
| 1.2 | beta3-remediation-BC-amendments | 2026-09-16 | product-owner | Operations feature gate: amended §Description to capture absent-feature default. Added feature-gate postcondition block (absent = NOT_YET_AVAILABLE_TOOLS &[], 14-tool catalog, -32601 for unknown tool; enabled = existing -32003 fast-fail). Replaced stale "registered in tools/list" invariant with INV-OPERATIONS-FEATURE-GATE. Anchors: S-MCP-TOOL-GATE-001 AC-001/AC-003/AC-004/AC-005 (RG-GATE-001, RG-GATE-003, RG-GATE-004). Resolves beta.3 issues 1 and 2 (beta3-remediation-delta-analysis.md §Issue 1). |
| 1.1 | PR-203-fix-burst-F-P2R2-HIGH-001 + PR-203-post-merge-POL-14 | 2026-06-26 | product-owner + state-manager | D-1110 reality-drift reconciliation (F-P2R2-HIGH-001 sibling-sweep miss). §Preconditions: removed fictional `mpsc::Sender<AuditEntry>`; reflects `Option<Arc<dyn AuditWriter>>`. §Postconditions: Option B `try_send` mandate dropped; shipped implementation is Option A (guard-reorder — `emit_tool_audit` never reached for NOT_YET_AVAILABLE tools). INV-AUDIT-NON-BLOCKING: rewritten to reflect Arc-DI async-await path with no mpsc/try_send; non-blocking property is satisfied by guard-reorder. §Architecture Anchors: cite real `emit_tool_audit` function signature + behavioral anchor; drop stale `try_send` prescription. BC-2.10.016 received its D-1110 reconciliation in the prior burst; this BC (2.10.017) is the sibling-sweep closure. EC-10-017-001 (and §Edge Cases sweep) reconciled to guard-reorder reality — "audit channel buffer full" was a fictional condition; rewritten to describe the slow-AuditWriter scenario and the invariant that `emit_tool_audit` is never reached for NOT_YET_AVAILABLE tools (completes the v1.1 D-1110 sweep, closes F-002). **POL-14 auto-promotion: draft → active** on squash-merge of PR #203 to develop@7e60df03 (2026-06-26; CI 43/43 green; `status: draft → active`; no additional behavioral change). |
| 1.0 | demo-readiness-2026-06-24 | 2026-06-24 | product-owner | Initial contract. Authored per demo-readiness-remediation-design-2026-06-24.md. Closes BLOCKER-004. Root cause: `emit_tool_audit` blocking `send()` before fast-fail guard; fix: reorder guard before audit OR change to `try_send`. |
