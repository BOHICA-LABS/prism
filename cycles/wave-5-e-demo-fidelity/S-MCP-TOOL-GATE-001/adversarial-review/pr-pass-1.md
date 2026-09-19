---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-09-18T02:00:00Z
phase: 3
inputs: []
input-hash: "[live-state]"
traces_to: ""
pass: 1
previous_review: null
pr: "297"
pr_head: "41541f496"
base_branch: develop
base_head: "528f9bdd3"
story: S-MCP-TOOL-GATE-001
cycle: wave-5-e-demo-fidelity
clean_strict: false
clean_pr_merge: false
---

# PR-LEVEL Adversary Pass 1 — S-MCP-TOOL-GATE-001 (PR #297)

**Frozen PR HEAD reviewed:** 41541f496 (base develop@528f9bdd3)
**Date:** 2026-09-18

## Convergence Verdict

| Criterion | Result |
|-----------|--------|
| CLEAN(strict) | NO |
| CLEAN(PR-merge) | NO |
| BC-5.39.001 PR-LEVEL streak | 0/3 on frozen HEAD 41541f496 |

## Finding ID Convention

Finding IDs for this PR-LEVEL pass use the format: `F-<SEQ>` (MED) and `OBS-<SEQ>` (LOW) per prism PR-LEVEL review convention. Findings fixed in the pass-1 fix-burst are recorded as CLOSED.

## Part A — Fix Verification (pass >= 2 only)

_Pass 1 — no prior PR-LEVEL findings to verify._

## Part B — New Findings (or all findings for pass 1)

### MEDIUM

#### F-001 [MED] — Story §File Structure / subsystem / risk anchors not reconciled after B-001 scripts/ fix

- **Severity:** MED
- **Category:** spec-fidelity / sibling-sweep (TD-VSDD-097 Dim-1)
- **Location:** Story spec S-MCP-TOOL-GATE-001.md §File Structure, §Subsystem, §Risk Anchors
- **Description:** The story §File Structure section did not include `scripts/t13-preflight-audit.py`, which was modified as part of the B-001 blocking fix pushed to the PR branch (commit 41541f496). The frozen-HEAD reference in the story pointed at `6a0986ace` rather than the post-B-001 PR HEAD. The §Subsystem anchor and §Risk Anchors sections were inconsistent with the actual 18-file diff in the PR (which includes `scripts/t13-preflight-audit.py` and `docs/demo-evidence/` additions).
- **Evidence:** PR diff contains modifications to `scripts/t13-preflight-audit.py` and `docs/demo-evidence/` artifacts; story §File Structure at HEAD 41541f496 lists neither; story frozen-HEAD field still shows `6a0986ace`.
- **Proposed Fix:** story-writer reconcile §File Structure 1:1 with 18-file PR diff; update frozen-HEAD to post-fix-burst HEAD; correct §Subsystem and §Risk Anchors.
- **Status:** CLOSED (story-writer v1.10→v1.11 @5f01ccad5; new PR HEAD ce4a945e1).

### LOW

#### OBS-001 [LOW] — `scripts/t13-preflight-audit.py` stale tool-count comment

- **Severity:** LOW
- **Category:** code-quality / stale-comment (TD-VSDD-091)
- **Location:** `scripts/t13-preflight-audit.py` (comment text)
- **Description:** Post-operations-feature-gate the preflight script retained a comment referencing the pre-gate 54-macro count and 2 registered groups. Post-story the catalog is 57 macros / 3 registered groups (14 LIVE_TOOLS unconditional + operations group feature-gated). The NOT_YET_AVAILABLE_TOOLS cite was pinned to a specific count that is no longer valid.
- **Evidence:** Comment text references "54 macros" and "2 registered groups"; post-merge catalog count is 57/3.
- **Proposed Fix:** Update comment: 56→57 macro count, 2→3 groups; add feature-aware wording; de-pin NOT_YET_AVAILABLE_TOOLS cite.
- **Status:** CLOSED (implementer @ce4a945e1; same fix-burst as OBS-002).

#### OBS-002 [LOW] — `scripts/t13-preflight-audit.py` stale `-32003` NYA assumption — BLOCKING default-build (B-001 class)

