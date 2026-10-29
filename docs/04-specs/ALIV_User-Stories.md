# User Stories
## Stories and acceptance criteria by epic

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active

---

## Overview

Stories are grouped by epic, which maps to the [Feature Spec](ALIV_Feature-Spec.md) modules. Each story has acceptance criteria in Given/When/Then form. **Release:** R0 is shipped; R1–R4 are planned ([Implementation Phases](../07-implementation/ALIV_Implementation-Phases.md)). **Priority:** P0 must ship in its release, P1 should, P2 may slip.

| Epic | Module(s) | Stories |
|---|---|---|
| E1 Matters & policy | FS-1 | US-101–104 |
| E2 Documents in | FS-2 | US-201–205 |
| E3 Find & ask | FS-3, FS-5 | US-301–307 |
| E4 Register | FS-4 | US-401–404 |
| E5 Control & audit | FS-6, FS-7 | US-501–505 |
| E6 Hybrid AI | FS-8–10 | US-601–606 |
| E7 Review tables | FS-11 | US-701–708 |
| E8 Workflows & drafting | FS-12–15 | US-801–807 |
| E9 Agents & monitors | FS-16–18 | US-901–905 |
| E10 Finance | FS-19 | US-1001–1003 |
| E11 Firm scale | FS-20–22 | US-1101–1104 |

---

## E1 Matters & policy

**US-101 · Create a private matter** (R0, P0) ✅
*As a lawyer, I want each client engagement in its own matter, so that documents and answers never mix across clients.*
- Given I create a matter, when I add documents and ask questions, then search and answers use only that matter's documents.
- Given I delete a matter, then its index is removed, my files on disk are untouched, and the audit log keeps a record of the deletion.

**US-102 · Local only by default** (R0, P0) ✅
*As a risk-conscious firm, I want new matters to keep data on this computer, so that no one sends client data to the cloud by accident.*
- Given a new matter, then its policy is Local only.
- Given a Local only matter and a cloud model selected, when I ask, then the call is refused, the refusal is logged, and I'm told how to proceed.

**US-103 · Allow cloud per matter** (R0, P0) ✅
*As a partner, I want to allow cloud models on matters where the client permits it, so that I get stronger answers there.*
- Given I switch a matter to Hybrid, then the change is audited with the old and new value, and Ask shows "Cloud · matching passages only, logged".

**US-104 · Ethical walls** (R4, P1)
*As legal ops, I want to restrict a matter to named users, so that conflicted lawyers can't see it.*
- Given a matter restricted to group A, when a user outside A searches, then the matter does not appear and any access attempt is logged.

## E2 Documents in

**US-201 · Add a folder** (R0, P0) ✅
- Given I add a folder, then supported files are indexed, unreadable ones are listed with a reason, and the toast summarises indexed / unchanged / skipped / OCR counts.

**US-202 · Read scans** (R0, P0) ✅
- Given Tesseract is installed and I add scanned PDFs and photos, then they're indexed with an OCR % badge, and low-confidence values go to Needs review.

**US-203 · Re-add only reads changes** (R0, P0) ✅
- Given I re-add a folder, then unchanged files are skipped, and changed files are re-read with their embeddings re-queued.

**US-204 · Email archives** (R2, P1)
*As a litigator, I want to add a .pst mailbox, so that emails and attachments become searchable with their threads.*
- Given a .pst, then each message and attachment is a document with sender, recipients, date and thread id; attachments link to their message.

**US-205 · Ten thousand documents** (R2, P1)
- Given a 10,000-document folder, then ingestion shows progress, can pause and resume, and survives an app restart without re-reading finished files.

## E3 Find & ask

**US-301 · Cited answers** (R0, P0) ✅
- Given I ask a question, then every statement in the answer has a [n] citation, and clicking it shows the source passage.

**US-302 · Find by meaning** (R0, P0) ✅
- Given the local embedding model is available, when I ask "can we exit early?", then a "termination for convenience" clause is among the sources, and the answer says it searched by keyword and meaning.

**US-303 · Big matters still work** (R0, P0) ✅
- Given more matching sources than the model can read, then the most relevant are sent and the answer says how many were left out.

**US-304 · @-mention** (R2, P1)
- Given I type @ in Ask, then I can pick documents, review tables or knowledge bases, and the answer uses only those.

**US-305 · Suggest the right tool** (R2, P1)
- Given I ask "list the governing law of every contract", then ALIV offers to create a review table with that column.

**US-306 · Save an answer** (R2, P1)
- Given an answer, when I choose Save as DOCX, then the file has the answer with citations as footnotes.

**US-307 · Spot unsupported claims** (R2, P1)
- Given the model writes a sentence with no citation, then that sentence is visibly marked "uncited".

## E4 Register

**US-401 · Key terms at a glance** (R0, P0) ✅
- Given indexed contracts, then the register shows type, parties, dates, renewal deadline, amount, notice period, governing law and penalty, each with confidence and a source link.

**US-402 · What's due** (R0, P0) ✅
- Given the register, when I filter "Due in 90 days", then only documents with a deadline in that window are listed, soonest first.

**US-403 · Correct a value** (R2, P1)
- Given an extracted date is wrong, when I edit it, then the correction is kept across re-ingestion, shown as edited, and audited.

**US-404 · XLSX export** (R2, P1)
- Given the register, then XLSX export keeps types (dates as dates, amounts as numbers).

## E5 Control & audit

**US-501 · Approve before anything changes** (R0, P0) ✅
- Given proposed renames, then nothing on disk changes until I approve and apply; applying never overwrites; each can be undone.

**US-502 · Tamper-evident log** (R0, P0) ✅
- Given the audit log, when I run Verify integrity, then it confirms the chain or names the first broken entry.

