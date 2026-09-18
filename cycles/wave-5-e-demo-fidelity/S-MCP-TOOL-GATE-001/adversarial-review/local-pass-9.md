---
document_type: adversarial-review-pass
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-09-18T22:00:00Z
story: S-MCP-TOOL-GATE-001
pass: 9
frozen_head: "0af76be5c"
story_version_at_review: "v1.9"
clean_strict: false
clean_pr_merge: true
streak_before: 1
streak_after: 0
findings_count: 1
traces_to: STATE.md
---

# Adversarial Review — S-MCP-TOOL-GATE-001 LOCAL Pass 9

## Pass Metadata

| Field | Value |
|-------|-------|
| Pass | 9 (LOCAL) |
| Date | 2026-09-18 |
| Frozen feature HEAD | 0af76be5c |
| Story version at review | v1.9 (story-writer @946fe7b16) |
| CLEAN(strict) | NO |
| CLEAN(PR-merge) | YES |
| Findings | 1 (F-LOCAL-LOW-001 [LOW]) |
| Streak before | 1/3 (pass-8 was CLEAN(strict)) |
| Streak after | 0/3 RESET (BC-5.39.001 — any finding resets streak) |

## Summary

Pass 9 on frozen feature HEAD `0af76be5c` with story at v1.9 (`@946fe7b16`).

This pass found 1 LOW finding: a prose-vs-test-body drift in the §Red Gate description for RG-GATE-004. The story described asserting `NOT_YET_AVAILABLE_TOOLS.len() == 0` (a private `const` item), but the shipped test body asserts `production_tool_catalog().len() == 14`. The private const is unreachable from an external test crate — the story description was never accurate to the shipped code. Zero behavioral impact; the test itself is load-bearing and correct.

All targeted scrutiny PASSES, including the highest-risk item: the `operations`-feature-off RG file (`bc_2_10_017_operations_feature_gate.rs`) genuinely runs in `just check` / `check-ci` default-feature legs AND `--no-default-features` legs.

This is the 7th consecutive code-clean pass (no code defects found since pass-3).

**Pass-8 context (CLEAN — streak reached 1/3):** Immediately prior pass on the same frozen HEAD `0af76be5c` + story v1.9 found zero findings. Streak advanced 0/3 → 1/3. That streak advance is now reset by this pass-9 finding.

## Findings

### F-LOCAL-LOW-001 [LOW] — §Red Gate RG-GATE-004 prose↔test-body drift

**Severity:** LOW  
**Category:** spec-drift (docs-only, zero behavioral impact)  
**Affected artifact:** Story S-MCP-TOOL-GATE-001 v1.9 §Red Gate planning list, RG-GATE-004 description bullet  

**Finding:**  
The §Red Gate planning list describes RG-GATE-004 with the following prose (paraphrased): "asserts `NOT_YET_AVAILABLE_TOOLS.len() == 0` (the private const is empty when operations feature is absent)."

The shipped test body for the test identified as covering RG-GATE-004 asserts:
```rust
assert_eq!(production_tool_catalog().len(), 14);
// plus 14 EXPECTED_LIVE_TOOLS presence assertions
// plus wire-shape assertion on the JSON MCP list-tools response
```

The item `NOT_YET_AVAILABLE_TOOLS` is a private `const` in the production module — it is not accessible from an external test crate. The story description citing it was never an accurate description of what the test actually asserts. The pre-gate pre-implementation prose described a design intent; the shipped implementation correctly chose a public API (`production_tool_catalog()`) and the test was written against that.

**Evidence:**  
- `production_tool_catalog().len() == 14`: verified present in shipped test  
- `NOT_YET_AVAILABLE_TOOLS`: private const, inaccessible from external test crate (E0603-family access violation would result)  
- All 14 `EXPECTED_LIVE_TOOLS` items: verified present in shipped test  
- Wire-shape assertion: verified present  

**Impact:** Zero behavioral impact. The test is correct and load-bearing. The finding is prose-only.  

**Proposed fix:** Story-writer corrects RG-GATE-004 description to accurately state: asserts `production_tool_catalog().len() == 14`, verifies all 14 `EXPECTED_LIVE_TOOLS` are present, and includes wire-shape assertion on the MCP list-tools response. Remove the `NOT_YET_AVAILABLE_TOOLS.len() == 0` language. No code change required.

## Targeted Scrutiny Results (All PASS)

| Check | Result | Notes |
|-------|--------|-------|
| Operations-off RG file runs in `just check` default-feature legs | PASS | `bc_2_10_017_operations_feature_gate.rs` confirmed in default build |
| Operations-off RG file runs in `check-ci --no-default-features` leg | PASS | Confirmed; file is not gated behind operations feature |
| 14 EXPECTED_LIVE_TOOLS present in shipped test | PASS | Verified all 14 entries |
| `production_tool_catalog().len() == 14` assertion present | PASS | Load-bearing assertion confirmed |
| Wire-shape assertion present | PASS | JSON envelope assertions verified |
| BC-2.10.017 §Operations-absent -32602 wire behavior | PASS | RG-GATE-003 covers this; discharged |
| AC-002 wire round-trip test | PASS | Test added in pass-5 code fix, present |
| AC-001 14-tool registration | PASS | production_tool_catalog().len() == 14 |
| No code mechanism defects | PASS | 7th consecutive code-clean pass |
| TD-VSDD-097 Dim-1 (sibling pair) | PASS | No sibling twin for this story |
| TD-VSDD-097 Dim-2 (downstream copy) | PASS | RG-GATE-004 description not copy-sourced elsewhere |
| TD-VSDD-097 Dim-3 (mandate anchor) | PASS | No new MUSTs; existing anchors intact |

## Verdict

**CLEAN(strict): NO** (1 LOW finding — streak RESETS 0/3)  
**CLEAN(PR-merge): YES** (zero CRIT/HIGH/MED findings)

**Streak:** 1/3 → RESET 0/3 per BC-5.39.001 (any finding resets streak regardless of severity or behavioral impact).

**Next action:** story-writer fixes RG-GATE-004 description in story v1.9 → v1.10; no code change; feature HEAD `0af76be5c` remains FROZEN. After fix-burst complete, adversary LOCAL pass-10 re-gates on frozen HEAD `0af76be5c` + story v1.10.
