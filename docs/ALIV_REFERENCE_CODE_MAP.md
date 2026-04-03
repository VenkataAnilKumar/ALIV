# ALIV Reference Code Map

## Purpose
This document lists high-signal reference repositories and code patterns for building ALIV in a product-only, local-first way. It maps each reference to ALIV PRD phases and clarifies what to adopt, adapt, or avoid.

## Quick Priority (Phase 0 -> Phase 1)
1. sqlite-vec: local vector indexing and nearest-neighbor query patterns.
2. Tauri examples: command architecture, state management, and app structure.
3. PyMuPDF + Tesseract: document text extraction plus OCR fallback pipeline.
4. text-embeddings-inference: local embedding/rerank service patterns and performance knobs.
5. PrivateGPT and LocalGPT: API and pipeline patterns to borrow selectively (not full architecture copy).

## Development Execution Rules (Reference-First)
Use these rules for all implementation work to keep output consistent, maintainable, and phase-gated.

1. Resolve technical uncertainty with focused research first.
2. Prefer vetted patterns from official or well-maintained sources.
3. Reuse or adapt proven implementations before writing from scratch.
4. Reuse references already identified in this repository and in current discussions.
5. Keep syntax and structure consistent with existing project modules.
6. Favor small, composable changes over large rewrites.
7. Build strictly phase-wise and do not move forward until current phase exit criteria pass.
8. Validate changes against PRD thresholds before declaring a phase complete.

Primary governance references:
- Phase plan and exit gates: docs/ALIV_PRODUCT_PHASES.md
- Master thresholds and validation protocol: docs/prd/PRD-000-MASTER.md

## Curated References

### 1) sqlite-vec
- Repository: https://github.com/asg017/sqlite-vec
- ALIV fit: PRD-001 ingestion/indexing, PRD-002 retrieval baseline.
- Reuse now:
	- `vec0` virtual table design and index creation flow.
	- KNN query shape using `MATCH` + distance ordering + top-k limit.
	- Local-first storage strategy in SQLite process.
- Adapt for ALIV:
	- Keep metadata and vectors in coordinated tables (document/chunk lineage + vector table).
	- Add deterministic chunk IDs for reproducibility and benchmark repeatability.
- Avoid:
	- Depending on unstable APIs without pinning versions.

### 2) Tauri + examples
- Repository: https://github.com/tauri-apps/tauri
- Examples directory: https://github.com/tauri-apps/tauri/tree/dev/examples
- ALIV fit: shell orchestration for PRD-001/002/003.
- Reuse now:
	- Command handler pattern (`invoke`) for frontend-backend contracts.
	- Shared state and lifecycle event patterns from examples.
	- Multi-module desktop app organization.
- Adapt for ALIV:
	- Define strict command surface for ingestion, retrieval, extraction, preview, approve, rollback, audit.
	- Keep long-running parse/OCR/extract in worker subprocesses and report progress events.
- Avoid:
	- Mixing UI concerns into command implementation; keep command layer thin.

### 3) PyMuPDF
- Repository: https://github.com/pymupdf/PyMuPDF
- ALIV fit: PRD-001 parsing stage, baseline text extraction.
- Reuse now:
	- Per-page extraction flow and page iteration patterns.
	- Fast local extraction path before OCR fallback.
- Adapt for ALIV:
	- Add parser quality score per page (empty text ratio, noise heuristics).
	- Route low-quality pages to OCR worker.
- Avoid:
	- Treating extracted text as trustworthy without normalization and quality checks.
- License note:
	- Validate commercial/license fit before production distribution.

### 4) Tesseract OCR
- Repository: https://github.com/tesseract-ocr/tesseract
- ALIV fit: PRD-001 OCR fallback and PRD-002 extraction quality recovery.
- Reuse now:
	- CLI-based OCR invocation pattern and language pack usage.
	- Structured output handoff back into parse pipeline.
- Adapt for ALIV:
	- Apply OCR only when parser confidence is below threshold.
	- Cache OCR artifacts keyed by file hash + page image hash.
- Avoid:
	- Running OCR on all pages by default (latency/cost hit).

