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

## 6. Upstream reuse: OpenGeni and OpenAgents

Reviewed October 29, 2026 at **OpenGeni** [`Cloudgeni-ai/opengeni@6c5a6b5`](https://github.com/Cloudgeni-ai/opengeni/tree/6c5a6b5284d2c818f9b5ba62d43f780480bb0eac) and **OpenAgents** [`openagents-org/openagents@c94dcbd`](https://github.com/openagents-org/openagents/tree/c94dcbd91b9680ef6b50e76fe6e074114dba836a). Both are Apache-2.0.

**Decision:** neither project is a dependency. OpenGeni's service needs Bun/TypeScript, Postgres, Temporal, NATS and Docker; OpenAgents' SDK is Python and its shared workspace is hosted. ALIV is Rust on one local SQLite file. We reuse **designs**, and port **code** only where it is Rust, self-contained and clearly better than writing our own.

**Reuse levels:** 1 = design rebuilt in Rust · 2 = code ported with attribution · 3 = run as a service · 4 = protocol interop only.

| Upstream piece | What it does | ALIV target | Level | When |
|---|---|---|---|---|
| OpenGeni `docs/human-input.md`, `packages/runtime` (`request_human_input`) | A step asks structured questions and resumes the **same** tool call; outcomes *answered / skipped / let the agent decide / expired / cancelled* | `aliv-agents`, AGT-08 | 1 | R2 (S6) |
| OpenGeni `docs/tool-approvals.md`, `packages/db` action policy | Per-tool Allow / Ask / Block; most specific rule wins, ties Block; arguments can't select a looser rule; prepared actions keep their decision | `aliv-gateway` + `aliv-mcp`, MCP-08 | 1 | R1 (S5) |
| OpenGeni event history ("every event stored; live streams backfill from it") | Interrupt, resume, replay from durable events | `aliv-agents` event-sourced runs in SQLite, AGT-09 | 1 | R2 (S6) |
| OpenGeni `docs/goals.md` | Goal-driven sessions that continue until completed with evidence or paused | `aliv-agents`, AGT-10 | 1 | R3 (S19) |
| OpenGeni `docs/tool-search.md` | Small visible tool set; load more by name or meaning (keyword + embeddings fused, as in ALIV search) | `aliv-mcp`, MCP-09 | 1 | R3 |
| OpenGeni `packages/tool-gateway` catalog digests | Tool catalog hashed so the approved set is pinned and auditable | `aliv-mcp`: record a digest of each matter's allowed tool catalog in the audit log | 1 | R1 |
| OpenGeni `packages/decisions` typed judgments | Predicate / choice / score answers with probabilities and refusal | Review-table answer types and confidence (TBL-02/03), evaluation judge (EVAL-03) | 1 | R2 |
| OpenGeni `agent/crates/opengeni-agent-update` (Rust, ~2.5k lines) | Self-update: signed manifest, minisign + sha256, version monotonicity, staged rollout, atomic swap, rollback | Desktop updater (GA track) | **Superseded:** ALIV uses Tauri's official `tauri-plugin-updater` (already minisign-signed); OpenGeni's staged-rollout and rollback ideas are design reference only | GA |
| OpenGeni `agent/` Runner + `opengeni-relay` (Rust) | Machines only dial out; no durable credentials on the machine | Desktop ↔ Firm Server link | 1, possibly 2 | R4 |
| OpenAgents A2A connector, converters, task store (Python) | Agent-to-agent task exchange | A2A in Rust against the spec; use their tests as reference cases | 4 | R4 option |
| OpenAgents network exposed as an MCP server | Network tools over MCP | Compare with ALIV as an MCP server (MCP-06) | 1 | R4 |
| OpenAgents hosted workspace, launcher daemon | Shared cloud workspace for coding agents | — | ✕ (conflicts with Local only) | — |

**Rules for any ported code (level 2):**
- record the upstream path and commit in a header comment, keep the original copyright notice, and mark ALIV's changes;
- add the component to `THIRD_PARTY_NOTICES.md` (Apache-2.0), including any upstream NOTICE text;
- port only what is needed; add ALIV tests and run it through the same security review as our own code (the updater and sandbox especially);
- never take on upstream infrastructure (Postgres, Temporal, NATS, Docker) as a desktop requirement.

## 7. Hybrid architecture: four planes

| Plane | Runs where | Holds client data? | Technology |
|---|---|---|---|
| **1. Local** (desktop) | User's computer | Yes, the matter vault | Tauri 2, Rust, React, SQLite (+ SQLCipher), llama.cpp / MLX |
| **2. Firm** (Firm Server, R4) | Firm's server room or **the firm's own cloud tenancy** | Yes, shared matters | Same Rust crates, axum, PostgreSQL + pgvector, Docker / Helm, Terraform (**Azure first**, then AWS, GCP) |
| **3. Model** (gateway) | Inside planes 1 and 2 | Passes only what policy allows | Rust gateway: policy, routing, redaction, audit; adapters via `genai` for local engines, OpenAI-compatible, Anthropic, Azure OpenAI, AWS Bedrock, Google, open-model hosts; OpenRouter only as an explicitly allowed cloud provider |
| **4. ALIV Cloud** (control plane) | ALIV's cloud | **Never** | Update server, licences and accounts, model and skill catalogue, optional connection relay (machines only dial out), opt-in crash reports; small API + managed PostgreSQL on Azure, Terraform |

An optional **managed ALIV cloud** (a dedicated tenant per firm, firm-held keys) may follow R4; it is not required by the plan.

## 8. Data and supporting technology

| Need | Desktop | Firm Server | ALIV Cloud |
|---|---|---|---|
| Database | **SQLite** (WAL); one file per matter for very large vaults; **SQLCipher** encryption (GA) | **PostgreSQL 16+** with **pgvector**, row-level security per matter, point-in-time backups; SQLite allowed for < 20 users | Managed PostgreSQL (no client data) |
| Data access | **SQLx** for both SQLite and PostgreSQL behind one repository interface in `aliv-vault` | same | — |
| Keyword search | SQLite FTS5 | PostgreSQL full-text or **Tantivy** | — |
| Vectors | Exact search now; **LanceDB** or **sqlite-vec** by benchmark (R2); int8/binary compressed vectors | **pgvector** (HNSW) | — |
| Files | Read in place | Read-only mounts or connectors; outputs in S3-compatible storage (MinIO on-prem, Azure Blob in cloud) | Release artifacts |
| Jobs and schedules | Tokio + job table; **apalis**, **tokio-cron-scheduler** | Same code on PostgreSQL (`SKIP LOCKED` workers) | — |
| UI updates | Tauri IPC events | Server-Sent Events / WebSockets | — |
| Secrets | OS keychain | Azure Key Vault / AWS KMS / HashiCorp Vault, firm-held keys (BYOK) | Cloud KMS |
| Identity | Local user | OIDC / SAML (Entra ID, Okta, Google); roles admin / member / viewer | Accounts |
| Transport | — | TLS 1.3 (rustls) | TLS 1.3 |
| Supply chain | Signed updates (`tauri-plugin-updater`) | Signed images (**cosign**), SBOMs (**syft**, `cargo-auditable`) | Same |
| Observability | OpenTelemetry (no content), opt-in crash reports | OTel to the firm's own tools (Grafana, Datadog, Azure Monitor) | Same |

**SQLite scaling:** a 10,000-document matter is roughly 0.5–1 million passages and 2–5 GB, well within SQLite's limits. Its real limits are concurrent writers and network-shared files, so multi-user deployments use the Firm Server on PostgreSQL; a vault file must never be shared over a network drive.

**Deliberately not used:** MongoDB and other NoSQL stores (legal data is relational); Elasticsearch / OpenSearch (heavy to run); Pinecone and other hosted vector databases, Firebase / hosted Supabase (client data in a third-party cloud); Redis, Kafka, Temporal, NATS (not needed at ALIV's scale); semantic caching shared across matters (could leak one matter into another; any cache is per matter and per policy).

## 9. Agent framework decision

ALIV builds on the **existing Rust agent runtime** (`src-tauri/src/core/agent`, about 46k lines: loop, sub-agents, plan, goals, compaction, memory, skills, hooks, OTel), evolving it into `crates/aliv-agents`, with every model and tool call going through `aliv-gateway`.

| Library | Use |
|---|---|
| `genai` (Rust) | Model provider adapters (in use) |
| `rmcp` (Rust, official MCP SDK) | MCP client, later ALIV as an MCP server (in use) |
| `cedar-policy` (Rust) | Tool and agent policy, Allow / Ask / Block (R1) |
| `rig-core` (Rust) | Evaluate in R2 for tool, embedding or extraction helpers |
| A2A SDK | Agent-to-agent interop (R4) |
| LangGraph, CrewAI, OpenAI / Claude Agent SDKs, Microsoft Agent Framework, Google ADK | **Not in the shipped app** (Python/TS runtimes). Allowed in CI, evals and prototypes, or as a firm's external agents connecting through MCP or A2A. Their designs inform `aliv-agents` (checkpointed step graphs, handoffs, guardrails) |

**Conditions for any agent library:** all model and tool calls go through the gateway; built-in telemetry off (e.g. LangSmith, CrewAI telemetry); permissive licence and active maintenance; runs locally; wrapped behind ALIV interfaces so it can be replaced; recorded here and in `THIRD_PARTY_NOTICES.md`.

**OpenGeni as a foundation:** not adopted. Re-evaluate at R4 only for an optional managed-cloud edition, once it has tagged releases and stable APIs.

## 10. Reuse policy and component map

| Kind | Rule |
|---|---|
| Whole products and platforms (OpenGeni, OpenAgents, LiteLLM, Portkey, Bifrost, TensorZero, OpenRouter) | **Reference specific parts only**: read the code, write ALIV's own version; small ported pieces keep their notice |
| Building-block libraries, including agent-framework libraries | **Use when they help**, under the conditions in §9 |
| External services that receive data | **No**, except cloud models through the gateway on Hybrid matters |

| Part of ALIV | Library to use | Licence |
|---|---|---|
| Tool policy | Cedar (`cedar-policy`) | Apache-2.0 |
| Signed auto-update | `tauri-plugin-updater` | MIT / Apache |
| Skill-script sandbox | Wasmtime; Landlock on Linux | Apache-2.0 |
| Vault encryption | SQLCipher via `rusqlite` | BSD |
| Data access | SQLx | MIT / Apache |
| Jobs and schedules | apalis, tokio-cron-scheduler | MIT / Apache |
| Server search | Tantivy | MIT |
| Vectors | LanceDB or sqlite-vec (benchmark); pgvector on server | Apache / MIT / PostgreSQL |
| Embeddings, rerankers, PII model | llama.cpp (in use); `ort` (ONNX Runtime), fastembed-rs; GLiNER-style PII model | MIT / Apache |
| OCR | Tesseract (in use); evaluate pure-Rust `ocrs` | Apache / MIT |
| Layout parsing | Docling / Granite-Docling or olmOCR (benchmark) | MIT / Apache |
| Word, Excel, PDF output | `docx-rs`, `rust_xlsxwriter`, Typst | MIT, MIT/Apache, Apache |
| Compare | `similar` | Apache-2.0 |
| Server and auth | axum, tower, `openidconnect`, `samael` or Keycloak / Zitadel | MIT / Apache |
| Tracing | `opentelemetry-rust` | Apache-2.0 |
| Review grid | TanStack Table + TanStack Virtual | MIT |
| Source viewer | PDF.js (`react-pdf`) | Apache-2.0 |
| Drafting editor | Tiptap | MIT |
| Agent Builder canvas | React Flow (`@xyflow/react`) | MIT |
| Command palette | cmdk | MIT |
| Evaluations (CI only) | promptfoo; Ragas | MIT / Apache |
| Word add-in | Office.js | MIT |

Every adoption is checked for licence and maintenance first and recorded in `THIRD_PARTY_NOTICES.md`.

**Gateway reference designs** (reference only): Portkey AI Gateway (MIT) for fallback chains, retries and guardrail hooks; LiteLLM for provider mappings and a model metadata table (context window, price, capabilities; data used only after a licence check); Bifrost (Apache-2.0) for budgets; TensorZero for a Rust gateway design (reportedly archived in 2026). OpenRouter is proprietary: usable only as an optional cloud provider, explicitly allowed by the firm and subject to ZDR and region rules.

