# UI Specification
## Screens, states and behaviour

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active (R0 screens shipped; R1–R3 screens specified)

---

## 1. Navigation

```
App shell (left sidebar from the base app)
├─ Matters                          ← ALIV home
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

## 2. Matters (home) ✅

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

## 6. Agent Builder (R3)

**Layout:**
- **Left:** a steps list (drag to reorder, add step).
- **Centre:** the StepEditor.
- **Right:** a test panel (pick a sample matter, run, RunLog).

**Validation:**
- every step has a type and an output;
- tools that change things are shown as "proposes only";
- the run is blocked if a step would need cloud in a Local only test matter (the reason is shown).

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