- **Severity:** LOW (finding tier); BLOCKING (build impact — 5 checks HARD-FAIL on default operations-off build)
- **Category:** sibling-sweep gap (TD-VSDD-060 / POL-29) / error-code assumption
- **Location:** `scripts/t13-preflight-audit.py` checks [A2],[A19],[A20],[A21],[A23]
- **Description:** The 14-pass LOCAL adversarial cascade, story-level holdout gate, and all doc fix-bursts operated exclusively on `crates/` test files. `scripts/t13-preflight-audit.py` was not in the sibling-sweep scope (TD-VSDD-060). The script hardcodes the assumption that operations-absent builds return `-32003` (NOT_YET_AVAILABLE_TOOLS) for gated tool calls; post-story the correct code is `-32602` (method-not-found / tool-not-found per BC-2.10.017 §EC-2.10.017-003). All five checks [A2],[A19],[A20],[A21],[A23] HARD-FAIL on the default (operations-off) build — a BLOCKING default-build breakage caught only at PR-LEVEL.
- **Evidence:** Checks [A2],[A19],[A20],[A21],[A23] contain literal `-32003` comparisons against tool-call responses; BC-2.10.017 AC-003 mandates `-32602`; DEFAULT build omits `--features operations`.
- **Proposed Fix:** Replace `-32003` with `-32602` in all 5 checks; add feature-conditional prose.
- **Root cause (process):** Sibling-sweep enforcement (TD-VSDD-060 / POL-29) scoped to `crates/` tests only; `scripts/` and non-crate tooling encoding tool-catalog / error-code / count assumptions excluded. See PROCESS-GAP D-2582a for codification candidate.
- **Status:** CLOSED (implementer @ce4a945e1; same fix-burst as OBS-001).

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 1 (F-001 — CLOSED in fix-burst) |
| LOW | 2 (OBS-001/OBS-002 — CLOSED in fix-burst) |

**Overall Assessment:** pass-with-findings (3 findings; all CLOSED in fix-burst @ce4a945e1)
**Convergence:** FINDINGS_REMAIN — streak reset; PR-LEVEL pass-2 required on new frozen HEAD ce4a945e1
**Readiness:** Fix-burst complete; re-gate on ce4a945e1 for pass-2

### Production Code Assessment

**AC-001 through AC-005 verified CORRECT:** Gate mechanism sound. Operations Cargo feature properly gates 40 tool stubs. LIVE_TOOLS unconditionally registered (14). Gated tool call returns `-32602`. `not_registered_tools` field present as empty array (not null, not absent) in both client_id:null and unknown-client modes.

**Repo-wide sibling sweep:** No additional unswept executable-asserter for tool-catalog or error-code assumptions found outside `scripts/`. `tool_dispatch_tests.rs` count references (54) are source-text parameter strings (tool names in test params), not catalog-size assertions — not a break.

**S-001 OPS_GATED failure-path:** NOT material — fails safe (method-not-found is correct when feature absent). Non-blocking.

## Post-Pass-1 Fix-Burst Summary

| Fix | Agent | Commit | Finding(s) Closed |
|-----|-------|--------|-------------------|
| Story §File Structure + frozen-HEAD + subsystem/risk anchors | story-writer | 5f01ccad5 | F-001 [MED] |
| `scripts/t13-preflight-audit.py` stale counts + error-code | implementer | ce4a945e1 | OBS-001 + OBS-002 [LOW] |

**New PR HEAD after fix-burst:** ce4a945e1
**PR-LEVEL BC-5.39.001 streak:** 0/3 (reset; frozen-HEAD rule: streak counts only consecutive CLEAN(strict) passes against UNCHANGED HEAD)
**NEXT:** PR-LEVEL adversary pass-2 re-gate on frozen HEAD ce4a945e1.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 1 (PR-LEVEL) |
| **New findings** | 3 (F-001, OBS-001, OBS-002) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.00 (3/3) |
| **Median severity** | LOW (2 LOW, 1 MED) |
| **Trajectory** | 0 (LOCAL 3/3 CLEAN) → 3 (PR-LEVEL pass-1) |
| **Verdict** | FINDINGS_REMAIN — all CLOSED in fix-burst; pass-2 required on ce4a945e1 |
