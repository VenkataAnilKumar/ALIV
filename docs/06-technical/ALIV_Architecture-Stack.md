# Architecture Stack
## Technology choices and why

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active

---

## 1. Stack at a glance

| Layer | Technology | Version (as of this doc) | Status |
|---|---|---|---|
| App shell | **Tauri 2** | 2.8.x | ✅ |
| Core language | **Rust** | toolchain pinned in `rust-toolchain.toml` (MSRV 1.88) | ✅ |
| UI | **React 19** + TypeScript | React 19.0 | ✅ |
| Build | **Vite** | 6.4 | ✅ |
| Styling | **Tailwind CSS 4** + shadcn/Radix-style components | 4.1 | ✅ |
| Routing / state | TanStack Router · Zustand | 1.x · 5.0 | ✅ |
| Package manager | Yarn 4 (Corepack) | 4.x | ✅ |
| Vault store | **SQLite** via `rusqlite` (bundled) with **FTS5** | rusqlite 0.32 | ✅ |
| Vectors | Unit vectors as BLOBs in `chunk_vectors`; cosine in Rust; RRF fusion with BM25 | — | ✅ |
| OCR | **Tesseract** + **poppler** (`pdftoppm`), `image` crate for upscaling | system packages | ✅ |
| Local models | **llama.cpp** (all platforms), **MLX** (Apple silicon) | via plugins/extensions | ✅ |
| Embeddings | Local GGUF embedding model via llama.cpp (`@aliv/core` `embedTexts`) | — | ✅ |
| Cloud models | OpenAI-compatible and Anthropic adapters | — | R1 (gateway) |
| Secrets | OS keychain (Keychain, Credential Manager, Secret Service), AES-256-GCM file fallback on headless Linux (`provider_secrets.rs`) | — | ✅ storage; R1 makes the gateway their only user |
| Documents out | DOCX (tracked changes), XLSX, PDF writers in Rust | — | R2 |
| Email archives | PST/MBOX readers | — | R2 |
| Tools and skills | MCP client (`core/mcp`, rmcp-based, with OAuth); SKILL.md skills and git plugins (`core/agent`) | — | ✅ base app; R1 governance |
| Document parsing (R2) | Local layout model, e.g. Docling / Granite-Docling (258M) or olmOCR (7B), chosen by benchmark; Tesseract fallback | — | R2 |
| Reranking (R2) | Local cross-encoder via llama.cpp rerank (Qwen3-Reranker, bge-reranker-v2-m3) | — | R2 |
| ANN index (R2 decision) | LanceDB (Rust-native, hybrid search) or sqlite-vec (pre-v1); exact search until the budget is exceeded | — | R2 |
| Structured output | JSON schema / grammar-constrained decoding (llama.cpp GBNF; provider structured outputs) | — | R2 |
| Type sharing | TS types generated from Rust (specta / ts-rs) | — | R1 |
| Evaluation | `evals/` harness, LLM-as-judge with human calibration, CI gates | — | R1–R2 |
| Tracing | OpenTelemetry GenAI conventions (version pinned; still *Development* status) | — | R1 |
| Server | Rust (axum) + PostgreSQL or SQLite, OIDC/SAML | — | R4 |
| Word add-in | Office.js (TypeScript) | — | R4 |
| Tests | Vitest, Rust `cargo test`, Tauri mock runtime IPC test, Playwright (e2e) | — | ✅ |
| CI | GitHub Actions (`.github/workflows/desktop.yml`) | — | ✅ |

## 2. Why these choices

