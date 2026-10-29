# Implementation Checklist
## Per-phase done list

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active. Tick items in the PR that completes them.

---

## Definition of done (every PR)

- [ ] Tests added or updated (Vitest / `cargo test`); CI green (lint, types, web tests, Rust jobs)
- [ ] New IPC commands are in the plugin permissions, the capabilities and `tests/ipc.rs`
- [ ] State changes and model calls write audit entries; no document text in audit or logs
- [ ] UI follows the [Design System](../05-design/ALIV_Design-System.md) (tokens, pills, empty and error states, keyboard, light and dark)
- [ ] User-facing docs updated (`user-guide.md`, `security.md` if behaviour changes)
- [ ] Feature IDs referenced in the PR description

## Phase 0: Private matter vault (R0) ✅

- [x] Matters with Local only / Hybrid policy (MAT-01–05)
- [x] Ingestion of PDF, DOCX, XLSX, PPTX, HTML, text, CSV, RTF and EML; change detection by hash; unreadable files listed with a reason (ING-01/02/05)
- [x] OCR for scans and images, with confidence carried into field confidence (ING-03/04)
- [x] Passages + FTS5 keyword search (SRCH-01)
- [x] Local embeddings, RRF hybrid search, background embedding, keyword fallback (SRCH-02–05)
- [x] Register extraction with confidence and evidence; deadline filters (REG-01–03)
- [x] Formula-safe CSV export (REG-04)
- [x] Extractive summaries (REG-05); context budget with "left out" count (ASK-03)
- [x] Cited answers, clickable sources, register answers without a model, route shown (ASK-01/02/04/05); policy check via `authorize_model_call` (MAT-02)
- [x] Rename proposals, approve, execute, undo (ACT-01–03)
- [x] Hash-chained audit with Verify integrity (AUD-01/02)
- [x] Vault health diagnostics, no text (AUD-03)
- [x] Legacy app retired; docs set; CI with OCR tools

## Phase 1: Hybrid foundation (R1)

- [ ] Root Cargo workspace; vault moved to `crates/aliv-vault`; generated TS types
- [ ] Eval harness with retrieval and redaction sets; CI gate (EVAL-01/02/04)
- [ ] OpenTelemetry GenAI spans from the gateway (pinned version); prompt-injection checks; ZDR-only and region-pinning options
- [ ] Gateway crate (`crates/aliv-gateway`); llama.cpp, OpenAI-compatible and Anthropic adapters; streaming (GW-01/03/04)
- [ ] Provider keys used only by the gateway (they are already stored in the OS keychain by `provider_secrets.rs`)
- [ ] CI check: no provider URLs or keys in `web-app`
- [ ] Policy check and audit in the gateway; `model_calls` table (GW-02/05, AUD-06)
- [ ] Step types, default routes, matter overrides (RTR-01/02/04)
- [ ] Matter settings UI; RoutePill
- [ ] Entity store, detectors, placeholders, restore (RED-01–03)
- [ ] Redaction preview (RED-05)
- [ ] Redaction recall ≥ 98% gate in CI (RED-06)
- [ ] Spend limits, fallback, hardware-aware model choice (GW-06/07, RTR-05)
- [ ] Per-tool Allow / Ask / Block policy (MCP-08)
- [ ] Firm-level routing defaults (RTR-03); redaction levels (RED-04); Skills tab in Hub (SKL-10)
- [ ] Matter chats, instructions, linked folders; convert chat projects (WS-01/02/03/05)
- [ ] Web search through the gateway: blocked on Local only, redacted and audited on Hybrid (SRCH-10)
- [ ] MCP trust levels, per-matter allow-lists, redaction and audit, approvals for side-effecting tools (MCP-01–04/07)
- [ ] Skills per matter: trust levels, review dialog, sandboxed scripts on approval, skills in audit (SKL-01–05, WS-06)
- [ ] Desktop Admin › AI policy and Skills & MCP pages
- [ ] `security.md`: remove the "policy check guards the Ask flow" limit and the "web search, MCP and chat projects outside matter policy" limit
- [ ] **Exit:** Local only never routes to cloud, and no chat, web search, MCP call or skill script bypasses the gateway (policy matrix test); preview equals what is sent

## Phase 2: Work product (R2)

