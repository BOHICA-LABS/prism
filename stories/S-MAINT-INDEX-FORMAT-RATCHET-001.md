---
document_type: story
story_id: "S-MAINT-INDEX-FORMAT-RATCHET-001"
title: "Corpus-Wide Records-Lint Cleanup — BC-INDEX Parenthetical Version Format + ARCH-INDEX ADR Inline-Changelog Collapse + records-lint L11 Gate"
wave: tbd
epic_id: maintenance
priority: P3
status: draft
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-09-16"
modified: "2026-09-16"
input-hash: "[live-state]"
inputs: []
traces_to: ""
cycle: "batch-0-spec-gate"
phase: 3
tdd_mode: standard
track: "Platform Engineering"
subsystems: []
crates_touched: []
target_module: ".factory/specs/behavioral-contracts/BC-INDEX.md, .factory/specs/architecture/ARCH-INDEX.md, scripts/records-lint.sh, .factory/policies.yaml"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship — POL-35 infra-only-holdout-exemption applies; pure records/tooling story; no behavioral contract required before dispatch per human direction 2026-09-16
verification_properties: []
holdout_scenarios: []
depends_on: []
blocks: []
points: 5
estimated_days: 1
risk: LOW
acceptance_criteria_count: 5
red_gate_tests: 0
estimated_passes: "tbd"
assumption_validations: []
risk_mitigations: []
tags:
  - records-lint
  - index-format
  - process-gap
  - factory-tooling
  - records-only
---

# S-MAINT-INDEX-FORMAT-RATCHET-001: Corpus-Wide Records-Lint Cleanup — BC-INDEX Parenthetical Version Format + ARCH-INDEX ADR Inline-Changelog Collapse + records-lint L11 Gate

## Origin

**Human-directed deferral:** Batch-0 spec-gate OBS-01 (consistency pass-16, 2026-09-16) + F-B0P14-LOW-001 (ARCH-INDEX parenthetical currency drift, pass-14).

**Decision context (2026-09-16):** The human accepted Batch-0 at CLEAN(PR-merge) and directed that the two pre-existing index-formatting anti-patterns identified as OBS-01 be filed as a dedicated maintenance story rather than fixed inside the scoped Batch-0 perimeter. Rationale: fixing one BC-INDEX row while leaving the other 8 sibling rows in parenthetical format would either break sibling consistency (leaving mixed formats) or require a corpus-wide sweep that exceeds the Batch-0 perimeter gate — both outcomes are worse than a clean dedicated story.

**Two pre-existing anti-patterns:**

1. **BC-INDEX `(vX.Y current)` parenthetical format (9 rows):** BC-2.02.006, BC-2.08.009, BC-2.15.010, BC-2.16.002, BC-2.16.003, BC-2.16.008, BC-2.16.009, BC-2.16.018 use `| active (vX.Y current) |` vs the canonical `| active vX.Y |` used by 69+ other rows. The parenthetical form is counted "unverifiable" by records-lint L10 (TD-VSDD-092), so those 9 rows contribute zero mechanical L10 pass coverage.

2. **ARCH-INDEX ADR-row inline-changelog chains:** ADR rows carry long inline `(vX.Y ...)(vX.Y-1 ...)...` changelog chains in the Status cell (pre-existing POL-40 anti-pattern; POL-40 as currently written scopes to STORY-INDEX/BC-INDEX only). Every ADR version bump requires manual leading-parenthetical currency curation that has repeatedly drifted (F-B0P14-LOW-001: ARCH-INDEX parenthetical currency drift). The full changelog already lives in each ADR body — the ARCH-INDEX row only needs a current-state pin.

**Relationship to sibling stories:** SIBLING to the proposed S-MAINT-RG-ANCHOR-DRIFT-GATE-001 (D-2524/F12) and the D-2528 formula-propagation-sweep candidate — same records-lint-gate family, distinct scope (this story is index-row FORMAT; those are RG-anchor/formula DRIFT). May be consolidated at materialization if the human prefers.

---

## Narrative

As a consistency-validator and adversary running corpus-wide index checks,
I want the BC-INDEX and ARCH-INDEX to use canonically formatted status/version cells corpus-wide with no mixed parenthetical forms,
so that records-lint L10 can mechanically verify every index row and adversarial passes stop minting recurring LOW/OBS findings for format drift that cannot be fixed inside any single scoped perimeter.

---

## Acceptance Criteria

### AC-001 — BC-INDEX parenthetical rows normalized to canonical bare form
(Traceability to BCs is pending PO authorship — POL-35 infra-only-holdout-exemption)

All 9 BC-INDEX rows currently using the `| active (vX.Y current) |` parenthetical format MUST be rewritten to the canonical bare form `| active vX.Y |` in a single corpus-wide burst. Affected rows: BC-2.02.006, BC-2.08.009, BC-2.15.010, BC-2.16.002, BC-2.16.003, BC-2.16.008, BC-2.16.009, BC-2.16.018 (9 rows total, to be confirmed against BC-INDEX at implementation time by grepping for `(v` in the Status column). Post-fix: `scripts/records-lint.sh --full-scan` L10 MUST report zero "unverifiable" rows for those 9 BCs.

