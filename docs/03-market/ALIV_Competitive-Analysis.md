# Competitive Analysis
## ALIV in the legal & finance AI market

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active

> **Sources:** vendor release notes, press releases, research profiles (Sacra, Contrary) and comparison sites, as of October 2026. Comparison sites are often run by competitors, which is noted where it matters. Verify pricing and capability claims with each vendor before using them externally.

---

## 1. Summary

- **The market has converged on four capabilities:**
  1. cited assistant chat;
  2. a document vault with **tabular review** (documents × questions);
  3. **workflows and agents** that run multi-step legal tasks;
  4. **research** grounded in licensed legal content.
- **Harvey, Legora and CoCounsel all ship all four.** They are cloud services sold to large firms on custom enterprise contracts.
- **The open position is the same capabilities, deployed inside the firm.** That means:
  - local models by default;
  - a governed path to cloud models;
  - an audit trail a client or regulator can inspect;
  - small-firm prices.

  Private legal-AI vendors (Alexi, ibl.ai, On Premises AI) show demand for the deployment model, but they are platforms or services, not polished workflow products. Generic local tools (AnythingLLM, GPT4All, Elephas) have the deployment but not the legal workflows.
- **ALIV's plan:** reach feature parity on the four capabilities that work well locally (assistant, tabular review, workflows, agents), differentiate on governance (per-step routing, redaction, audit), and partner, not build, for licensed research content.

---

## 2. Competitor profiles

### 2.1 Harvey (enterprise leader)

