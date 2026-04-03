# ALIV

Autonomous Local Intelligence Vault (ALIV) is a local-first desktop document intelligence system for legal and finance workflows that need AI-powered document understanding without sending data to external cloud services.

ALIV combines offline ingestion, retrieval, extraction, persistent memory, and preview-first file actions with citations, auditability, and rollback.

## What ALIV Does
- Ingests mixed local document sets including PDFs, DOCX files, emails, and images
- Retrieves evidence-backed answers with citation-aware ranking
- Extracts structured facts from documents with benchmarked precision targets
- Maintains persistent local memory for conversations, summaries, and pinned facts
- Supports preview-first file actions with rollback and safety controls

## Product Positioning
ALIV is designed as a desktop-first, privacy-preserving document intelligence product for sensitive workflows where local processing, reproducibility, and operator trust matter more than cloud convenience.

## Current Status
This repository contains the active ALIV product build, benchmark tooling, architecture docs, PRDs, and implementation artifacts under [docs](docs).

## Initial Structure
- `src-tauri/` Rust and Tauri orchestration layer
- `python/` parsing, OCR, extraction, and embedding workers
- `db/` schema and migration scripts
- `docs/` product, design, architecture, and implementation artifacts
- `logs/` local runtime logs

## Build Focus
Current phase progression:
1. Phase 1 complete: ingestion and indexing validation
2. Phase 2 complete: retrieval and extraction thresholds met
3. Phase 3 complete: preview-first file action command surface and rollback scaffolding
4. Phase 4 complete: end-to-end hardening with two consecutive passing full-suite runs
5. Active now: Phase 5 UX and operator experience improvements

## Local Desktop Run
- Install frontend dependencies from the repo root with `npm install`
- Start the desktop app from `src-tauri/` with `cargo tauri dev`

### Windows Note
If `cargo tauri` is not found after installing `tauri-cli`, add Cargo's bin directory to the current shell first:

```powershell
$env:PATH = "$HOME\.cargo\bin;" + $env:PATH
Set-Location "V:/AI Engineer/GitHub/ALIV/src-tauri"
cargo tauri dev
```

ALIV's Tauri crate also expects the desktop binary as the default run target, so launching through `cargo tauri dev` should resolve to `aliv-desktop` without extra `--bin` arguments.