### AC-002 — ARCH-INDEX ADR Status cells collapsed to one-line current-state pins
(Traceability to BCs is pending PO authorship — POL-35 infra-only-holdout-exemption)

Every ADR row in ARCH-INDEX MUST have its Status cell reduced to a single `ACCEPTED vX.Y (date; one-line current-state)` entry — removing inline `(vX.Y ...)(vX.Y-1 ...)...` changelog chains. The full revision history already lives in each ADR body; the ARCH-INDEX row is a current-state pin only. Post-fix: no ADR row in ARCH-INDEX MUST contain more than one parenthetical version reference in its Status cell.

### AC-003 — POL-40 scope extended to cover ARCH-INDEX ADR rows
(Traceability to BCs is pending PO authorship — POL-35 infra-only-holdout-exemption)

`.factory/policies.yaml` POL-40 MUST be amended to explicitly name ARCH-INDEX ADR Status cells as a covered scope, alongside the existing STORY-INDEX and BC-INDEX coverage. The amended policy MUST state the single-current-state-pin requirement for ARCH-INDEX ADR rows. POL-40 version bumped. policies.yaml `version:` frontmatter bumped. ARCH-INDEX `version:` frontmatter bumped as part of the AC-002 edit.

### AC-004 — records-lint L11 implemented and gating index-row format on commit
(Traceability to BCs is pending PO authorship — POL-35 infra-only-holdout-exemption)

