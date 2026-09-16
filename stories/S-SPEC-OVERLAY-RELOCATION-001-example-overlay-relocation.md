---
document_type: story
story_id: S-SPEC-OVERLAY-RELOCATION-001
title: "Relocate Armis example org overlays out of shipped spec tree to prevent E-SPEC-022 on source-build (post-beta.3 fast-follow)"
level: "L4"
wave: TBD
epic_id: E-REL
priority: P3
status: draft
# BC status: pending PO authorship — behavioral_contracts is empty. Per S-7.01, this story
# MUST remain draft until a product-owner authors or waives BCs (waiver acceptable for a
# path-restructuring / devex story). Possible waiver: ADR-authority infra story with no
# behavioral surface; PO must decide.
producer: story-writer
timestamp: "2026-09-15T00:00:00Z"
version: "0.1"
modified: "2026-09-15"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - "crates/prism-sensors/specs/customers/acme/"
  - "crates/prism-sensors/specs/customers/contoso/"
input-hash: "pending-recompute"
# input-hash: pending — to be computed from the two overlay paths after beta.3 ships
traces_to: []
# traces_to: references BC-2.06.015 / BC-2.06.016 (overlay validation behavioral contracts)
# and ADR-029 (overlay loading strategy). Precise BC authorship or waiver at materialization.
points: 2
# points: 2 estimated — file moves + CI update + install script validation.
# No behavioral changes. Armis is POST-v1 de-scoped so no active sensor test migration needed.
estimated_days: 0.5
tdd_mode: facade
# tdd_mode: facade — this story is a file restructuring + CI validation task.
# No non-trivial algorithmic function bodies are introduced. Combined scaffold+impl delivery
# is appropriate. Mutation testing at wave gate replaces Red Gate density check.
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Release Engineering + Distribution) owns this story's scope: the fix is in
#     the specs tarball packaging step, install scripts, and example-overlay location.
#     No prism-core / prism-spec-engine behavioral changes. SS-22 governs release
#     packaging, install tooling, and distribution artifacts per ARCH-INDEX.
target_module: prism-sensors
crates_touched: [prism-sensors]
# crates_touched: prism-sensors because crates/prism-sensors/specs/ is the canonical
# spec tree path. The move is within the repo; the CI release tarball step is in
# .github/workflows/release.yml.
behavioral_contracts: []
# BC status: pending PO authorship or waiver (S-7.01 gate — behavioral_contracts: []
# blocks status=ready). PO may waive for an ADR-authority infra story.
# Traceability context: BC-2.06.015 (E-SPEC-022 unknown org slug) and BC-2.06.016
# (overlay validation) are the correct-by-design contracts that this story preserves.
# The fix ensures the spec tree does not contain overlays for unregistered orgs.
verification_properties: []
holdout_scenarios: []
# holdout_scenarios: PO authors scenarios at remove-uncertainty time. Given facade
# tdd_mode, holdout scenarios may be waived if the story is treated as a devex fix
# with AC-level verification only. PO decides at materialization.
depends_on: []
# depends_on justification: no runtime dependency. This is a spec-tree restructuring
# that is independent of all other stories. Can ship in any wave after the current
# beta.3 W3 remediation cycle completes and the spec tree is stable.
blocks: []
acceptance_criteria_count: 0
# acceptance_criteria_count: 0 — draft stub; ACs to be authored at materialization
red_gate_tests: 0
# red_gate_tests: 0 — draft stub; facade mode; RG enumeration at materialization
risk: LOW
# Risk justification: pure file-move + tarball-exclusion change. The E-SPEC-022 error
# fires correctly-by-design when unregistered org overlay paths exist in the spec tree
# at runtime; this story prevents the example overlays from being in the tree in the
# first place. No behavioral change to the overlay loading logic. Packaged-release
# installs are unaffected (release tarball already ships only claroty.sensor.toml per
# the specs-tarball task in release.yml).
assumption_validations: []
risk_mitigations: []
---

# S-SPEC-OVERLAY-RELOCATION-001: Relocate Armis Example Org Overlays Out of Shipped Spec Tree

