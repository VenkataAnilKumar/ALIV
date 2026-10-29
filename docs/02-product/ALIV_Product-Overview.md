# Product Overview
## ALIV on one page

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active

---

**ALIV is a hybrid intelligent agent platform for legal and finance teams.** Every matter is a private vault on the firm's own machines. Agents, review tables and workflows run over it. A per-matter, per-step policy decides what runs on a local model and what may use a cloud model, after redaction. Every call is audited.

## The problem

Harvey, Legora and CoCounsel have shown what AI can do for legal work: bulk review, drafting, workflows, agents. They run in the vendor's cloud. Many firms, clients and regulated finance teams can't, or won't, send every document there. Local tools keep data private but lack the work product. ALIV closes that gap.

## Who it's for

| Persona | Job |
|---|---|
| M&A associate | Due diligence on hundreds of contracts |
| In-house counsel | Renewals, obligations, NDA turnaround |
| Litigation partner | Chronologies and fact-finding from productions |
| Finance controller | Invoices against contracts, covenants |
| Legal ops / IT | Adopt AI with control and evidence |

## What it does

| Shipped (R0) | Next (R1–R2) | Later (R3–R4) |
|---|---|---|
| Matters as private vaults with a Local only / Hybrid policy | **Model gateway**: every call through one audited Rust component | **Agent Builder**: multi-step firm agents |
| Ingestion: PDF, DOCX, scans (OCR), images, text | **Redaction** of parties, people and numbers before any cloud call | **Monitors**: deadline digests, new-document triage |
| Keyword + meaning search (local embeddings) | **Router**: right model per step | **Knowledge bases** of firm precedents |
| Cited answers (Ask) | **Review tables**: one question per column, cited cells, 10k rows | **Finance pack**: invoices, covenants |
| Contract register and deadlines | **Workflows**: chronology, DD issues list, memo, compare | **Firm Server**, SSO, ethical walls |
| Approvals for file changes, with undo | **Playbooks** with DOCX redlines | **Word add-in**, research connectors |
| Hash-chained audit log, vault health | Clause library and drafting | |

## Why ALIV

1. **Private by default:** documents are read in place, embeddings are local, and new matters are Local only.
2. **Hybrid when allowed:** use the strongest model for the step, with identities redacted.
3. **Cited and in control:** every output traces to a passage, and agents propose rather than act.
4. **Provable:** a tamper-evident audit log and an exportable trail for clients and regulators.
5. **Runs on your hardware:** desktop today, Firm Server and private cloud next.

## How it works

```
Documents (stay in place) → Vault: passages · fields · embeddings · audit (SQLite, on the machine)
                                 ↓
     Ask · Review tables · Workflows · Agents (agent runtime)
                                 ↓
     Model gateway → policy → router → (redact) → local model  |  cloud model (Hybrid only)
                                 ↓
     Cited outputs · proposals for approval · audit entries
```

## Business model

| Tier | For | Price (planned) |
|---|---|---|
| Solo | Individual lawyers, accountants | $39–59 / user / month, or perpetual licence + yearly updates |
| Team | Firms of 2–50 | $79–129 / user / month |
| Firm Server | Firms needing shared vaults and admin | Server licence + per-user fee |

## Learn more

[Product Definition](ALIV_Product-Definition.md) · [Competitive Analysis](../03-market/ALIV_Competitive-Analysis.md) · [Feature Spec](../04-specs/ALIV_Feature-Spec.md) · [System Architecture](../06-technical/ALIV_System-Architecture.md) · [Implementation Phases](../07-implementation/ALIV_Implementation-Phases.md)
