---
document_type: session-handoff
level: ops
version: "9.024"
status: current
timestamp: 2026-09-16T00:00:00Z
---

# Session Handoff — Prism VSDD Pipeline

> **D-2526 (2026-09-16): BETA.3 BATCH-0 RE-GATE PASS-2 FIX-BURST COMPLETE (TD-VSDD-053) — pass-2 findings closed. error-taxonomy v2.86 (E-QUERY-045 full POL-39 closure); BC-2.16.003 v1.30 (status_code string_t rationale); VP-162 v1.2 (DI-019 = Query Security Limits + ADR-066 §D3 anchor); ADR-066 v1.3 (§A..§H contiguous); ADR-058 v2.39 (§K5 exemption note); ADR-060 v1.22 (RG-QTT-004 MCP-field precision). BC-INDEX v10.11; ARCH-INDEX v2.388; VP-INDEX v2.25. Orchestrator grep-verified 4 fixes. Both reviewers CLEAN (corroborated). develop_head 561d8baccc UNCHANGED. records-lint L1/L7/L9/L10 PASS. STATE v10.031→v10.032; SESSION-HANDOFF v9.023→v9.024. §RESUME SNAPSHOT D-2525 SUPERSEDED by D-2526.**

_D-2321..D-2491 delta entries and superseded snapshots archived to cycles/wave-5-e-demo-fidelity/session-handoff-archive.md (D-2494 compaction 2026-09-08). D-2505 snapshot superseded by D-2509. D-2509 snapshot superseded by D-2510. D-2510 snapshot superseded by D-2511. D-2511 snapshot superseded by D-2512. D-2512 snapshot superseded by D-2513. D-2513 snapshot superseded by D-2514. D-2514 snapshot superseded by D-2515. D-2515 snapshot superseded by D-2516. D-2516 snapshot superseded by D-2517. D-2517 snapshot superseded by D-2518. D-2518 snapshot superseded by D-2519. D-2519 snapshot superseded by D-2520. D-2520 snapshot superseded by D-2521. D-2521 snapshot superseded by D-2522. D-2522 snapshot superseded by D-2523. D-2523 snapshot superseded by D-2524. D-2524 snapshot superseded by D-2525. D-2525 snapshot superseded by D-2526._

---

## §RESUME SNAPSHOT — D-2526 (2026-09-16 — BATCH-0 RE-GATE PASS-2 FIX-BURST COMPLETE; STATE v10.032) [supersedes D-2525]

### RESUME IN ONE BREATH
Prism at develop@561d8bac — BETA.3 BATCH-0 RE-GATE PASS-2 FIX-BURST COMPLETE (D-2526). Pass-2 re-gate findings closed (F-1 HIGH + F-2 MED + F-3 MED + OBS-1/2/3 + consistency-LOW). error-taxonomy v2.86 (E-QUERY-045 full POL-39 closure); BC-2.16.003 v1.30 (status_code string_t rationale); VP-162 v1.2 (DI-019 = Query Security Limits + ADR-066 §D3 anchor); ADR-066 v1.3 (§A..§H contiguous); ADR-058 v2.39 (§K5 exemption note); ADR-060 v1.22 (RG-QTT-004 MCP-field precision). BC-INDEX v10.11 / VP-INDEX v2.25 / ARCH-INDEX v2.388. CARRY-FORWARD: F11 OBS (S-CLAROTY-OCSF-STATUS-001 DTU is_online false/null fixture obligation at materialization); F12 PROCESS-GAP OPEN (S-MAINT-RG-ANCHOR-DRIFT-GATE-001; in Blocking Issues). story_index v3.037 / total_stories 344. FIRST NEXT ACTION: re-gate on NEW HEAD (fresh adversary + consistency; BC-5.39.001 3-CLEAN streak restarts at D-2526 commit — need 3 consecutive CLEAN(strict)); then F3 story materialization. HUMAN ACTION REQUIRED BEFORE W4: RELEASE_PROMOTE_TOKEN PAT re-scope for BOHICA-LABS.

**RESUME STEP 0:** `CronList` → re-arm heartbeat cron b98bd9dc (8,23,38,53 * * * *) if absent/expired (.factory/ops/vsdd-heartbeat-autorecovery.md).

**RESUME NEXT-ACTION:** re-gate corrected Batch-0 spec delta on D-2526 HEAD (fresh adversary + consistency; BC-5.39.001 3-CLEAN streak restarts at D-2526 commit — need 3 consecutive CLEAN(strict)). Canonical RG sets FROZEN: RG-JEX-001..011, RG-QTT-001..011, RG-COS-001..008. Then F3 story materialization (10 beta.3 stories + remove-uncertainty; attach F11/F12/Finding-5 obligations at materialization). NOTE: F1 delta-analysis in .factory/cycles/wave-5-e-demo-fidelity/beta3-remediation-delta-analysis.md. Post-beta.3 fast-follow stubs (D-2521) in STORY-INDEX v3.037; reconcile depends_on S-JSON-EXTRACT-UDF-001 + SS-01→SS-11 relabeling at F3 materialization.

