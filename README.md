# ALIV

Autonomous Local Intelligence Vault is a local-first document intelligence workbench for legal and finance teams that cannot risk sending sensitive files to external cloud AI services.

[Customer One-Pager](docs/CUSTOMER_ONE_PAGER.md) | [Demo Walkthrough](DEMO.md) | [Release Notes](CHANGELOG.md) | [Release Checklist](docs/PHASE6_RELEASE_CHECKLIST.md)

## The Problem We Solve

Most AI document tools start by asking you to upload proprietary contracts, invoices, and legal artifacts to a vendor-controlled cloud.

For many legal and finance organizations, that is a non-starter.

ALIV is built for those teams.

## What Customers Get

1. Faster document intelligence without changing existing folder workflows.
2. Local-first processing aligned with stricter trust and compliance expectations.
3. Operator-safe action workflow with preview, approval, audit, and rollback.
4. Better handoff quality from AI findings to human review teams.

## Why Teams Choose ALIV

1. Local by default: ingestion, retrieval, extraction, memory, and diagnostics remain on-device.
2. Evidence-backed outputs: retrieval responses include citation context.
3. Safe automation: file actions are staged before execution and fully auditable.
4. Recovery built in: action rollback and runtime diagnostics export are first-class workflows.

## Product Experience

ALIV is designed as an operator console, not a chat toy.

1. Onboard a workspace in minutes.
2. Ingest mixed files from existing local folders.
3. Ask questions and extract structured facts with citations.
4. Stage and approve actions with clear risk visibility.
5. Export diagnostics and logs for operations and support.

## Core Capabilities

1. Ingestion: indexes PDFs, DOCX, emails, images, and mixed folders.
2. Retrieval: returns evidence-backed hits with citation context.
3. Extraction: produces structured field outputs for downstream review.
4. Memory: stores conversation turns, summaries, and pinned facts.
5. Safe Actions: preview-first recommendations with execution audit and undo.
6. Diagnostics: exports runtime, database, and log-tail diagnostics as JSON.

## Ideal Customers

1. Small and mid-sized legal teams with confidentiality-heavy workflows.
2. Finance and accounting groups handling contract and invoice evidence trails.
3. In-house legal and operations teams that need local AI without cloud-first risk.

## Demo First

Use [DEMO.md](DEMO.md) for a buyer-facing walkthrough in 5 to 8 minutes.

The demo is structured to prove:

1. trust posture,
2. measurable workflow compression,
3. operator control and reversibility,
4. production supportability via diagnostics.

## Customer Narrative Pack

1. [docs/CUSTOMER_ONE_PAGER.md](docs/CUSTOMER_ONE_PAGER.md): concise buyer narrative and pilot framing.
2. [DEMO.md](DEMO.md): scripted product walkthrough.
3. [CHANGELOG.md](CHANGELOG.md): release notes and known limitations.
4. [PRODUCT_DOCUMENT.md](PRODUCT_DOCUMENT.md): full product requirements and architecture.

## Technical Snapshot

1. Frontend: React, TypeScript, Vite.
2. Desktop shell and orchestration: Rust, Tauri 2.
3. Data layer: SQLite WAL with sqlite-vec.
4. Worker layer: Python subprocess pipeline for parsing and embedding workflows.

## Quick Start

### Prerequisites

1. Node.js 18+ and npm.
2. Rust toolchain and Cargo.
3. Tauri CLI 2.x.
4. Ollama running locally for model-backed flows.

### Run in Development

```powershell
npm install
Set-Location "src-tauri"
cargo tauri dev
```

### Windows PATH Fix for tauri-cli

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

## Repository Layout

1. src: React desktop UI.
2. src-tauri: Rust and Tauri command/runtime layer.
3. python: parsing and embedding workers.
4. db: SQL schema and migrations.
5. benchmarks: benchmark corpus, scripts, and result snapshots.
6. docs: product, architecture, and implementation guides.

## Current Release Focus

1. Bundle-ready desktop packaging.
2. Structured runtime logging.
3. Diagnostics report generation and export.
4. First-run onboarding.
5. Frontend crash containment via error boundary.

## License

No explicit license file is currently included in this repository.
