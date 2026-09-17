---
document_type: holdout-scenario
level: L3
id: "HS-DESC-001-001"
title: "prism_describe _meta.total_results equals the count of tables in the wire response body (not 0)"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "E-BETA3-REMEDIATION"
story_source: "S-MCP-ENVELOPE-DESCRIBE-001"
version: "1.0"
status: active
used: false
last_evaluated: null
last_eval_satisfaction: null
single_use: true
producer: product-owner
timestamp: "2026-09-16T00:00:00Z"
modified: "2026-09-16"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.10.012-prism-describe-schema-discovery-tool.md"
input-hash: "TBD"
traces_to: "BC-2.10.012"
behavioral_contracts:
  - BC-2.10.012
verification_properties: []
lifecycle_status: active
introduced: "S-MCP-ENVELOPE-DESCRIBE-001"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-MCP-ENVELOPE-DESCRIBE-001 (HS-034 group). Validates Issue 3 fix: safety_envelope::wrap() was missing a 'tables' arm for the prism_describe object shape {client_id, tables: [...], pql_hints}, causing _meta.total_results to always be 0. Discriminating: pre-patch returns total_results=0 regardless of N tables; post-fix returns total_results==N (equal to tables.length in the same response body). Wire-level assertion on serialized JSON content text. BC-2.10.012 §Response envelope amended postcondition EC-10-032: total_results = tables.len(). Test-writer and implementer must NOT read this file."
---

# HS-DESC-001-001: prism_describe _meta.total_results equals the count of tables in the wire response body (not 0)

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-MCP-ENVELOPE-DESCRIBE-001 (HS-034 group)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.10.012 §Response envelope amended postcondition (EC-10-032):
`_meta.total_results` in the safety envelope output for a `prism_describe` call MUST equal
`tables.len()` — the number of entries in the `tables` array of the same response body.
It must NOT be 0 when tables are present.
**Gate:** Story-level holdout gate (HS-034) — runs after LOCAL 3-CLEAN convergence, before
demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the **total_results counting fix in safety_envelope::wrap()**
(BC-2.10.012 §Response envelope EC-10-032; S-MCP-ENVELOPE-DESCRIBE-001 Issue 3 / AC-001 / AC-003).

When `prism_describe` executes against a client with N configured tables:

1. The tool handler returns a JSON object with shape `{client_id, tables: [...N entries...], pql_hints: [...]}`.
2. `safety_envelope::wrap()` must detect the `tables` key and compute `total_results = tables.len()`.
3. The outer `_meta` envelope must carry `"total_results": N` — NOT `"total_results": 0`.
4. The `tables` array in the same parsed response body also has N entries.

**The defect this scenario catches:** Pre-patch `wrap()` has two arms for `total_results`:
bare-array path and `{rows: [...]}` object path. The `prism_describe` shape
`{client_id, tables: [...], pql_hints: [...]}` matches neither arm, so `else { 0 }` fires
unconditionally. An LLM agent receives `"0 results found"` in the prose summary even when
the client has multiple Claroty tables registered.

**Discriminating assertion:** `_meta.total_results` value is compared against
`tables.length` as measured from the SAME serialized response. Pre-patch: total_results=0
while tables.length≥1 — a clear mismatch. Post-fix: total_results == tables.length.

**Two assertions in this scenario:**

**Part A — total_results matches tables.length from same response:**
- Parse `result.content[0].text` as JSON to get the prism_describe response.
- Extract `_meta.total_results` (integer).
- Extract the length of the `tables` array.
- Assert `_meta.total_results == tables.length`.
- Assert `_meta.total_results > 0` (at least one table is configured for the test client).

**Part B — total_results is present as an integer in the raw wire bytes:**
- Assert the serialized JSON string (`result.content[0].text`) contains `"total_results"`.
- Assert the value is an integer (not null, not a string).
- Assert the value is NOT 0 when tables is non-empty.

**BDD supplement (Part A):**

**Given** prism is built from the S-MCP-ENVELOPE-DESCRIBE-001 story branch
**And** a test client is configured with at least one Claroty sensor table
**When** `tools/call prism_describe {client_id: "<test_client>"}` is issued via MCP stdio
**Then** the response is not a JSON-RPC error
**And** `result.content[0].text` parses as valid JSON
**And** `_meta.total_results` equals the length of `tables` in the same JSON
**And** `_meta.total_results` is greater than 0
**And** `_meta.total_results` is NOT the integer 0

---

## Setup Instructions

1. Build prism from the S-MCP-ENVELOPE-DESCRIBE-001 story branch (standard build, no special
   feature flags required for this scenario).

2. Prepare a `prism.toml` that configures at least one client with at least one Claroty
   sensor table. The standard `claroty.sensor.toml` spec shipped with the codebase registers
   multiple tables (claroty_alerts, claroty_devices, claroty_audit_logs, and others). Configure
   a test client (`client_id = "holdout-describe-test"`) with the Claroty sensor.
   Note: `prism_describe` reads from the loaded sensor spec at boot — it does NOT contact the
   live Claroty API or DTU. The DTU does NOT need to be running for this scenario.

