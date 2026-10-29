# UI Specification
## Screens, states and behaviour

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active (R0 screens shipped; R1–R3 screens specified)

---

## 1. Navigation

```
App shell (left sidebar from the base app)
├─ Home                 R1–R3  (one view across all matters; see W-0)
├─ Matters                          ← ALIV home today
│   └─ Matter
│       ├─ Overview      ✅
│       ├─ Ask           ✅  (R2: @-mentions, save, suggestions)
│       ├─ Register      ✅
│       ├─ Tables        R2
│       ├─ Workflows     R2  (runs and outputs for this matter)
│       ├─ Documents     ✅
│       ├─ Approvals     ✅
│       ├─ Inbox         R3  (monitor results)
│       └─ Audit log     ✅
├─ Library               R2/R3: Workflows · Playbooks · Clauses · Agents · Knowledge bases
├─ Chat, Models, Settings (base app)
└─ Vault health (dialog from Matters) ✅
```

**Global elements:** the matter header shows the name, client, type icon, PolicyPill and a ⋯ menu (settings, delete). The **RoutePill** appears wherever a model runs.

---

## 2. Home (R1–R3) and Matters ✅

**Home** becomes the landing screen once there is work to track across matters ([mockup](../assets/mockups/home.png)):

| Element | Behaviour |
|---|---|
| Quick Ask | Matter picker plus the RoutePill for that matter; @-mentions (R2) |
| Counters | Deadlines in 30 days, waiting for you, running now, model calls this month with the share run locally |
| Upcoming deadlines | All matters, 90 days, DaysPill; Open jumps to the clause |
| Waiting for you | Approvals, inbox items (R3), cells needing review (R2) |
| Running | Tables and workflows with progress; click opens the run log |
| Sidebar | Pinned matters with their policy icon |

**Matters** (shipped) lists every matter and is the landing screen until Home ships.

### Matters ✅

| Element | Behaviour |
|---|---|
| Header | "Vault health" (ghost) and "New matter" (primary) |
| Upcoming deadlines strip | Across all matters, next 90 days, DaysPill each; click → the document's register row |
| Matter cards | Name, client, type, PolicyPill, documents count, pending approvals; click → matter |
| Empty | "Create your first matter", explaining that matters are private vaults |

**New matter dialog:** name (required), client, type (Legal / Finance), policy radio (Local only selected, Hybrid with an explanation). Save → matter Overview.

**Vault health dialog:** stats grid, audit and OCR pills, unreadable reasons, Copy report.

---

## 3. Matter tabs

### 3.1 Overview ✅
StatCards: documents indexed (with needs-attention hint), upcoming deadlines, pending approvals, last activity. Lists: next deadlines, documents needing attention (skipped/failed with reason). Quick ask box → Ask tab.

### 3.2 Ask ✅ (R2 additions)
**Layout:**
- **Left (3/5):** model picker, RoutePill (R1) or routing pill (R0), question box, turns newest first.
- **Right (2/5):** sticky Sources list.

**States per turn:**

| State | Shows |
|---|---|
| streaming | Partial text and a stop button |
| answered | Text with CitationChips, a model/provider/location line, search mode, "N left out" note |
| computed | Register-based answer, "no model used" note |
| no_model / no_sources | Explanation and next action |
| blocked | Policy refusal (danger) with "pick a local model or switch to Hybrid" |
| error | Message, retry |

**R2 additions:**
- `@` menu (documents, tables, knowledge bases);
- suggestion chips ("Create a review table for this");
- Save as DOCX/PDF;
- uncited-sentence marking;
- follow-ups keep sources.

### 3.3 Register ✅
Filters (all, due in 90 days, passed, needs review) with counts. The grid has document, type and the fields as columns. A cell click shows its source passage. Export CSV (XLSX in R2). R2: edit a value inline, shown as edited.

### 3.4 Tables (R2)
- **List:** the tables in the matter, with progress, rows and columns.
- **Table view:** see the [Wireframes](ALIV_Wireframes.md), W-5.

| Area | Behaviour |
|---|---|
| Toolbar | + Column, Template, Filter, Sort, Group, Run / Pause, Export (XLSX, CSV, DOCX issues), RoutePill for the table |
| Column header | Question, type icon, condition badge, menu (edit, re-run, hide, delete) |
| Cell | Value + CitationChip + ConfidenceMeter; state styles: pending (skeleton), running (spinner), not found (muted "—"), n/a (muted), edited (accent dot), stale (amber outline), error (danger icon), comments (count badge) |
| Cell click | SourceDrawer with the passage; edit, comment, re-run |
| Add column dialog | Question, answer type (text, yes/no, date, amount, list, classification + options), optional condition, preview on 3 rows before running all |

### 3.5 Workflows (R2)
- **Library picker:** cards with name, description, inputs, outputs, typical time, and route summary (e.g. "local extract · cloud draft").
- **Run setup:** inputs, optional steps (toggles), **Preview what leaves** (Hybrid).
- **Run view:** RunLog with live progress, then output preview (DOCX/XLSX) and Save.

