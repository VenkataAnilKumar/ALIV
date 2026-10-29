# User Journeys
## How each persona gets work done in ALIV

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active

---

## Overview

These journeys follow the personas in the [Product Definition](../02-product/ALIV_Product-Definition.md) through their real tasks, end to end. Each step names the feature it relies on ([Feature Spec](../04-specs/ALIV_Feature-Spec.md) IDs), and whether that feature is **shipped (R0)** or **planned (R1–R4)**. The journeys are drawn from the same jobs competitors are built around: due diligence (Harvey Vault, CoCounsel bulk review), contract monitoring (Legora Monitors), playbook review (Luminance, Spellbook) and research. Each is done ALIV's way: local by default, cloud by policy, every step cited and audited.

| # | Journey | Persona | Release |
|---|---|---|---|
| J1 | First matter: from install to a cited answer | Any | R0 |
| J2 | Due diligence on a data room | Priya, M&A associate | R2 |
| J3 | Contract renewals and obligations | Daniel, in-house counsel | R0 → R3 |
| J4 | NDA triage against the firm playbook | Daniel / Priya | R2 |
| J5 | Litigation chronology from a production | Maria, litigation partner | R2 |
| J6 | Invoice-to-contract reconciliation | Sam, finance controller | R3 |
| J7 | Hybrid drafting with redaction | Priya | R1 → R2 |
| J8 | Building a firm agent | Lena, legal ops | R3 |
| J9 | Answering a client's AI-use audit | Lena | R0 → R2 |
| J10 | Data room on autopilot: connector + skills + automation | Priya | R3 |
| J11 | Governing skills, MCP and web search firm-wide | Lena | R1 → R4 |

---

## J1: First matter, from install to a cited answer (R0)

**Goal:** a new user gets a trustworthy, cited answer within 10 minutes of installing.

| Step | User does | ALIV does | Feature |
|---|---|---|---|
| 1 | Installs and opens ALIV | Checks hardware; recommends a local model and offers to download it | Setup (GA track) |
| 2 | Downloads the recommended model | Shows progress; the local embedding model is set up too | Models |
| 3 | Matters → **New matter**: "Acme lease review", client, Legal, **Local only** | Creates the vault; the audit log records it | MAT-01/02 ✅ |
| 4 | **Add folder** → picks the lease folder | Parses, OCRs scans, extracts the register, writes summaries; toast: "42 indexed (6 read by OCR)" | ING-01–05 ✅ |
| 5 | Opens **Overview** | Upcoming deadlines, documents needing review | REG-03 ✅ |
| 6 | **Ask**: "Which leases renew in the next 90 days and what notice is required?" | Finds passages by keyword and meaning; answers with [n] citations; pill shows "Runs on this computer" | ASK-01–05 ✅ |
| 7 | Clicks [2] | Opens the exact notice clause | ASK-02 ✅ |

**Moments that matter:**
- **Step 4:** every file is accounted for, and unreadable ones show why. This builds trust.
- **Step 7:** the citation lands on the right clause. This is the "aha" moment.

**Failure paths:**
- **No model yet:** deadline questions are answered from the register, labelled "computed, no model used".
- **Tesseract missing:** scans are listed with "install Tesseract OCR to read it".

---

## J2: Due diligence on a data room (R2)

**Persona:** Priya, M&A associate. **Goal:** a cited issues list for 300 target-company contracts in two days, not two weeks.

| Step | User does | ALIV does | Feature |
|---|---|---|---|
| 1 | Creates matter "Project Falcon", **Hybrid**, redaction *standard* | Policy and redaction level audited | MAT-02, RED-04 |
| 2 | Adds the data-room export (PDFs, scans, a .pst of key emails), or links the SharePoint data room (R3) | Ingests 300 documents and 2,000 emails; resumable progress | ING-01–07, CON-01/02 |
| 3 | **New review table** from template *M&A due diligence* | Columns: change of control, assignment, exclusivity, termination for convenience, liability cap, governing law, MFN | TBL-01/10 |
| 4 | Adds a custom column: "Does the customer have audit rights? (yes/no)" | Column typed yes/no; cells fill in the background, each cited | TBL-02/03/11 |
| 5 | Adds a **conditional column**: "If change of control = yes, what consent is required?" | Runs only on matching rows | TBL-05 |
| 6 | Filters "needs review"; opens low-confidence cells; edits two; comments on one for the partner | Edits keep the original value and are audited; comments are visible to the team | TBL-07/08 |
| 7 | Runs workflow **DD issues list** on the table, with the firm's *DD checklist* skill | Groups issues by severity and drafts a DOCX issues list with citations, saved as an artifact | WF-01/05, SKL-01, ART-01 |
| 8 | Exports the table to XLSX and the issues list to DOCX | Files saved to the matter | TBL-09 |

