# Feature Specification
## ALIV: hybrid intelligent agent platform for legal and finance

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active (R0 shipped, R1–R4 planned)

---

## Overview

This document specifies every ALIV feature module: its purpose, functional requirements, edge cases, how it treats local and cloud models, and what is out of scope. It is the engineering handoff for the [Product Definition](../02-product/ALIV_Product-Definition.md). The release plan is in [Implementation Phases](../07-implementation/ALIV_Implementation-Phases.md).

**Priority:** P0 = required for the release · P1 = should ship in the release · P2 = may slip.

**Rule for every module that calls a model:**
- every call goes through the **Model Gateway** (FS-8);
- the call is subject to the matter's policy and the **Router** (FS-10);
- every call is recorded in the audit chain (FS-7).

Nothing calls a provider directly.

## Module Index

| Module | ID prefix | Release | Status |
|---|---|---|---|
| [FS-1] Matters & Policy | MAT | R0 | ✅ |
| [FS-2] Ingestion & OCR | ING | R0 (+R2 archives) | ✅ |
| [FS-3] Hybrid Search | SRCH | R0 | ✅ |
| [FS-4] Register & Extraction | REG | R0 | ✅ |
| [FS-5] Assistant (Ask) | ASK | R0 (+R2 outputs) | ✅ |
| [FS-6] Approvals & File Actions | ACT | R0 | ✅ |
| [FS-7] Audit Log & Vault Health | AUD | R0 | ✅ |
| [FS-8] Model Gateway | GW | R1 | Planned |
| [FS-9] Redaction & Pseudonymisation | RED | R1 | Planned |
| [FS-10] Model Router | RTR | R1 | Planned |
| [FS-11] Review Tables | TBL | R2 | Planned |
| [FS-12] Workflows | WF | R2 | Planned |
| [FS-13] Playbooks & Redlines | PB | R2 | Planned |
| [FS-14] Clause Library & Drafting | DRF | R2 | Planned |
| [FS-15] Document Compare | CMP | R2 | Planned |
| [FS-16] Agent Builder | AGT | R3 | Planned |
| [FS-17] Monitors | MON | R3 | Planned |
| [FS-18] Knowledge Bases | KB | R3 | Planned |
| [FS-19] Finance Pack | FIN | R3 | Planned |
| [FS-20] Firm Server & Roles | SRV | R4 | Planned |
| [FS-21] Word Add-in | WORD | R4 | Planned |
| [FS-22] Research Connectors | RES | R4 | Planned |

---

## [FS-1] Matters & Policy ✅

**Purpose:** a matter is a private vault for one engagement. It is the boundary for data, policy, audit and agent runs.

| ID | Requirement | Priority | Status |
|---|---|---|---|
| MAT-01 | Create, rename, delete a matter with name, client and type (Legal / Finance) | P0 | ✅ |
| MAT-02 | Policy per matter: **Local only** (default) or **Hybrid** | P0 | ✅ |
| MAT-03 | Policy changes are audited with the old and new value | P0 | ✅ |
| MAT-04 | Deleting a matter removes its index (documents, passages, fields, embeddings, proposals) but never the user's files; its audit entries are kept so the chain stays verifiable | P0 | ✅ |
| MAT-05 | Search, answers, tables and agents never read across matters unless the user selects several matters explicitly | P0 | ✅ (single-matter today) |
| MAT-06 | Ethical walls: users and groups allowed per matter | P1 | R4 (FS-20) |
| MAT-07 | Retention date per matter, with reminders before it (cf. Harvey retention notifications) | P2 | R3 (FS-17) |

**Edge cases:** deleting a matter with an agent mid-run cancels the run first. Changing a Hybrid matter to Local only takes effect for the next model call, including within running agents.

---

## [FS-2] Ingestion & OCR ✅

**Purpose:** turn folders of documents into searchable, citable passages without moving or changing the files.

