---
document_type: session-handoff
level: ops
version: "9.019"
status: current
timestamp: 2026-09-15T00:00:00Z
---

# Session Handoff — Prism VSDD Pipeline

> **D-2521 (2026-09-15): FAST-FOLLOW STUBS RECORDED (human-directed, TD-VSDD-053) — 3 draft post-beta.3 stories added to STORY-INDEX v3.037 (total_stories 344): S-JSON-EXTRACT-TYPED-001, S-JSON-EXTRACT-NESTED-001, S-SPEC-OVERLAY-RELOCATION-001. No BC/ADR/VP changes; develop_head 561d8baccc UNCHANGED. records-lint L1/L7/L9/L10 PASS. STATE v10.026→v10.027; SESSION-HANDOFF v9.018→v9.019. §RESUME SNAPSHOT D-2520 SUPERSEDED by D-2521.**

_D-2321..D-2491 delta entries and superseded snapshots archived to cycles/wave-5-e-demo-fidelity/session-handoff-archive.md (D-2494 compaction 2026-09-08). D-2505 snapshot superseded by D-2509. D-2509 snapshot superseded by D-2510. D-2510 snapshot superseded by D-2511. D-2511 snapshot superseded by D-2512. D-2512 snapshot superseded by D-2513. D-2513 snapshot superseded by D-2514. D-2514 snapshot superseded by D-2515. D-2515 snapshot superseded by D-2516. D-2516 snapshot superseded by D-2517. D-2517 snapshot superseded by D-2518. D-2518 snapshot superseded by D-2519. D-2519 snapshot superseded by D-2520. D-2520 snapshot superseded by D-2521._

---

## §RESUME SNAPSHOT — D-2521 (2026-09-15 — FAST-FOLLOW STUBS RECORDED; STATE v10.027) [supersedes D-2520]

