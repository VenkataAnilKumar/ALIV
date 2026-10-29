# ALIV — Interactive Demo Guide

> **Note:** this walkthrough was written for the legacy ALIV app at the repository root, now retired. The sample corpus below also powers the new desktop app: run `yarn dev:web` in `apps/desktop` and open `/matters/` for the Matters demo. An updated walkthrough will replace this guide.

> Run a complete buyer-facing or technical demo in under 8 minutes using the included sample corpus.

---

## What's in This Demo Folder

```
demo/
├── DEMO_GUIDE.md          ← you are here
└── corpus/                ← drop this folder into ALIV as your workspace
    ├── lease_acme_2025.txt              Lease agreement — expiry, penalty, renewal clause
    ├── nda_axiom_legal_2025.txt         NDA — confidentiality, $50k penalty, expiry
    ├── invoice_harbourview_2026.txt     Invoice — vendor, amount, SLA penalty, due date
    ├── service_agreement_delta_2025.txt Service agreement — penalty clause, IP terms
    └── email_renewal_notice_2025.txt    Email — renewal deadline, holdover warning
```

These 5 documents are designed to produce realistic, citation-backed results across all ALIV workflows.

---

## Pre-Demo Setup (2 minutes)

```bash
# 1. Clone and install
git clone https://github.com/VenkataAnilKumar/ALIV.git && cd ALIV
pip install -r python/requirements.txt
npm install

# 2. Start Ollama with the required model
ollama serve
ollama pull llama3.2:3b     # ~2 GB download, one-time

# 3. Launch ALIV
cargo tauri dev
```

On first launch, the onboarding modal will guide you through workspace registration.
Point the workspace at `ALIV/demo/corpus/` (use the absolute path on your machine).

---

## Stage 1 — Register Workspace (30 seconds)

**Screen:** Workspace Control

1. Enter the absolute path to `demo/corpus/` in the Workspace path field
2. Click **Add** then **Save workspace**
3. Confirm: Runtime shows `Workspaces: 1`
4. Verify: Ollama badge turns **green** in the top bar

**What to say:**
> ALIV never sends files to a cloud API. The workspace path stays on your machine.
> Ollama runs the LLM locally — no API keys, no data exfiltration.

---

## Stage 2 — Ingest the Corpus (60 seconds)

**Screen:** Ingestion

1. Click **Incremental ingest**
2. Watch the telemetry update: Discovered → Parsed → Chunks → Embeddings
3. Expected result after ingest:
   - Files discovered: **5**
   - Files parsed: **5**
   - Parser confidence avg: **≥ 90%**

**What to say:**
> ALIV parsed, chunked, and embedded 5 documents in seconds.
> On a real matter folder with 500 files, this runs incrementally — only new or changed files are re-processed.

---

## Stage 3 — Query with Citations (90 seconds)

**Screen:** Query & Assemble

Run each query and highlight the cited source + page reference in the result.

### Query A — Expiring agreements
```
Which agreements are expiring in 2026 and what are the renewal deadlines?
```
**Expected:** Lease (March 1, 2026, renewal by Jan 1), NDA (July 15, 2026), Service Agreement (Dec 31, 2025, 90-day notice). Sources cited.

### Query B — Penalty clauses
```
List every penalty clause across all documents with the dollar amounts.
```
**Expected:**
| Document | Penalty |
|---|---|
| Lease — Acme | $2,800/day holdover |
| NDA — Axiom Legal | $50,000 per breach |
| Invoice — Harbourview | 10% monthly credit for SLA breach |
| Service Agreement — Delta | $5,000/day delivery delay |

### Query C — Vendor lookup
```
Who are the vendors named in these documents?
```
**Expected:** Acme Commercial Properties, Axiom Legal LLP, Harbourview IT Services, Delta Advisory Group

**What to say:**
> Every answer is grounded in evidence. The source document and chunk are cited.
> If the LLM hallucinates, the citation breaks — that's the check.

---

## Stage 4 — Structured Extraction (60 seconds)

**Screen:** Extraction

1. Set template to **Lease summary**
2. Enter instruction:
   ```
   Extract the vendor name, client name, effective date, expiry date, renewal window, and penalty clause from the lease agreement.
   ```
3. Click **Run extraction**

