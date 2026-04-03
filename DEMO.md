# ALIV Demo Guide

Use this guide to run a buyer-facing demo that feels credible, fast, and operationally mature.

## Demo Objective

Show that ALIV delivers document intelligence for legal and finance teams while preserving local data control and operator safety.

## Audience Promise

By the end of this demo, the audience should believe:

1. sensitive data can stay local,
2. retrieval and extraction are evidence-backed,
3. actions are controlled and reversible,
4. support and diagnostics are production-ready.

## Demo Setup

1. App launches successfully.
2. Ollama is running.
3. A mixed workspace folder is available.
4. Use benchmark sample folders for deterministic runs when needed.

Recommended sample sources:

1. benchmarks/corpus
2. benchmarks/sample_corpus

## 6-Stage Storyline

### Stage 1: Open with risk and trust (45 to 60 seconds)

1. Launch ALIV.
2. Position the core issue: most AI document tools require cloud upload.
3. Explain that ALIV keeps retrieval, memory, and actions local.
4. Show onboarding and set the workspace.

Narration example:

ALIV is purpose-built for teams that cannot trade confidentiality for convenience.

### Stage 2: Prove speed to value with ingestion (60 to 90 seconds)

1. Open Ingestion.
2. Run incremental ingestion.
3. Call out discovered, indexed, and parsed counts.

Proof point:

The system converts folder chaos into searchable local intelligence in one flow.

### Stage 3: Prove answer quality with retrieval and extraction (90 to 120 seconds)

1. Open Query and submit a practical legal or finance prompt.
2. Highlight citation-backed hits and confidence context.
3. Open Extraction and run a structured template.

Suggested prompts:

1. Find agreements expiring in the next 90 days and extract penalty clauses.
2. Extract vendor name, effective date, renewal window, and cancellation terms.

Proof point:

Outputs are not only generated, they are traceable back to evidence.

### Stage 4: Prove safety with action controls (60 to 90 seconds)

1. Open Actions Preview.
2. Generate recommendations.
3. Approve and execute a small safe set.
4. Open Audit and show logged outcomes.
5. Run undo to show rollback.

Proof point:

Automation is governed, reviewable, and reversible.

### Stage 5: Prove memory and continuity (45 to 60 seconds)

1. Save a memory fact.
2. Show memory listing and reuse in follow-up context.

Proof point:

ALIV compounds value over time instead of resetting every session.

### Stage 6: Prove operational readiness (45 to 60 seconds)

1. Open Diagnostics.
2. Generate report.
3. Export JSON and open log folder.

Proof point:

The platform is built for real operations, not just one-off demos.

## Objection Handling

### What if Ollama is offline?

1. Show onboarding guidance for model availability.
2. Retry health and continue.

### What if ingestion returns zero files?

1. Switch to benchmarks/sample_corpus.
2. Re-run incremental ingestion and continue demo.

### What if an action looks risky?

1. Highlight preview-first controls.
2. Skip execution and explain approval gates.

## Closing Script

Use this close after Stage 6:

ALIV gives your team local AI document intelligence with evidence, controls, and rollback. If this aligns with your trust requirements, the next step is a scoped pilot on one legal or finance workflow.

## Demo Success Checklist

1. Workspace registered.
2. Ingestion completed successfully.
3. Retrieval returned cited evidence.
4. Extraction produced structured fields.
5. Safe action preview and audit demonstrated.
6. Rollback path demonstrated.
7. Diagnostics JSON exported.

## Command Snippets

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