| ID | Requirement | Priority | Status |
|---|---|---|---|
| ING-01 | Add folders (recursive) or files; supported: PDF, DOCX, XLSX, PPTX, HTML, TXT/MD, CSV, RTF, EML | P0 | ✅ |
| ING-02 | Skip unchanged files by content hash; re-read only changed files | P0 | ✅ |
| ING-03 | OCR images (PNG, JPEG, TIFF) and PDFs without a text layer with Tesseract; enlarge low-resolution scans; up to 50 PDF pages | P0 | ✅ |
| ING-04 | Record OCR confidence per document and scale extracted-field confidence by it | P0 | ✅ |
| ING-05 | Unreadable files stay listed with the reason | P0 | ✅ |
| ING-06 | **Mailbox archives** (.pst, .mbox): each message and attachment becomes a document, keeping thread and sender metadata (cf. Harvey .pst upload) | P1 | R2 |
| ING-07 | Large-matter ingestion: 10,000+ documents with progress, pause and resume (cf. CoCounsel bulk review) | P1 | R2 |
| ING-08 | Watch folders: new or changed files are ingested automatically (feeds FS-17) | P1 | R3 |
| ING-09 | Duplicate and near-duplicate detection across a matter | P2 | R3 |

**Edge cases:**
- **Password-protected files:** skipped with "protected".
- **Files over 200 MB:** skipped with the size.
- **Mixed-language scans:** OCR uses the configured languages (default English) and reports low confidence.

**Local/cloud:** all ingestion is local. Never calls a model.

---

## [FS-3] Hybrid Search ✅

**Purpose:** find the passages that answer a question, whether they share its words or only its meaning.

| ID | Requirement | Priority | Status |
|---|---|---|---|
| SRCH-01 | Keyword search with FTS5 (porter stemming), scoped to one matter | P0 | ✅ |
| SRCH-02 | Meaning search over passage embeddings from the **local** embedding model, never a cloud provider | P0 | ✅ |
| SRCH-03 | Fuse the two rankings with reciprocal-rank fusion (keyword 0.55, k = 60) | P0 | ✅ |
| SRCH-04 | Background embedding of new passages; coverage shown in Documents | P0 | ✅ |
| SRCH-05 | Fall back to keyword-only, and say so, when no embedding model is available | P0 | ✅ |
| SRCH-06 | Filters: document type, date range, party, folder | P1 | R2 |
| SRCH-07 | Search across several selected matters (explicit user choice) | P2 | R3 |
| SRCH-08 | ANN index when a matter exceeds ~200k passages | P2 | R4 |

---

## [FS-4] Register & Extraction ✅

**Purpose:** a structured register of each document's key terms, every value linked to its source.

| ID | Requirement | Priority | Status |
|---|---|---|---|
| REG-01 | Rule-based extraction: type, parties, effective/expiry dates, renewal deadline, amount, notice period, governing law, penalty | P0 | ✅ |
| REG-02 | Each value carries confidence and the passage it came from | P0 | ✅ |
| REG-03 | Filters: due in 90 days, passed, needs review | P0 | ✅ |
| REG-04 | CSV export with formula neutralisation | P0 | ✅ |
| REG-05 | Per-document summary from fields and opening words (no model) | P0 | ✅ |
| REG-06 | Users correct a value; the correction is kept across re-ingestion and audited | P1 | R2 |
| REG-07 | Model-assisted second pass for fields the rules left empty, always cited | P1 | R2 (as Review Table columns) |
| REG-08 | XLSX export | P1 | R2 |

---

## [FS-5] Assistant (Ask) ✅

**Purpose:** cited answers to questions about a matter.

| ID | Requirement | Priority | Status |
|---|---|---|---|
| ASK-01 | Answer only from numbered sources (passages and register rows), citing each statement [n] | P0 | ✅ |
| ASK-02 | Click a citation to open the source passage | P0 | ✅ |
| ASK-03 | Fit sources to the model's context window, most relevant first; report what was left out | P0 | ✅ |
| ASK-04 | Without a model: answer deadline questions from the register; otherwise show matching passages | P0 | ✅ |
| ASK-05 | Show where the answer runs (local / cloud) and the search mode | P0 | ✅ |
| ASK-06 | **@-mention** documents, tables or knowledge bases to scope a question (cf. Harvey) | P1 | R2 |
| ASK-07 | Suggest a Review Table or Workflow when the question fits one ("list the change-of-control clauses in all contracts") | P1 | R2 |
| ASK-08 | Save an answer as a DOCX or PDF note with citations as footnotes | P1 | R2 |
| ASK-09 | Follow-up questions keep the conversation's sources | P1 | R2 |
| ASK-10 | Flag statements the model made without a citation | P1 | R2 |

**Local/cloud:** through FS-8/10. Under Hybrid, the router may send the question and the selected sources, redacted (FS-9), to the cloud.

---

## [FS-6] Approvals & File Actions ✅

