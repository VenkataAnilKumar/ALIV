# Product Definition Document
## ALIV: the hybrid intelligent agent platform for legal and finance

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active

---

## Table of Contents

1. Executive Summary
2. Problem
3. Target Audience & Personas
4. Product Principles
5. Core Capabilities & Value Proposition
6. Features by Release
7. Market Positioning
8. Business Model
9. Technical & Operational Requirements
10. Success Metrics (KPIs)
11. Risks, Dependencies & Mitigations
12. Research Basis
13. Appendix: Glossary

---

## 1. Executive Summary

**ALIV** gives legal and finance teams the capabilities of enterprise legal AI: cited answers over a document set, review tables, one-click workflows, playbook redlines and custom agents. It does this without requiring them to hand their clients' documents to a vendor's cloud.

Every **matter** is a private vault on the firm's own machines. A **hybrid model router** decides, per matter and per step, whether a model runs locally (llama.cpp, MLX) or in the cloud (OpenAI, Anthropic and others). When the cloud is allowed, only the minimum text is sent, after **redaction**, through one **audited gateway**. Every answer is **cited** to the passage it came from.

| | |
|---|---|
| **Category** | Legal & finance AI: agent platform with document vault |
| **Deployment** | Desktop (macOS, Windows, Linux) → Firm Server (planned) → private cloud (planned) |
| **Primary buyers** | Small and mid-size law firms, in-house legal, accounting and tax firms, regulated finance teams |
| **Core promise** | *Harvey-class work product. Your client data stays yours. Every step on the record.* |
| **Today** | Matters, ingestion with OCR, hybrid search, extraction register, cited Ask, approvals with undo, audit log, vault health |
| **Next** | Model gateway, redaction and router → Review Tables, Workflows, Playbooks → Agent Builder → Firm Server |

---

## 2. Problem

