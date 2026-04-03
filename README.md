# ALIV

Autonomous Local Intelligence Vault (ALIV) is a local-first desktop document intelligence system for ingestion, retrieval, extraction, and safe file actions.

## Current Status
This repository is scaffolded from the ALIV product, architecture, and PRD documents under [docs](docs).

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
