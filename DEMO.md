# ALIV Demo Guide

This guide is a polished walkthrough for presenting ALIV in 5 to 8 minutes.

## Demo Goal

Show how ALIV handles sensitive local documents end to end without cloud upload.

## Demo Prerequisites

1. Build succeeded and app launches.
2. Ollama is running locally.
3. A small test workspace exists with mixed files.
4. You can access the app from src-tauri via cargo tauri dev.

Recommended sample source in this repo:

1. benchmarks/corpus
2. benchmarks/sample_corpus

## Demo Runbook

### Step 1: Launch and trust framing (45 to 60 seconds)

1. Start ALIV.
2. Explain local-first posture and no cloud vector dependency.
3. Show onboarding and select a workspace folder.

Suggested narration:

ALIV is a local desktop workbench for legal and finance document operations. Indexing, retrieval, memory, and actions stay on-device.

### Step 2: Ingestion (60 to 90 seconds)

1. Open Ingestion.
2. Run incremental ingestion first.
3. Call out discovered and indexed counts.

Success signal:

Telemetry updates and run status completes without errors.

### Step 3: Retrieval and extraction (90 to 120 seconds)

1. Open Query and run a natural-language question.
2. Highlight evidence-backed hits and citations.
3. Open Extraction and run a template-based extraction.

Suggested queries:

1. Find agreements expiring in 90 days and show penalty clauses.
2. Extract vendor, effective date, and renewal terms.

Success signal:

Relevant hits appear with confidence and citation context.

### Step 4: Safe actions and rollback trust (60 to 90 seconds)

1. Open Actions Preview.
2. Generate recommendations.
3. Approve and execute a small safe batch.
4. Open Audit and show traceability.
5. Trigger undo for a batch to demonstrate reversibility.

Success signal:

Execution is logged and rollback pathway is visible.

### Step 5: Memory and diagnostics (60 to 90 seconds)

1. Pin one extracted fact in Memory.
2. Open Diagnostics.
3. Generate report and export JSON.
4. Open log folder from the diagnostics page.

Success signal:

JSON export contains runtime, DB counts, platform info, and log tail.

## Demo Script Version (Short)

1. Launch ALIV and set workspace.
2. Run ingestion.
3. Ask a retrieval question.
4. Run extraction.
5. Preview and execute one safe action.
6. Show audit and undo.
7. Save a memory fact.
8. Export diagnostics JSON.

## Failure Recovery Talking Points

1. If Ollama is offline, show onboarding step guidance and retry.
2. If ingestion is empty, switch to benchmarks/sample_corpus and rerun.
3. If retrieval quality drops, explain benchmark suite and threshold validation.
4. If action risk is high, show preview gating and skip execution.

## Demo Checklist

1. Onboarding appears or has been previously completed intentionally.
2. Workspace is registered.
3. Ingestion completes.
4. Retrieval returns at least one relevant cited hit.
5. Extraction returns structured fields.
6. Action preview and audit are visible.
7. Diagnostics JSON is exported.

## Optional Command Snippets

```powershell
npm install
Set-Location "src-tauri"
cargo tauri dev
```

```powershell
npm run build
Set-Location "src-tauri"
cargo tauri build
```
