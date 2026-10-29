# ALIV plan

**Decision:** [`apps/desktop`](../../../apps/desktop/README.md) is the product. The legacy app that lived at the repository root (`src/`, `src-tauri/`, `python/`) has been retired, after the few things it did that the new app didn't were rebuilt inside the new app. Full product docs are written after that, so they're written once.

Each step is one pull request into `main`.

## Steps

| # | Pull request | What it delivers | Done when |
|---|---|---|---|
| A | **Docs cleanup, plan and CI** | Obsolete docs moved to `docs/archive/`; root README describes the new app; GitHub Actions runs the desktop app's tests on every PR | `main` is green and the README matches the product |
| B ✅ | **OCR** | Scanned PDFs and images are read during ingestion, instead of being listed as unreadable. Passages read by OCR are marked as such, with a confidence score | The 40 scanned documents in `benchmarks/` are indexed and their fields extracted |
| C ✅ | **Hybrid search** | Meaning-based search next to the existing keyword search (FTS5), combined with reciprocal-rank fusion. Falls back to keyword-only when no embedding model is available | Questions phrased differently from the contract text still find the right clause |
| D ✅ | **Summaries and context budget** | A summary per document, built at ingestion from its extracted fields and opening (no model), so it changes only when the file does; Ask fits its sources into the model's context window, most relevant first | Ask works on a 500-document matter without overflowing the context |
| E ✅ | **Diagnostics; retire the legacy app** | A vault health report in the new app (Matters → Vault health); `src/`, `src-tauri/`, `python/`, `db/` and their configs removed | One app, one build |
| F ✅ | **Product docs** | User guide, security and privacy overview, developer architecture and build docs | Every feature in the app has a page |

## What the legacy app has that the new app doesn't

| Capability | Legacy app | Rebuilt in |
|---|---|---|
| OCR for scanned PDFs and images | `python/workers/ocr_worker.py` (Tesseract) | B: done, `tauri-plugin-aliv-vault/src/ocr.rs` |
| Meaning-based (embedding) search, fused with keyword search | `python/workers/embedding_worker.py`, `src-tauri/src/core/retrieval.rs` | C: done, `tauri-plugin-aliv-vault/src/semantic.rs` |
| Document summaries, conversation memory, token budget | `python/workers/summarization_worker.py`, `src-tauri/src/core/memory.rs` | D: done. Summaries are extractive (`extract::summarize`), not model-written; context budget in `web-app/src/aliv/ask.ts`. Conversation memory is covered by the app's chat threads |
| Diagnostics export and benchmark views | `src/pages/DiagnosticsPage.tsx`, `BenchmarksPage.tsx` | E: done. Vault health report (`vault_diagnostics`); app logs and hardware were already in the new app; benchmarks run from [`benchmarks/`](../../../benchmarks/README.md) |

Already covered by the new app, so nothing to port: field extraction (the register is a superset of the legacy templates), safe file actions with undo, email parsing, workspace and ingestion.

Each capability is rebuilt in Rust inside `tauri-plugin-aliv-vault` rather than carried over as a Python sidecar, so the installer ships one runtime.

## After the plan

Signed installers and an ALIV update server; first-run setup; the cloud gateway that routes and redacts cloud calls; accuracy benchmarks on real contracts; design-partner pilots. See the roadmap in [`apps/desktop/README.md`](../../../apps/desktop/README.md#roadmap).
