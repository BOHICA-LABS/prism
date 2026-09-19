---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-09-19T02:04:00Z
phase: 3
inputs: []
input-hash: "[live-state]"
traces_to: ""
pass: 2
previous_review: pr-pass-1.md
pr: "297"
pr_head: "ce4a945e1"
base_branch: develop
base_head: "528f9bdd3"
story: S-MCP-TOOL-GATE-001
cycle: wave-5-e-demo-fidelity
clean_strict: true
clean_pr_merge: true
---

# PR-LEVEL Adversary Pass 2 — S-MCP-TOOL-GATE-001 (PR #297)

**Frozen PR HEAD reviewed:** ce4a945e1 (base develop@528f9bdd3)
**Date:** 2026-09-18

## Convergence Verdict

| Criterion | Result |
|-----------|--------|
| CLEAN(strict) | YES |
| CLEAN(PR-merge) | YES |
| BC-5.39.001 PR-LEVEL streak | 0/3 → 1/3 on frozen HEAD ce4a945e1 |

## Finding ID Convention

Finding IDs for this PR-LEVEL pass use the format: `F-<SEQ>` (MED/HIGH/CRIT) and `OBS-<SEQ>` (LOW) per prism PR-LEVEL review convention. Pass 2 has zero new findings.

## Part A — Fix Verification

Verifying all pass-1 findings are correctly resolved in the post-fix-burst HEAD ce4a945e1.

**F-001 [MED] — Story §File Structure / subsystem / risk anchors:** VERIFIED CLOSED.
Story v1.11 (@5f01ccad5, committed as part of ce4a945e1 PR branch): §File Structure now lists `scripts/t13-preflight-audit.py` and `docs/demo-evidence/` additions 1:1 with the 18-file PR diff. Frozen-HEAD updated to ce4a945e1. §Subsystem and §Risk Anchors corrected. Story-writer fix is accurate and complete.

**OBS-001 [LOW] — `scripts/t13-preflight-audit.py` stale tool-count comment:** VERIFIED CLOSED.
Comment updated to 57 macros / 3 registered groups; feature-aware wording added; NOT_YET_AVAILABLE_TOOLS cite de-pinned. Text is accurate to the post-story catalog.

**OBS-002 [LOW] — `scripts/t13-preflight-audit.py` stale `-32003` NYA assumption:** VERIFIED CLOSED.
All five checks [A2],[A19],[A20],[A21],[A23] updated from `-32003` to `-32602`. Verified correct against BC-2.10.017 AC-003 mandate. Default (operations-off) build no longer HARD-FAILs on these checks.

## Part B — New Findings

**None.**

Fresh-context review of full 18-file PR diff on frozen HEAD ce4a945e1:

**Gate mechanism (AC-001..005):** PASS. `operations` Cargo feature correctly gates 40 tool stubs. `LIVE_TOOLS` unconditionally registered (14 tools). Gated tool call returns `-32602` per BC-2.10.017 EC-2.10.017-003. `not_registered_tools` field present as empty array (not null, not absent) in both client_id:null and unknown-client modes. AC-005 token-budget constants correct.

**BC-2.10.017 compliance:** PASS. §EC-2.10.017-001 (list returns all unconditional tools), §EC-2.10.017-002 (operations-gated tools absent from list when feature off), §EC-2.10.017-003 (-32602 on gated call), §EC-2.10.017-004 (not_registered_tools empty array) — all satisfied.

**BC-2.10.011 compliance:** PASS. Health-check infrastructure unaffected by story changes; no regression.

**`scripts/t13-preflight-audit.py` (post-fix):** PASS. All five previously-HARD-FAILING checks now use `-32602`; feature-conditional branching added; macro/group counts accurate.

**Story v1.11 §File Structure audit:** PASS. 1:1 correspondence with 18-file PR diff confirmed. Frozen-HEAD field correctly shows ce4a945e1. §Subsystem and §Risk Anchors consistent.

**Repo-wide sibling sweep (feature-aware audit, TD-VSDD-060 / D-2582a):** PASS. No additional unswept executable-asserter for tool-catalog counts or error-code assumptions outside `scripts/` identified. `tool_dispatch_tests.rs` count references are string-parameter test names, not catalog-size assertions — no break.

**SAP-1 (tracing emission catalog):** PASS. No new `event_type =` emissions introduced by story.

**SAP-3 (spec-arm reachability):** PASS. All BC arms reachable from public surface (MCP stdio tool-call).

**TD-VSDD-097 sweep (pass-level, all CLEAR):**
- Dim-1: CLEAR — S-MCP-TOOL-GATE-001 has no sibling twin story.
- Dim-2: CLEAR — story §File Structure is not a copy-source section for any downstream artifact.
- Dim-3: CLEAR — no unanchored MUSTs; existing story anchors intact.

**TD-VSDD-091/POL-39:** CLEAN — no volatile line-cites or version pins in staged changes.

**SAC-1 (enumerated Red Gate list):** PASS. RG-GATE-001..004 present in story with correct test names; BC-5.38.001 density check paragraph present; red-before-green task ordering verified.

**SAC-2 (ADR anchor_stories):** PASS — no ADR authoring in this story's scope.

**Security:** PASS. No credential exposure; no injection surface; `reqwest` timeout rules not violated (no HTTP client in scope); rustls-tls usage unchanged.

**Non-exhaustive / #[non_exhaustive] discipline:** PASS. No new public TOML-deserialized types added.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 0 |

**Overall Assessment:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES
**Convergence:** Streak advances 0/3 → 1/3 on frozen HEAD ce4a945e1
**Readiness:** PR-LEVEL pass-3 required for streak continuation

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 2 (PR-LEVEL) |
| **New findings** | 0 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | n/a (zero-finding pass) |
| **Median severity** | n/a |
| **Trajectory** | 3 (PR pass-1) → 0 (PR pass-2) |
| **Verdict** | CLEAN — streak 0/3 → 1/3; pass-3 required |
