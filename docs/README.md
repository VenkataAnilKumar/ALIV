# ALIV documentation

ALIV is a **hybrid intelligent agent platform for legal teams, and the finance work in their contracts** (invoices against contract terms, covenants, payment terms, audit evidence). Every matter is a private vault on the firm's own machines. Agents, workflows and review tables run over it, with a per-matter, per-step policy that decides what runs locally and what may use a cloud model, and every model call is audited.

The product documents run from brand to product, market, specs, design, technical and implementation. Feature scope is based on research into Harvey, Legora, CoCounsel, Luminance, Spellbook and private legal-AI vendors (see the [Competitive Analysis](03-market/ALIV_Competitive-Analysis.md)).

## Product documents

| # | Document | What it answers | Status |
|---|---|---|---|
| **01 Brand** | | | |
| 1 | [Brand Identity](01-brand/ALIV_Brand-Identity.md) | Who ALIV is, how it speaks, how it looks | ✅ |
| **02 Product** | | | |
| 2 | [Product Overview](02-product/ALIV_Product-Overview.md) | ALIV on one page | ✅ |
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
| 11 | [System Architecture](06-technical/ALIV_System-Architecture.md) | Components, sequences, non-functional requirements | ✅ |
| 12 | [Hybrid AI Architecture](06-technical/ALIV_Hybrid-AI-Architecture.md) | Model gateway, router, redaction, agent runtime | ✅ |
| 13 | [Architecture Stack](06-technical/ALIV_Architecture-Stack.md) | Technology choices and why | ✅ |
| 14 | [Data Model](06-technical/ALIV_Data-Model.md) | Vault schema, current and planned | ✅ |
| 15 | [API Design](06-technical/ALIV_API-Design.md) | Vault commands, agent tools, local API, CLI | ✅ |
| 16 | [Security & Compliance](06-technical/ALIV_Security-Compliance.md) | Threat model, controls, compliance mapping | ✅ |
| 17 | [Deployment Architecture](06-technical/ALIV_Deployment-Architecture.md) | Desktop, Firm Server, private cloud | ✅ |
| 18 | [Repo Structure](06-technical/ALIV_Repo-Structure.md) | Where everything lives | ✅ |
| **07 Implementation** | | | |
| 19 | [Implementation Phases](07-implementation/ALIV_Implementation-Phases.md) | Phases, scope and exit criteria | ✅ |
| 20 | [Sprint Plan](07-implementation/ALIV_Sprint-Plan.md) | Phases broken into PRs | ✅ |
| 21 | [Implementation Checklist](07-implementation/ALIV_Implementation-Checklist.md) | Per-phase done list | ✅ |

## Guides

| Document | For |
|---|---|
| [User guide](guides/user-guide.md) | People using ALIV |
| [Security and privacy](guides/security.md) | A firm's IT or risk team: what is true today |
| [Developer guide](guides/developer.md) | Contributors: build, test, CI |

## Folder layout

```
docs/
├─ 01-brand … 07-implementation   Product documents (table above)
├─ guides/                        User, security and developer guides
├─ assets/                        Screenshots (shipped app) and mockups/ (planned screens)
└─ archive/                       History only: superseded docs and the legacy app
```

Older strategy, market research, the customer one-pager and the completed rebuild plan are kept in [archive/](archive/README.md). The numbered documents above replace them.