- [ ] Agent runtime: definitions, runs, steps, citation verifier, resume
- [ ] Review tables: question columns, cited cells, register columns, filters, XLSX (TBL-01–04/08/09)
- [ ] Conditional and manual columns, comments, edits, templates, 10k rows (TBL-05–07/10/11)
- [ ] ≥ 95% cell accuracy on the 200-document benchmark
- [ ] Workflows library with golden-output tests; optional steps (WF-01–05)
- [ ] Compare versions (CMP-01)
- [ ] Playbooks, batch runs, tracked-changes DOCX, starter playbooks (PB-01–05)
- [ ] NDA playbook ≥ 90% deviation recall
- [ ] Clause library and drafting (DRF-01–04)
- [ ] Ask upgrades (ASK-06–10)
- [ ] PST/MBOX; 10k-document ingestion with resume (ING-06/07)
- [ ] Field corrections kept across re-ingestion (REG-06); XLSX register export (REG-08)
- [ ] Audit trail export, PDF and JSON (AUD-05)
- [ ] Structured mid-run questions; interrupt, resume and replay (AGT-08/09)
- [ ] Agent and workflow steps audited (AUD-04); model second pass as table columns (REG-07); duplicate workflow as agent (WF-06); compare against template, DOCX (CMP-02/03)
- [ ] Folder moves as proposals (ACT-04); search filters (SRCH-06); matter export (WS-08)
- [ ] Artifacts: store, versions, diff, tab, sandboxed HTML, export, output folder (ART-01–07, WS-04)
- [ ] Global search across matters and types (SRCH-09/11/12)
- [ ] Layout-aware parsing and structure-aware chunks (ING-10/11)
- [ ] Local reranker; ANN index decision by benchmark (SRCH-13/14)
- [ ] Contract-finance basics: invoice fields, calculation tool, fee schedules as tables (FIN-01 fields, FIN-06/07)
- [ ] Schema-constrained extraction (REG-09); LLM-as-judge calibrated (EVAL-03/05)
- [ ] Starter legal and finance skill packs with golden-output tests (SKL-06)
- [ ] **Exit:** a pilot DD matter end to end inside ALIV

## Phase 3: Agents (R3)

- [ ] Agent Builder: steps, tools, routing, test runs, versions, sharing (AGT-01–06)
- [ ] Agents only propose; proposals appear in Approvals
- [ ] Monitors: scheduler, watch folders, digest, triage, inbox (MON-01–04, ING-08)
- [ ] Knowledge bases, read-only and cited (KB-01–03)
- [ ] Finance pack (FIN-01–05) with ≥ 95% invoice field accuracy
- [ ] Automation engine: triggers, actions, history, retries, templates; proposals only (AUTO-01–07)
- [ ] Document connectors: iManage, SharePoint, NetDocuments, Outlook, then Drive, Box, Gmail (CON-01–06)
- [ ] MCP tools in agents (MCP-05); firm skill library and Skill Builder (SKL-07/08)
- [ ] Goal-driven runs (AGT-10); tool search for large catalogs (MCP-09)
- [ ] Agent proposals in the approval queue (ACT-05); retention reminders (MAT-07); cross-matter search (SRCH-07); duplicate detection (ING-09); firm-run evals (EVAL-06)
- [ ] **Exit:** a 4-step agent built in under 15 minutes by a non-technical user

## Phase 4: Scale & reach (R4)

- [ ] Firm Server: shared matters, roles, ethical walls, central audit, backups (SRV-01/02/04/06, MAT-06)
- [ ] SSO with two identity providers; server GPU models (SRV-03/05)
- [ ] Word add-in on Windows and Mac (WORD-01–03)
- [ ] Research connectors (RES-01/02/04)
- [ ] ALIV as an MCP server; connector source permissions (MCP-06, CON-04)
- [ ] Shared and signed skills (SKL-09)
- [ ] Firm-wide view-only knowledge bases (KB-04)
- [ ] **Exit:** 20 concurrent users on one server; access lists enforced in tests

## GA track

- [ ] ALIV Cloud on Vercel (website, docs, update manifest, licences) and public demo
- [ ] Signed self-update with verification and rollback
- [ ] Signed and notarised installers (macOS, Windows)
- [ ] Update server and signed `latest.json`
- [ ] Vault encryption at rest (key in the OS keychain)
- [ ] First-run onboarding: hardware check, model download, sample matter
- [ ] Opt-in crash reporting without document data
- [ ] OCR tools bundled or guided install
- [ ] `cargo audit` / `yarn npm audit` in CI
- [ ] Real-document accuracy benchmark with pilot firms
