# ALIV documentation

ALIV is a **hybrid intelligent agent platform for legal and finance teams**. Every matter is a private vault on the firm's own machines. Agents, workflows and review tables run over it, with a per-matter, per-step policy that decides what runs locally and what may use a cloud model, and every model call is audited.

The product documents run from brand to product, market, specs, design, technical and implementation. Feature scope is based on research into Harvey, Legora, CoCounsel, Luminance, Spellbook and private legal-AI vendors (see the [Competitive Analysis](03-market/ALIV_Competitive-Analysis.md)).

## Product documents

| # | Document | What it answers | Status |
|---|---|---|---|
| **01 Brand** | | | |
| 1 | Brand Identity | Who ALIV is, how it speaks, how it looks | Planned (batch 3) |
| **02 Product** | | | |
| 2 | Product Overview | ALIV on one page | Planned (batch 3) |
| 3 | [Product Definition](02-product/ALIV_Product-Definition.md) | Vision, personas, value, features by release, KPIs, risks | ✅ |
| **03 Market** | | | |
| 4 | [Competitive Analysis](03-market/ALIV_Competitive-Analysis.md) | Harvey and the field: where ALIV wins and loses | ✅ |
| **04 Specs** | | | |
| 5 | [Feature Spec](04-specs/ALIV_Feature-Spec.md) | Every feature module: requirements, edge cases, scope | ✅ |
| 6 | [User Journeys](04-specs/ALIV_User-Journeys.md) | End-to-end flows for each persona | ✅ |
| 7 | [User Stories](04-specs/ALIV_User-Stories.md) | Stories with acceptance criteria | ✅ |
| **05 Design** | | | |
| 8 | [Design System](05-design/ALIV_Design-System.md) | Tokens, components, states, accessibility | ✅ |
| 9 | [UI Spec](05-design/ALIV_UI-Spec.md) | Screen-by-screen behaviour | ✅ |
| 10 | [Wireframes](05-design/ALIV_Wireframes.md) | Layouts of current and planned screens | ✅ |
| **06 Technical** | | | |
| 11 | System Architecture | Components, sequences, non-functional requirements | Planned (batch 3) |
| 12 | [Hybrid AI Architecture](06-technical/ALIV_Hybrid-AI-Architecture.md) | Model gateway, router, redaction, agent runtime | ✅ |
| 13 | Architecture Stack | Technology choices and why | Planned (batch 3) |
| 14 | Data Model | Vault schema, current and planned | Planned (batch 3) |
| 15 | API Design | Vault commands, agent tools, local API, CLI | Planned (batch 3) |
| 16 | Security & Compliance | Threat model, controls, compliance mapping | Planned (batch 3) |
| 17 | Deployment Architecture | Desktop, Firm Server, private cloud | Planned (batch 3) |
| 18 | Repo Structure | Where everything lives | Planned (batch 3) |
| **07 Implementation** | | | |
| 19 | [Implementation Phases](07-implementation/ALIV_Implementation-Phases.md) | Phases, scope and exit criteria | ✅ |
| 20 | Sprint Plan | Phases broken into PRs | Planned (batch 3) |
| 21 | Implementation Checklist | Per-phase done list | Planned (batch 3) |

## Guides

| Document | For |
|---|---|
| [User guide](user-guide.md) | People using ALIV |
| [Security and privacy](security.md) | A firm's IT or risk team |
| [Developer guide](developer.md) | Contributors |
| [Plan (completed)](PLAN.md) | The rebuild of the legacy app into the desktop app (steps A–F, done) |

## Background

[Market research](MARKET_RESEARCH.md) (Oct 2026 snapshot, partly about the legacy app), [future strategy](ALIV_FUTURE_STRATEGY.md), [customer one-pager](CUSTOMER_ONE_PAGER.md). Legacy-app documents are in [archive/](archive/README.md).
