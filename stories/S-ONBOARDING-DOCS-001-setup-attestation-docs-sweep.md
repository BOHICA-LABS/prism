---
document_type: story
story_id: S-ONBOARDING-DOCS-001
title: "docs: SETUP.md §9 placeholder + base_url + attestation download prereq + docs sweep (beta.3 W5)"
wave: W5
epic_id: E-BETA3-REMEDIATION
priority: P1
status: ready
# BC status: N/A — pure documentation correction story.
# No subsystem behavioral contract governs operator installation documentation.
# The validator behavior cited in AC-001 (E-SPEC-022) is governed by BC-2.06.015/016;
# this story adds user-facing guidance about an existing error, it does NOT amend those BCs.
# Conforming per S-REL-011 pure-docs story precedent (behavioral_contracts: [] permitted).
# S-7.01 gate: status remains draft with behavioral_contracts: [] — this is the project-
# established pattern for docs-only stories with no new executable behavioral surface.
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-09-17T00:00:00Z"
modified: "2026-09-17"
phase: 3
cycle: wave-5-e-demo-fidelity
tdd_mode: facade
# tdd_mode: facade — justified because:
#   (1) All deliverables are markdown file edits; there is no compilable Rust code to
#       drive with Red Gate stub/test discipline.
#   (2) Quality gate is verifiable ACs assertable via grep/read on the modified files,
#       not Red Gate Rust tests.
#   (3) Mutation testing at wave gate (W5) replaces Red Gate density check per the
#       facade mode definition.
#   (4) The delta analysis (beta3-remediation-delta-analysis.md Part 2 Story Breakdown
#       table) explicitly specifies tdd_mode: facade for this story.
#   (5) Consistent with S-BETA3-RELEASE-001 facade mode in the same remediation wave.
#   No override of project CLAUDE.md production-grade default: facade is correct for
#   docs-correction stories; the "correctness" criterion is whether the doc text is
#   accurate and guiding, not whether a Rust test passes.
subsystems: []
# Subsystem anchor justification:
#   docs/SETUP.md and RELEASING.md are operator-facing documentation files.
#   No ARCH-INDEX subsystem owns installation runbooks or operator docs.
#   subsystems: [] per S-REL-011 docs-story precedent.
crates_touched: []
target_module: docs
capabilities: []
behavioral_contracts: []
# BC status: pending — pure docs story; no new BCs required or authored.
# The S-7.01 spec-first gate requires behavioral_contracts to be non-empty before
# status transitions to ready. For this story the gate is satisfied by the project
# convention that pure docs-correction stories carry behavioral_contracts: [] and
# remain status: draft until the full delivery gate (demo-recorder → pr-manager).
# This convention is established by S-REL-011 (docs/SETUP.md) and S-REL-005/S-REL-006
# precedents. A product-owner DOES NOT need to author BCs for this story to be
# dispatched — the AC verifiability criterion (grep-assertable doc changes) substitutes.
verification_properties: []
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Command accuracy: every command added or changed in SETUP.md §3 and §9 must be
    verified against the actual beta.3 binary and the live-tenant-validation-runbook
    before the story is closed. Aspirational commands or flags that don't exist in the
    real CLI are a defect."
  - "base_url example accuracy (Issue 17): the replacement base_url example must reflect
    the correct Claroty xDome API base URL format. Cross-reference
    .factory/ops/live-tenant-validation-runbook.md §5 Variant-1 which documents the
    canonical API URL (https://api.claroty.com). Do NOT use a dashboard URL."
  - "gh release download prerequisite (Issue 18): the added download step must use
    the exact gh CLI flag syntax that exists in the installed gh CLI version. Verify
    `gh release download --help` before documenting the step."
  - "Beta.2 attestation note (Issue 19): the RELEASING.md note must accurately state
    that beta.2 attestation references the drbothen/prism workflow (verified by
    checking the GitHub Actions attestation metadata for v1.0.0-beta.2), not the
    current BOHICA-LABS/prism workflow. Do not state facts about the attestation that
    cannot be verified."
depends_on: []
# Dependency anchor justification:
#   This story has no technical dependencies on other beta.3 stories.
#   Documentation corrections to SETUP.md and RELEASING.md can be authored and merged
#   independently of any code change. The delta analysis dependency graph explicitly
#   states: "S-ONBOARDING-DOCS-001 (no deps; can run any time)". This story is in
#   Batch 1 (parallel execution) per the delta analysis Part 2 Parallel Execution Batches.
blocks: [S-BETA3-RELEASE-001]
# Blocks anchor justification:
#   S-BETA3-RELEASE-001 depends on all Batch 1 stories being merged to develop
#   before the beta.3 release tag is created. S-ONBOARDING-DOCS-001 is in Batch 1.
#   The doc fixes should ship inside the beta.3 release, not as a post-release patch.
points: 2
estimated_days: 0.5
risk: LOW
# Risk justification: Documentation-only story. No Rust code changes. No CI changes.
# No behavioral contract amendments. Primary risk is command-text inaccuracy
# (mitigated by verify-before-document discipline). No blast radius across crates.
acceptance_criteria_count: 4
red_gate_tests: 0
# red_gate_tests: 0 — facade mode; no Red Gate Rust tests. ACs are grep-assertable
# doc changes. Wave-gate mutation testing applies at W5 gate.
inputs:
  - "docs/SETUP.md"
  - "RELEASING.md"
  - ".factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md §§Issue 16-19"
  - ".factory/ops/live-tenant-validation-runbook.md"
input-hash: ""
# input-hash: pending state-manager compute-input-hash pass
---

# S-ONBOARDING-DOCS-001: SETUP.md §9 Placeholder Clarity + API base_url + Attestation Download Prereq + Docs Sweep (Beta.3 W5)

## Problem Statement

Four onboarding defects were identified during the beta.2 Monroe live-test triage
(D-2520, Group F). All are documentation inaccuracies in `docs/SETUP.md` and
`RELEASING.md` discovered when a real operator followed the published guide against a
live Claroty xDome tenant.

**Issue 16 — SETUP.md §9 `my-client` placeholder too literal**
SETUP.md §9 uses `my-client` as the example `org_slug` in directory names, the
customer overlay TOML `instance_id`, and the `prism credential set --org-slug` command.
When an operator copy-pastes without substituting, Prism's validator correctly rejects
the spec with `E-SPEC-022` (org_id not found in registry) because `my-client` was
registered in §8 as their actual `org_slug`, but the spec's `instance_id` still says
`claroty@my-client`. The error message is technically correct but the guide gives no
hint that E-SPEC-022 means a name mismatch. The placeholder is not visually distinct
enough to prevent copy-paste without substitution.

**Issue 17 — SETUP.md §9 example `base_url` points to dashboard host → 403**
The Step 2 overlay TOML example `base_url = "https://your-tenant.claroty.com"` is
formatted as a tenant subdomain but was interpreted by operators as the xDome dashboard
URL. The Claroty xDome API base URL is `https://api.claroty.com` (generic) or a
tenant-specific API subdomain. The dashboard host returns 403 for API calls. The
example needs to be the API endpoint form, not the dashboard form, with an explicit
note distinguishing the two.

**Issue 18 — `gh attestation verify` runbook step missing download prerequisite**
`docs/SETUP.md §3` and `RELEASING.md §5` both show a `gh attestation verify` command
that assumes the archive file is already present in the current working directory.
`gh attestation verify` requires the artifact to be a local file. No download step
precedes the verify command in either document, causing "file not found" failures for
operators following the runbook literally.

**Issue 19 — RELEASING.md missing beta.2 attestation non-verifiability note + residual sweep**
The `v1.0.0-beta.2` SLSA attestation was generated by the `drbothen/prism` GitHub
Actions workflow (before the org rename to BOHICA-LABS). After the rename, `gh
attestation verify --repo BOHICA-LABS/prism` fails against the beta.2 archive because
the attestation references the old workflow namespace. RELEASING.md has no note
documenting this. Additionally, the full docs tree requires a sweep to confirm no
residual `drbothen` references remain after the org rename (D-2516).

---

## Narrative

As an operator deploying Prism against a Claroty xDome tenant for the first time,
I want the SETUP.md onboarding guide to use unambiguous placeholder notation, correct
API endpoint examples, and a correct attestation verification procedure, so that I
can complete setup on the first attempt without encountering confusing errors or
running commands that fail because a prerequisite step was omitted.

---

## Acceptance Criteria

### AC-001 — SETUP.md §9: `<YOUR-CLIENT-ID>` placeholder + E-SPEC-022 guidance (Issue 16)

`docs/SETUP.md §9 "Onboard a Claroty xDome Client"` is updated such that:

1. All occurrences of the literal string `my-client` that represent the operator's
   chosen `org_slug` are replaced with the visually-distinct placeholder
   `<YOUR-CLIENT-ID>`. This applies to: the `mkdir` directory name in Step 1
   (`/etc/prism/specs/customers/<YOUR-CLIENT-ID>`), the comment line at the top of
   the Step 2 overlay TOML (`# Per-org overlay for claroty sensor — <YOUR-CLIENT-ID>`),
   the `instance_id` field in the overlay TOML (`claroty@<YOUR-CLIENT-ID>`), and
   the `--org-slug` flag in Step 4's `prism credential set` command.
2. A bold callout is present in Step 1 or at the top of §9 that explicitly states:
   **"Replace `<YOUR-CLIENT-ID>` with the exact `org_slug` you declared under `[[orgs]]`
   in `prism.toml` (§6). These two values must match exactly."**
3. A note is present (inline or in a troubleshooting callout) that explains:
   `E-SPEC-022` from `prism validate-config` or `prism start` means the `org_slug`
   in the customer overlay directory name or `instance_id` does not match any registered
   `org_slug` in `prism.toml`. The fix is to confirm the directory path and `instance_id`
   use exactly the same slug as the `[[orgs]]` entry.

Verification: `grep -c '<YOUR-CLIENT-ID>' docs/SETUP.md` returns ≥ 4 (one per
occurrence in §9 Step 1, Step 2 overlay comment, Step 2 `instance_id`, Step 4 `--org-slug`).
`grep -c 'E-SPEC-022' docs/SETUP.md` returns ≥ 1.

No occurrence of the literal string `my-client` remains in the §9 walkthrough steps
(the literal may still appear in §12 "First Smoke Query" examples — that section is
out of scope for this AC and must be evaluated separately at delivery time).

### AC-002 — SETUP.md §9: Correct `base_url` API endpoint form (Issue 17)

`docs/SETUP.md §9 Step 2 "Create the overlay TOML"` is updated such that:

1. The example overlay TOML `base_url` value is the Claroty xDome API base URL format
   `https://api.claroty.com` (or the tenant-specific API subdomain form documented in
   `.factory/ops/live-tenant-validation-runbook.md §5 Variant-1`). The placeholder
   must NOT be a value that could be interpreted as the xDome dashboard URL.
2. A note immediately follows the overlay TOML block stating: the `base_url` must be
   the **API** base URL for the Claroty xDome instance, not the dashboard or portal
   URL. If `check_sensor_health` returns `E-SENSOR-030` with an HTTP 403 from the
   Claroty API, the most common cause is using the dashboard URL as `base_url`.
   Cross-reference to `.factory/ops/live-tenant-validation-runbook.md` (or `docs/DEMO-RUNBOOK.md`
   if the runbook URL is documented there) for the correct API endpoint format.

Verification: `grep 'base_url' docs/SETUP.md` in the §9 overlay TOML block returns a
value that matches `api.claroty.com` or equivalent API-path form (not a `/dashboard`
or `/portal` URL). `grep -c 'dashboard' docs/SETUP.md` in the §9 Step 2 block returns
0 (no dashboard reference in the overlay example itself; a note warning NOT to use
the dashboard URL is allowed and expected).

### AC-003 — SETUP.md §3 + RELEASING.md §5: `gh release download` prerequisite step (Issue 18)

Both `docs/SETUP.md §3 "Verify the Installation"` and `RELEASING.md §5 "Release Notes
Convention" → "Verify build provenance"` are updated such that:

1. An explicit `gh release download` step appears immediately before the
   `gh attestation verify` command in each location. The step must include:
   - The `gh release download` command with `--pattern` targeting the platform-specific
     archive (e.g. `--pattern 'prism-*-linux-x86_64.tar.gz'`), `--dir` specifying a
     local output directory, and the release tag as the first positional argument.
   - A `cd` or working-directory note so the verify command runs from the same
     directory as the downloaded file.
2. The `gh attestation verify` command that follows must reference the local file
   path produced by the download step (e.g. `prism-<version>-<triple>.tar.gz` inside
   the `--dir` directory).
3. The exact command syntax must be compatible with the `gh` CLI version documented
   in `docs/SETUP.md §1` prerequisites table (any version listed as "any" means the
   technical-writer must verify the flags exist in the minimum supported version).

Verification: in `docs/SETUP.md §3`, `grep -A5 'attestation verify'` shows a
`gh release download` command appearing before the `gh attestation verify` command.
Same pattern holds in `RELEASING.md §5` verify provenance block. No version of the
`gh attestation verify` command in either file appears without a preceding download
step within the same documentation block.

### AC-004 — RELEASING.md: Beta.2 attestation non-verifiability note + drbothen residual sweep (Issue 19)

1. `RELEASING.md` contains a note — placed at or near the `§5 Verify build provenance`
   section — documenting that the `v1.0.0-beta.2` SLSA attestation is **not verifiable**
   under `BOHICA-LABS/prism` because the attestation was generated by the
   `drbothen/prism` GitHub Actions workflow before the org rename. The note must:
   - State that `gh attestation verify --repo BOHICA-LABS/prism` fails against the
     beta.2 archive (the attestation references the old workflow namespace).
   - State that `v1.0.0-beta.3` is the first release with a verifiable attestation
     under `BOHICA-LABS/prism`.
2. A sweep of `docs/`, `README.md`, and `RELEASING.md` confirms zero remaining
   occurrences of `drbothen` in any human-facing documentation file (excluding
   `.factory/` archive files and git history). The technical-writer MUST run
   `grep -rn "drbothen" docs/ README.md RELEASING.md` and confirm exit 1 (no matches)
   before declaring this AC satisfied. If any drbothen reference is found, it must
   be replaced with `BOHICA-LABS` before the story PR is opened.

Verification: `grep -c 'drbothen.*beta.2\|beta.2.*non-verif\|v1.0.0-beta.2.*attestation'
RELEASING.md` returns ≥ 1 (a note about the beta.2 attestation is present).
`grep -rn "drbothen" docs/ README.md RELEASING.md` returns exit 1 (no matches).

---

## Behavioral Contracts

| ID | Title | Relationship |
|----|-------|--------------|
| *(none)* | N/A — pure docs-correction story | S-7.01 status=draft with behavioral_contracts: [] per S-REL-011 precedent |

This story corrects documentation that describes existing binary behavior and existing
CLI procedures. No new behavioral contracts are introduced. The validator error
E-SPEC-022 referenced in AC-001 is governed by BC-2.06.015/016, which this story does
not amend. The `gh attestation verify` procedure is an operator UX concern with no
governing MCP behavioral contract. The S-7.01 spec-first gate is satisfied by the
project convention that pure docs-correction stories carry `behavioral_contracts: []`
and remain `status: draft` through the delivery cycle.

---

## Architecture Mapping

| Deliverable | File | Pure/Effectful |
|-------------|------|---------------|
| §9 placeholder + E-SPEC-022 note (AC-001) | `docs/SETUP.md` | Pure (markdown) |
| §9 base_url API endpoint example (AC-002) | `docs/SETUP.md` | Pure (markdown) |
| §3 gh attestation download prereq (AC-003) | `docs/SETUP.md` | Pure (markdown) |
| §5 attestation verify download prereq (AC-003) | `RELEASING.md` | Pure (markdown) |
| Beta.2 attestation note + drbothen sweep (AC-004) | `RELEASING.md` | Pure (markdown) |

No architecture section files are affected. No modules, crates, or ADRs are amended.

---

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~5,000 |
| `docs/SETUP.md` (766 lines) | ~8,000 |
| `RELEASING.md` (742 lines) | ~9,000 |
| `.factory/ops/live-tenant-validation-runbook.md` (250 lines, §5 reference for base_url) | ~3,000 |
| `CHANGELOG.md` (delivery task) | ~3,000 |
| BC files (0 BCs) | 0 |
| **Total** | **~28,000 tokens** |

Well within 20–30% of a standard 200k-token agent context window. No story splitting required.

---

## Tasks

> **tdd_mode: facade** — no Red Gate stubs or test-first phase. Delivery is direct doc edits.

1. [ ] **Read `docs/SETUP.md` in full** — confirm the current state of §9 (Steps 1–4,
   directory layout) and §3 (attestation verify block) before making any edits.

2. [ ] **Read `RELEASING.md §5`** — confirm the current state of the "Verify build
   provenance" code block before editing.

3. [ ] **Read `.factory/ops/live-tenant-validation-runbook.md §5 Variant-1`** — confirm
   the canonical Claroty API base URL (`https://api.claroty.com`) before editing §9.

4. [ ] **Edit `docs/SETUP.md §9` — AC-001 (Issue 16):**
   - Replace all `my-client` occurrences in §9 walkthrough steps with `<YOUR-CLIENT-ID>`
     (mkdir directory, overlay TOML comment, instance_id, --org-slug flag, directory layout)
   - Add bold callout: **"Replace `<YOUR-CLIENT-ID>` with the exact `org_slug` you declared
     under `[[orgs]]` in `prism.toml` (§6). These two values must match exactly."**
   - Add E-SPEC-022 guidance note in §9 or in the §10 First Boot troubleshooting table

5. [ ] **Edit `docs/SETUP.md §9` — AC-002 (Issue 17):**
   - Update overlay TOML `base_url` example to `https://api.claroty.com`
   - Add inline note: "Set `base_url` to the API base URL for your Claroty xDome
     instance, **not** the dashboard or portal URL. If `check_sensor_health` returns
     E-SENSOR-030 with HTTP 403, a dashboard URL is the most common cause."

6. [ ] **Edit `docs/SETUP.md §3` — AC-003 (Issue 18):**
   - Add `gh release download` step immediately before the `gh attestation verify`
     command block, using the exact command from the delta analysis (Issue 18 Fix
     block in `beta3-remediation-delta-analysis.md`)
   - Ensure working-directory context is clear (cd to the download dir before verify)

7. [ ] **Edit `RELEASING.md §5` — AC-003 (Issue 18):**
   - Add the same `gh release download` prerequisite step before the verify command
     in the "Verify build provenance" code block

8. [ ] **Edit `RELEASING.md` — AC-004 (Issue 19):**
   - Add a note near §5 "Verify build provenance" documenting that `v1.0.0-beta.2`
     SLSA attestation is not verifiable under `BOHICA-LABS/prism`; the attestation
     was generated by `drbothen/prism` GitHub Actions workflow before the org rename
   - State that `v1.0.0-beta.3` is the first release with a verifiable attestation
     under `BOHICA-LABS/prism`

9. [ ] **Run drbothen sweep — AC-004:**
   - `grep -rn "drbothen" docs/ README.md RELEASING.md` — must return exit 1 (no matches)
   - If matches found: replace with `BOHICA-LABS` (fix in scope per CLAUDE.md Rule 4)

10. [ ] **AC self-verification pass** before opening the PR:
    - `grep -c '<YOUR-CLIENT-ID>' docs/SETUP.md` → ≥ 4
    - `grep -c 'E-SPEC-022' docs/SETUP.md` → ≥ 1
    - `grep 'base_url' docs/SETUP.md` in §9 block → contains `api.claroty.com`
    - `grep -A5 'attestation verify' docs/SETUP.md` → `gh release download` appears before it
    - `grep -A5 'attestation verify' RELEASING.md` → `gh release download` appears before it
    - `grep -c 'beta.2\|drbothen' RELEASING.md` (drbothen line) → beta.2 attestation note present
    - `grep -rn "drbothen" docs/ README.md RELEASING.md` → exit 1 (no matches)

11. [ ] **Add CHANGELOG entry** under `## [Unreleased]` > `### Fixed`:
    ```
    - Fix SETUP.md §9 placeholder clarity: replace `my-client` with `<YOUR-CLIENT-ID>`
      and add E-SPEC-022 guidance for org_slug mismatch errors
    - Fix SETUP.md §9 Claroty overlay base_url example to reference API endpoint
      (https://api.claroty.com) with note distinguishing API URL from dashboard URL
    - Add `gh release download` prerequisite step before `gh attestation verify` in
      SETUP.md §3 and RELEASING.md §5 to prevent "file not found" failures
    - Add RELEASING.md note documenting that v1.0.0-beta.2 SLSA attestation is not
      verifiable under BOHICA-LABS/prism (generated by drbothen/prism workflow
      before org rename); v1.0.0-beta.3 is the first verifiable release
    ```

12. [ ] Open PR targeting `develop`.

---

## Previous Story Intelligence

No previous story in this epic exists yet. This is the first story in `E-BETA3-REMEDIATION`
to be dispatched to technical-writer. The parent delta analysis
(`.factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md`) is the
complete design authority for the beta.3 remediation scope.

**Key prior-art lessons from `S-REL-011` (SETUP.md original authoring story):**
- Every command in the setup guide must be verified against the actual binary and scripts
  before the story is closed. Aspirational commands are a defect.
- The setup guide describes `v1.0.0` model behavior. Future changes (e.g., S-REL-010
  embedded specs) should not be backfilled into this doc.
- `prism.toml.example` and the credential model must remain as-is; this story only
  touches §9 placeholder text and §3 attestation procedure — no other sections.

---

## Architecture Compliance Rules

This story produces no Rust code. The following rules are extracted as a reminder for
the technical-writer who reviews commands for accuracy:

1. **AD-017 — No credentials in docs:** The overlay TOML `base_url` is a hostname
   only. Do NOT add bearer tokens, API keys, or any credential values to `docs/SETUP.md`
   under any circumstances. The note about E-SPEC-022 must not reproduce any credential
   values or tenant-specific identifiers from the live test run.

2. **No CLI flags that don't exist:** Every flag added to the `gh release download`
   step (AC-003) must be verified against the `gh` CLI help output. The flags
   `--pattern`, `--dir`, and the positional release tag are standard `gh release download`
   flags as of gh CLI v2.x. Verify before documenting.

3. **Command sequence is the specification:** The exact command sequences in SETUP.md
   and RELEASING.md are consumed by operators verbatim. Do not abbreviate steps, merge
   commands onto one line "for brevity," or add `&&` chains that silently swallow
   intermediate errors. Each step is its own shell command.

4. **SETUP.md §9 `my-client` scope:** Only §9 walkthrough steps (Steps 1–4 and the
   directory layout) use `my-client` as a placeholder to replace. §12 "First Smoke
   Query" also uses `my-client` in example queries ("Run check_sensor_health for the
   my-client client"). Whether to update §12 is left to the technical-writer's
   judgment at delivery time — it is NOT required by AC-001. If §12 is updated,
   the same `<YOUR-CLIENT-ID>` placeholder should be used for consistency.

---

## Library & Framework Requirements

This story has no Rust, npm, or Python dependencies. The only tooling required is:

| Tool | Version | Purpose |
|------|---------|---------|
| `gh` CLI | any (verify ≥ 2.0 supports `gh release download --pattern --dir`) | AC-003 command syntax verification |
| `grep` | any | AC self-verification steps |

No Cargo.toml changes. No new crates.

---

## File Structure Requirements

| File | Action | Description |
|------|--------|-------------|
| `docs/SETUP.md` | MODIFY | AC-001 (§9 placeholder), AC-002 (§9 base_url), AC-003 (§3 download prereq) |
| `RELEASING.md` | MODIFY | AC-003 (§5 download prereq), AC-004 (beta.2 attestation note) |
| `CHANGELOG.md` | MODIFY | Add [Unreleased] > Fixed entries (Task 11) |

No new files are created. No other files are modified. In particular:
- `.factory/ops/live-tenant-validation-runbook.md` is read-only reference; not modified
- `docs/DEMO-RUNBOOK.md` is read-only reference; not modified unless drbothen sweep (Task 9) finds a hit
- All `.factory/specs/behavioral-contracts/` files are NOT touched
- All crate source files (`crates/**/*.rs`) are NOT touched

---

## Edge Cases

| ID | Description | Expected Handling |
|----|-------------|-------------------|
| EC-001 | §12 "First Smoke Query" also uses `my-client` — out of scope for AC-001 | Do not update §12 unless the technical-writer judges it necessary for consistency; document the decision in the PR description |
| EC-002 | `gh release download` flags differ between gh CLI v1.x and v2.x | Verify flags against `gh release download --help`; if v1.x incompatible, add a minimum version note |
| EC-003 | Drbothen sweep finds a hit in `docs/demo-evidence/` (historical records) | Historical demo-evidence records documenting prior behavior are exempt from the sweep; only update live-operator-facing docs (docs/, README.md, RELEASING.md) |
| EC-004 | Base_url note references the runbook which is a `.factory/ops/` path (not operator-accessible in a binary release) | Cross-reference `docs/DEMO-RUNBOOK.md` instead if it contains the correct API URL; otherwise use the inline note only |
| EC-005 | RELEASING.md beta.2 note is placed and then immediately superseded by beta.3 CHANGELOG | The note should be in the `§5 Verify build provenance` section or a `§5.1 Note on pre-beta.3 attestations` sub-section so it remains visible after CHANGELOG updates prepend new content |

---

## Story-Level Holdout Gate Assessment

**Assessment: N/A — holdout gate does not apply to this story.**

The story-level holdout gate (human-approved 2026-07-13, CLAUDE.md §Pipeline Authority) applies to stories that deliver **observable product behavior** against which hidden wire-level MCP scenarios can be evaluated. A pure documentation correction story has no binary behavioral surface for holdout evaluation:

- No new MCP tool is being registered or changed.
- No Rust function is being added or modified.
- No sensor data pipeline behavior changes.
- The "deliverable" is correct text in markdown files — not a behavior that can be asserted at the wire level.

Holdout scenarios require "real MCP stdio + DTU, wire-level assertions, scoped to the story's touched surface." There is no MCP stdio surface for a docs edit. Accordingly, product-owner MUST NOT author holdout scenarios for this story. No hidden holdout pool is created. The story proceeds directly from demo-recorder (docs PR screenshot) to pr-manager without a holdout gate.

---

## History

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-09-17 | story-writer | Initial draft. Issues 16-19 from beta3-remediation-delta-analysis.md §§Issue 16-19. 4 ACs, tdd_mode: facade, no behavioral contracts (pure docs-correction story per S-REL-011 precedent), depends_on: [], blocks: [S-BETA3-RELEASE-001]. Story-level holdout gate N/A — no executable behavioral surface. |