| ID | Requirement | Priority | Status |
|---|---|---|---|
| ACT-01 | Propose `Type_Counterparty_Date` renames with a risk level | P0 | ✅ |
| ACT-02 | Nothing changes on disk until approved and applied | P0 | ✅ |
| ACT-03 | Never overwrite; every applied action can be undone | P0 | ✅ |
| ACT-04 | Move into folders by type, party or year (proposed, approved, undoable) | P1 | R2 |
| ACT-05 | Agents propose actions into the same approval queue; they can never apply them directly | P0 | R3 |

---

## [FS-7] Audit Log & Vault Health ✅

| ID | Requirement | Priority | Status |
|---|---|---|---|
| AUD-01 | Hash-chained (SHA-256) log of ingestion, policy changes, model calls (allowed and refused), proposals and file changes | P0 | ✅ |
| AUD-02 | Integrity check reports the first broken entry | P0 | ✅ |
| AUD-03 | Vault health report: counts, unreadable reasons, OCR tools, meaning coverage, DB size, audit status; copyable with no document text | P0 | ✅ |
| AUD-04 | Every agent and workflow step is an audit entry with its inputs' identifiers, model, location and output size | P0 | R2 |
| AUD-05 | Export a matter's audit trail as a signed PDF or JSON for a client or regulator | P1 | R2 |
| AUD-06 | Record the redacted outbound text (or its hash) for every cloud call | P0 | R1 |

---

## [FS-8] Model Gateway (R1)

**Purpose:** one place in the Rust core through which every model call passes, so the policy cannot be bypassed and every call is audited. This closes the main gap listed in [`security.md`](../guides/security.md).

| ID | Requirement | Priority |
|---|---|---|
| GW-01 | All completions and embeddings, local and cloud, are made by the gateway; the UI and agents call the gateway, never a provider | P0 |
| GW-02 | Before each call, check the matter policy and router decision; refuse and audit if not allowed | P0 |
| GW-03 | Provider adapters: llama.cpp, MLX, OpenAI-compatible, Anthropic; streaming | P0 |
| GW-04 | Provider keys held in the OS keychain, never exposed to the web layer | P0 |
| GW-05 | Audit each call: matter, step, provider, model, location, tokens in/out, redaction applied, outcome | P0 |
| GW-06 | Per-matter and per-day spend limits for cloud calls | P1 |
| GW-07 | Timeouts, retries and fallback to local when a cloud call fails (if the policy allows the local model) | P1 |

**Edge cases:**
- **Policy changed mid-stream:** the current call completes, and the next call is checked again.
- **Provider unreachable:** fail with a clear message; no silent fallback to a different cloud provider.

---

## [FS-9] Redaction & Pseudonymisation (R1)

**Purpose:** let Hybrid matters use strong cloud models without revealing who or what the matter is about.

| ID | Requirement | Priority |
|---|---|---|
| RED-01 | Before any cloud call, replace parties, people, organisations, emails, phone numbers, account numbers and (optionally) amounts and dates with stable placeholders (`[PARTY_1]`, `[AMOUNT_3]`) | P0 |
| RED-02 | Entities come from the register (known parties) plus local named-entity detection | P0 |
| RED-03 | Restore placeholders in the response locally before display | P0 |
| RED-04 | Per-matter redaction level: off (Hybrid only, explicit), standard, strict | P1 |
| RED-05 | "Show what will be sent" preview for any cloud step | P1 |
| RED-06 | Measured recall on a labelled set; release gate ≥ 98% for parties and people | P0 |

**Edge cases:**
- **Questions that depend on a name** ("is Acme an affiliate of Beta?") work, because placeholders keep identity consistent within the call.
- **Redaction cannot hide everything:** citation text quoted in an answer is restored locally, never sent back to the provider.

---

## [FS-10] Model Router (R1)

**Purpose:** decide, for each step, which model runs: local where possible, cloud where needed and allowed.

| ID | Requirement | Priority |
|---|---|---|
| RTR-01 | Rules per matter: *Local only* forces local; *Hybrid* allows cloud per step type | P0 |
| RTR-02 | Step types: extract, classify, summarise, answer, draft, reason, compare; default routing per type (e.g. extract → local, draft → cloud in Hybrid) | P0 |
| RTR-03 | Firm-level defaults, overridable per matter and per workflow step | P1 |
| RTR-04 | Show the route for each step before a workflow runs, and in the run log | P1 |
| RTR-05 | Hardware-aware: pick the largest local model the machine runs at acceptable speed | P1 |

---

## [FS-11] Review Tables (R2)