| Decision | Alternatives considered | Why |
|---|---|---|
| **Tauri over Electron** | Electron | Small binaries, Rust core for data and policy, lower memory; the base app is already Tauri |
| **Rust owns data and policy** | Node backend | Memory-safe, one place to enforce policy, reusable in the Firm Server |
| **SQLite + FTS5** | Postgres, Elasticsearch, Tantivy | Single file, zero admin, fast enough for 10k documents per vault, works offline; FTS5 gives BM25 |
| **Vectors in SQLite, brute-force cosine** | sqlite-vec, LanceDB, Qdrant | Simple and exact at desktop scale; no extra native dependency. An ANN index is benchmarked in R2 (SRCH-14): LanceDB is Rust-native with hybrid search; sqlite-vec is tiny but pre-v1 |
| **Reranker after RRF** | No reranker; cloud rerank APIs | The largest accuracy gain per cost in 2026 retrieval; runs locally, so it works for Local only matters |
| **Layout-aware parsing** | Tesseract only; cloud parsing APIs | Legal documents are structured (clauses, schedules, tables); local layout models keep structure without sending documents out |
| **Workspace of crates** | Code inside the desktop app | The Firm Server, Admin Console and Word add-in reuse the same domain code (see Repo Structure §4) |
| **RRF hybrid search** | Pure vector, learned reranker | Robust without tuning; keyword precision for legal terms, meaning for paraphrase |
| **Tesseract OCR** | Cloud OCR, PaddleOCR | Local, mature, multilingual; cloud OCR would break the privacy promise |
| **llama.cpp + MLX** | Ollama only | Bundled, no separate install; MLX is fastest on Apple silicon; Ollama and LM Studio still usable as local providers |
| **Local embeddings only** | Cloud embeddings | Meaning search must work for Local only matters with zero exceptions |
| **Rust gateway for all model calls** | Calls from the web layer | Unbypassable policy, complete audit, keys out of the web layer |
| **Extractive summaries (R0)** | LLM summaries | Instant, deterministic, no model needed at ingest; LLM summaries become an optional workflow |

## 3. Local model tiers (guidance)

| Hardware | Chat model class | Typical use |
|---|---|---|
| 8 GB RAM, no GPU | 3–4B, Q4 | Ask on small matters, extraction |
| 16 GB / Apple M-series | 7–8B, Q4–Q5 | Default: Ask, review-table extraction |
| 32 GB+ / 12 GB+ VRAM | 14B–32B | Drafting and comparison locally |
| Firm Server GPU | 70B-class | Local-only firms that want cloud-like quality |

The router (R1) picks the tier from `tauri-plugin-hardware` and the matter's routes.

## 4. Dependency policy

- Pin versions in lockfiles (`yarn.lock`, `Cargo.lock`). CI installs with `--immutable`.
- Only one `libsqlite3-sys` may be linked (vault and vector-db plugins must match).
- New native dependencies need a licence check (`THIRD_PARTY_NOTICES.md`) and a note in this document.
- No telemetry SDKs that send document data, ever.

## 5. Sources for the October 2026 technology check

| Topic | Finding | Source |
|---|---|---|
| Embedded vector search | LanceDB is written in Rust with Rust, Python and JS clients; sqlite-vec is a dependency-free C extension, pre-v1 | [LanceDB FAQ](https://docs.lancedb.com/faq/faq-oss), [Embedded vector search guide (Oct 2026)](https://www.pistack.xyz/posts/2026-10-13-embedded-vector-search-lancedb-sqlite-vec-pgvector-guide/) |
| Local rerankers | Qwen3-Reranker and bge-reranker-v2-m3 are common local choices; llama.cpp serves rerankers with `--pooling rank` | [Redis: reranking models 2026](https://redis.io/blog/top-reranking-models-rag-accuracy.md), [bge-reranker-v2-m3 GGUF](https://huggingface.co/gpustack/bge-reranker-v2-m3-GGUF) |
| Layout-aware parsing | Docling is an MIT-licensed local pipeline; Granite-Docling is a 258M-parameter document VLM; olmOCR is a 7B VLM with its own benchmark | [IBM Granite-Docling](https://ibm.com/new/announcements/granite-docling-end-to-end-document-conversion), [olmOCR paper](https://arxiv.org/pdf/2502.18443) |
| GenAI tracing | OpenTelemetry GenAI semantic conventions are still in *Development* status in 2026 and moved to their own repository | [Status in 2026](https://dev.to/azena-ai/opentelemetrys-genai-semantic-conventions-are-not-stable-yet-heres-what-actually-shipped-in-2026-3mke), [Dash0 explainer](https://www.dash0.com/knowledge/opentelemetry-genai-semantic-conventions-explained) |

These sources are mostly secondary; each choice is confirmed by ALIV's own benchmark before adoption.

