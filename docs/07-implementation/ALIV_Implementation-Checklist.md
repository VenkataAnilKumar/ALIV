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

- [x] Matters with Local only / Hybrid policy (MAT-01–03)
- [x] Ingestion of PDF, DOCX and text; change detection by hash (ING-01/02/04)
- [x] OCR for scans and images with confidence (ING-03/05)
- [x] Passages + FTS5 keyword search
- [x] Local embeddings and RRF hybrid search (SRCH)
- [x] Register extraction with confidence and evidence; deadlines (REG-01–03)
- [x] Formula-safe CSV export (REG-04)
- [x] Extractive summaries; context budget with "left out" count (ASK-04/05)
- [x] Cited answers with policy check (`authorize_model_call`) (ASK-01–03)
- [x] Rename proposals, approve, execute, undo (ACT)
- [x] Hash-chained audit with Verify integrity (AUD-01/02)
- [x] Vault health diagnostics (no text)
- [x] Legacy app retired; docs set; CI with OCR tools

## Phase 1: Hybrid foundation (R1)

- [ ] Gateway crate; llama.cpp, OpenAI-compatible and Anthropic adapters; streaming (GW-01/03/04)
- [ ] Provider keys in the OS keychain only
- [ ] CI check: no provider URLs or keys in `web-app`
- [ ] Policy check and audit in the gateway; `model_calls` table (GW-02/05, AUD-06)
- [ ] Step types, default routes, matter overrides (RTR-01/02/04)
- [ ] Matter settings UI; RoutePill
- [ ] Entity store, detectors, placeholders, restore (RED-01–03)
- [ ] Redaction preview (RED-05)
- [ ] Redaction recall ≥ 98% gate in CI
- [ ] Spend limits, fallback, hardware-aware model choice (GW-06/07, RTR-05)
- [ ] `security.md`: remove the "policy check guards the Ask flow" limit
- [ ] **Exit:** Local only never routes to cloud (policy matrix test); preview equals what is sent

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
- [ ] Field overrides kept across re-ingestion; XLSX register export
- [ ] Audit trail export, PDF and JSON (AUD-05)
- [ ] **Exit:** a pilot DD matter end to end inside ALIV

## Phase 3: Agents (R3)

- [ ] Agent Builder: steps, tools, routing, test runs, versions, sharing (AGT-01–06)
- [ ] Agents only propose; proposals appear in Approvals
- [ ] Monitors: scheduler, watch folders, digest, triage, inbox (MON-01–04, ING-08)
- [ ] Knowledge bases, read-only and cited (KB-01–03)
- [ ] Finance pack (FIN-01–05) with ≥ 95% invoice field accuracy
- [ ] **Exit:** a 4-step agent built in under 15 minutes by a non-technical user

## Phase 4: Scale & reach (R4)

- [ ] Firm Server: shared matters, roles, ethical walls, central audit, backups (SRV-01/02/04/06, MAT-06)
- [ ] SSO with two identity providers; server GPU models (SRV-03/05)
- [ ] Word add-in on Windows and Mac (WORD-01–03)
- [ ] Research connectors (RES-01/02/04)
- [ ] **Exit:** 20 concurrent users on one server; access lists enforced in tests

## GA track

- [ ] Signed and notarised installers (macOS, Windows)
- [ ] Update server and signed `latest.json`
- [ ] Vault encryption at rest (key in the OS keychain)
- [ ] First-run onboarding: hardware check, model download, sample matter
- [ ] Opt-in crash reporting without document data
- [ ] OCR tools bundled or guided install
- [ ] `cargo audit` / `yarn npm audit` in CI
- [ ] Real-document accuracy benchmark with pilot firms