> **DRAFT STUB — POST-BETA.3 FAST-FOLLOW.** Full ACs and task list are authored at
> story-materialization time. Human-directed deferral: Armis is POST-v1 de-scoped,
> making this a genuine deferral per Canonical Principle Rule 3 (concrete future
> dependency = stable spec tree + Armis re-scoping decision). Status MUST remain
> `draft` until `behavioral_contracts:` is populated or a PO waiver is recorded per S-7.01.

## Authority

**BC-2.06.015** (`E-SPEC-022` `unknown_org_slug` error contract) and **BC-2.06.016**
(overlay validation contract) govern the correct behavior that this story preserves —
those BCs are correct by design. **ADR-029** (overlay loading strategy) defines the
overlay path resolution and org-slug validation logic in `overlay.rs`.

**D-2520** (beta.3 live-test triage / W3 remediation cycle) is the source decision record
that identified this issue during source-build validation. The `E-SPEC-022` protection
fires for `acme` and `contoso` org slugs because those orgs are not registered in the
live prism.toml, causing source-build `validate-config` to fail with a typo-protection
error on a clean checkout.

## Objective

Move `crates/prism-sensors/specs/customers/acme/` and
`crates/prism-sensors/specs/customers/contoso/` (Armis example org overlays) from the
buildable spec tree to a test-only fixtures path so that:

1. A fresh source-build pointing `spec_dir` at `crates/prism-sensors/specs` does NOT fail
   `boot` or `validate-config` with `E-SPEC-022` for unregistered org slugs `acme`/`contoso`.
2. The example overlays remain available in the repository for Armis integration testing
   when Armis is re-scoped in a future cycle.
3. The release tarball step in `.github/workflows/release.yml` continues to ship only the
   intended Claroty-only spec tree (unaffected by this move but must be verified).
4. `docs/SETUP.md §9` (any reference to the example overlay paths) is updated if present.

## Rationale

`E-SPEC-022` (implemented in `overlay.rs::make_e_spec_022_unknown_org_slug`) is the
correct-by-design typo protection for overlay org slugs — it prevents a misconfigured
production deployment from silently loading wrong overlays. The fix is NOT to weaken
`E-SPEC-022` but to not commit example overlays that reference unregistered orgs into
the path that a source-build reads as its `spec_dir`.

Packaged installs (release tarball produced by `.github/workflows/release.yml`'s
`specs-tarball` step) are already unaffected because that step packages only
`claroty.sensor.toml` from the specs root — the `customers/` subdirectory is not included
in the packaged release. This story fixes the **source-build** developer experience.

Armis is POST-v1 de-scoped, making this a genuine deferral per Canonical Principle Rule 3.

## Source

Origin: D-2520 live-test triage / beta.3 remediation cycle (W3). Human-directed deferral
2026-09-15: Armis is POST-v1 de-scoped; fix is a source-build devex improvement rather
than a production correctness issue.

## Dependency

None. This story is independent and can ship in any wave after the beta.3 W3 cycle.

## Scope (to be refined at materialization)

- Move `crates/prism-sensors/specs/customers/acme/` to
  `crates/prism-sensors/tests/fixtures/overlay-examples/customers/acme/` (or equivalent
  test-fixture path — exact destination to be confirmed at materialization per the
  existing test fixture layout in `crates/prism-sensors/tests/`).
- Move `crates/prism-sensors/specs/customers/contoso/` to the same fixture path.
- Update any test or CI references to the old paths.
- Verify the `release.yml` specs-tarball step does not need changes (it already excludes
  the `customers/` directory — verify and document).
- Update `docs/SETUP.md §9` if it references the example overlay paths.
- The `overlay.rs` `E-SPEC-022` logic MUST NOT be changed — it is correct by design.

## Architecture Anchors

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Spec tree path | `crates/prism-sensors/specs/customers/` | N/A (data files) |
| Test fixture path | `crates/prism-sensors/tests/fixtures/` | N/A (data files) |
| Release tarball step | `.github/workflows/release.yml §specs-tarball` | Effectful (CI) |
| Install docs | `docs/SETUP.md §9` | N/A (docs) |