**HEADS (backup boundary):**
- develop HEAD `561d8baccc` (UNCHANGED; RECONCILIATION-PENDING: +2 out-of-session; #284 @daac70dc7; #287 @561d8bacc; tag v1.0.0-nightly.20260910). `main`: `bdf24cec8` (stub).
- `factory-artifacts`: run `git -C .factory log -1 --format='%h'` for current HEAD (TD-VSDD-053).
- Open PRs: #292 (ci clippy all-targets), #291 (dev-setup protoc), #288 (dtu embed fixtures), #282 (dependabot taiki-e); PR #255 (OBSOLETE — verify/close); Dependabot: #266–#274 (UNTRIAGED).
- WORKTREES: REMOVABLE-POST-MERGE: E-REL-NOTES (#264), S-CLAROTY-VULNS-001 (#245), S-ENGINE-LIMIT-EARLY-STOP-001 (#243), S-REL-NIGHTLY-001 (#276), S-REL-NIGHTLY-NOTES-001. PARKED: S-3.09, W3-FIX-S307-001 (DIRTY do-NOT-touch), S-ENGINE-H2-LARGE-RESPONSE-001.
- Releases: v1.0.0-beta.1 + v1.0.0-beta.2 + v1.0.0-nightly.* (multiple) + edge-nightly. NEXT: BOHICA-LABS beta.3 (all 20-issue fixes bundled).
- DEMO ASSETS (external — /Users/jmagady/Dev/test-soc/live-soc/ only; NOT repo state): (a) bin/prism beta.2 binary; (b) .prism-live/specs/claroty.sensor.toml; (c) monroe-demo-script.md v1.1.

**BETA.3 REMEDIATION — SPEC-GATE LOCKED (D-2522); BATCH-0 COMPLETE (D-2523+D-2524):**
- **3 gate decisions locked (D-2522):** (1) is_online FULL FIX IN BETA.3 — OPTION A: device_is_online VENDOR-EXT Boolean via ocsf_field "device.is_online" (pure-TOML; no code); S-CLAROTY-OCSF-STATUS-001 SIMPLIFIED. (2) ADR-060 §D8 + ADR-058 §K5 amendments AUTHORIZED (AMENDED+FROZEN after 3-CLEAN). (3) S-CLAROTY-OCSF-REMEDIATION-001 SPLIT → STATUS-001 + TOML-001.
- **Batch-0 canonical spec versions (D-2523+D-2524+D-2525+D-2526):** ADR-066 v1.3; VP-162 v1.2 (draft); BC-2.11.025 v1.2; ADR-060 v1.22; ADR-058 v2.39; BC-2.11.001 v1.34; BC-2.16.003 v1.30; error-taxonomy v2.86.
- **W1/strict:** S-MCP-TOOL-GATE-001 (issues 1,2 / 3pts)
- **W2/strict:** S-MCP-ENVELOPE-DESCRIBE-001 (issues 3,5 / 3pts); S-MCP-NULL-ENCODING-001 (issue 7 / 3pts)
- **W2-arch/strict:** S-DESCRIBE-EXAMPLE-DEDUP-001 (issue 4 / 2pts); S-QUERY-TRUE-TOTAL-001 (issue 6 / 8pts; BC-2.11.001 v1.34+ADR-060 v1.21 §D8.11.6; RG-QTT-001..011)
- **W3/strict:** S-CLAROTY-OCSF-STATUS-001 (issues 9,10; ADR-058 v2.38 §K5; RG-COS-001..008; OBLIGATION: seed is_online false+absent/null DTU fixtures at materialization); S-CLAROTY-OCSF-TOML-001 (issues 8,11,12,13; EC-016-013-004; live re-val 11/13); S-JSON-EXTRACT-UDF-001 (issue 20 / 5pts; BC-2.11.025 v1.2+ADR-066 v1.2+VP-162 v1.1; RG-JEX-001..011)
- **W4/facade:** S-BETA3-RELEASE-001 (issues 14,15 / 2pts; HUMAN ACTION: RELEASE_PROMOTE_TOKEN PAT before W4)
- **W5/facade:** S-ONBOARDING-DOCS-001 (issues 16-19 / 2pts)
- **POST-BETA.3 FAST-FOLLOWS (D-2521):** S-JSON-EXTRACT-TYPED-001 (P2/5pts; SS-01→SS-11 at materialization), S-JSON-EXTRACT-NESTED-001 (P2/8pts; SS-01→SS-11 at materialization), S-SPEC-OVERLAY-RELOCATION-001 (P3/2pts)

**PENDING / OPEN ITEMS:**
- **(a) FIRST NEXT ACTION:** re-gate on D-2526 HEAD (fresh adversary + consistency; BC-5.39.001 3-CLEAN; streak restarted at D-2526 HEAD — need 3 consecutive CLEAN(strict)). Then F3 story materialization (10 stories + remove-uncertainty; attach F11/F12/Finding-5 obligations).
- **(b) RECONCILIATION:** develop +2 out-of-session (#284 @daac70dc7; #287 @561d8bacc; tag v1.0.0-nightly.20260910).
- **(c) Open PRs + Dependabot:** Triage #292, #291, #288, #282; Dependabot #266–#274; verify/close PR #255.
- **(d) PROCESS-GAP D-2503 (JUSTIFIED DEFERRAL D-2515):** combined candidates (a) adjust branch-protection; (b) pr-manager-completion-guard honor orchestrator-scoped partial dispatch — target cycle-close/human.
- **(e) PROCESS-GAP D-2524/F12 OPEN:** S-MAINT-RG-ANCHOR-DRIFT-GATE-001 draft stub at story materialization; in Blocking Issues.
- **(f) RELEASE_PROMOTE_TOKEN:** PAT re-scope for BOHICA-LABS owner REQUIRED before W4 beta.3 release-tag dispatch.
- **(g) beta.2 redacted log external only:** test_prism-20260915.redacted.log stays external per D-2410.
- **(h) fast-follow depends_on S-JSON-EXTRACT-UDF-001:** reconcile at F3 materialization.
- **(i) F11 OBS obligation:** S-CLAROTY-OCSF-STATUS-001 must seed is_online false+absent/null DTU fixtures at materialization.
- **(j) Finding-5 OBS:** fast-follow stubs S-JSON-EXTRACT-TYPED-001/NESTED-001 mislabeled SS-01; correct to SS-11 at materialization.

**HEARTBEAT:** durable cron b98bd9dc (8,23,38,53 * * * *) in .claude/scheduled_tasks.json. CLAUDE.md §Orchestrator Auto-Recovery Heartbeat is authoritative standing rule.

**DECISION-LOG DELTA THIS BURST:** D-2526 (BETA.3 BATCH-0 RE-GATE PASS-2 FIX-BURST COMPLETE — F-1 HIGH/F-2 MED/F-3 MED/OBS-1/2/3/consistency-LOW closed; error-taxonomy v2.86/BC-2.16.003 v1.30/VP-162 v1.2/ADR-066 v1.3/ADR-058 v2.39/ADR-060 v1.22; BC-INDEX v10.11/ARCH-INDEX v2.388/VP-INDEX v2.25; both reviewers CLEAN corroborated; orchestrator grep-verified 4 fixes; develop_head 561d8baccc UNCHANGED; STATE v10.031→v10.032; SESSION-HANDOFF v9.023→v9.024). All recorded.

**STANDING DECISIONS (carry forward):** (a) No pragmatic convergence / fix all issues (production-grade default). (b) D-989 + D-2445 autonomy grant in force (merge+tag on green gates; force-push any branch still needs human). (c) D-2410 DO NOT SAVE LIVE-TEST OUTPUT INTO REPO. (d) Live xDome validation: canonical runbook .factory/ops/live-tenant-validation-runbook.md. (e) AUTONOMOUS MERGE + TAG (D-2445): all PRs autonomous on green gates; force-push STILL requires explicit human approval. (f) DEFECT-1 RESOLVED by PR #237. (g) POST-v1: TD-SENSOR-SORTBY-PUSHDOWN-001, TD-DI019-RECORDS-CAP-001, TD-CONFIG-SURFACE-EPIC-001. (h) Sensor scope v1: Claroty xDome ONLY (D-2440/D-2443). (i) RELEASING.md at repo root; release-config quality_gates vsdd-partial. (j) No registry publish in v1. (k) Demo bundle + DTU parity DEFERRED post-beta.1. (l) BETA channel; v1.0.0-rc.1 = immutable never-published ghost (D-2452). (m) ADR-063 v1.14 §D7 SHIPPED PR #281; ADR-064; PRISM_VERSION COMPLETE. (n) DEMO-SCOPE.md v2.1; live-monroe-capstone-runbook.md v1.0. (o) S-REL-DOCS-CI-WIRE-001 v0.1. (p) S-REL-HOLDOUT-HARNESS-001 v0.1. (q) Nightly lane LIVE; S-REL-NIGHTLY-NOTES-001 COMPLETE. (r) S-REL-SPECS-TARBALL-001 COMPLETE. (s) PROCESS-GAP D-2503 JUSTIFIED DEFERRAL. (t) S-REL-CHANGELOG-CHANNEL-SCOPE-001 MERGED PR #281. (u) v1.0.0-beta.2 PRE-RELEASE PUBLISHED. (v) ORG RENAME COMPLETE: BOHICA-LABS/prism D-2516. (w) RECONCILIATION-PENDING: develop +2 out-of-session. (x) vsdd-factory rc.25 D-2518. (y) BETA.1 BOOT-LOG NON-BUG D-2519. (z) LIVE-TEST TRIAGE COMPLETE D-2520. (aa) FAST-FOLLOW STUBS RECORDED D-2521. (ab) BETA.3 SPEC-GATE APPROVED D-2522. (ac) BATCH-0 F2 SPEC PRE-WORK COMPLETE D-2523: NEW ADR-066 v1.0/VP-162 v1.0/BC-2.11.025 v1.0 draft; is_online OPTION A ratified; POL-9 DISCHARGED. (ad) BATCH-0 SPEC-GATE FIX-BURST COMPLETE D-2524: ALL findings closed; ADR-060 v1.20/ADR-066 v1.1/ADR-058 v2.38/VP-162 v1.1/BC-2.11.001 v1.33/BC-2.11.025 v1.1/BC-2.16.003 v1.29/error-taxonomy v2.84; RG-JEX-001..011+RG-QTT-001..009+RG-COS-001..008 FROZEN; BC-INDEX v10.08/VP-INDEX v2.24/ARCH-INDEX v2.386. (ae) BATCH-0 RE-GATE FIX-BURST COMPLETE D-2525: pass-1 findings closed; ADR-060 v1.21/ADR-066 v1.2/BC-2.11.001 v1.34/BC-2.11.025 v1.2/error-taxonomy v2.85; BC-INDEX v10.09/ARCH-INDEX v2.387; RG-QTT expanded to 001..011; orchestrator grep-verified CLEAN; 3-CLEAN streak restarts on D-2525 HEAD. (af) BATCH-0 RE-GATE PASS-2 FIX-BURST COMPLETE D-2526: F-1 HIGH/F-2 MED/F-3 MED/OBS-1/2/3/consistency-LOW closed; error-taxonomy v2.86/BC-2.16.003 v1.30/VP-162 v1.2/ADR-066 v1.3/ADR-058 v2.39/ADR-060 v1.22; BC-INDEX v10.11/ARCH-INDEX v2.388/VP-INDEX v2.25; both reviewers CLEAN corroborated; orchestrator grep-verified 4 fixes; 3-CLEAN streak restarts on D-2526 HEAD.

---

## Standing Rules (Permanent — Do Not Archive)

### Production-Grade Default
Default behavior is enterprise/production-grade correctness. Speed lives in feature ordering, not feature completeness. See CLAUDE.md §CANONICAL PRINCIPLE for full rule set.

### Pipeline Authority
Orchestrator coordinates all phases; specialist agents do the writing. Orchestrator does NOT write files itself. See CLAUDE.md §Pipeline Authority + Agent Routing Table.

### BC-5.39.001 3-CLEAN Convergence
Adversarial cascades require three consecutive CLEAN(strict) passes for convergence. Any finding resets streak to 0/3. Frozen-HEAD streak rule (DRIFT-ORCH-PRLEVEL-PUSH-001): 3-CLEAN only counts consecutive passes on UNCHANGED HEAD. CLEAN(strict) = ZERO findings of ANY severity. CLEAN(PR-merge) = zero CRIT+HIGH+MED (LOW/OBS non-blocking). Streak requires CLEAN(strict).

### TD-VSDD-053 Single-Commit-Per-Burst
Each logical burst → ONE commit in .factory/. MULTI_COMMIT_CHAIN_NOT_ALLOWED detector blocks consecutive commits containing "backfill"/"Stage 1"/"Stage 2". STATE.md no longer cites the current HEAD SHA — run `git -C .factory log -1 --format='%h %s'` for live HEAD.

### TD-VSDD-091/POL-39 Anti-Volatile-Pin
Narrative spec content must cite function names + behavioral anchors, NOT filename.ext:NNN line numbers. All record-tier text (adversary pass reports, changelog rows, STATE.md entries) MUST use section/symbol/anchor cites ONLY.

### TD-VSDD-097 Three-Dimension Sweep
Every fix-burst MUST explicitly discharge all THREE dimensions: (1) Sibling pair sweep; (2) Downstream copy target sweep; (3) Mandate anchor (every MUST names story + AC + Red Gate test, or cites a real story ID deferral).

### Heartbeat SOP
First action every session: `CronList` → re-arm if absent/expired. Durable recurring cron b98bd9dc (8,23,38,53 * * * *). CLAUDE.md §Orchestrator Auto-Recovery Heartbeat is the authoritative standing rule.
