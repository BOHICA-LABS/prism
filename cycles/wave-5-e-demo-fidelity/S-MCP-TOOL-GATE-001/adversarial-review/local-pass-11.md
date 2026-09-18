---
document_type: adversarial-review-pass
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-09-18T22:30:00Z
story: S-MCP-TOOL-GATE-001
pass: 11
frozen_head: "0af76be5c"
story_version_at_review: "v1.10"
clean_strict: false
clean_pr_merge: true
streak_before: 1
streak_after: 0
findings_count: 1
traces_to: STATE.md
---

# Adversarial Review — S-MCP-TOOL-GATE-001 LOCAL Pass 11

## Pass Metadata

| Field | Value |
|-------|-------|
| Pass | 11 (LOCAL) |
| Date | 2026-09-18 |
| Frozen feature HEAD | 0af76be5c |
| Story version at review | v1.10 (story-writer @8afffe10b) |
| CLEAN(strict) | NO |
| CLEAN(PR-merge) | YES |
| Findings | 1 (LOW-001 [LOW]) |
| Streak before | 1/3 (pass-10 was CLEAN(strict)) |
| Streak after | 0/3 RESET (BC-5.39.001 — any finding resets streak) |

## Pass Context

**Pass-10 (immediately prior):** CLEAN(strict)=YES on frozen HEAD `0af76be5c` + story v1.10. Streak advanced 0/3 → 1/3. This pass-11 finding resets the streak.

**Code-clean streak:** This is the 8th consecutive code-clean pass (no code mechanism defects found in passes 4–11). All findings since pass-4 have been docs-tier only.

## Summary

Pass 11 on frozen feature HEAD `0af76be5c` with story at v1.10 (`@8afffe10b`).

This pass found 1 LOW finding: the module-level doc comment in the story-modified file `crates/prism-mcp/src/tools/mod.rs` is stale in two respects. First (L4 context), the top-of-module doc describes the tool registration mechanism using old single-macro `server_handler` phrasing, whereas the story introduced a two-router-block plus combiner architecture. Second (L12-L14 context), the doc mentions "NotImplemented" as a fallback behavior, while the actual behavior after the story's feature gate is default-absent (the operations router block is simply absent when the `operations` feature is off, rather than returning a NotImplemented variant). The module doc was not updated as part of the story's changes.

All targeted scrutiny passes, including all AC-001..005, BC-2.10.017 v1.5 operations-absent −32602 wire behavior, BC-INDEX drift check, sibling-sweep (TD-VSDD-097), CI both-state coverage (default-features + no-default-features), POL-7/32/39/40/42, SAP-1 (tracing emission catalog), SAP-3 (spec-arm reachability), and SAC-1 (Red Gate list enumeration). No code mechanism defects.

**Out-of-diff observation (non-blocking, pre-existing workspace issue):** The same stale `server_handler` single-macro phrasing appears in 5 other tool submodule docs that are NOT part of this story's diff: `sensor_health.rs`, `config.rs`, `write.rs`, `query.rs`, and `operations.rs`. These are pre-existing inaccuracies outside S-MCP-TOOL-GATE-001's scope. Deferred to maintenance story S-MAINT-TOOLS-MODULE-DOC-SWEEP-001 (draft-stub).

## Findings

### LOW-001 [LOW] — `tools/mod.rs` module doc stale (story-modified file)

**Severity:** LOW
**Category:** docs-drift (doc-comment only, zero behavioral impact)
**Affected artifact:** `crates/prism-mcp/src/tools/mod.rs` — module-level doc comment
**In diff:** YES (this file was modified by the story)

**Finding:**

The module doc comment in `crates/prism-mcp/src/tools/mod.rs` contains two stale passages:

1. **L4 context — single-macro `server_handler` phrasing:** The doc describes tool registration with old phrasing referencing `server_handler` as a single macro entry point. The story introduced a two-router-block architecture (a query-capable router and an operations router) combined via a combiner. The doc still reads as if there is one unified `server_handler` call site.

2. **L12-L14 context — "NotImplemented" fallback description:** The doc references "NotImplemented" as a fallback when the operations feature is absent. The actual behavior is default-absent: when `features = []` (no `operations` feature), the operations router block is simply not included, and there is no NotImplemented variant in the tool dispatch path. The description incorrectly implies a runtime fallback rather than compile-time absence.

**Impact:** Zero behavioral impact. The gate mechanism is correct; only the module doc describing it is stale.