## Previous Story Intelligence

N/A — first story in this overlay-relocation track. Predecessor context: S-REL-SPECS-TARBALL-001
(merged PR #279) established the specs-tarball packaging model and is the reference for
verifying that the release tarball step is unaffected.

## Architecture Compliance Rules

- `overlay.rs::make_e_spec_022_unknown_org_slug` MUST NOT be modified (correct-by-design
  contract; CLAUDE.md §Source-of-Truth Precedence — spec wins over code).
- The moved example overlays MUST NOT be added to the release tarball (verify the
  `release.yml` step excludes them by default).
- Test fixtures MUST NOT be committed to a path that `spec_dir` points to in CI.

## Library & Framework Requirements

N/A — this story makes no dependency changes. No new crates. No version pins required.

## File Structure Requirements

| File / Path | Action |
|-------------|--------|
| `crates/prism-sensors/specs/customers/acme/` | Move to test fixture path |
| `crates/prism-sensors/specs/customers/contoso/` | Move to test fixture path |
| `crates/prism-sensors/tests/fixtures/overlay-examples/customers/` | New destination (confirm at materialization) |
| `.github/workflows/release.yml` | Verify specs-tarball step unaffected; document |
| `docs/SETUP.md` | Update §9 if example overlay paths are referenced |
| `CHANGELOG.md` | Add [Unreleased] > Fixed row describing the source-build boot fix |

## Token Budget Estimate

| Item | Tokens |
|------|--------|
| This story spec | ~2 000 |
| `overlay.rs` (reference read only) | ~2 000 |
| `release.yml` specs-tarball step | ~1 000 |
| `docs/SETUP.md §9` | ~500 |
| Existing test fixtures layout | ~1 000 |
| BC-2.06.015 / BC-2.06.016 (reference read) | ~2 000 |
| **Total estimate** | **~8 500** |

Well within the 20-30% window cap. Straightforward facade delivery.

## Acceptance Criteria

*N/A — draft stub. ACs to be authored at materialization. Placeholder scope:*

- *AC-001 (placeholder): A fresh source-build pointing `spec_dir` at
  `crates/prism-sensors/specs` with no `prism.toml` defining `acme` or `contoso` as org
  slugs completes `boot`/`validate-config` without `E-SPEC-022` errors.*
- *AC-002 (placeholder): The example overlays remain accessible in the repository at the
  new fixture path for future Armis integration test use.*
- *AC-003 (placeholder): The `release.yml` specs-tarball step produces an identical tarball
  (the `customers/` directory was already excluded — verify with a checksum or diff).*
- *AC-004 (placeholder): `docs/SETUP.md §9` (if it references the old paths) is updated
  to the new fixture path.*

## Red Gate Tests

*N/A — draft stub. Facade mode; mutation testing at wave gate per tdd_mode: facade.*
*Key verification at materialization: a CI job or local test that validates the `spec_dir`
source path boots cleanly without E-SPEC-022 for unregistered example orgs.*

## Edge Cases

*N/A — draft stub. To be elaborated at materialization:*

- *Existing CI test that loads the example overlays by old path — must be updated.*
- *Any grep/find in scripts that references `customers/acme` or `customers/contoso` — must sweep.*

## Tasks

(Enumerated at materialization.)

1. Remove-uncertainty pass: confirm the exact fixture layout in `crates/prism-sensors/tests/`;
   identify all references to the old overlay paths.
2. Move the two overlay directories.
3. Update any test or CI references to the old paths.
4. Verify `release.yml` specs-tarball step is unaffected (document the verification).
5. Update `docs/SETUP.md §9` if applicable.
6. Run `just check` to confirm no regressions.
7. Add CHANGELOG entry under [Unreleased] > Fixed.
8. Push + PR.

## Changelog

| Version | Date | Change |
|---------|------|--------|
| v0.1 | 2026-09-15 | Initial draft stub registered (D-2521 post-beta.3 fast-follow batch). |