`scripts/records-lint.sh` MUST gain an **L11** check that mechanically enforces index-row current-state-pin format (one-line, canonical version pin, no inline chains) for BC-INDEX, ARCH-INDEX, and STORY-INDEX rows on every commit. L11 MUST:
- Hard-block any staged addition to a BC-INDEX or ARCH-INDEX row that contains an inline changelog chain (detected by `(v` appearing more than once in the same row's Status cell).
- Hard-block any staged addition to a BC-INDEX row that uses the parenthetical form `(vX.Y current)` instead of the bare form `active vX.Y`.
- Include at least 4 `--self-probe` PASS/FAIL cases covering the two new check arms.
- Be documented in the L11 config block at the top of `records-lint.sh`.

### AC-005 — Full records-lint L10 corpus coverage improves (zero unverifiable rows from the 9 BC rows)
(Traceability to BCs is pending PO authorship — POL-35 infra-only-holdout-exemption)

After AC-001 and AC-004 land, `scripts/records-lint.sh --full-scan` L10 MUST report zero "unverifiable" rows for the 9 BCs identified in AC-001. This is a success criterion of AC-001, not a separate implementation task — recorded here as an observable gate for the delivering agent to verify before declaring the story done.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| BC-INDEX | `.factory/specs/behavioral-contracts/BC-INDEX.md` | Pure (data records — format normalization only) |
| ARCH-INDEX | `.factory/specs/architecture/ARCH-INDEX.md` | Pure (data records — ADR row collapse) |
| records-lint.sh | `scripts/records-lint.sh` | Effectful (commit gate; runs in pre-commit hook chain) |
| policies.yaml | `.factory/policies.yaml` | Pure (governance config) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A BC-INDEX row uses `(vX.Y current)` but the BC's actual frontmatter version has since advanced | Normalize to bare `active vX.Y-ACTUAL` where vX.Y-ACTUAL is read from the BC's frontmatter `version:` field at implementation time; do not preserve a stale parenthetical pin |
| EC-002 | An ARCH-INDEX ADR row's Status cell has no inline changelog chain — just a single current-state entry | Skip; no edit needed for that row |
| EC-003 | L11 is added but triggers on pre-existing uncovered lines in BC-INDEX or ARCH-INDEX (ratchet scope) | L11 MUST use ratchet mode (staged-additions-only, same as L9) so pre-existing lines that are not staged are not hard-blocked |
| EC-004 | A `--full-scan` run after AC-001 still shows "unverifiable" rows for other BCs not in the AC-001 list | Expected — AC-005 only guarantees the 9 listed rows; other "unverifiable" rows are out of scope for this story |

---

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~3,000 |
| BC-INDEX (read to identify 9 rows) | ~15,000 |
| ARCH-INDEX (read to identify ADR rows) | ~8,000 |
| records-lint.sh (read + edit for L11) | ~6,000 |
| policies.yaml (read + edit for POL-40) | ~2,000 |
| Total | ~34,000 |

Within a single agent context window. No split required.

---

## Tasks

### Red Gate tests

N/A — this story has `tdd_mode: standard` (records and tooling sweep; no Rust production code; POL-35 infra-only-holdout-exemption). The L11 implementation in `records-lint.sh` MUST have `--self-probe` cases that exercise both L11 check arms before the script is merged (self-probe doubles as the test vehicle for the bash gate logic). Red Gate density check deferred to implementation-time when `tdd_mode` is confirmed and the implementing module is designated.

### Implementation tasks

- [ ] T-01: Read BC-INDEX; confirm all 9 rows using `(vX.Y current)` form; record exact list. Normalize each to bare `active vX.Y` reading the BC's actual frontmatter `version:`. Verify `scripts/records-lint.sh --full-scan` L10 passes on those rows post-edit.
- [ ] T-02: Read ARCH-INDEX; identify all ADR rows with inline changelog chains `(vX.Y ...)(vX.Y-1 ...)`. Collapse each to `ACCEPTED vX.Y (date; one-line current-state)`. Bump ARCH-INDEX `version:` frontmatter.
- [ ] T-03: Amend `policies.yaml` POL-40 to add ARCH-INDEX ADR Status cells to scope. Bump POL-40 version and policies.yaml `version:` frontmatter.
- [ ] T-04: Add L11 check to `scripts/records-lint.sh` — two arms: (a) inline-changelog-chain detection (multiple `(v` in same Status cell), (b) parenthetical `(vX.Y current)` form detection. Add at least 4 `--self-probe` PASS/FAIL cases. Verify `--self-probe` exits 0 with expected results.
- [ ] T-05: Run `scripts/records-lint.sh --full-scan` end-to-end. Confirm L10 reports zero unverifiable rows for the 9 BCs from AC-001. Confirm L11 reports no violations on the now-normalized corpus.
- [ ] T-06: Add a CHANGELOG entry under `[Unreleased] > Changed` describing the index-format normalization and L11 gate, before creating the PR.

---

## Previous Story Intelligence

**S-MAINT-RG-LIST-GATE-001** — structural precedent for maintenance stories that combine a corpus cleanup with a gate implementation. Same pattern: identify corpus drift → fix in one sweep → add mechanical gate to prevent recurrence.

**S-MAINT-ADR-ANCHOR-GATE-001** — closest sibling: another records-tooling story with no BCs, behavioral_contracts: [], POL-35 exemption. Frontmatter template is the direct pattern for this story.

**S-MAINT-ANTIPIN-SWEEP-001/002** — prior corpus sweeps of POL-39 version pins. Established the pattern: identify all instances via grep → fix in one burst → confirm via lint gate. Same approach applies here for AC-001 and AC-002.

**S-MAINT-L11-GATE-001** — the already-shipped L11 narrative-version-pin gate. This story adds a SECOND L11 arm (index-row format), not a competing implementation. Implementing agent MUST read S-MAINT-L11-GATE-001 and `scripts/records-lint.sh` to understand the existing L11 structure before extending it.

**N/A — first story in the index-FORMAT enforcement sub-family.** The sibling stories (S-MAINT-RG-ANCHOR-DRIFT-GATE-001, D-2528 formula sweep) cover DRIFT; this story covers FORMAT. No prior story covers the specific `(vX.Y current)` parenthetical normalization.

---

## Architecture Compliance Rules

1. **No `crates/` modifications.** This story MUST NOT add, remove, or edit any file under `crates/`. Scope is `.factory/` records and `scripts/records-lint.sh` only.
2. **records-lint.sh ratchet mode.** L11 MUST operate in ratchet mode (staged-additions-only by default) matching L9's existing design (TD-VSDD-092 §L9 config). `--full-scan` flag enables corpus-wide audit.
3. **Single atomic commit.** Per TD-VSDD-053, all edits (BC-INDEX, ARCH-INDEX, policies.yaml, records-lint.sh) MUST land in ONE `.factory/` commit. Multi-commit chains blocked by factory-dispatcher.
4. **POL-40 canonical form.** POL-40 amendment MUST follow the existing policies.yaml entry format. Version bump follows the existing `vX.Y -> vX.Y+1` convention for policy amendments.
5. **L10 `--self-probe` parity.** Adding new L11 self-probe cases MUST NOT break the existing 34/34 L10 self-probe case count. Run `--self-probe` before and after to verify.

---

## Library & Framework Requirements

| Tool | Version | Note |
|------|---------|------|
| bash | system (>=3.2) | records-lint.sh is a bash script; no external deps beyond standard POSIX tools |
| grep / awk | system | Used by L11 check arms for inline-changelog detection |

N/A — no Rust crate dependencies. No new npm/cargo dependencies introduced by this story.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.factory/specs/behavioral-contracts/BC-INDEX.md` | MODIFY | Normalize 9 `(vX.Y current)` rows to bare `active vX.Y` form |
| `.factory/specs/architecture/ARCH-INDEX.md` | MODIFY | Collapse inline changelog chains in ADR Status cells; bump `version:` |
| `scripts/records-lint.sh` | MODIFY | Add L11 check (two arms); add `--self-probe` cases; update L11 config-block comment |
| `.factory/policies.yaml` | MODIFY | Amend POL-40 scope; bump `version:` |