3. Start prism in MCP stdio mode with the prepared config.
   SETUP-FAILURE condition: if prism fails to start (exit non-zero or no MCP handshake).

4. Complete the MCP `initialize` handshake:
   `{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"holdout-evaluator","version":"1.0"}}}`
   Await non-error response before proceeding.

5. Issue the `prism_describe` tool call:
   `{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"prism_describe","arguments":{"client_id":"holdout-describe-test"}}}`
   Capture the full raw wire-level JSON response bytes.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.10.012 | §Response envelope EC-10-032 (amended): `_meta.total_results` = `tables.len()` in `wrap()` tables-arm | Part A: total_results == tables.length |
| BC-2.10.012 | §Response envelope: total_results is non-zero when ≥1 table configured | Part B: value > 0 assertion |

---

## Verification Approach

1. Parse the wire-level JSON-RPC response. Verify the response is NOT a JSON-RPC error:
   `error` key absent and `result` key present.

2. Extract `result.content[0].text` as a raw string. This is the safety-envelope-wrapped
   prism_describe response.

3. Parse the `result.content[0].text` string as JSON to obtain the prism_describe response object.

4. **Part A — Consistency check:**
   Extract `_meta.total_results` as an integer.
   Extract `tables` as an array; count its length (`N`).
   Assert `_meta.total_results == N`.
   If `_meta.total_results != N`: record FAIL with both values.

5. **Part B — Non-zero assertion:**
   Assert `_meta.total_results > 0` (at least one table loaded at boot).
   If `_meta.total_results == 0` AND `tables.length > 0`: record FAIL — the classic pre-patch defect.
   If `tables.length == 0`: record SETUP-FAILURE (client has no tables configured).

6. Wire-shape assertion: perform all assertions on the PARSED JSON from `result.content[0].text`,
   not on pre-serialization Rust struct inspection.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: >= 0.75.

- **Response is non-error (prerequisite)** (weight: 0.10):
  Full credit (1.0): `result` key present, `error` key absent, `result.content[0].text` is valid JSON.
  Zero credit (0.0): error response, no response, or content text is not valid JSON (SETUP-FAILURE).

- **total_results > 0** (weight: 0.30):
  Full credit (1.0): `_meta.total_results` is a positive integer.
  Zero credit (0.0): `_meta.total_results == 0` — the classic Issue 3 defect.

- **total_results == tables.length** (weight: 0.60):
  Full credit (1.0): `_meta.total_results` exactly equals `tables` array length.
  Partial credit (0.3): value is positive but != tables.length (some other counting error).
  Zero credit (0.0): value == 0 when tables.length >= 1 (pre-patch behavior).

---

## Edge Conditions

- **Client not found / no tables configured:** prism_describe may return an error or an empty
  tables array. If `tables.length == 0` and `_meta.total_results == 0`, this is SETUP-FAILURE,
  not a behavioral FAIL (the fix handles the non-empty case; 0 tables → 0 results is correct
  per EC-001 of the story's edge case table).

- **total_results == tables.length == 0:** SETUP-FAILURE — no tables configured for test client.
  Verify the prism.toml sensor configuration is correct.

- **Tool returns an MCP error code:** If `prism_describe` tool itself errors (not JSON-RPC
  transport error), record SETUP-FAILURE with the error details.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-DESC-001-001 (satisfaction: X.XX) — prism_describe response envelope mismatch; check safety_envelope::wrap() tables-arm for the {tables: [...]} shape (BC-2.10.012 §Response envelope EC-10-032: total_results must equal tables.len() for the prism_describe object shape)"`

Do NOT disclose: the specific table count, the exact assertion values, or whether
the failure was on Part A vs Part B.

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | prism binary from S-MCP-ENVELOPE-DESCRIBE-001 branch + standard claroty.sensor.toml (codebase-shipped spec) |
| corpus_size | N tables from claroty.sensor.toml loaded at boot (exact count depends on spec; ≥3 expected from claroty_alerts, claroty_devices, claroty_audit_logs plus Wave expansions) |
| known_edge_cases | Empty client (0 tables) = SETUP-FAILURE not behavioral FAIL |
| false_positive_threshold | Zero: total_results vs tables.length is an unambiguous integer equality check |
| false_negative_threshold | Near-zero: pre-patch always returns 0; post-fix returns N >= 1 |

**Known-good corpus:** prism binary built from S-MCP-ENVELOPE-DESCRIBE-001 story branch
with the wrap() tables-arm fix applied. Expected: `_meta.total_results == tables.length > 0`.

**Known-problematic corpus:** pre-patch prism binary (develop HEAD before this story's fix).
Expected: `_meta.total_results == 0` despite `tables.length >= 1` — the Issue 3 defect.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | beta3-w2-holdout-authoring | 2026-09-16 | product-owner | Initial authoring. HS-034 group for S-MCP-ENVELOPE-DESCRIBE-001. Issue 3 (total_results=0 defect): prism_describe response must have _meta.total_results == tables.length > 0 at the serialized JSON wire level. Discriminates pre-patch (always 0) from post-fix (equals N). BC-2.10.012 §Response envelope EC-10-032. SINGLE-USE. |
