# ALIV

Autonomous Local Intelligence Vault is a local-first desktop intelligence workbench for legal and finance document operations.

[Release Candidate Changelog](CHANGELOG.md) | [Demo Walkthrough](DEMO.md) | [Phase 6 Release Checklist](docs/PHASE6_RELEASE_CHECKLIST.md)

## Why ALIV

Most AI document tools assume your files can be uploaded.
ALIV is designed for teams that cannot do that.

1. Runs locally on your machine.
2. Keeps ingestion, retrieval, and memory in local storage.
3. Provides operator-safe action previews before any rename or move.
4. Preserves citations, audit logs, and rollback pathways.

## Core Capabilities

1. Ingestion: Indexes PDFs, DOCX, emails, images, and mixed folders.
2. Retrieval: Returns evidence-backed hits with citation context.
3. Extraction: Produces structured field outputs for downstream review.
4. Memory: Stores conversation turns, summaries, and pinned facts.
5. Safe Actions: Preview-first recommendations with execution audit and undo.
6. Diagnostics: Exports runtime, database, and log-tail diagnostics as JSON.

## Product Snapshot

1. Frontend: React, TypeScript, Vite
2. Desktop shell and orchestration: Rust, Tauri 2
3. Data layer: SQLite (WAL) with sqlite-vec
4. Worker layer: Python subprocess pipeline for parsing and embedding workflows

## Quick Start

### Prerequisites

1. Node.js 18+ and npm
2. Rust toolchain and Cargo
3. Tauri CLI 2.x
4. Ollama running locally for model-backed flows

### Run in Development

```powershell
npm install
Set-Location "src-tauri"
cargo tauri dev
```

### Windows PATH fix for tauri-cli

```powershell
$env:PATH = "$HOME\.cargo\bin;" + $env:PATH
Set-Location "src-tauri"
cargo tauri dev
```

## Build and Package

```powershell
npm run build
Set-Location "src-tauri"
cargo build
cargo tauri build
```

Expected installer outputs:

1. src-tauri/target/release/bundle/msi/ALIV_0.1.0_x64_en-US.msi
2. src-tauri/target/release/bundle/nsis/ALIV_0.1.0_x64-setup.exe

## Demo

Use the guided script in [DEMO.md](DEMO.md) for a structured 5-8 minute product demo.

The demo walks through:

1. Workspace setup and onboarding
2. Ingestion and retrieval
3. Extraction and safe actions
4. Audit, memory, and diagnostics export

## Repository Layout

1. src: React desktop UI
2. src-tauri: Rust and Tauri command/runtime layer
3. python: parsing and embedding workers
4. db: SQL schema and migrations
5. benchmarks: benchmark corpus, scripts, and result snapshots
6. docs: product, architecture, and implementation guides

## Documentation Index

1. [DEMO.md](DEMO.md): polished demo flow for internal and external walkthroughs
2. [CHANGELOG.md](CHANGELOG.md): release notes and known limitations
3. [docs/PHASE6_RELEASE_CHECKLIST.md](docs/PHASE6_RELEASE_CHECKLIST.md): critical-path release checks
4. [PRODUCT_DOCUMENT.md](PRODUCT_DOCUMENT.md): product requirements and architecture narrative

## Current Release Focus

1. Bundle-ready desktop packaging
2. Structured runtime logging
3. Diagnostics report generation and export
4. First-run onboarding
5. Frontend crash containment via error boundary

## License

No explicit license file is currently included in this repository.
