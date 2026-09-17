---
document_type: story
story_id: S-BETA3-RELEASE-001
title: "release: dispatch v1.0.0-beta.3 under BOHICA-LABS to fix orphaned SLSA attestation and bundle all 20 beta.2 issue fixes"
wave: 4
epic_id: E-BETA3-REMEDIATION
version: "1.0"
status: ready
producer: story-writer
phase: 3
priority: P0
points: 2
tdd_mode: facade
# tdd_mode: facade — this story dispatches a GitHub Actions workflow (release-tag.yml)
# and updates a documentation file (RELEASING.md). No Rust production code is modified;
# crates_touched: []. Quality gate is structural + live: CI green on develop, SLSA
# attestation verification under BOHICA-LABS/prism, binary version identity check, and
# live-tenant validation post-release. Mutation testing at wave gate replaces Red Gate
# density check per BC-8.30.001. There is no Red Gate list because there is no
# production Rust code to write or stub.
target_module: devops
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Process Lifecycle) governs the release build pipeline, binary distribution
#   channels, and SLSA provenance per ARCH-INDEX Subsystem Registry. Dispatching a
#   release tag, generating a CHANGELOG section, and verifying build attestations are
#   all squarely within SS-22's scope. No other subsystem is crossed.
crates_touched: []
# crates_touched: [] — all delivery actions are CI workflow dispatch, docs edit, and
# external service verification. Zero Rust source files are modified.
estimated_days: 0.5
depends_on:
  - S-MCP-TOOL-GATE-001
  - S-MCP-ENVELOPE-DESCRIBE-001
  - S-MCP-NULL-ENCODING-001
  - S-DESCRIBE-EXAMPLE-DEDUP-001
  - S-QUERY-TRUE-TOTAL-001
  - S-CLAROTY-OCSF-STATUS-001
  - S-CLAROTY-OCSF-TOML-001
  - S-JSON-EXTRACT-UDF-001