**Purpose:** answer the same questions across every document in a matter, as a grid with each cell cited. This is the most-developed feature at Harvey and core at Legora, CoCounsel and Hebbia.

| ID | Requirement | Priority |
|---|---|---|
| TBL-01 | Create a table over a matter, a folder or a selection; rows = documents | P0 |
| TBL-02 | Add columns as questions ("Is there a change-of-control clause?", "Liability cap") with an answer type: text, yes/no, date, amount, list, classification | P0 |
| TBL-03 | Each cell answered from that document's passages only, with a citation and confidence | P0 |
| TBL-04 | Built-in register fields appear as ready-made columns | P0 |
| TBL-05 | **Conditional columns**: run only when another column meets a condition (cf. Harvey, May 2026) | P1 |
| TBL-06 | **Manual columns** for reviewer notes and status (cf. Harvey) | P1 |
| TBL-07 | **Comments** on cells; **inline edits** with the original kept and the edit audited (cf. Harvey) | P1 |
| TBL-08 | Filter, sort, group; flag cells needing review (low confidence, no citation) | P0 |
| TBL-09 | Export XLSX and CSV; issues list as DOCX | P0 |
| TBL-10 | Column templates: due diligence, leases, NDAs, employment, financing, invoices | P1 |
| TBL-11 | Scale to 10,000 rows with background processing, progress and resume | P1 |
| TBL-12 | Ask a question about the table ("which contracts have no cap?") | P2 |

**Edge cases:**
- **A document too long for one call:** the column is answered from the top-k passages; the cell says so.
- **Column edited after it has run:** cells are marked stale and can be re-run.

**Local/cloud:** each column is a routed step type (extract/classify → local by default).

---

## [FS-12] Workflows (R2)

**Purpose:** one-click, multi-step tasks with a fixed method and output format, so users get expert work without writing prompts.

| ID | Requirement | Priority |
|---|---|---|
| WF-01 | Library: **Summarise matter**, **Key-dates chronology**, **Compare two documents**, **Draft client memo**, **NDA triage**, **Due-diligence issues list**, **Lease abstract** | P0 |
| WF-02 | Each workflow declares inputs, steps, step types (for routing), and output (DOCX, XLSX, table) | P0 |
| WF-03 | Run log shows each step, its route and citations; outputs saved to the matter | P0 |
| WF-04 | **Optional steps** the user can skip (cf. Harvey) | P1 |
| WF-05 | Workflows can create or read Review Tables | P1 |
| WF-06 | Duplicate and edit a workflow (becomes a custom agent, FS-16) | P1 |

---

## [FS-13] Playbooks & Redlines (R2)

**Purpose:** check contracts against the firm's standard positions and propose redlines. This is Luminance's and Spellbook's core use.

| ID | Requirement | Priority |
|---|---|---|
| PB-01 | Playbook = list of positions: clause topic, preferred position, acceptable fallbacks, unacceptable terms | P0 |
| PB-02 | Run a playbook on one contract or a batch; result per position: compliant / deviation / missing, with citation | P0 |
| PB-03 | For each deviation, a suggested redline grounded in the playbook's preferred language | P0 |
| PB-04 | Export redlines as DOCX with tracked changes, and a deviation report | P0 |
| PB-05 | Starter playbooks: NDA (mutual / one-way), SaaS, services agreement, lease | P1 |
| PB-06 | Build a playbook from the firm's past executed contracts (proposed positions for approval) | P2 |

---

## [FS-14] Clause Library & Drafting (R2)

| ID | Requirement | Priority |
|---|---|---|
| DRF-01 | Save clauses from documents to a library with tags (topic, jurisdiction, favourable-to) | P0 |
| DRF-02 | Draft a clause or short document grounded in library clauses and matter facts, citing the clauses used | P0 |
| DRF-03 | Output to DOCX or PDF | P0 |
| DRF-04 | Suggest library clauses when a playbook deviation is found | P1 |

---

## [FS-15] Document Compare (R2)

| ID | Requirement | Priority |
|---|---|---|
| CMP-01 | Compare two versions: text diff and a plain-language summary of material changes, cited to both | P0 |
| CMP-02 | Compare a document against a template or playbook (uses FS-13) | P1 |
| CMP-03 | Export as DOCX with changes highlighted | P1 |

---

## [FS-16] Agent Builder (R3)

**Purpose:** firms build their own multi-step agents without code. Harvey and Legora both ship agent builders.