1. **Confidentiality blocks adoption.**
   - About 55% of lawyers name confidentiality and security as their top cloud concern ([ABA 2024 Tech Survey](https://www.americanbar.org/groups/law_practice/resources/tech-report/2024/2024-cloud-computing-techreport/)).
   - ABA Formal Opinion 512 makes assessing that risk a professional duty before using generative AI on client data.
   - Many engagement letters and outside-counsel guidelines forbid sending client documents to third-party AI.
2. **Enterprise legal AI is priced and built for large firms.**
   - Harvey, Legora and Luminance sell enterprise contracts, reported in the hundreds of thousands of dollars a year for Harvey (see [Competitive Analysis](../03-market/ALIV_Competitive-Analysis.md)).
   - Small firms and in-house teams are priced out, or can't accept the cloud terms.
3. **Local "chat with your PDFs" tools are generic.**
   - AnythingLLM, GPT4All and Elephas answer questions over files.
   - None of them offer legal workflows, extraction registers, playbooks, approvals or an audit trail.
4. **Accuracy is the first concern.**
   - Lawyers rank accuracy above privacy (75% vs 47% in ABA survey summaries). An answer that can't be traced to a source is not usable.
   - Small local models alone struggle with long, complex documents.

**The gap:** no product combines enterprise-grade legal workflows, local-first data control, a policy-governed path to strong cloud models when allowed, and a complete audit trail.

---

## 3. Target Audience & Personas

### Beachhead
- **Small and mid-size law firms**, 2–50 lawyers: corporate, commercial, real estate, employment.
- **In-house legal teams** at regulated companies: financial services, healthcare, government contractors.
- **Accounting and tax firms**, and **finance teams** that review contracts, invoices and filings.

### Personas

| Persona | Role | Goals | Pains today | What ALIV does for them |
|---|---|---|---|---|
| **Priya, M&A associate** | 4th-year associate, 20-lawyer firm | Get through a 300-document data room in days, not weeks | Manual review, spreadsheets, no time to read everything | Review Tables across the data room, cited; *Due diligence* workflow; issues list export |
| **Daniel, in-house counsel** | Sole counsel at a fintech | Track renewals and obligations; answer business questions fast | Contracts scattered across drives; can't use cloud AI under the bank partner's terms | Register with deadline alerts; Local only matters; NDA triage agent |
| **Maria, litigation partner** | Partner, 8-lawyer boutique | Build chronologies, find key facts in productions | Thousands of emails and scans; paralegal time is expensive | OCR, *Chronology* workflow, cited Ask across a production |
| **Sam, finance controller** | Controller, mid-market company | Reconcile invoices to contracts; monitor covenants | Spreadsheet matching, missed price escalations | Finance pack: invoice-to-contract checks, covenant tracker |
| **Lena, legal ops / IT** | Runs tools and risk for a 40-lawyer firm | Adopt AI without a confidentiality incident | Vendor questionnaires, no audit evidence | Per-matter policy, gateway with audit log, health report, self-hosted Firm Server |

---

## 4. Product Principles

1. **The matter is the boundary.** Data, policy, audit and agent runs are scoped to a matter. Nothing crosses matters unless a user does it.
2. **Local by default, cloud by permission.** A new matter is *Local only*. Cloud use is a deliberate, per-matter choice, narrowed per step by the router.
3. **Minimum necessary.** When a step does use the cloud, it sends the fewest passages, redacted. Whole documents never leave.
4. **Everything cited.** Every generated statement links to a source passage or register entry. Uncited output is labelled as such.
5. **Everything on the record.** Model calls (allowed and refused), file changes and agent steps go into a hash-chained audit log.
6. **Humans approve changes.** File changes, redlines and outbound actions are proposals until a person approves them, and they can be undone.
7. **Deterministic where possible.** Parsing, OCR, extraction rules, search and summaries are deterministic and testable. Models are used where judgement is needed.

---

## 5. Core Capabilities & Value Proposition

| Capability | What it is | Value |
|---|---|---|
| **Matter Vault** | Private, per-matter index of documents, passages, extracted fields and vectors | Fast, cited access to everything in a matter, without uploading it |
| **Assistant (Ask)** | Cited Q&A over a matter | Answers in seconds, each statement traceable |
| **Review Tables** | Documents × questions grid; each cell answered and cited; conditional and manual columns, comments, inline edits | Diligence and contract review at scale: the most-developed feature at Harvey, and core at Legora and CoCounsel |
| **Workflows** | One-click tasks: summarise, chronology, compare, memo, NDA triage | Expert output without prompt-writing |
| **Playbooks & Redlines** | Firm positions checked clause by clause; suggested redlines | Consistent negotiation, faster first pass |
| **Clause Library & Drafting** | Approved clauses from the firm's own documents; grounded drafting | Drafts in the firm's voice, from its precedents |
| **Agent Builder** | Multi-step agents from tools, steps and model rules | Automate the firm's own processes |
| **Monitors** | Agents that run on a schedule or when files arrive: deadline scans, new-document triage | Nothing is missed between reviews (cf. Legora Monitors, Harvey retention notifications) |
| **Knowledge bases** | Firm precedents, playbooks and clause banks, shared read-only across matters | Answers and drafts grounded in the firm's own work |
| **Hybrid AI layer** | Gateway, router and redaction | Strong cloud reasoning where allowed, local everywhere else, all audited |
| **Governance** | Policies, approvals, audit chain, health report | Evidence for clients, insurers and regulators |

**Value proposition:** *Do Harvey-class work on your own terms: local by default, cloud only by policy, every answer cited, every step audited, at a price a 10-lawyer firm can pay.*

---

## 6. Features by Release

Feature IDs refer to the [Feature Spec](../04-specs/ALIV_Feature-Spec.md). The phases are detailed in [Implementation Phases](../07-implementation/ALIV_Implementation-Phases.md).

| Release | Theme | Features | Status |
|---|---|---|---|
| **R0** (shipped) | Private matter vault | FS-1 Matters & policy · FS-2 Ingestion & OCR · FS-3 Hybrid search · FS-4 Register & extraction · FS-5 Assistant · FS-6 Approvals & file actions · FS-7 Audit & health | ✅ |
| **R1** | Hybrid foundation | FS-8 Model gateway · FS-9 Redaction · FS-10 Model router | Phase 1 |
| **R2** | Harvey-class work product | FS-11 Review Tables · FS-12 Workflows · FS-13 Playbooks & redlines · FS-14 Clause library & drafting · FS-15 Document compare | Phase 2 |
| **R3** | Agents | FS-16 Agent Builder · FS-17 Monitors · FS-18 Knowledge bases · FS-19 Finance pack | Phase 3 |
| **R4** | Scale & reach | FS-20 Firm Server & roles · FS-21 Word add-in · FS-22 Research connectors | Phase 4 |

---

## 7. Market Positioning

```
                         data stays with the firm
                                   ▲
         AnythingLLM ●             │            ● ALIV (target: R2+)
         GPT4All     ●             │     ● ALIV (today)
         Elephas     ●             │  ● LocalApps
   ────────────────────────────────┼──────────────────────────────► legal workflow depth
         ChatGPT / Copilot ●       │          ● Spellbook   ● Luminance
                                   │                 ● Legora   ● Harvey
                                   │
                         vendor cloud by default
```

- **Against Harvey, Legora and Luminance:** comparable workflows, but deployable inside the firm, with per-step cloud control and an audit chain, at small-firm prices.
- **Against generic local tools:** legal and finance workflows, extraction, playbooks, approvals and an audit trail, not just chat.
- **Positioning line:** *The legal AI you can run inside your firm.*

---

## 8. Business Model

| Tier | For | Includes | Indicative price* |
|---|---|---|---|
| **Solo** | Individual lawyers and accountants | Desktop app, local models, R0–R2 features | $39–59 / user / month, or a perpetual licence with a yearly update plan |
| **Team** | Firms of 2–50 | Solo, plus Agent Builder, shared clause library and playbooks, priority support | $79–129 / user / month |
| **Firm Server** | Firms needing shared vaults and admin | Self-hosted server, roles, SSO, central audit | Per-server licence plus per-user fee |
| **Enterprise / private cloud** | Larger firms, in-house at regulated companies | Private deployment, compliance pack, custom agents | Annual contract |

\*Placeholders to validate in pilots. The market research suggests anchoring between free open-source tools and Spellbook.

Cloud model usage is billed by the provider under the customer's own API key (bring your own key). ALIV does not resell tokens in R1–R3.

---

## 9. Technical & Operational Requirements

| Area | Requirement |
|---|---|
| **Platforms** | macOS 13+ (Apple Silicon recommended), Windows 10/11, Ubuntu 22.04+ |
| **Hardware** | Minimum 16 GB RAM. Recommended 32 GB, or Apple Silicon M-series Pro/Max, for 14B+ local models |
| **Local models** | llama.cpp (GGUF) and MLX. A recommended model per hardware tier. A local embedding model is required for meaning search |
| **Cloud models** | OpenAI-compatible and Anthropic APIs via the gateway. Customer-supplied keys |
| **Data at rest** | One SQLite vault per installation (R0). Encryption at rest planned for R1; until then, full-disk encryption |
| **Performance** | Ingestion ≥ 5 text documents/s on a 2023 laptop. Ask first token ≤ 3 s with a local 7–8B model. Search ≤ 300 ms on 10k passages |
| **Reliability** | Ingestion resumable; unchanged files skipped; no data loss on crash (SQLite WAL) |
| **Auditability** | 100% of model calls and file changes in the hash chain; integrity check on demand |
| **Accessibility** | WCAG 2.1 AA for core screens |
| **Updates** | Signed installers and an update server (planned before general availability) |

---

## 10. Success Metrics (KPIs)

| Area | KPI | Target (12 months after R2) |
|---|---|---|
| **Activation** | Users who index a matter and get a cited answer in their first session | ≥ 70% |
| **Value** | Hours saved per diligence matter (self-reported, pilot cohort) | ≥ 50% vs baseline |
| **Accuracy** | Review Table cells judged correct and correctly cited in pilot audits | ≥ 95% |
| **Trust** | Matters set to Local only that never recorded a cloud call | 100% (a hard guarantee, measured from the audit log) |
| **Engagement** | Weekly active users / paid seats | ≥ 60% |
| **Workflow adoption** | Share of active users running ≥ 1 workflow or agent a week | ≥ 40% |
| **Retention** | Logo retention at 12 months | ≥ 85% |
| **Revenue** | Paying firms | 150 (Solo + Team) |

---

## 11. Risks, Dependencies & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|---|---|---|---|
| Local models too weak for complex reasoning | Poor answers, lost trust | High (on small hardware) | Hybrid router: cloud for heavy steps where allowed; hardware tiers; citations make errors visible |
| A policy bypass (a cloud call from a Local only matter) | Fatal to the brand | Low after FS-8 | All calls through the Rust gateway; tests; audit; third-party review |
| Redaction misses an identifier | Confidential data reaches a cloud provider | Medium | Conservative default (redact all detected entities); preview of outbound text; Local only remains available |
| Harvey or Legora ship a private or on-prem edition | Narrows the differentiator | Medium | Win on price, small-firm fit, auditability and Windows desktop; move fast on R2 |
| Microsoft builds local Q&A into Windows | Generic local search becomes a commodity | Medium | Moat is the workflow layer (tables, playbooks, agents), not retrieval |
| Extraction rules fail on real-world documents | Low register accuracy | Medium | Model-assisted extraction as a second pass (FS-11); benchmark on real anonymised documents |
| Support load from diverse hardware | Cost, churn | Medium | Hardware check at setup; vault health report; recommended configurations |
| Dependence on the open-source base (Apache-2.0) | Upstream divergence | Low | Isolated vault plugin; keep the base thin; licence compliance |

---

## 12. Research Basis

The capability set is drawn from what the leading products ship in 2026, then filtered through ALIV's local-first constraint. Vendor and aggregator sources; verify before quoting externally.

| Observation | Source | Effect on ALIV |
|---|---|---|
| Harvey's review tables gained conditional columns, manual-input columns, cell comments and inline editing; Assistant can run review tables and suggests them | [Harvey release notes](https://help.harvey.ai/release-notes/category/vault), [The Brief, May 2026](https://www.harvey.ai/en-US/blog/the-brief-may-2026), [July 2026](https://www.harvey.ai/blog/the-brief-july-2026) | FS-11 scope includes all four |
| Harvey workflows have a builder with optional steps and can embed review tables | [Harvey workflow release notes](https://help.harvey.ai/release-notes/category/workflows), [releases.sh](https://releases.sh/release/rel__Oof-GkMaR7xOAYbNMZbZ-agentic-workflow-builder-scc-online-and-lovdata-integrations) | FS-12 has a library and a builder; FS-16 steps can produce tables |
| Harvey Vault accepts .pst email archives; retention-deadline notifications | [The Brief, June/July 2026](https://www.harvey.ai/blog/the-brief-july-2026) | FS-2 adds mailbox archives; FS-17 Monitors |
| Legora: Agent, Tabular Review, Workflows, Legal Research, Monitors; seat-based custom pricing | [Sacra](https://sacra.com/c/legora/), [Vaquill review](https://www.vaquill.ai/blog/legora-legal-ai-review) | Monitors (FS-17); confirms tabular review as table stakes |
| CoCounsel: guided agentic workflows, deep research on Westlaw, bulk review of up to 10,000 documents | [Thomson Reuters, Nov 2025](https://www.thomsonreuters.com/en/press-releases/2025/november/thomson-reuters-advances-ai-market-leadership-with-new-agentic-ai-solutions), [Aug 2026](https://www.thomsonreuters.com/en/press-releases/2026/august/thomson-reuters-launches-next-generation-of-cocounsel-legal-the-ai-ecosystem-built-for-legal-professionals) | FS-11 scales to thousands of documents; research needs licensed content (FS-22) |
| Spellbook's moat is working inside Word | [aivortex](https://www.aivortex.io/legal/compare/spellbook-alternatives-2026/) | FS-21 Word add-in in R4 |
| Private / on-prem legal AI is a growing category; residency does not settle who can be compelled to produce data | [ibl.ai](https://ibl.ai/blog/on-premise-legal-ai-platform), [Alexi](https://alexi.com/platform/containers) | Validates the Firm Server / private-cloud path; the security doc must not over-claim |

---

## 13. Appendix: Glossary

| Term | Meaning |
|---|---|
| **Matter** | A client engagement or project: a private vault with its own policy and audit |
| **Passage** | A chunk of a document, the unit of search and citation |
| **Register** | Structured fields extracted from each document (parties, dates, amounts…) |
| **Policy** | *Local only* or *Hybrid*, set per matter; the router narrows it further per step |
| **Gateway** | The single Rust component every model call passes through (FS-8) |
| **Redaction** | Replacing names, parties and amounts with placeholders before a cloud call, then restoring them (FS-9) |
| **Workflow** | A predefined multi-step task with fixed prompts and output format |
| **Agent** | A user-defined, multi-step process with tools, model rules and optional schedule |