### 3.6 Documents ✅
Table: name and summary, type, status (indexed / OCR % / skipped reason), passages, size, open. Header: counts and meaning-index coverage line. Add folder.

### 3.7 Approvals ✅
Proposals list with RiskPill and ActionStatusPill, "from → to", reason. Actions: approve, reject, "Apply N approved", undo. R3: proposals from agents show the agent name and run link.

### 3.8 Inbox (R3)
Monitor results grouped by monitor and date: deadline digest items (DaysPill + clause link), triaged documents (type, playbook result), errors. Mark done / snooze.

### 3.9 Audit log ✅
Entries: time, kind, summary, expandable detail. Verify integrity → success or danger pill naming the broken entry. R2: filter by kind or route; Export audit trail (PDF/JSON).

---

## 4. Library (R2–R3)

| Section | List | Editor |
|---|---|---|
| Workflows | Built-in and custom | Read-only for built-ins; "Duplicate as agent" |
| Playbooks | Name, contract type, positions count | Positions table: topic, preferred, fallbacks, unacceptable, sample language |
| Clauses | Search, tags, source | Clause text, tags, source link |
| Agents (R3) | Name, version, last run, shared | StepEditor list, test-run panel with RunLog, Publish |
| Knowledge bases (R3) | Name, documents, used-by | Add folder, read-only badge |

---

## 5. Hybrid controls (R1)

| Where | Control |
|---|---|
| Matter settings | Policy (Local only / Hybrid); redaction level (standard / strict / off, the last only with confirmation); monthly cloud limit; default routes per step type |
| Before any cloud step | RoutePill "Cloud · redacted" plus a **Preview** link opening the RedactionPreview |
| After a run | RunLog lists each step's route, model and redaction |
| Settings → AI routing (firm defaults) | Step type × route table; hardware tier detected; local model per tier |

---

## 5b. Admin interface

Admins set policy and see metadata (counts, routes, costs, audit entries), **never document text**.

### Desktop: Settings › Admin (R1) · [mockup](../assets/mockups/admin-desktop.png) · W-13

| Page | Contents | Feature IDs |
|---|---|---|
| AI policy | Defaults for new matters (policy, redaction level), allowed cloud providers, routing by step type, hardware tier and local model | MAT-02, RED-04, RTR-02/03/05 |
| Spend | Firm and per-matter monthly cloud limits; usage by matter; calls refused at a limit | GW-06 |
| Audit | Verify all matters; export a matter's trail | AUD-02/05 |
| Health | Vault health, OCR tools, model status, copy report | AUD-03 |

Settings pushed by the firm's policy file (MDM) are greyed out and labelled "locked by firm". A matter can be stricter than the defaults, never looser.

### Firm Server: Admin Console (R4) · [mockup](../assets/mockups/admin-console.png) · W-14

| Page | Contents | Feature IDs |
|---|---|---|
| Dashboard | Active users, matters by policy, model calls by route, refusals, spend, recent policy events, GPU server, backups | SRV-04 |
| Users & groups | Invite, remove, roles (admin, member, viewer), SSO group mapping | SRV-02/03 |
| Matters & walls | Access lists per matter (ethical walls), ownership, retention dates | MAT-06/07 |
| AI policy · Providers · Spend | As on the desktop, applied firm-wide; server GPU models | RTR-03, SRV-05, GW-06 |
| Audit | Search across matters, export a matter's trail, chain status | SRV-04, AUD-05 |
| Health & backups · Updates · SSO | Backup schedule and restore, release channel and mirror, identity provider | SRV-06, SRV-03 |

---

## 6. Agent Builder (R3)

**Layout:**
- **Left:** a steps list (drag to reorder, add step).
- **Centre:** the StepEditor.
- **Right:** a test panel (pick a sample matter, run, RunLog).

**Validation:**
- every step has a type and an output;
- tools that change things are shown as "proposes only";
- in a Local only test matter, cloud steps run on this computer instead, and a notice says so (the router never sends a Local only matter to the cloud); a run is blocked only if a step is pinned to a cloud model, with the reason shown.

**Publish** creates a version and requires a change note.

---

## 7. Responsiveness and windows

- The desktop window is at least 1024 × 700. Panels stack below 1024 px (Ask: sources move under the answer).
- The Review grid is virtualised and keeps a sticky first column. Horizontal scroll is fine there and nowhere else.
- The SourceDrawer is 420 px wide and takes the full width below 900 px.

---

## 8. Performance budgets

| Interaction | Budget |
|---|---|
| Tab switch | < 150 ms |
| Search / filter in grids up to 10k rows | < 300 ms |
| Open SourceDrawer | < 200 ms |
| First streamed token (local 7–8B) | < 3 s |
| Table cell update rendering | Batched; ≤ 1 render per 250 ms during runs |