| ID | Requirement | Priority |
|---|---|---|
| AGT-01 | An agent = steps; each step has a type (for routing), instructions, tools and an output | P0 |
| AGT-02 | Tools: search matter, read register, create/update review table, run workflow, draft, propose file action, notify | P0 |
| AGT-03 | Agents can only **propose** file changes and outbound messages; a person approves | P0 |
| AGT-04 | Test-run on a sample matter with the full run log before publishing | P0 |
| AGT-05 | Share agents across the firm (Team tier / FS-20) | P1 |
| AGT-06 | Versioning; each run records the agent version | P1 |
| AGT-07 | Import and export agents as files | P2 |

---

## [FS-17] Monitors (R3)

**Purpose:** agents that run on a schedule or when something changes (cf. Legora Monitors, Harvey retention notifications).

| ID | Requirement | Priority |
|---|---|---|
| MON-01 | Triggers: schedule (daily, weekly), new files in a watched folder, a register date approaching | P0 |
| MON-02 | Built-in monitors: **deadline digest** (renewals, notices, expiries in the next N days), **new-document triage** (classify, extract, flag against playbook) | P0 |
| MON-03 | Results appear in an inbox per matter; optional OS notification | P0 |
| MON-04 | Runs when the app is open; on Firm Server, runs server-side | P1 |

---

## [FS-18] Knowledge Bases (R3)

| ID | Requirement | Priority |
|---|---|---|
| KB-01 | A knowledge base is a read-only collection (precedents, templates, internal guidance) usable across matters | P0 |
| KB-02 | Ask, tables, workflows and agents can @-mention a knowledge base | P0 |
| KB-03 | Knowledge bases never receive matter data | P0 |
| KB-04 | Shared firm-wide on Firm Server, with view-only access (cf. Harvey sandbox vaults) | P1 |

---

## [FS-19] Finance Pack (R3)

**Purpose:** the second wedge: finance and accounting teams.

| ID | Requirement | Priority |
|---|---|---|
| FIN-01 | Invoice register: vendor, invoice number, dates, amounts, tax, PO | P0 |
| FIN-02 | **Invoice-to-contract check**: price, escalation, payment terms vs the contract; exceptions table | P0 |
| FIN-03 | **Covenant tracker** for loan and facility agreements: covenants, test dates, thresholds; monitor reminders | P1 |
| FIN-04 | **KYC / onboarding file review** table template | P1 |
| FIN-05 | XLSX output designed for accounting import | P1 |

---

## [FS-20] Firm Server & Roles (R4)

| ID | Requirement | Priority |
|---|---|---|
| SRV-01 | Self-hosted server holding shared matters, knowledge bases and agents; desktop apps connect over the LAN or VPN | P0 |
| SRV-02 | Users, groups, roles (admin, member, viewer); matter access lists (ethical walls) | P0 |
| SRV-03 | SSO (SAML/OIDC) | P1 |
| SRV-04 | Central audit chain and firm-wide health report | P0 |
| SRV-05 | Server-side local models (GPU) shared by the firm | P1 |
| SRV-06 | Backups and restore | P0 |

---

## [FS-21] Word Add-in (R4)

| ID | Requirement | Priority |
|---|---|---|
| WORD-01 | Run a playbook on the open document; show deviations and insert redlines as tracked changes | P0 |
| WORD-02 | Ask about the open document and its matter; insert clauses from the library | P0 |
| WORD-03 | Talks to the local ALIV app or Firm Server; no separate cloud | P0 |

---

## [FS-22] Research Connectors (R4)

| ID | Requirement | Priority |
|---|---|---|
| RES-01 | Connector interface for external sources, results cited with source links | P0 |
| RES-02 | Free public sources first (e.g. court opinions, legislation, company filings) | P1 |
| RES-03 | Licensed sources through partners; ALIV does not host licensed content | P2 |
| RES-04 | Research queries never include matter text unless the matter is Hybrid and the user confirms | P0 |

---

## Cross-cutting requirements

| Area | Requirement |
|---|---|
| **Citations** | Every generated statement in Ask, tables, workflows and drafts links to a source passage, register entry or library clause. Uncited text is visibly marked |
| **Human approval** | File changes, redlines applied to documents, and outbound messages are proposals until approved |
| **Audit** | Every model call, agent step and change is a chain entry |
| **Performance** | Interactive actions answer in < 300 ms; model output starts streaming in < 3 s locally |
| **Accessibility** | WCAG 2.1 AA |
| **Data portability** | Matters export to an open format (JSON plus original files) |