**Routing (Hybrid):**
- **Extraction columns:** run locally.
- **Issues-list drafting:** goes to a cloud model after redaction (parties appear as `[PARTY_n]`).
- **Run log:** shows the route of every step.

**Success:**
- ≥ 95% of cells correct on spot-check;
- issues list ready for partner review on day 2.

---

## J3: Contract renewals and obligations (R0 → R3)

**Persona:** Daniel, sole in-house counsel. **Goal:** never miss a renewal or notice deadline.

| Step | User does | ALIV does | Feature |
|---|---|---|---|
| 1 | Creates "Vendor contracts", **Local only** (bank partner forbids cloud AI) | | MAT-02 ✅ |
| 2 | Adds the shared contracts folder | Register: parties, expiry, renewal-notice deadline, notice period | REG-01/02 ✅ |
| 3 | Register → **Due in 90 days** | List of upcoming deadlines, each linked to its clause | REG-03 ✅ |
| 4 | Exports CSV for the finance team | Formula-safe CSV | REG-04 ✅ |
| 5 | (R3) Turns on the **Deadline digest** monitor, weekly on Monday | Each Monday: an inbox item listing deadlines in the next 60 days | MON-01/02 |
| 6 | (R3) Turns on **watch folder** | New contracts are ingested and appear in the next digest | ING-08, MON-01 |

**Guarantee:** the audit log shows **zero cloud calls** for this matter, which is the evidence for the bank partner.

---

## J4: NDA triage against the firm playbook (R2)

**Persona:** Daniel (in-house) or Priya (firm). **Goal:** turn around inbound NDAs in minutes.

| Step | User does | ALIV does | Feature |
|---|---|---|---|
| 1 | Drops the counterparty NDA into the matter | Ingested and classified as NDA | ING-01, REG-01 |
| 2 | Runs **Playbook: Mutual NDA** | Per position: compliant / deviation / missing, cited (term, definition of confidential information, residuals, non-solicit, governing law) | PB-01/02 |
| 3 | Reviews deviations; accepts two suggested redlines and edits one | Redlines grounded in the playbook's preferred wording | PB-03 |
| 4 | **Export DOCX** with tracked changes | Ready to send to the counterparty | PB-04 |
| 5 | (R3) Turns on **New-document triage** for the NDA inbox folder | Each new NDA is pre-checked and waits in the inbox | MON-02 |

**Batch variant:** run the playbook on 50 legacy NDAs. ALIV produces a review table of deviations per NDA (TBL + PB).

---

## J5: Litigation chronology from a production (R2)

**Persona:** Maria, litigation partner. **Goal:** a cited chronology of events from 5,000 produced documents.

| Step | User does | ALIV does | Feature |
|---|---|---|---|
| 1 | Matter "Smith v. Northwind", **Local only** | | MAT-02 |
| 2 | Adds the production (scanned PDFs, emails) | OCR; email threads kept | ING-03/06/07 |
| 3 | Runs workflow **Key-dates chronology** with the date range and parties | Extracts dated events locally, deduplicates, orders, cites each event | WF-01 |
| 4 | Asks "When did Northwind first learn of the defect?" | Cited answer from emails and memos | ASK-01 |
| 5 | Exports the chronology to DOCX and XLSX | | WF-02 |

**Routing:** all local (the policy). The chronology uses the largest local model the workstation supports.

---

## J6: Invoice-to-contract reconciliation (R3)

**Persona:** Sam, finance controller. **Goal:** catch invoices that don't match contract terms before paying them.

| Step | User does | ALIV does | Feature |
|---|---|---|---|
| 1 | Matter "AP Q4", Finance, **Local only** | | MAT-01 |
| 2 | Adds the supplier contracts folder and the Q4 invoices folder | Contract register and invoice register | FIN-01 |
| 3 | Runs **Invoice-to-contract check** | Exceptions table: price above rate card, missed escalation cap, wrong payment terms, no contract found. Each row cites the invoice and the contract | FIN-02 |
| 4 | Reviews exceptions; adds a manual column "Action" | | TBL-06 |
| 5 | Exports XLSX for the AP system | | FIN-05 |

---

## J7: Hybrid drafting with redaction (R1 → R2)

**Persona:** Priya. **Goal:** use a strong cloud model for a first-draft memo without exposing the client.