### 5) text-embeddings-inference
- Repository: https://github.com/huggingface/text-embeddings-inference
- ALIV fit: PRD-001 embeddings, PRD-002 rerank/quality experiments.
- Reuse now:
	- `/embed` and `/rerank` API shapes.
	- Batching and throughput tuning controls.
- Adapt for ALIV:
	- Wrap with ALIV adapter interface so model/provider changes do not leak across modules.
	- Track embedding model version in benchmark metadata.
- Avoid:
	- Tight coupling to one model endpoint in core domain layer.

### 6) PrivateGPT
- Repository: https://github.com/zylon-ai/private-gpt
- ALIV fit: reference architecture for API layering and DI boundaries.
- Reuse now:
	- Separation between API router layer and service layer.
	- Ingestion and retrieval primitive decomposition.
- Adapt for ALIV:
	- Keep ALIV's SQLite + sqlite-vec strategy (instead of adopting external vector DB assumptions).
	- Preserve ALIV safety-first command gate for file actions.
- Avoid:
	- Copying full framework stack if it conflicts with ALIV's modular monolith direction.

### 7) LocalGPT
- Repository: https://github.com/PromtEngineer/localGPT
- ALIV fit: practical end-to-end local RAG operational patterns.
- Reuse now:
	- Batch indexing scripts and health-check operational patterns.
	- Hybrid retrieval/rerank configuration ideas.
	- API-first index/session management patterns.
- Adapt for ALIV:
	- Replace their vector layer assumptions with sqlite-vec-first implementation.
	- Use only patterns aligned with your benchmark protocol.
- Avoid:
	- Pulling full dependency surface into core MVP.

### 8) Open WebUI (study later)
- Repository: https://github.com/open-webui/open-webui
- ALIV fit: optional future UI/ops and plugin ecosystem ideas.
- Reuse later:
	- Offline mode operational settings.
	- Extensibility and observability ideas for post-MVP.
- Avoid now:
	- Large-scope UI/platform adoption that can distract from Phase 0 benchmark gates.
- License note:
	- Review current multi-license terms before any code-level reuse.

## Not Recommended for ALIV Core
- Chroma: good ecosystem project, but not aligned with current sqlite-vec-first local architecture.
- LangChain/LlamaIndex frameworks: useful patterns, but full adoption can add abstraction overhead too early.
- Stable Diffusion WebUI: unrelated to ALIV document intelligence core scope.

## Adopt / Adapt / Avoid Matrix
- Adopt directly:
	- sqlite-vec query/index patterns.
	- Tauri command + state patterns from official examples.
	- PyMuPDF page extraction baseline.
- Adapt:
	- PrivateGPT and LocalGPT service/API layering and operational scripts.
	- text-embeddings-inference deployment/tuning patterns behind ALIV adapter contracts.
- Avoid for MVP:
	- Heavy framework lock-in and unrelated UI/image-generation stacks.

## Suggested Implementation Order by PRD

### PRD-001 Ingestion & Indexing
1. Build parser pipeline: PyMuPDF primary, Tesseract fallback.
2. Build chunk lineage and deterministic IDs.
3. Add embeddings adapter and sqlite-vec storage/query pipeline.
4. Add benchmark harness for ingestion success and latency metrics.

### PRD-002 Retrieval & Extraction
1. Implement top-k retrieval over sqlite-vec.
2. Add rerank experiment path (via text-embeddings-inference rerank API if needed).
3. Add citation/confidence contract and extraction precision checks.

### PRD-003 Safe File Actions
1. Use Tauri command boundaries for preview/approve/rollback/audit.
2. Add explicit risk scoring and policy blocks before mutation commands.
3. Persist immutable audit trail with operation and evidence references.

## License and Compliance Checklist Before Copying Code
- Confirm repository license compatibility with ALIV distribution model.
- Prefer reimplementation of ideas over verbatim copying.
- Preserve attribution when required.
- Record third-party components and licenses in a dedicated NOTICE file.

## Final Guidance
For Phase 0, prioritize small, verifiable patterns from official/maintained sources (sqlite-vec, Tauri examples, PyMuPDF, Tesseract, text-embeddings-inference). Borrow architecture ideas from large RAG apps only when they map directly to ALIV benchmark gates.
