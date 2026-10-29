# Sprint Plan
## Phases broken into two-week sprints and PRs

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active (plan; dates are targets for a team of 2–3 engineers + 1 designer)

---

## Assumptions

- **Sprint length:** 2 weeks. Each sprint ends with a demo on the demo corpus and green CI.
- **Pull requests:** one PR per row, small enough to review in a day. Each has tests, docs and a changelog line.
- **Not counted:** the GA track runs in parallel with roughly 20% of capacity.
- **Phase 0 (R0):** complete (PRs #12–#17).

## Phase 1: Hybrid foundation (R1), sprints 1–4

| Sprint | PRs | Demo |
|---|---|---|
| **S1** | 1.1a Gateway crate skeleton: `ModelRequest`, `Route`, llama.cpp adapter, IPC streaming · 1.1b Ask uses the gateway for local models | Ask runs through the gateway locally |
| **S2** | 1.1c OpenAI-compatible and Anthropic adapters, keys in the OS keychain · 1.1d CI check: no provider URLs or keys in `web-app` · 1.2 `model_calls` table, policy check moved into the gateway, audit per call | Hybrid matter asks a cloud model via the gateway; Local only refused |
| **S3** | 1.3a Step types and default routes · 1.3b Matter settings UI (W-12), RoutePill · 1.4a Entity store from the register, regex detectors | Route shown before and after each call |
| **S4** | 1.4b Local NER, placeholders and restore · 1.4c RedactionPreview (W-8) · 1.4d Redaction recall benchmark in CI (≥ 98%) · 1.5 Spend limits, fallback, hardware-aware model choice · `security.md` update | Preview shows exactly what leaves; recall gate green |

## Phase 2: Harvey-class work product (R2), sprints 5–12

| Sprint | PRs | Demo |
|---|---|---|
| **S5** | 2.1a Agent runtime: definitions, runs, steps, fake-model replay harness · 2.1b Citation verifier · 2.1c Resume after crash | A two-step workflow runs and resumes |
| **S6** | 2.2a Review tables data model and commands · 2.2b ReviewGrid (virtualised), question columns with types · 2.2c SourceDrawer, CitationChip, ConfidenceMeter | Table with 3 question columns on 50 documents |
| **S7** | 2.2d Register columns, filters, needs-review · 2.2e XLSX writer and export · 200-document accuracy benchmark | DD table exported to XLSX |
| **S8** | 2.3a Conditional and manual columns · 2.3b Comments, inline edits with originals · 2.3c Templates · 2.3d 10k-row background runs | 10k-row run with pause and resume |
| **S9** | 2.4a Workflows library UI and RunLog · 2.4b Summarise, chronology, compare (CMP-01) · 2.4c DOCX writer | Chronology on the litigation demo set |
| **S10** | 2.4d Memo, NDA triage, DD issues list, lease abstract · optional steps · golden-output tests | DD issues list from a table |
| **S11** | 2.5 Playbooks: editor, single and batch runs, tracked-changes DOCX, starter playbooks · 2.6 Clause library and drafting | NDA playbook with redlines opened in Word |
| **S12** | 2.7 Ask upgrades (@-mentions, suggestions, save, follow-ups, uncited marks) · 2.8 PST/MBOX, 10k-document ingestion with resume · pilot hardening | Pilot DD matter end to end |

## Phase 3: Agents (R3), sprints 13–17

| Sprint | PRs | Demo |
|---|---|---|
| **S13** | 3.1a Agent Builder: steps list, StepEditor, tools · 3.1b Test runs on a sample matter | Build a 4-step agent |
| **S14** | 3.1c Versions, publish, share (Team) · proposals from agents into Approvals | Published agent with versioned runs |
| **S15** | 3.2 Scheduler, watch folders, deadline digest, new-document triage, Inbox (W-11) | Weekly digest survives restarts |
| **S16** | 3.3 Knowledge bases (read-only, @-mention, citations) | Answers cite firm precedents |
| **S17** | 3.4 Finance pack: invoice register, invoice-to-contract check, covenant tracker, KYC template | Exceptions table on the labelled invoice set |

## Phase 4: Scale & reach (R4), sprints 18–24

| Sprint | PRs | Demo |
|---|---|---|
| **S18–S19** | 4.1a `apps/server`: API, auth skeleton, vault crate on Postgres · 4.1b Desktop server-backed service | Two desktops share a matter |
| **S20** | 4.1c Users, roles, ethical walls, central audit, backups | Wall blocks a conflicted user |
| **S21** | 4.2 SSO (Entra ID, Okta), server GPU models via the gateway | SSO login; 70B model on the server |
| **S22–S23** | 4.3 Word add-in: playbook panel, tracked-change insertion (Windows and Mac) | Redlines inserted in Word |
| **S24** | 4.4 Research connector API, first public source | Cited research results |

## GA track (parallel, sprints 6–14)

Signed installers and notarisation (S6–S7) · updater and update server (S8) · vault encryption at rest (S9–S10) · first-run onboarding with hardware check and sample matter (S11–S12) · opt-in crash reporting (S13) · real-document accuracy benchmark with pilots (S12–S14).

## Milestones

| Milestone | After sprint | Meaning |
|---|---|---|
| **M1 Hybrid-safe** | S4 | Cloud models usable with redaction and full audit |
| **M2 Pilot** | S12 | Pilot firms run DD matters end to end |
| **M3 GA** | S14 | Signed installers, encryption at rest, onboarding |
| **M4 Agents** | S17 | Agent Builder, monitors, finance pack |
| **M5 Firm** | S24 | Firm Server, SSO, Word add-in |

Progress against this plan is tracked in the [Implementation Checklist](ALIV_Implementation-Checklist.md).