**US-503 · Health for support** (R0, P0) ✅
- Given Vault health, when I copy the report, then it contains counts and reasons and no document text.

**US-504 · Export a matter's audit** (R2, P1)
- Given a matter, when I export its audit trail, then I get a PDF and JSON listing every model call (location, model, redaction level, sizes), with no document text.

**US-505 · Agents only propose** (R3, P0)
- Given an agent run that wants to rename files or send a message, then those appear in Approvals and are never applied automatically.

## E6 Hybrid AI

**US-601 · One gateway** (R1, P0)
*As a security reviewer, I want every model call to pass through one audited component, so that the policy can't be bypassed.*
- Given any feature calls a model, then the call is made by the gateway and audited; a CI check fails if web code contains provider keys or provider URLs.

**US-602 · Redact before cloud** (R1, P0)
- Given a Hybrid matter and a cloud step, then party, person, email, phone and account identifiers are replaced by placeholders before sending, and restored in the result shown to me.

**US-603 · Preview what leaves** (R1, P1)
- Given a cloud step, when I choose Preview, then I see the exact redacted text that will be sent.

**US-604 · Right model per step** (R1, P0)
- Given a Hybrid matter, then extraction runs locally and drafting may use the cloud, and the route of each step is shown before and after it runs.

**US-605 · Never cloud for Local only** (R1, P0)
- Given a Local only matter, then no step type, workflow or agent can route to a cloud model; attempts are refused and logged.

**US-606 · Spend limits** (R1, P1)
- Given a monthly cloud limit on a matter, when it's reached, then further cloud steps are refused with a clear message and logged.

## E7 Review tables

**US-701 · Table from a matter** (R2, P0)
- Given a matter, when I create a review table, then each document is a row and register fields are available as ready-made columns.

**US-702 · Ask a column** (R2, P0)
- Given I add a column "Liability cap" of type amount, then each cell is filled from that document only, with a citation and confidence; cells without an answer say "not found" rather than guess.

**US-703 · Conditional column** (R2, P1)
- Given a column that runs only when "Change of control = yes", then other rows show "not applicable" and aren't sent to a model.

**US-704 · Review and edit** (R2, P1)
- Given a cell, when I edit it, then the original is kept, the edit is shown and audited; when I comment, teammates see it.

**US-705 · Needs review** (R2, P0)
- Given low-confidence or uncited cells, then the "needs review" filter shows exactly those.

**US-706 · Templates** (R2, P1)
- Given the template "M&A due diligence", then the standard columns are created and can be edited.

**US-707 · Export** (R2, P0)
- Given a table, then XLSX export includes values, citations (document and page/passage) and review status.

**US-708 · Big tables** (R2, P1)
- Given 10,000 rows, then cells fill in the background with progress, and a restart resumes without redoing finished cells.

## E8 Workflows & drafting

**US-801 · One-click workflows** (R2, P0)
- Given the library, when I run "Summarise matter", then I get a cited summary saved to the matter, with a run log of steps and routes.

**US-802 · Chronology** (R2, P0)
- Given a production, then "Key-dates chronology" lists dated events in order, each cited and deduplicated.

**US-803 · Compare versions** (R2, P0)
- Given two versions, then I get a diff and a plain-language summary of material changes, cited to both.

**US-804 · Playbook check** (R2, P0)
- Given a playbook and a contract, then each position is compliant / deviation / missing with a citation and, for deviations, a suggested redline.

**US-805 · Redlines in Word format** (R2, P0)
- Given accepted redlines, then the DOCX export opens in Word with tracked changes.

**US-806 · Clause library** (R2, P0)
- Given a clause in a document, when I save it to the library with tags, then drafting can use and cite it.

**US-807 · Optional steps** (R2, P1)
- Given a workflow with optional steps, then I can skip them before running, and the run log shows them as skipped.

## E9 Agents & monitors

**US-901 · Build an agent** (R3, P0)
- Given the Agent Builder, then I can add steps with type, instructions, tools and output, test-run on a sample, and publish a version.

**US-902 · Deadline digest** (R3, P0)
- Given the digest monitor is on weekly, then each week the inbox lists deadlines in the next N days, cited to the clauses.

**US-903 · New-document triage** (R3, P0)
- Given a watched folder, when a file arrives, then it's ingested, classified, checked against the chosen playbook and placed in the inbox.

**US-904 · Firm knowledge** (R3, P0)
- Given a precedents knowledge base, when I @-mention it, then answers cite it, and no matter data is ever written into it.

**US-905 · Share agents** (R3, P1)
- Given a published agent, then teammates on the Team tier can run it; runs record the agent version.

## E10 Finance

**US-1001 · Invoice register** (R3, P0)
- Given invoices, then vendor, invoice number, dates, amounts, tax and PO are extracted, with ≥ 95% field accuracy on the labelled set.

**US-1002 · Invoice vs contract** (R3, P0)
- Given invoices and contracts, then the exceptions table lists mismatches (price, escalation, terms, missing contract), each citing both documents.

**US-1003 · Covenant tracker** (R3, P1)
- Given loan agreements, then covenants, thresholds and test dates are tracked, with reminders before each test date.

## E11 Firm scale

**US-1101 · Shared vault** (R4, P0)
- Given a Firm Server, then permitted users open the same matter from their desktops, with one audit chain.

**US-1102 · Roles and SSO** (R4, P0)
- Given SSO, then users sign in with the firm's identity provider, and roles (admin, member, viewer) limit what they can do.

**US-1103 · Word add-in** (R4, P0)
- Given a contract open in Word, when I run a playbook, then deviations appear in a side panel and accepted redlines are inserted as tracked changes.

**US-1104 · Public research** (R4, P1)
- Given a research connector, when I search, then results are cited with source links, and no matter text is sent unless the matter is Hybrid and I confirm.