**Expected output:**
| Field | Value | Confidence |
|---|---|---|
| vendor_name | Acme Commercial Properties LLC | ~95% |
| client_name | Redstone Enterprises Inc. | ~95% |
| effective_date | 2025-03-01 | ~97% |
| expiry_date | 2026-03-01 | ~97% |
| renewal_window | 60 days written notice by Jan 1, 2026 | ~90% |
| penalty_clause | $2,800/day holdover after expiry | ~93% |

4. Click **Export CSV** — show the downloaded file

**What to say:**
> This is structured output with confidence scoring. You can pipe this directly into a spreadsheet, a CRM, or a compliance dashboard.

---

## Stage 5 — Safe File Actions (90 seconds)

**Screen:** Actions Preview

1. Click **Generate preview**
2. The table shows rename recommendations in the format `{DocType}_{Vendor}_{Date}`
3. Click **Re-evaluate risk** — risk levels appear (low/medium/high)
4. Click **Select low-risk** — high-risk rows stay unchecked
5. Click **Approve selection**
6. Click **Execute approved** → confirm in the modal

**Expected:** Undo banner appears with batch ID and success count.

**Screen:** Actions Audit

7. Open the Audit screen — all executed renames logged with status
8. Click **Undo** on the batch — all renames reversed

**What to say:**
> No file moves without an approval gate. Every action is logged.
> If something goes wrong, one click reverses the entire batch.

---

## Stage 6 — Memory Persistence (30 seconds)

**Screen:** Memory

1. Enter a finding:
   ```
   Lease with Acme expires March 1 2026. Renewal deadline is January 1 2026. Holdover penalty $2,800/day.
   ```
2. Set scope to **Permanent**
3. Click **Save fact**
4. Close and reopen ALIV → the fact is still there

**What to say:**
> ALIV remembers what matters across sessions.
> This is scoped memory — session, week, or permanent — with token-budget assembly so the context stays within model limits.

---

## Stage 7 — Diagnostics (30 seconds)

**Screen:** Diagnostics

1. Click **Generate Report**
2. Show: DB stats (files, chunks, embeddings), platform info, version
3. Click **Export to file** → `aliv-diagnostics.json`

**What to say:**
> When something goes wrong, your team can export this JSON and attach it to a support ticket.
> Logs are written locally to the platform app-data directory — no telemetry is phoned home.

---

## Demo Timing Summary

| Stage | Screen | Time |
|---|---|---|
| Setup | — | 2 min (pre-demo) |
| 1. Register workspace | Workspace | 30 sec |
| 2. Ingest | Ingestion | 60 sec |
| 3. Query | Query | 90 sec |
| 4. Extract | Extraction | 60 sec |
| 5. Safe actions + undo | Actions Preview + Audit | 90 sec |
| 6. Memory | Memory | 30 sec |
| 7. Diagnostics | Diagnostics | 30 sec |
| **Total** | | **~8 min** |

---

## Objection Handling

| Objection | Response |
|---|---|
| "Can it handle PDFs with scanned images?" | Yes — Tesseract OCR fallback is wired into the ingestion pipeline. See `python/workers/`. |
| "What if Ollama is slow on our hardware?" | llama3.2:3b runs on 8 GB RAM. Upgrade to mistral:7b for better quality with 16 GB. Gemma 4 support is on the roadmap. |
| "How does it handle 10,000 files?" | Incremental indexing processes only changed files. sqlite-vec ANN search scales without full re-scan. |
| "What about Word and email files?" | .docx via python-docx, .eml via email stdlib. Both are in the ingestion pipeline. |
| "Is this production-ready?" | Phases 0–5 complete, all benchmark gates passed. Phase 6 (release candidate) in progress. See `docs/archive/legacy-app/ALIV_PRODUCT_PHASES.md`. |

---

## Sample Benchmark Queries (Technical Audience)

Run these against the demo corpus after ingestion to validate retrieval quality:

```
1. Does any document mention a penalty for delivery delays? What is the amount?
2. What is the total amount due across all invoices?
3. Which agreements require 90 days written notice for renewal?
4. Extract all expiry dates and sort them chronologically.
5. What confidentiality obligations apply after termination?
```

---

## Running the Benchmark

See [`benchmarks/README.md`](../benchmarks/README.md): the vault scans the labelled corpus and `score_vault_extraction.py` scores the extracted fields.