# depends_on anchor justifications:
#   S-MCP-TOOL-GATE-001 (W1): Operations stub gating — must merge before release bundles
#     the develop HEAD. Beta.3 must not ship the -32003 catalog-pollution defect (issue 1/2).
#   S-MCP-ENVELOPE-DESCRIBE-001 (W2): prism_describe total_results + virtual fields fix
#     (issues 3/5) — must be in develop HEAD at tag time.
#   S-MCP-NULL-ENCODING-001 (W2): Null encoding fix (issue 7b) — must be in develop HEAD.
#   S-DESCRIBE-EXAMPLE-DEDUP-001 (W2-arch): Example query dedup (issue 4) — must be in
#     develop HEAD. No cross-crate build conflict with W2 stories.
#   S-QUERY-TRUE-TOTAL-001 (W2-arch): True upstream total propagation (issue 6) —
#     high-blast-radius story (touches engine.rs, materialization.rs, fanout.rs,
#     spec_driven_adapter.rs); must fully merge and CI-green before release dispatch
#     to avoid a partially-shipped multi-struct change in the beta.3 binary.
#   S-CLAROTY-OCSF-STATUS-001 (W3): Claroty devices is_online + retired demotion
#     (issues 9/10) — live-tenant visible fix; must ship in beta.3.
#   S-CLAROTY-OCSF-TOML-001 (W3): Claroty TOML field-mapping corrections
#     (issues 8/11/12/13) — live-tenant visible fix; must ship in beta.3.
#   S-JSON-EXTRACT-UDF-001 (W3): json_extract_string ScalarUDF (issue 20) —
#     must ship in beta.3; S-JSON-EXTRACT-TYPED-001 and S-JSON-EXTRACT-NESTED-001
#     depend on it; those fast-follows are post-beta.3 and are NOT in depends_on here.
blocks: []
# blocks: [] — S-ONBOARDING-DOCS-001 (W5) is explicitly scheduled as runnable in
# parallel with or after S-BETA3-RELEASE-001 per delta analysis §Batch 2 note.
# Its content (SETUP.md placeholders, attestation runbook) does not require the
# beta.3 tag to be created first; version placeholders are used throughout.
risk: LOW
# Risk justification:
#   LOW — no Rust production code modified; no schema changes; no new crates.
#   The release-tag.yml workflow is mechanically proven by prior beta.1 and beta.2
#   dispatches. The channel-scoped tag-pattern (--tag-pattern) is in place from
#   S-REL-CHANGELOG-CHANNEL-SCOPE-001 (PR #281). The PRISM_VERSION injection
#   mechanism is proven from ADR-064. Main risk: RELEASE_PROMOTE_TOKEN PAT requires
#   human re-scope action before dispatch can succeed.
behavioral_contracts: []
# BC status: N/A — release orchestration / CI workflow governance.
# No behavioral contract in the BC-S.SS.NNN namespace governs the act of dispatching
# a pre-release tag workflow. Release pipeline authority is:
#   RELEASING.md §4 (step-by-step: pre-release lane = release-tag.yml)
#   ADR-063 §D7 v1.14 (channel-scoped git-cliff --tag-pattern for beta lane)
#   ADR-064 §D2 v2.3 (pre-release binary version identity; PRISM_VERSION injection)
#   delta-analysis §Issue 14 + §Issue 15 (scope)
# S-7.01 gate: status remains draft pending product-owner review; no BC-TBD
# placeholders (none needed — release dispatch has no applicable behavioral contract).
# Per project convention for release-lane facade stories (cf. S-REL-CHANGELOG-CHANNEL-SCOPE-001,
# S-REL-NIGHTLY-NOTES-001, S-REL-SPECS-TARBALL-001): behavioral_contracts: [] is
# correct for CI/release-tooling stories whose authority is ADR-level, not BC-level.
verification_properties: []
assumption_validations: []
risk_mitigations:
  - "RELEASE_PROMOTE_TOKEN PAT MUST be re-scoped for BOHICA-LABS/prism before any
    release-tag.yml dispatch. The PAT grants Contents:write permissions, enabling the
    workflow to push the CHANGELOG commit to develop and push the annotated tag to
    origin. Without it, the workflow exits early with a permissions error and no
    CHANGELOG commit or tag is created. This is the single blocking prerequisite for
    this story. See §Prerequisites for the exact GitHub admin steps."
  - "Use release-tag.yml for beta.3, NOT release-promote.yml. RELEASING.md §1
    pre-release exception and the release-tag.yml comment header are both explicit:
    pre-release channels tag develop only; release-promote.yml is for stable tags
    (X.Y.Z without hyphen) only. Dispatching release-promote.yml for a beta tag
    would attempt a develop→main promotion, which is incorrect and will fail the
    version guard (develop prism-bin stays at 1.0.0-dev per ADR-064 §D2)."
  - "The channel-scoped --tag-pattern for the beta channel is already in
    release-tag.yml from S-REL-CHANGELOG-CHANNEL-SCOPE-001 (PR #281). No changes to
    release-tag.yml are required. The step 5b channel-detect block sets
    TAG_PATTERN='\\^v[0-9]+\\.[0-9]+\\.[0-9]+(-beta\\.[0-9]+)?\\$' for a beta tag,
    preventing cross-channel tag masking that affected beta.2."
  - "The empty-section guard in release-tag.yml step 5b will exit 1 if the new
    CHANGELOG section has no qualifying commits since the last beta tag
    (v1.0.0-beta.2). Confirm there are at least 8 non-skip-type commits between
    beta.2 and the beta.3 dispatch HEAD (fix: commits from W1-W3 stories satisfy
    this requirement)."
inputs:
  - "RELEASING.md"
  - ".github/workflows/release-tag.yml"
  - ".github/workflows/release.yml"
  - ".factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md"
  - ".factory/specs/architecture/decisions/ADR-063-changelog-release-notes-architecture.md"
  - ".factory/specs/architecture/decisions/ADR-064-pre-release-binary-version-identity.md"
input-hash: "[pending-state-manager-compute]"
acceptance_criteria_count: 9
red_gate_tests: 0
# red_gate_tests: 0 — tdd_mode: facade; crates_touched: []; no Rust production code.
# Quality gate: structural verification (AC-001..AC-009) + live-tenant validation (Task T-10).
holdout_scenarios: []
# HOLDOUT-N/A: tdd_mode=facade; crates_touched=[]; no prism binary produced by this
# story itself. The release asset binary is built by release.yml (CI) from develop HEAD,
# which has already passed all story-level holdout gates for W1-W3 stories individually.
# The release dispatch is a tag + asset packaging operation, not a code change.
# Consistent with S-REL-NIGHTLY-NOTES-001, S-REL-CLIFF-001,
# S-REL-CHANGELOG-CHANNEL-SCOPE-001 holdout determinations.
traces_to: STATE.md D-2520
cycle: wave-5-e-demo-fidelity
---

# S-BETA3-RELEASE-001: Dispatch v1.0.0-beta.3 Release Under BOHICA-LABS

**Story ID:** S-BETA3-RELEASE-001
**Status:** draft
**Version:** v1.0
**Wave:** W4 (after all W1+W2+W2-arch+W3 stories merged)
**Priority:** P0
**Points:** 2
**TDD Mode:** facade

---

## Problem Statement

**Issue 14 — Orphaned SLSA attestation after org rename:**

The v1.0.0-beta.2 release was cut while the repository lived at `drbothen/prism`.
The SLSA build-provenance attestation generated by `actions/attest-build-provenance`
during that release references the workflow URI
`drbothen/prism/.github/workflows/release.yml`. After the org rename to
BOHICA-LABS/prism (D-2516), `gh attestation verify --repo BOHICA-LABS/prism` fails:
the attestation certificate's subject workflow does not match the new org namespace.
Operators cannot verify the supply-chain integrity of the beta.2 binaries under the
BOHICA-LABS identity.

**Fix:** Create a new release `v1.0.0-beta.3` under BOHICA-LABS/prism. The
`release.yml` workflow triggered by the new tag will generate a fresh SLSA
attestation with the URI `BOHICA-LABS/prism/.github/workflows/release.yml`, which
verifies correctly under the BOHICA-LABS org. The beta.2 attestation remains
non-verifiable; this is documented in RELEASING.md.

**Issue 15 — `main` branch stale; install.sh raw URL concern:**

The `main` branch is ~307 commits behind develop and may contain `drbothen/prism`
references in `scripts/install.sh`. The `develop` branch already has
`REPO="BOHICA-LABS/prism"` (confirmed pre-fixed). The beta.3 release asset
(built from the tagged develop HEAD) will include the correct BOHICA-LABS install.sh
in each platform archive. The raw `https://raw.githubusercontent.com/BOHICA-LABS/prism/main/`
URL concern is fully resolved only when the first stable release promotes develop→main
via `release-promote.yml`. For beta.3 users (who install from the release download
page, not the raw main URL), the release asset's install.sh is correct.

**Note on delta analysis language (spec clarification):**
The delta analysis §S-BETA3-RELEASE-001 scope item 2 states "Develop→main promotion
via PR." This language is imprecise and contradicts RELEASING.md §1 pre-release
exception: **pre-release beta tags MUST use `release-tag.yml` and NEVER touch
`main`.** The correct mechanism is `gh workflow run release-tag.yml --ref develop -f
tag=v1.0.0-beta.3`. The `release-promote.yml` (develop→main) flow is reserved for
stable tags only. This story follows the authoritative RELEASING.md procedure, not
the imprecise language in the delta analysis.

---

## Prerequisites

### REQUIRED HUMAN ACTION — BLOCKING

**`RELEASE_PROMOTE_TOKEN` PAT must be re-scoped for BOHICA-LABS/prism.**

**This story CANNOT be delivered until this action is complete.**

The `release-tag.yml` workflow requires `RELEASE_PROMOTE_TOKEN` to push the
git-cliff CHANGELOG commit to develop and push the annotated tag to origin. The
default `GITHUB_TOKEN` cannot trigger downstream `on:push:tags` workflows
(release.yml), so a PAT is required.

| Step | Actor | Action |
|------|-------|--------|
| 1 | Human | Create or re-scope a Fine-Grained PAT for BOHICA-LABS/prism with `Repository contents: Read and write` (or Classic PAT: `repo`) |
| 2 | Human | Navigate to github.com/BOHICA-LABS/prism → Settings → Secrets and variables → Actions |
| 3 | Human | Set (or update) the secret `RELEASE_PROMOTE_TOKEN` to the new PAT value |
| 4 | Human | Confirm to the orchestrator: "RELEASE_PROMOTE_TOKEN configured for BOHICA-LABS" |

**Authority:** RELEASING.md §3 Mandatory Invariants ("RELEASE_PROMOTE_TOKEN secret
must be configured"); release-tag.yml header comment ("REQUIRED SECRET: RELEASE_PROMOTE_TOKEN");
D-2520 open item (e).

The orchestrator MUST NOT dispatch TDD delivery (test-writer / implementer) for this
story until the human confirms the PAT is in place. The story `status: draft` reflects
this blocking gate.

---

## Authority

**`beta3-remediation-delta-analysis.md` §Issue 14 + §Issue 15 + §S-BETA3-RELEASE-001**
is the authoritative scope document for this story.
Path: `.factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md`

**`RELEASING.md`** §1 (pre-release exception), §4 (step-by-step release procedure)
governs the release workflow. Pre-release beta tags use `release-tag.yml` directly
(NOT `release-prep.yml` + `release-promote.yml`). `main` is NEVER touched by a pre-release.
Path: `RELEASING.md`

**ADR-063 §D7 v1.14** governs channel-scoped `--tag-pattern` for git-cliff invocations.
The beta channel pattern `^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$` is already wired
in `release-tag.yml` step 5b from S-REL-CHANGELOG-CHANNEL-SCOPE-001 (PR #281).
Path: `.factory/specs/architecture/decisions/ADR-063-changelog-release-notes-architecture.md`

**ADR-064 §D2 v2.3** governs pre-release binary version identity. No `prism-bin`
`Cargo.toml` version bump is required between pre-releases on the same X.Y.Z cycle.
`PRISM_VERSION` is injected at build time via `GITHUB_REF_NAME` (tag name minus `v`).
Path: `.factory/specs/architecture/decisions/ADR-064-pre-release-binary-version-identity.md`

---

## Narrative

As a release engineer, I want to cut the `v1.0.0-beta.3` release under the
BOHICA-LABS/prism GitHub organization, so that operators can verify the supply-chain
integrity of all beta.3 binaries via `gh attestation verify --repo BOHICA-LABS/prism`
and all 20 beta.2 live-test fixes are bundled in a single, attestation-valid release.

---

## Acceptance Criteria

### AC-001 — All upstream stories merged and CI green
All eight W1+W2+W2-arch+W3 story PRs (S-MCP-TOOL-GATE-001, S-MCP-ENVELOPE-DESCRIBE-001,
S-MCP-NULL-ENCODING-001, S-DESCRIBE-EXAMPLE-DEDUP-001, S-QUERY-TRUE-TOTAL-001,
S-CLAROTY-OCSF-STATUS-001, S-CLAROTY-OCSF-TOML-001, S-JSON-EXTRACT-UDF-001) are merged
to develop and all 24 required CI status checks are green on develop HEAD before
`release-tag.yml` is dispatched.

### AC-002 — RELEASE_PROMOTE_TOKEN configured (human-validated prerequisite)
The `RELEASE_PROMOTE_TOKEN` secret is set in github.com/BOHICA-LABS/prism repository
secrets with sufficient scope (Fine-Grained PAT: `Contents: Read and write`; or
Classic PAT: `repo`). Human confirmation is on record before dispatch. See §Prerequisites.

### AC-003 — release-tag.yml dispatched and CHANGELOG committed
`gh workflow run release-tag.yml --ref develop -f tag=v1.0.0-beta.3` dispatches
successfully. Step 5b completes: the git-cliff CHANGELOG section for v1.0.0-beta.3
is generated with the beta channel `--tag-pattern`, committed to develop with message
`chore: update CHANGELOG for v1.0.0-beta.3`, and pushed to origin/develop. The
annotated tag `v1.0.0-beta.3` is created on that CHANGELOG-updated commit and pushed,
triggering `release.yml`.

### AC-004 — All 4 release build legs succeed
`release.yml` completes successfully across all 4 matrix legs: `aarch64-apple-darwin`,
`x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `x86_64-pc-windows-msvc`.
`gh release view v1.0.0-beta.3 --repo BOHICA-LABS/prism` returns HTTP 200.

### AC-005 — All required release assets attached
All 7 required asset categories from RELEASING.md §4 Step 6 are present on the
GitHub Release:
1. `prism-v1.0.0-beta.3-aarch64-apple-darwin.tar.gz`
2. `prism-v1.0.0-beta.3-x86_64-unknown-linux-gnu.tar.gz`
3. `prism-v1.0.0-beta.3-x86_64-unknown-linux-musl.tar.gz`
4. `prism-v1.0.0-beta.3-x86_64-pc-windows-msvc.zip`
5. `checksums.txt`
6. `install.sh` and `install.ps1` (per S-REL-003)
7. `prism-specs-v1.0.0-beta.3.tar.gz` (per S-REL-SPECS-TARBALL-001)

### AC-006 — SLSA attestation verifies under BOHICA-LABS (Issue 14 fix)
`gh attestation verify prism-v1.0.0-beta.3-x86_64-unknown-linux-gnu.tar.gz --repo BOHICA-LABS/prism --signer-workflow BOHICA-LABS/prism/.github/workflows/release.yml`
exits 0 with a PASSED result. The attestation certificate references the
`BOHICA-LABS/prism` workflow URI, confirming the supply-chain integrity is no longer
orphaned to the old `drbothen` org namespace.

### AC-007 — Binary version identity correct (ADR-064 §D2)
`prism --version` extracted from the `aarch64-apple-darwin` or `x86_64-unknown-linux-gnu`
archive reports `prism 1.0.0-beta.3`. This confirms the PRISM_VERSION injection chain
in `crates/prism-bin/build.rs` correctly resolved `GITHUB_REF_NAME` (`v1.0.0-beta.3`
with leading `v` stripped) to `1.0.0-beta.3` on the tag build. No `prism-bin`
`Cargo.toml` version bump was required (ADR-064 §D2 pre-release exception).

### AC-008 — Bundled specs present in archive (Issue 15 — install.sh correct)
`tar tzf prism-v1.0.0-beta.3-x86_64-unknown-linux-gnu.tar.gz` output includes
`specs/claroty.sensor.toml` and `prism.toml.example` (per RELEASING.md §4 Step 6
item 8). The bundled `install.sh` in the same archive contains
`REPO="BOHICA-LABS/prism"` (confirming the BOHICA-LABS org rename is present in
the release asset; Issue 15 mitigated for beta.3 users).

### AC-009 — RELEASING.md updated with beta.2 attestation non-verifiability note
`RELEASING.md` contains a note documenting that the v1.0.0-beta.2 SLSA attestation
is non-verifiable under BOHICA-LABS/prism (attestation references `drbothen/prism`;
superseded by beta.3). The note must be placed in §6 Recovery Procedures or a new
§Known Issues subsection before §4, with the text: "v1.0.0-beta.2 SLSA attestation:
the beta.2 release was cut before the org rename from drbothen to BOHICA-LABS. Its
build-provenance attestation references `drbothen/prism/.github/workflows/release.yml`
and is non-verifiable under `--repo BOHICA-LABS/prism`. Use v1.0.0-beta.3 or later
for verifiable attestations."

---

## Architecture Compliance Rules

Extracted from RELEASING.md, ADR-063 §D7, ADR-064 §D2. These are mandatory constraints
for this story's delivery.

| Rule | Source | Enforcement |
|------|--------|-------------|
| Beta tags use `release-tag.yml` ONLY; NEVER `release-promote.yml` | RELEASING.md §1 pre-release exception; release-tag.yml header | Workflow selection at dispatch time |
| `main` MUST NOT be touched by this story | release-tag.yml header; RELEASING.md §2 branch strategy | release-tag.yml design guarantee |
| No `prism-bin` Cargo.toml version bump | ADR-064 §D2 pre-release exception | Pre-dispatch check: verify Cargo.toml still at `1.0.0-dev` |
| `--tag-pattern` for beta channel MUST be `^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$` | ADR-063 §D7 v1.14 | Already wired in release-tag.yml step 5b (PR #281); verify intact before dispatch |
| Verify 24 required CI status checks green BEFORE dispatch | RELEASING.md §3 mandatory invariants | `gh api repos/BOHICA-LABS/prism/branches/develop/protection --jq '.required_status_checks.contexts[]'` |
| Never skip git hooks | CLAUDE.md TD-FACTORY-HOOK-BYPASS-001 P0 | No `--no-verify` permitted |
| `attestation verify` must use `--repo BOHICA-LABS/prism` and `--signer-workflow BOHICA-LABS/prism/.github/workflows/release.yml` | RELEASING.md §5 "Verify build provenance" | Post-release AC-006 verification |

---

## Token Budget Estimate

| Artifact | Estimated Lines | Estimated Tokens |
|----------|----------------|-----------------|
| This story spec | ~250 lines | ~3,500 |
| RELEASING.md | ~742 lines | ~9,000 |
| release-tag.yml | ~200 lines | ~2,500 |
| release.yml (relevant sections) | ~200 lines | ~2,500 |
| ADR-063 (§D7 only) | ~50 lines | ~700 |
| ADR-064 (§D2 only) | ~40 lines | ~600 |
| Delta analysis (§Issue 14+15 only) | ~60 lines | ~800 |
| **Total** | **~1,542** | **~19,600** |

Well within the agent context window for a 2-point facade delivery story. No split
required.

---

## Tasks

| # | Task | Actor | Blocking |
|---|------|-------|---------|
| T-01 | Read RELEASING.md §1 + §4 in full before any action | implementer | yes |
| T-02 | Confirm all 8 depends_on story PRs merged to develop (`gh pr list --state merged`) | implementer | yes |
| T-03 | Confirm 24 required CI status checks green on develop HEAD (`gh api repos/BOHICA-LABS/prism/branches/develop/protection --jq '.required_status_checks.contexts[]'`; `gh run list --branch develop --limit 5`) | implementer | yes |
| T-04 | **HUMAN ACTION — BLOCKING** Verify `RELEASE_PROMOTE_TOKEN` secret is configured for BOHICA-LABS/prism; obtain human confirmation before proceeding | orchestrator | yes |
| T-05 | Verify `crates/prism-bin/Cargo.toml` version is still `1.0.0-dev` (ADR-064 §D2 pre-release exception; no bump required) | implementer | yes |
| T-06 | Verify release-tag.yml step 5b has the beta channel `--tag-pattern` in place from PR #281 | implementer | yes |
| T-07 | Edit RELEASING.md: add beta.2 attestation non-verifiability note per AC-009 | implementer | no |
| T-08 | Commit RELEASING.md change on a feature branch (`feature/S-BETA3-RELEASE-001`); open PR; merge after CI green | implementer | no |
| T-09 | Dispatch release-tag.yml: `gh workflow run release-tag.yml --repo BOHICA-LABS/prism --ref develop -f tag=v1.0.0-beta.3` | implementer | yes |
| T-10 | Monitor release.yml run: `gh run watch --repo BOHICA-LABS/prism` | implementer | yes |
| T-11 | Verify all release assets (AC-005): `gh release view v1.0.0-beta.3 --repo BOHICA-LABS/prism` | implementer | yes |
| T-12 | Download archive and verify SLSA attestation (AC-006): `gh release download v1.0.0-beta.3 --repo BOHICA-LABS/prism --pattern 'prism-*-linux-x86_64.tar.gz' --dir /tmp/beta3_verify && cd /tmp/beta3_verify && gh attestation verify prism-v1.0.0-beta.3-x86_64-unknown-linux-gnu.tar.gz --repo BOHICA-LABS/prism --signer-workflow BOHICA-LABS/prism/.github/workflows/release.yml` | implementer | yes |
| T-13 | Verify binary version identity (AC-007): extract and run `./prism --version`; confirm `prism 1.0.0-beta.3` | implementer | yes |
| T-14 | Verify install.sh in archive (AC-008): `tar tzf ... | grep 'specs/claroty.sensor.toml'`; `tar xOf ... scripts/install.sh | grep BOHICA-LABS` | implementer | yes |
| T-15 | Dispatch live-tenant validation per `.factory/ops/live-tenant-validation-runbook.md` against beta.3 binary (post-release gate; confirms all 20 issues observable in live Claroty xDome tenant) | implementer | post-release |
| T-16 | Add CHANGELOG entry under `[Unreleased] > Changed` in `CHANGELOG.md`: "v1.0.0-beta.3 released: 20 beta.2 live-test fixes bundled; SLSA attestation re-issued under BOHICA-LABS/prism org" | implementer | before PR |
| T-17 | Create story PR targeting develop; paste AC verification results in PR description | implementer | — |

**Note on T-07/T-08 vs T-09 ordering:** The RELEASING.md edit (T-07/T-08) should be
committed to develop via PR BEFORE the release-tag.yml dispatch (T-09), so the
documentation update is in the develop HEAD that the beta.3 tag points at. This
ensures the bundled release asset reflects the updated RELEASING.md.

---

## Previous Story Intelligence

**From S-REL-CHANGELOG-CHANNEL-SCOPE-001 (PR #281 — shipped 2026-09-09):**
- The manual `--tag-pattern` workaround required for beta.2 is now automatic. The
  release-tag.yml step 5b channel-detect block will set the beta pattern without
  any manual intervention.
- The idempotency guard in step 5b (checks if `## [VERSION]` already present in
  CHANGELOG.md) makes re-dispatching with the same tag safe if step 6 or 7 fails
  transiently.
- The empty-section guard exits 1 before any git commit if no qualifying commits
  exist. With 8 W1-W3 story fix-commits since beta.2, this will not fire.

**From S-REL-SPECS-TARBALL-001 (release that shipped beta.2):**
- The `publish-release` job in release.yml uses `gh release create --clobber` for
  idempotent re-runs if the publish job partially fails mid-upload.
- `checksums.txt` must come from CI — do not manually construct it.
- The specs tarball `prism-specs-<tag>.tar.gz` is uploaded by a separate step in
  the same `publish-release` job; wait for the full job to complete before checking
  for it in the release assets.

**From ADR-064 §D2 v2.3 (PRISM_VERSION injection, shipped during prior release work):**
- `prism-bin/Cargo.toml` version stays at `1.0.0-dev` on develop throughout the
  beta series. No bump is needed. The beta.3 binary reports `1.0.0-beta.3` because
  `build.rs` resolves `GITHUB_REF_NAME = v1.0.0-beta.3` → strips `v` → `1.0.0-beta.3`.
- On non-tag runs (PRs, branch pushes) the binary reports `1.0.0-dev`. This is correct.

**From S-MCP-TOOL-GATE-001 (W1 of this same beta.3 batch, first to land):**
- The `just check` pre-push gate is the correct final gate before release dispatch.
  Confirm develop passes `just check` locally before triggering.

---

## Library and Framework Requirements

No new libraries or framework changes. All toolchain dependencies are pre-existing:

| Tool | Version | Source |
|------|---------|--------|
| git-cliff | 2.14.1 | Pinned in release-tag.yml via `taiki-e/install-action` (SHA-pinned) |
| gh CLI | authenticated to BOHICA-LABS | Pre-existing; operator environment |
| Rust toolchain | per `rust-toolchain.toml` | Pre-existing; pinned by dtolnay/rust-toolchain in release.yml |
| cargo-zigbuild | 0.23.0 | Cached in release.yml (musl leg only; Cache cargo-zigbuild binary step) |
| actions/attest-build-provenance | existing pin in release.yml | No change; generates SLSA attestation per leg |

No new Cargo dependencies. No new Rust features. `crates_touched: []`.

---

## File Structure Requirements

| File | Action | Rationale |
|------|--------|-----------|
| `RELEASING.md` | MODIFY | Add beta.2 attestation non-verifiability note per AC-009 |
| `.github/workflows/release-tag.yml` | NO CHANGE | Channel-scoped --tag-pattern already in place (PR #281) |
| `.github/workflows/release.yml` | NO CHANGE | SLSA attestation steps already present; BOHICA-LABS org is implicit from repository context |
| `crates/prism-bin/Cargo.toml` | NO CHANGE | ADR-064 §D2 pre-release exception: stays at `1.0.0-dev` |
| `CHANGELOG.md` | AUTOMATED | git-cliff in release-tag.yml step 5b generates and commits the beta.3 section |

**No new files to create.**

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `RELEASE_PROMOTE_TOKEN` not configured or expired when dispatch runs | release-tag.yml step 5b exits with authentication error; no CHANGELOG commit, no tag. Resolution: re-scope PAT (§Prerequisites), re-dispatch with same tag (idempotency guard in step 5b makes re-dispatch safe). |
| EC-002 | Empty-section guard fires (no qualifying commits since beta.2 in the tag universe) | release-tag.yml step 5b exits 1 before any commit or push; no tag created. Resolution: verify W1-W3 story commits are `fix:` type (not skip types). W1-W3 commits are `fix:` by convention — this should not fire. |
| EC-003 | One or more of the 4 release.yml build legs fails | The `publish-release` job has `needs: build-release` — a partial matrix failure means no release is created. Recovery: RELEASING.md §6 "A build-release matrix leg failed" procedure (diagnose, fix via PR, delete tag, prefer new patch tag). |
| EC-004 | `publish-release` job fails mid-upload (partial assets) | Re-run the failed job via `gh run rerun <failed-run-id> --repo BOHICA-LABS/prism --failed`. The `--clobber` flag in the job handles partial uploads idempotently. |
| EC-005 | `gh attestation verify` exits non-zero despite release.yml succeeding | May indicate the attestation was not generated (permissions error on `attestations: write`). Check release.yml job logs for the `attest-build-provenance` step. The `id-token: write` + `attestations: write` permissions are declared at the job level. |
| EC-006 | `prism --version` reports `1.0.0-dev` from the archive | PRISM_VERSION injection failed. Possible cause: the tag was not a `refs/tags/` ref when release.yml ran (e.g., dispatch via non-tag push). Verify the tag trigger: `gh run list --repo BOHICA-LABS/prism --event push --limit 5`. |
| EC-007 | git-cliff generates v1.0.0-beta.3 section but includes nightly-tagged commits | This would indicate the beta channel `--tag-pattern` is not applied in step 5b. Verify PR #281's release-tag.yml changes are present in develop HEAD before dispatch. |

---

## History

| Version | Date | Author | Change |
|---------|------|--------|--------|
| v1.0 | 2026-09-17 | story-writer | Initial draft — authoritative scope: beta3-remediation-delta-analysis.md §Issue 14 + §Issue 15 + §S-BETA3-RELEASE-001; ADR-063 §D7 v1.14; ADR-064 §D2 v2.3; RELEASING.md §4; 9 ACs; tdd_mode=facade; RELEASE_PROMOTE_TOKEN human prerequisite captured; release-tag.yml (not release-promote.yml) clarified per RELEASING.md §1 pre-release exception; delta-analysis "develop→main promotion" language flagged as imprecise. |