| Step | User does | ALIV does | Feature |
|---|---|---|---|
| 1 | In a **Hybrid** matter, runs **Draft client memo** | The router plans: research and extraction local, drafting cloud | RTR-02/04 |
| 2 | Clicks **Preview what will be sent** | Shows the redacted prompt: `[PARTY_1] acquires [PARTY_2] for [AMOUNT_1]…` | RED-05 |
| 3 | Confirms | The gateway sends the redacted text and records its hash in the audit log | GW-01/05, AUD-06 |
| 4 | Reads the draft | Placeholders restored locally; each paragraph cites sources | RED-03 |
| 5 | Saves as DOCX | | DRF-03 |

**If the matter were Local only:** step 1 routes drafting locally, and the preview says "nothing leaves this computer".

---

## J8: Building a firm agent (R3)

**Persona:** Lena, legal ops. **Goal:** automate the firm's lease-abstract process.

| Step | User does | ALIV does | Feature |
|---|---|---|---|
| 1 | **Agent Builder → New agent** "Lease abstract v1" | | AGT-01 |
| 2 | Adds steps: (1) review table with lease columns, (2) flag deviations against the *Lease* playbook, (3) draft a 1-page abstract, (4) propose renaming the file | Each step has a type, so routing is automatic; step 4 only *proposes* | AGT-01–03 |
| 3 | Test-runs on a sample matter | Full run log with routes and citations | AGT-04 |
| 4 | Publishes; shares with the real-estate group | Versioned; each run records its version | AGT-05/06 |

---

## J9: Answering a client's AI-use audit (R0 → R2)

**Persona:** Lena. **Goal:** prove to a client what AI did with their documents.

| Step | User does | ALIV does | Feature |
|---|---|---|---|
| 1 | Opens the client's matter → **Audit log** | Every ingestion, model call (allowed and refused), route and file change | AUD-01 ✅ |
| 2 | **Verify integrity** | Confirms the chain is unbroken | AUD-02 ✅ |
| 3 | (R2) **Export audit trail** | Signed PDF and JSON: calls by location, models, redaction level, no document text | AUD-05 |
| 4 | Attaches the matter's policy history | Policy changes are in the log | MAT-03 ✅ |

---

## J10: Data room on autopilot (R3)

**Persona:** Priya. **Goal:** keep a live diligence picture while the seller keeps uploading.

| Step | User does | ALIV does | Feature |
|---|---|---|---|
| 1 | Links the seller's SharePoint data room to "Project Falcon" | Read-only sync; documents indexed locally; counts audited | CON-01–03, WS-03 |
| 2 | Enables the *DD checklist* and *House memo style* skills on the matter | Skills reviewed and recorded with versions | SKL-01–05 |
| 3 | Creates an automation: "every morning, triage new uploads into the DD table" | Trigger + agent; proposals only | AUTO-01–03 |
| 4 | Next morning, opens Inbox | New documents classified, table rows filled and cited, deviations flagged | AUTO-03, TBL-03 |
| 5 | Approves logging 0.5 h to the billing system | MCP tool call runs after approval; audited | MCP-04 |
| 6 | Re-runs the issues memo | New artifact version; diff against yesterday's | ART-03 |

---

## J11: Governing skills, MCP and web search (R1 → R4)

**Persona:** Lena, legal ops. **Goal:** let lawyers use skills and tools without opening a side door.

| Step | User does | ALIV does | Feature |
|---|---|---|---|
| 1 | Admin › AI policy: web search allowed on Hybrid matters only | Local only matters refuse web search; Hybrid shows the redacted query | SRCH-10 |
| 2 | Admin › MCP: allows the firm's billing server (*firm* trust); blocks third-party servers | Matters can only allow listed servers | MCP-01/02/07 |
| 3 | Publishes *House memo style v4* to the firm skill library | Versioned; lawyers can enable it per matter | SKL-07 |
| 4 | Reviews the audit: which skills and tools shaped last month's client memos | Each artifact lists skills, versions, tools and routes | ART-02, SKL-05 |
| 5 | (R4) Admin Console: shares skills and allowed MCP servers firm-wide | Central policy, central audit | SKL-09, SRV-04 |

---

## Journey-level requirements

- **Show where work runs, every time:** "on this computer" or "cloud, redacted".
- **Make every output traceable:** a citation, or a visible "uncited" marker.
- **Never surprise the user:** file changes, redlines and messages are proposals until approved.
- **No side doors:** web search, MCP tools and skill scripts follow the same policy, redaction and audit as model calls.
- **Make long jobs resumable:** 10k-document tables, large ingestions and agent runs survive restarts.