| | |
|---|---|
| **Positioning** | Full-stack legal AI for large firms and corporate legal departments |
| **Deployment** | Vendor cloud on Microsoft Azure ([Harvey newsroom](https://www.harvey.ai/en-US/newsroom)); EU instance ([eu.help.harvey.ai](https://eu.help.harvey.ai/release-notes)) |
| **Models** | Originally OpenAI models fine-tuned on legal data; multi-model by 2026, with Anthropic Claude Sonnet 4.6 live (Feb 2026) and newer OpenAI models selectable ([newsroom](https://www.harvey.ai/en-US/newsroom), [release notes](https://eu.help.harvey.ai/release-notes/page/4)) |
| **Pricing** | Enterprise contracts, not published. A third-party review cites about $288k a year for a team ([LawXY](https://www.lawxyai.com/articles/harvey-ai-review-2026-is-it-worth-288k-a-year-for-legal-teams)) |

**Product areas** (from release notes, Jan–Jul 2026):
- **Assistant:**
  - chat over files;
  - @-mention sources;
  - suggests and runs review tables from a prompt;
  - creates and edits PDFs;
  - understands charts and diagrams.
- **Vault:**
  - stores and bulk-analyses document sets;
  - 500 MB upload limit;
  - `.pst` email archives;
  - retention-deadline and sharing notifications.
- **Review tables:**
  - conditional columns;
  - manual-input columns;
  - comments on cells;
  - inline editing of every cell;
  - can be built over knowledge bases and sandbox vaults with view-only access.
- **Workflows and agents:**
  - agentic workflow builder with optional steps;
  - workflows can embed review tables and knowledge sources;
  - "Harvey Agents execute legal work end to end".
- **Knowledge:**
  - licensed and jurisdictional sources (e.g. Lovdata for Norway, SCC Online for India) ([releases.sh](https://releases.sh/release/rel__Oof-GkMaR7xOAYbNMZbZ-agentic-workflow-builder-scc-online-and-lovdata-integrations)).

**Architecture (inferred):** a cascade of fine-tuned models, retrieval over customer documents and public legal databases, and an orchestration layer ([Contrary Research](https://research.contrary.com/company/harvey)). Customer data is not used for training.

**Where ALIV can't match it:**
- breadth of agents;
- licensed research content;
- enterprise sales reach;
- frontier-model quality on every step.

**Where ALIV can win:**
- data stays inside the firm by default;
- per-step cloud control;
- inspectable audit chain;
- price;
- Windows desktop for small firms.

### 2.2 Legora (fast-growing challenger)

| | |
|---|---|
| **Positioning** | Collaborative legal AI workspace for firms and in-house teams ([Sacra](https://sacra.com/c/legora/)) |
| **Product** | Agent (end-to-end execution), **Tabular Review**, Workflows, Legal Research, **Monitors**, firm-wide search ([Vaquill review](https://www.vaquill.ai/blog/legora-legal-ai-review), [Capterra](https://www.capterra.com/p/10043592/Legora/reviews/)) |
| **Pricing** | No public prices; demo-led, seat-based custom quotes. A competitor's comparison claims consumption pricing for the agentic tier ([Bind](https://bindlegal.com/resources/comparisons/legora-pricing-2026/), unverified) |

Takeaways for ALIV:
- **Tabular review is now table stakes.** Teams drag in folders of contracts and get a filterable grid; reviewers note no general chatbot matches it.
- **Monitors** (agents that watch for changes) are a distinct product line. That's an opening for local folder-watching agents.

### 2.3 CoCounsel Legal (Thomson Reuters)

| | |
|---|---|
| **Positioning** | AI ecosystem grounded in Westlaw and Practical Law content |
| **Product** | Guided agentic workflows; Deep Research on Westlaw; **bulk document review of up to 10,000 documents in tabular form** (beta, Nov 2025); Westlaw Brief Builder; a Claude connector (Aug 2026) ([TR, Nov 2025](https://www.thomsonreuters.com/en/press-releases/2025/november/thomson-reuters-advances-ai-market-leadership-with-new-agentic-ai-solutions), [TR, Aug 2026](https://www.thomsonreuters.com/en/press-releases/2026/august/thomson-reuters-launches-next-generation-of-cocounsel-legal-the-ai-ecosystem-built-for-legal-professionals)) |
| **Scale** | Thomson Reuters reports one million professionals using CoCounsel (Feb 2026) |

Takeaway: **licensed content is CoCounsel's moat.** ALIV should integrate research sources through partners and connectors, not compete on them.

### 2.4 Luminance

- An end-to-end contract platform covering 1,000+ legal concepts.
- Strong at **autonomous, high-volume review** (e.g. NDAs at volume) ([dupple](https://dupple.com/learn/best-ai-contract-review), [aivortex](https://www.aivortex.io/legal/compare/luminance-alternatives-2026/)).
- One analysis cites training on 150M+ legal documents and on-prem-friendly deployments.
- Reported at about $30M ARR ([Sacra](https://sacra.com/research/luminance-at-30m-arr/)).

Takeaway: **playbook-driven, autonomous first-pass review** sells. FS-13 should support batch runs, not only one contract at a time.

### 2.5 Spellbook

- Contract drafting and review **inside Microsoft Word** for in-house teams and firms.
- Its Word integration is described as its moat ([aivortex](https://www.aivortex.io/legal/compare/spellbook-alternatives-2026/), [rfp.wiki](https://www.rfp.wiki/vendors/spellbook/luminance)).
- Self-serve pricing, reported at roughly $99–299 per user per month.

Takeaway: lawyers draft in Word. A Word add-in (FS-21) is required to compete on drafting. Until then, ALIV exports redlines as DOCX.

### 2.6 Hebbia

- Described as a specialised **data extraction / matrix analysis** tool rather than a drafting assistant ([Spellbook comparison](https://www.spellbook.legal/briefs/hebbia-vs-harvey), a competitor-authored source).
- Strong in finance (investment research).

Takeaway: finance teams value grid-style extraction across many documents, which is FS-11 plus the finance pack (FS-19).

### 2.7 Private and on-prem legal AI

| Vendor | Offer |
|---|---|
| **Alexi** | Private cloud or self-hosted containers ([alexi.com](https://alexi.com/platform/containers), [press release](https://www.silicon.co.uk/press-release/alexi-launches-private-cloud-to-protect-legal-teams-sensitive-data)) |
| **ibl.ai** | Air-gapped and on-prem agentic AI for law firms ([ibl.ai](https://ibl.ai/blog/on-premise-legal-ai-platform)) |
| **On Premises AI** | Private AI Knowledge Portal for law firms ([LegalTech Hub](https://www.legaltechnologyhub.com/vendors/private-ai-knowledge-portal-by-on-premises-ai/)) |
| **Sandlabs** (AU) | Cloud-private with local residency; self-hosted where a matter requires it ([sandlabs](https://sandlabs.com.au/ai-for-lawyers)) |

Points worth keeping (mostly vendor marketing):
- **Three deployment models recur:** on-prem, virtual private cloud, and vendor-hosted.
- **Residency ≠ protection.** A residency promise says *where* data sits, not *who can be compelled to produce it*. ALIV's security doc must not over-claim.
- **Counterpoint:** self-hosting's cost and staffing burden often outweighs its benefits for small firms ([LawXY](https://www.lawxyai.com/articles/why-zero-data-retention-is-the-future-of-private-legal-ai)). That argues for a **desktop app that needs no IT**, which is ALIV's starting point, with Firm Server as an option.

### 2.8 Generic local tools

| Tool | Strength | Gap vs ALIV |
|---|---|---|
| AnythingLLM | Free, desktop and Docker, workspaces, agents | No legal workflows, extraction, approvals or audit |
| GPT4All LocalDocs | Simplest setup | Collection-level chat only |
| Elephas | Polished Mac app | Mac-only; no extraction, review tables or audit |
| DEVONthink | Lawyer favourite document manager with local AI (2025) | Mac-only; AI is an add-on |
| LocalApps (NL) | On-device legal AI for small firms | Closest direct competitor. Track closely |

---

## 3. Feature comparison

✅ = shipped · ◐ = partial · ○ = planned in ALIV · — = not offered (as far as public sources show)

| Capability | Harvey | Legora | CoCounsel | Luminance | Spellbook | Local tools | **ALIV today** | **ALIV plan** |
|---|---|---|---|---|---|---|---|---|
| Cited assistant over documents | ✅ | ✅ | ✅ | ◐ | ◐ | ◐ | ✅ | ✅ |
| Document vault / matters | ✅ | ✅ | ✅ | ✅ | — | ◐ | ✅ | ✅ |
| OCR for scans | ✅ | ✅ | ✅ | ✅ | — | ◐ | ✅ | ✅ |
| Email archives (.pst) | ✅ | ? | ? | ? | — | — | ◐ (.eml) | ○ FS-2 |
| Tabular review, custom columns | ✅ | ✅ | ✅ (10k docs) | ✅ | — | — | ◐ fixed fields | ○ FS-11 |
| Prebuilt workflows | ✅ | ✅ | ✅ | ◐ | ◐ | — | — | ○ FS-12 |
| Workflow / agent builder | ✅ | ✅ | ◐ | — | — | ◐ | ◐ CLI agent | ○ FS-16 |
| Monitors / scheduled agents | ◐ | ✅ | — | — | — | — | — | ○ FS-17 |
| Playbook review & redlines | ◐ | ◐ | ◐ | ✅ | ✅ | — | — | ○ FS-13 |
| Drafting in Word | ◐ | ◐ | ◐ | ✅ | ✅ | — | — | ○ FS-21 |
| Licensed legal research | ✅ | ✅ | ✅ (Westlaw) | — | — | — | — | ○ FS-22 (partners) |
| DMS / SharePoint / email connectors | ✅ | ✅ | ◐ | ✅ | — | — | — | ○ FS-23 |
| Saved, versioned outputs (artifacts) | ◐ | ◐ | ◐ | ? | — | — | ◐ exports | ○ FS-25 |
| Firm know-how as reusable skills | ? | ? | ? | ? | ? | ◐ | ◐ base-app skills | ○ FS-28 (governed, per matter) |
| MCP (tools in, vault out) | ? | ? | ? | ? | ? | ◐ | ◐ base-app MCP client | ○ FS-24 (trust levels, allow-lists) |
| Event-driven automation | ◐ | ✅ | — | ◐ | — | — | — | ○ FS-26 |
| Runs fully on the firm's machines | — | — | — | ◐ | — | ✅ | ✅ | ✅ |
| Per-matter cloud policy | — | — | — | — | — | — | ✅ | ✅ |
| Per-step model routing + redaction | — | — | — | — | — | — | — | ○ FS-9/10 |
| Tamper-evident audit of every model call | ? | ? | ? | ? | — | — | ✅ | ✅ |
| Approval-gated, undoable file actions | — | — | — | — | — | — | ✅ | ✅ |
| Self-serve small-firm pricing | — | — | ◐ | — | ✅ | ✅ | ○ | ○ |

Rows for connectors, artifacts, skills, MCP and automation were added without new research; their competitor cells are best-known status and should be re-checked before external use. "?" means no public evidence either way; enterprise products likely log internally, but don't expose a verifiable chain to the customer.

---

## 4. Positioning

| Segment | Who wins today | ALIV's angle |
|---|---|---|
| AmLaw 100 / Magic Circle | Harvey, Legora, CoCounsel | Not the beachhead. Later via private cloud for matters that can't leave the firm |
| Mid-size firms (50–300) | Legora, CoCounsel, Luminance | Firm Server with an audit chain; Hybrid routing gives cloud quality with control |
| **Small firms (2–50) and solos** | Spellbook, generic tools, nothing | **Beachhead.** Desktop, no IT, Harvey-class workflows, honest pricing |
| **Regulated in-house teams** | Often nothing (cloud AI banned by policy) | **Beachhead.** Local only matters, audit evidence |
| **Finance teams, accounting and tax** | Hebbia (high end), spreadsheets | Review Tables + finance pack over contracts, invoices and filings |

**Message:** *The legal AI you can run inside your firm.* Lead with cited accuracy and review tables (what users buy), and close on governance (what risk teams approve).

---

## 5. Threats and responses

| Threat | Response |
|---|---|
| Harvey or Legora launch a private-deployment edition | Compete on price, desktop simplicity, Windows, and the per-step audit chain; ship R2 quickly |
| Microsoft adds on-device document Q&A to Windows/Copilot | Generic retrieval becomes a commodity. ALIV's moat is the workflow and governance layer |
| Competitors adopt open standards (MCP, Agent Skills) | ALIV already ships both in the base app; the difference is governance: trust levels, per-matter allow-lists, redaction and audit on every tool and skill |
| Local models stay too weak for complex reasoning | Hybrid routing with redaction; hardware tiers; citations expose errors |
| Licensed content locks users into CoCounsel/Westlaw | Partner and connect (FS-22); focus ALIV on the firm's own documents, where licensed content doesn't help |
| A new local-first legal startup (e.g. LocalApps) wins small firms first | Speed on R1–R2, an open data format, and finance coverage as a second wedge |
