---
document_type: adversarial-review-pass
story: S-MCP-TOOL-GATE-001
pass: 2
scope: LOCAL
frozen_head: 09658db3a
reviewed_by: adversary
date: 2026-09-18
clean_strict: false
clean_pr_merge: true
finding_count: 1
finding_severity_breakdown: "LOW: 1"
streak_effect: "NOT CLEAN(strict) — story-text-only fix (v1.4→v1.5 @8d9320277); feature HEAD 09658db3a FROZEN/UNCHANGED; LOCAL streak 0/3 UNCHANGED"
---

# S-MCP-TOOL-GATE-001 LOCAL Adversarial Review — Pass 2

**Frozen HEAD reviewed:** 09658db3a
**CLEAN(strict):** NO
**CLEAN(PR-merge):** YES (zero CRIT/HIGH/MED findings)
**Finding count:** 1 LOW

---

## CLEAN Report (all targeted scrutiny)

- **AC-004 P0 (14 LIVE_TOOLS ungated):** CLEAN — 14 tools registered unconditionally in `LIVE_TOOLS`; no operations gate on the live-tool slice; wire-confirmed.
- **AC-003 −32602 wire behavior:** CLEAN — `get_diagnostics` and all operations stubs return `MethodNotFound (−32602)` under `operations` feature absent; `RG-GATE-003` exercises this path; wire-level assertion present.
- **BC-2.10.017 v1.4 tables consistent + BC-INDEX pin matches:** CLEAN — BC-2.10.017 at v1.4 in both the artifact and BC-INDEX row; operations-absent `−32602` rows present in §Error Cases and §Canonical Test Vectors; no POL-40/L10 drift.
- **OBS-2 e2e round-trip load-bearing:** CLEAN — e2e assertion in `mcp_infrastructure.rs` tests serialized JSON wire output; load-bearing (not a no-op).
- **Justfile operations-off leg non-redundant + runs RG tests:** CLEAN — `just check` operations-off leg is non-redundant (distinct `--no-default-features` invocation targeting `RG-GATE-003` path); no duplication.
- **Sibling-sweep complete:** CLEAN — swept all three `mcp_infrastructure` ops-gated tests (server.rs inline test + two `mcp_infrastructure.rs` tests); no ungated residuals.
- **POL-12/14/32/36/42:** CLEAN.
- **SAP-1 (tracing emission catalog completeness):** CLEAN — no new `event_type =` sites added.
- **SID-1 (no-ignored-test rationalization):** CLEAN.
- **SID-2 (composed-output assertions):** CLEAN.
- **SAC-1 (enumerated Red Gate list):** CLEAN — RG-GATE-001..RG-GATE-004 enumerated and anchored.

---

## Findings

### OBS-A [LOW] — Story AC-005/T-D01 prose: unsatisfiable operations-absent assertion

**Severity:** LOW
**Category:** Story text / specification prose defect
**Affects:** Story file only; feature code at HEAD 09658db3a is CORRECT

**Description:**

Story AC-005 and Task T-D01 prose contained an instruction of the form "assert that calling `get_diagnostics` when the `operations` feature is absent returns an error" — phrased as though a test should invoke `get_diagnostics` in a context where the feature is compiled out. This instruction is unsatisfiable: `get_diagnostics` is a method on a type that does not compile into the binary when `operations` is gated out (`#[cfg(feature = "operations")]`), meaning any test attempting `get_diagnostics` in the absent-feature configuration would produce a compile error (`E0599: no method named 'get_diagnostics'`).

**Code behavior (CORRECT — no code change required):**
The implementation is correct. When `operations` is absent, the operations stubs are absent from the binary. The existing inline arm in the tool-dispatch match asserts catalog-absence for the operation names, and `RG-GATE-003` exercises the `−32602` wire behavior via the MCP protocol surface (calling the tool by name, not by method). This is the correct test approach.

**Root cause:**
Story-writer prose described a test calling a Rust method directly (`get_diagnostics`) rather than describing the observable wire behavior test (MCP tool call returning `−32602`). The distinction matters because the former requires the method to be compiled in, while the latter works regardless of feature flag state.

**Fix:**
Story-writer corrects AC-005 and T-D01 prose to describe the `RG-GATE-003` wire behavior approach (invoke the MCP tool by name; assert `−32602` response) rather than a direct Rust method call.

**TD-VSDD-097 (3-dim sweep):**
- Dim-1 (sibling pair): No sibling story twin for S-MCP-TOOL-GATE-001. CLEAR.
- Dim-2 (downstream copy target): AC-005/T-D01 are not copy-sources for downstream artifacts. CLEAR.
- Dim-3 (mandate anchor): No new MUSTs introduced by the prose fix. CLEAR.

---

## Adversary Assessment

The code at frozen HEAD 09658db3a is effectively converged. All behavioral contracts (BC-2.10.017 v1.4 + BC-2.10.011 v1.7), wire behavior, and test coverage are correct. The single finding is a story-text precision defect only — the test described in AC-005/T-D01 would not compile as written, but the *intended* test (RG-GATE-003 wire behavior) already exists and is correct.

**Effectively converged.** After story-text correction (v1.4→v1.5), re-gate pass-3 on frozen HEAD 09658db3a is the first candidate for CLEAN(strict).
