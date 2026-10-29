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
| **Vectors in SQLite, brute-force cosine** | sqlite-vec, LanceDB, Qdrant | Simple and exact at desktop scale (≤ 1M passages); no extra native dependency. Revisit with an ANN index if latency exceeds budget |
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
