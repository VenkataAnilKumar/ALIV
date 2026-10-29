# Implementation Phases
## From private matter vault to hybrid agent platform

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active

---

## Phase overview

| Phase | Release | Theme | Features | Status |
|---|---|---|---|---|
| 0 | R0 | Private matter vault | FS-1 to FS-7 | ✅ Complete (PRs #12–#17) |
| 1 | R1 | Hybrid foundation | FS-8 Gateway · FS-9 Redaction · FS-10 Router | Next |
| 2 | R2 | Harvey-class work product | FS-11 Review Tables · FS-12 Workflows · FS-13 Playbooks · FS-14 Drafting · FS-15 Compare (+ ING-06/07, ASK-06–10) | Planned |
| 3 | R3 | Agents | FS-16 Agent Builder · FS-17 Monitors · FS-18 Knowledge Bases · FS-19 Finance Pack | Planned |
| 4 | R4 | Scale & reach | FS-20 Firm Server · FS-21 Word Add-in · FS-22 Research Connectors | Planned |
| — | GA | Release readiness | Signed installers, update server, vault encryption, onboarding | Runs alongside Phases 2–3 |

**Why this order:**
- **Phase 1 first:** it makes "hybrid" real and safe. Without the gateway and redaction, every cloud-using feature in Phase 2 would widen the gap in [`security.md`](../guides/security.md).
- **Phase 2 next:** it delivers the features buyers compare against Harvey.
- **Phases 3–4:** they extend reach once the core is trusted.

---

## Phase 0: Private matter vault ✅

| Delivered | PR |
|---|---|
| Desktop app; matter vault (ingestion, register, Ask, approvals, audit) | #12 |
| OCR for scans; docs cleanup; CI | #13 |
| Hybrid keyword and meaning search | #14 |
| Summaries; context budget | #15 |
| Vault health; legacy app removed | #16 |
| User guide, security overview, developer guide | #17 |

---

## Phase 1: Hybrid foundation (R1)

**Goal:** every model call goes through one audited Rust gateway, with routing per step and redaction before any cloud call.

| # | Work | Feature IDs | Exit criteria |
|---|---|---|---|
| 1.1 | **Gateway core:** `ModelRequest`, provider adapters (llama.cpp, OpenAI-compatible, Anthropic), streaming over IPC, keys in the OS keychain | GW-01/03/04 | Ask runs end to end through the gateway; no provider call or key left in the web layer (CI check) |
| 1.2 | **Policy and audit in the gateway:** refusal, `model_calls` table, audit entry per call | GW-02/05, AUD-06 | Policy-matrix test passes for every step type |
| 1.3 | **Router:** step types, default routing, matter overrides, route shown in Ask | RTR-01/02/04 | Local only never routes to cloud; Hybrid routes per the table in the Hybrid AI Architecture |
| 1.4 | **Redaction:** entity store from the register, local NER, placeholders, restore, preview | RED-01/02/03/05 | ≥ 98% recall for parties and people on the labelled set; preview matches what is sent |
| 1.5 | **Spend limits, fallback, hardware-aware local model choice** | GW-06/07, RTR-05 | Limits enforced and audited; fallback behaviour tested |

**Phase exit:**
- `security.md` no longer lists the "policy check guards the Ask flow" limitation;
- redaction recall gate passes in CI.

---

## Phase 2: Harvey-class work product (R2)

**Goal:** the features buyers compare: review tables, workflows, playbooks, drafting, compare.

| # | Work | Feature IDs | Exit criteria |
|---|---|---|---|
| 2.1 | **Agent runtime v1:** definitions, runs, steps, citation verification, resume | (runtime) | Fake-model replay tests; resume after a forced crash |
| 2.2 | **Review Tables core:** create, question columns with types, cited cells, register columns, filters, XLSX export | TBL-01/02/03/04/08/09 | 200-document benchmark matter: ≥ 95% of cells correct and correctly cited (human-checked sample) |
| 2.3 | **Review Tables advanced:** conditional and manual columns, comments, inline edits, templates, 10k-row background runs | TBL-05/06/07/10/11 | 10,000-row run completes with resume; edits audited |
| 2.4 | **Workflows library:** summarise, chronology, compare, memo, NDA triage, DD issues list, lease abstract; DOCX/XLSX outputs; optional steps | WF-01–05, CMP-01 | Each workflow has a golden-output test on the demo corpus |
| 2.5 | **Playbooks and redlines:** positions, single and batch runs, DOCX tracked changes, starter playbooks | PB-01–05 | NDA playbook flags known deviations in a labelled NDA set (≥ 90% recall) |
| 2.6 | **Clause library and drafting** | DRF-01–04 | Drafts cite library clauses; DOCX output opens cleanly in Word |
| 2.7 | **Ask upgrades:** @-mentions, suggest tables and workflows, save as DOCX/PDF, follow-ups, uncited-claim flags | ASK-06–10 | Usability test with 5 pilot users |
| 2.8 | **Ingestion at scale:** .pst/.mbox archives; 10k-document matters with pause and resume | ING-06/07 | Enron-style public mailbox sample ingests with thread metadata |

**Phase exit:** a pilot firm runs a due-diligence matter end to end (ingest → review table → issues list → memo) without leaving ALIV.

---

## Phase 3: Agents (R3)

| # | Work | Feature IDs | Exit criteria |
|---|---|---|---|
| 3.1 | **Agent Builder UI:** steps, tools, routing, test-runs, versions | AGT-01–06 | A non-technical user builds a 4-step agent in under 15 minutes (usability test) |
| 3.2 | **Monitors:** schedules, watch folders, date triggers, deadline digest, new-document triage, inbox | MON-01–04, ING-08 | Monitors fire reliably across app restarts |
| 3.3 | **Knowledge bases** | KB-01–03 | KB content is cited and never mixed with matter data |
| 3.4 | **Finance pack:** invoice register, invoice-to-contract check, covenant tracker, KYC template | FIN-01–05 | Labelled invoice set: ≥ 95% field accuracy; known mismatches flagged |

---

## Phase 4: Scale & reach (R4)

| # | Work | Feature IDs | Exit criteria |
|---|---|---|---|
| 4.1 | **Firm Server:** shared matters, users, roles, ethical walls, central audit, backups | SRV-01/02/04/06, MAT-06 | 20 concurrent users on one server; access lists enforced in tests |
| 4.2 | **SSO and server-side GPU models** | SRV-03/05 | SAML/OIDC with two identity providers |
| 4.3 | **Word add-in** | WORD-01–03 | Playbook redlines inserted as tracked changes in Word on Windows and Mac |
| 4.4 | **Research connectors:** connector API, public sources first | RES-01/02/04 | Cited results with source links; no matter text sent without confirmation |

---

## GA track (alongside Phases 2–3)

| Work | Why |
|---|---|
| Signed and notarised installers (macOS, Windows), update server | Required to ship to customers |
| Vault encryption at rest (key in the OS keychain) | Closes the second limitation in `security.md` |
| First-run onboarding: hardware check, model download, sample matter | Activation KPI |
| Crash reporting (opt-in, no document data) | Support |
| Real-document accuracy benchmark (anonymised, with pilot firms) | Credible accuracy claims |

---

## Architecture decisions by phase

| Phase | Decision | Reason |
|---|---|---|
| 1 | All model calls in the Rust gateway | Makes the policy unbypassable and the audit complete |
| 1 | Redaction with stable placeholders, restored locally | Cloud reasoning without identities |
| 1 | Embeddings always local | Meaning search for Local only matters with no exceptions |
| 2 | One agent runtime for tables, workflows and agents | One place for routing, citations, audit and resume |
| 2 | Agents propose, people approve | Professional responsibility; matches the existing file-action model |
| 3 | Monitors run in-app on desktop, server-side on Firm Server | No background daemon needed for solo users |
| 4 | Firm Server reuses the vault crate | One data model across desktop and server |
| 4 | Licensed research through partners | Don't compete on content; keep the firm's documents the focus |

---

## Delivery practice

- **One PR per numbered item:**
  - tests in the same PR;
  - CI green (web, Rust desktop, CLI, vault);
  - docs updated (user guide, security, developer guide as relevant).
- **Every phase updates** the [Feature Spec](../04-specs/ALIV_Feature-Spec.md) status column and this document.
- **Accuracy claims** need a benchmark in `benchmarks/` and its numbers in the PR.