### RESUME IN ONE BREATH
Prism at develop@561d8bac — beta.2 live-test triage COMPLETE (D-2520). 3 post-beta.3 fast-follow stubs RECORDED per human direction (D-2521; don't lose: S-JSON-EXTRACT-TYPED-001, S-JSON-EXTRACT-NESTED-001, S-SPEC-OVERLAY-RELOCATION-001). story_index v3.037 / total_stories 344. FIRST NEXT ACTION: run F1 delta-analysis (architect) + story decomposition (story-writer) for beta.3 remediation cycle; present full story set at spec gate review.

**RESUME STEP 0:** `CronList` → re-arm heartbeat cron b98bd9dc (8,23,38,53 * * * *) if absent/expired (.factory/ops/vsdd-heartbeat-autorecovery.md).

**RESUME NEXT-ACTION:** Run F1 delta-analysis + story decomposition for the beta.3 remediation cycle; present full story set at spec gate review. NOTE: fast-follow stubs (D-2521) already recorded in STORY-INDEX v3.037; reconcile depends_on S-JSON-EXTRACT-UDF-001 at decomposition time.

**HEADS (backup boundary):**
- develop HEAD `561d8baccc` (UNCHANGED; RECONCILIATION-PENDING: +2 out-of-session; #284 ci nightly+fuzz-nightly @daac70dc7; #287 fix dtu-claroty @561d8bacc; tag v1.0.0-nightly.20260910). `main`: `bdf24cec8` (stub).
- `factory-artifacts`: run `git -C .factory log -1 --format='%h'` for current HEAD (TD-VSDD-053).
- Open PRs: #292 (ci clippy all-targets), #291 (dev-setup protoc), #288 (dtu embed armis/cyberint/nvd fixtures), #282 (dependabot taiki-e); PR #255 (OBSOLETE — verify/close); Dependabot: #266–#274 (UNTRIAGED).
- WORKTREES: REMOVABLE-POST-MERGE: E-REL-NOTES (#264), S-CLAROTY-VULNS-001 (#245), S-ENGINE-LIMIT-EARLY-STOP-001 (#243), S-REL-NIGHTLY-001 (#276), S-REL-NIGHTLY-NOTES-001. PARKED: S-3.09, W3-FIX-S307-001 (DIRTY do-NOT-touch), S-ENGINE-H2-LARGE-RESPONSE-001.
- Releases: v1.0.0-beta.1 + v1.0.0-beta.2 + v1.0.0-nightly.* (multiple) + edge-nightly. NEXT RELEASE: BOHICA-LABS beta.3 (all 20-issue fixes bundled).
- DEMO ASSETS (external — /Users/jmagady/Dev/test-soc/live-soc/ only; NOT repo state): (a) bin/prism beta.2 binary (SHA-256 5108c6a60fbf32dd249c1cf28fb74658275fb9873f6ab6e151482065b8b31a2f); (b) .prism-live/specs/claroty.sensor.toml; (c) monroe-demo-script.md v1.1.

**BETA.3 REMEDIATION — 20 ISSUES (D-2520 inventory):**
- W1: Tool-gating (issues 1,2) — gate 40 -32003 stubs (prism-mcp/src/server.rs not_yet_available_msg); keep 14 live tools; rmcp cfg compile-spike
- W2: Envelope (issues 3,5,7) — total_results:0 (safety_envelope.rs); virtual fields (inject_virtual_fields); null/list encoding (spec_driven_adapter.rs + column_mapping.rs)
- W2-arch: (issues 4,6) — example_query dedupe (build_example_with_note); total_available FULL FIX (BC-2.11.001 + ADR-060 + interface-definitions amend)
- W3: OCSF + JSON accessor (issues 8-13,20) — claroty.sensor.toml: finding_info_uid type, status_code type+field, time column, join keys, severity; json_extract_string UDF; new BC+ADR+VP
- W4: Release (issues 14,15) — new BOHICA-LABS beta.3; install.sh drbothen→BOHICA-LABS URL fix
- W5: Docs (issues 16,17,18,19) — gh attestation verify runbook; SETUP.md §9 my-client+base_url fix
- POST-BETA.3 FAST-FOLLOWS (D-2521; recorded in STORY-INDEX v3.037): S-JSON-EXTRACT-TYPED-001 (P2/5pts), S-JSON-EXTRACT-NESTED-001 (P2/8pts), S-SPEC-OVERLAY-RELOCATION-001 (P3/2pts)

**PENDING / OPEN ITEMS:**
- **(a) FIRST NEXT ACTION:** F1 delta-analysis (architect) + story decomposition (story-writer) → human spec gate review of full beta.3 story set + architect OCSF status(9/10) proposal + JSON-accessor design.
- **(b) RECONCILIATION:** develop +2 out-of-session (#284 ci nightly+fuzz-nightly @daac70dc7; #287 fix dtu-claroty @561d8bacc; tag v1.0.0-nightly.20260910).
- **(c) Open PRs + Dependabot:** Triage #292, #291, #288, #282; Dependabot #266–#274; verify/close PR #255.
- **(d) PROCESS-GAP D-2503 (JUSTIFIED DEFERRAL D-2515):** combined candidates (a) adjust branch-protection; (b) pr-manager-completion-guard honor orchestrator-scoped partial dispatch — target cycle-close/human.
- **(e) FLAG: RELEASE_PROMOTE_TOKEN** PAT may need re-scope for BOHICA-LABS owner before beta.3 release-tag dispatch.
- **(f) beta.2 redacted log external only:** test_prism-20260915.redacted.log stays external per D-2410 (DO NOT SAVE LIVE-TEST OUTPUT INTO REPO).
- **(g) fast-follow depends_on S-JSON-EXTRACT-UDF-001:** reconcile at F1/story-decomposition (either create S-JSON-EXTRACT-UDF-001 or re-point to the actual minimal-accessor story ID).

**HEARTBEAT:** durable cron b98bd9dc (8,23,38,53 * * * *) in .claude/scheduled_tasks.json. CLAUDE.md §Orchestrator Auto-Recovery Heartbeat is authoritative standing rule.

**DECISION-LOG DELTA THIS BURST:** D-2521 (FAST-FOLLOW STUBS RECORDED — human-directed; 3 draft post-beta.3 stories added to STORY-INDEX v3.037; total_stories 341→344; depends_on S-JSON-EXTRACT-UDF-001 flagged; develop_head 561d8baccc UNCHANGED; STATE v10.026→v10.027; SESSION-HANDOFF v9.018→v9.019). All recorded.

**STANDING DECISIONS (carry forward):** (a) No pragmatic convergence / fix all issues (production-grade default). (b) D-989 + D-2445 autonomy grant in force (merge+tag on green gates; force-push any branch still needs human). (c) D-2410 DO NOT SAVE LIVE-TEST OUTPUT INTO REPO. (d) Live xDome validation: canonical runbook .factory/ops/live-tenant-validation-runbook.md. (e) AUTONOMOUS MERGE + TAG (D-2445): all PRs autonomous on green gates; force-push STILL requires explicit human approval. (f) DEFECT-1 RESOLVED by PR #237. (g) POST-v1: TD-SENSOR-SORTBY-PUSHDOWN-001, TD-DI019-RECORDS-CAP-001, TD-CONFIG-SURFACE-EPIC-001. (h) Sensor scope v1: Claroty xDome ONLY (D-2440/D-2443). (i) RELEASING.md at repo root; release-config quality_gates vsdd-partial. (j) No registry publish in v1. (k) Demo bundle + DTU parity DEFERRED post-beta.1. (l) BETA channel; v1.0.0-rc.1 = immutable never-published ghost (D-2452). (m) ADR-063 v1.14 git-cliff hybrid model + §D7 per-channel scoping SHIPPED PR #281; cargo-release 1.1.5 (ADR-064 §D3); PRISM_VERSION COMPLETE; docs VERSION-AGNOSTIC. (n) DEMO-SCOPE.md v2.1; live-monroe-capstone-runbook.md v1.0. (o) S-REL-DOCS-CI-WIRE-001 v0.1. (p) S-REL-HOLDOUT-HARNESS-001 v0.1. (q) Nightly lane LIVE; S-REL-NIGHTLY-NOTES-001 COMPLETE. (r) S-REL-SPECS-TARBALL-001 COMPLETE: PR #279 @54523dccd; all 9 ACs. (s) PROCESS-GAP D-2503 (4th recurrence D-2515): JUSTIFIED DEFERRAL; cycle-close target. (t) S-REL-CHANGELOG-CHANNEL-SCOPE-001 MERGED develop@09e9b28d2 (PR #281 2026-09-10). (u) v1.0.0-beta.2 PRE-RELEASE PUBLISHED. (v) ORG RENAME COMPLETE: BOHICA-LABS/prism (D-2516; PR #283 develop@85f30ea7f); .factory swept exhaustively. (w) RECONCILIATION-PENDING: develop +2 out-of-session (#284, #287) to 561d8bac + tag v1.0.0-nightly.20260910. (x) vsdd-factory plugin rc.25 INSTALLED + RE-ACTIVATED (darwin-arm64; D-2518 records-only micro-burst). (y) BETA.1 BOOT-LOG CONFIRMED NON-BUG: stale artifact from rc.1 provisioning run (D-2519). (z) LIVE-TEST TRIAGE COMPLETE (D-2520): beta.2 Monroe demo 20-issue inventory; beta.3 remediation cycle scoped into 5 waves; no repo changes in triage burst. (aa) FAST-FOLLOW STUBS RECORDED (D-2521, human-directed): 3 draft post-beta.3 stories committed to STORY-INDEX v3.037 (total_stories 344); depends_on S-JSON-EXTRACT-UDF-001 flagged for reconciliation at F1/story-decomposition.

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
