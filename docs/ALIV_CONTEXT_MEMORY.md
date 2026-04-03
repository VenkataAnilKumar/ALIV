# ALIV Context Memory

## Product
- Autonomous Local Intelligence Vault (ALIV)
- Local-first and offline-capable document intelligence product
- Three core workflows: Legal/Finance Agent, Messy Inbox Organizer, Persistent Memory Layer

## PRDs
- PRD-000: Master thresholds and validation protocol
- PRD-001: Ingestion and Indexing
- PRD-002: Retrieval and Extraction
- PRD-003: Safe File Actions
- PRD-004: Persistent Memory and Long Context ← NEW

## Current Source Docs
- docs/ALIV_PRODUCT_REQUIREMENTS.md — includes local LLM spec (Ollama llama3.2:3b, hardware requirements)
- docs/ALIV_BUILD_WEEK1_PLAN.md — historical Week 1 implementation plan
- docs/ALIV_PRODUCT_PHASES.md — canonical phase tracker (Phases 1–4 complete)
- docs/ALIV_REFERENCE_CODE_MAP.md
- docs/ALIV_IMPLEMENTATION_BACKLOG.md — Workstreams A–F (F = Memory)
- docs/ALIV_FILE_LEVEL_IMPLEMENTATION_CHECKLIST.md — Section 5.5 = Memory checklist
- docs/PHASE4_HARDENING_REPORT.md — phase 4 completion evidence and metrics

## Design and Architecture Docs
- docs/architecture/ALIV_SYSTEM_ARCHITECTURE.md
- docs/architecture/ALIV_TECHNICAL_DESIGN.md
- docs/design/ALIV_PRODUCT_DESIGN.md — 7 screens with full component + state specs
- docs/design/ALIV_DESIGN_PRINCIPLES.md
- docs/prd/README.md

## Local AI Model Stack
- LLM: Ollama + llama3.2:3b (fallback: mistral:7b-instruct-q4_K_M), localhost:11434
- Embeddings: sentence-transformers all-MiniLM-L6-v2, 384 dims, batch 32
- OCR: Tesseract 5.x, min confidence 60
- Summarization: Ollama (same model), post-ingestion per file

## Memory Architecture (PRD-004)
Three memory layers — all local SQLite, no cloud:
1. Conversation memory — multi-turn history across sessions (conversations + conversation_turns tables)
2. Document summary cache — 150-200 word Ollama summaries per file (document_summaries table)
3. User memory facts — pinned facts with citations, scoped permanent/week/session (memory_facts table)

Token budget: 8192 context window → 6912 usable → facts 10%, history 20%, chunks 60%, summaries 10%

New files:
- db/migrations/0005_memory_context.sql
- src-tauri/src/core/memory.rs (MemoryService, context assembler, token budget)
- src-tauri/src/commands/memory.rs (8 Tauri commands)
- python/workers/summarization_worker.py (Ollama HTTP, document/section granularities)

## Current Direction
- Phase 1: complete
- Phase 2: complete
- Phase 3: complete
- Phase 4: complete (hardening gate passed with two consecutive full-suite runs)
- Active phase: Phase 5 (UX and operator experience)

## Week 1 Validation Targets
- Ingestion success rate >= 98%
- Retrieval relevance (top-5) >= 85%
- Extraction precision (critical fields) >= 95%
- Harmful file action rate < 1%
- Query latency p95 < 8s on reference machine

## Safety Constraints
- Keep file actions preview-first until safety threshold passes
- Maintain rollback and undo capability for all file actions
- Memory never leaves device — all storage is local SQLite

## Target Architecture
- Rust and Tauri backend shell
- React and TypeScript frontend
- SQLite with sqlite-vec (ANN search via vec0 virtual tables)
- Python subprocess bridge for document parsing and ML tasks
- Ollama for local LLM inference (localhost:11434)