**Proposed fix:** Implementer updates `crates/prism-mcp/src/tools/mod.rs` module doc comment: (a) replace single-macro `server_handler` phrasing with description of the two-router-block + combiner architecture; (b) remove "NotImplemented" fallback language and replace with accurate description of feature-gate-driven default-absent behavior. Doc-comment change only — no logic, no gate, no test change.

## Out-of-Diff Observation (Pre-existing, Non-Blocking)

**OBS-001 [OUT-OF-DIFF] — `server_handler` phrasing in 5 unmodified tool submodule docs**

**Status:** Non-blocking, pre-existing, deferred. NOT a reset trigger for this pass (out-of-diff per-story-scope boundary).

The following files, which are NOT modified by S-MCP-TOOL-GATE-001, carry the same stale `server_handler` single-macro phrasing in their module-level doc comments:
- `crates/prism-mcp/src/tools/sensor_health.rs`
- `crates/prism-mcp/src/tools/config.rs`
- `crates/prism-mcp/src/tools/write.rs`
- `crates/prism-mcp/src/tools/query.rs`
- `crates/prism-mcp/src/tools/operations.rs`

This is a workspace-wide pre-existing inaccuracy. It falls outside S-MCP-TOOL-GATE-001's diff scope. Recommended action: track as deferred finding anchored to maintenance story S-MAINT-TOOLS-MODULE-DOC-SWEEP-001 (draft-stub) for a future doc-sweep burst covering all tool submodule docs.

## Targeted Scrutiny Results (All PASS)

| Check | Result | Notes |
|-------|--------|-------|
| AC-001: 14 tools registered | PASS | production_tool_catalog().len() == 14 confirmed |
| AC-002: wire round-trip | PASS | Test present; 14 tools round-trip via MCP list-tools |
| AC-003: operations-off gate | PASS | Operations feature gate mechanically correct |
| AC-004: unregistered tool -32602 error | PASS | BC-2.10.017 §Operations-absent wire behavior verified |
| AC-005: CI both-state coverage | PASS | Default-features + no-default-features both exercised |
| BC-2.10.017 v1.5 contract drift | PASS | No drift; BC row pin matches artifact |
| BC-INDEX no drift (L10 check) | PASS | No stale/phantom rows for this story's BCs |
| TD-VSDD-097 Dim-1 (sibling pair) | PASS | No sibling twin story for S-MCP-TOOL-GATE-001 |
| TD-VSDD-097 Dim-2 (downstream copy) | PASS | tools/mod.rs doc not copy-sourced in any downstream spec |
| TD-VSDD-097 Dim-3 (mandate anchor) | PASS | No new unanchored MUSTs introduced |
| SAP-1: tracing emission catalog | PASS | No new event_type emitters introduced; existing catalog intact |
| SAP-3: spec-arm reachability | PASS | All BC arms reachable from public MCP surface |
| SAC-1: Red Gate list enumeration | PASS | Story v1.10 carries enumerated RG-GATE-001..004 list |
| POL-7 (verbatim BC H1 titles) | PASS | BC H1 citations in story verified accurate |
| POL-32 (no phantom BCs) | PASS | All referenced BCs exist |
| POL-39 (no volatile version pins) | PASS | No new volatile version pins introduced |
| POL-40 (no story-level scope inflation) | PASS | Story scope unchanged |
| POL-42 (traceability complete) | PASS | All ACs trace to BCs |
| Code mechanism defects | PASS | 8th consecutive code-clean pass; zero logic/gate defects |

## Verdict

**CLEAN(strict): NO** (1 LOW finding — streak RESETS 0/3)
**CLEAN(PR-merge): YES** (zero CRIT/HIGH/MED findings)

**Streak:** 1/3 → RESET 0/3 per BC-5.39.001 (any finding resets streak regardless of severity or behavioral impact).

**Feature HEAD at review:** `0af76be5c` (unchanged from pass-10). LOW-001 is a doc-comment fix in the story-modified file `crates/prism-mcp/src/tools/mod.rs` — implementing the fix advances the feature HEAD.

**Next action:** Implementer updates `tools/mod.rs` module doc (L4 mechanism description + L12-14 default-absent semantics); doc-only change; no logic/gate modification; both builds must compile; 6/6 `bc_2_10_017` tests must pass. After fix committed, feature HEAD advances (0af76be5c → new SHA). LOCAL 3-CLEAN streak resets 0/3 on the new frozen HEAD (BC-5.39.001 frozen-HEAD rule). Adversary LOCAL pass-12 re-gates on new frozen HEAD + story v1.10.
