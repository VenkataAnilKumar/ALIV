# System Architecture
## Components, flows and non-functional requirements

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active. R0 is as built; R1+ is the target design.

---

## 1. Context

```
                ┌───────────────────────── Firm's machine ─────────────────────────┐
  User ───────▶ │  ALIV desktop app                                                 │
                │   ├─ Web UI (React)  ──IPC──▶  Rust core (Tauri)                  │
                │   │                              ├─ Vault (SQLite)                │
                │   │                              ├─ Local engines (llama.cpp/MLX) │
                │   │                              └─ OCR (Tesseract, pdftoppm)     │
                │  Documents on disk / file shares (read in place)                  │
                └──────────────────────────────┬────────────────────────────────────┘
                                               │ Hybrid matters only (R1: via gateway, redacted)
                                               ▼
                                Cloud model providers (OpenAI-compatible, Anthropic)
```

**Later deployments:** the Firm Server (R4) hosts shared vaults for many desktops. See [Deployment Architecture](ALIV_Deployment-Architecture.md).

## 2. Components

| Component | Where | Responsibility | State |
|---|---|---|---|
| **Matters UI** | `web-app/src/aliv/` | Matters, Ask, Register, Documents, Approvals, Audit, Vault health | ✅ R0 |
| **ALIV service** | `web-app/src/aliv/service.ts`, `tauri-service.ts`, `demo-service.ts` | Typed client for vault commands; demo implementation for the browser build | ✅ |
| **Ask orchestration** | `web-app/src/aliv/ask.ts` | Search → context budget → policy check → model call → citations | ✅ (moves to the gateway in R1) |
| **Meaning index** | `web-app/src/aliv/semantic.ts`, `useMeaningIndex.ts` | Embeds pending passages with the local embedder; stores vectors in the vault | ✅ |
| **Vault plugin** | `src-tauri/plugins/tauri-plugin-aliv-vault` | Matters, ingestion, extraction, OCR, passages, FTS5 + vector search with RRF, register, summaries, actions, policy decisions, audit chain, diagnostics | ✅ |
| **Local engines** | `tauri-plugin-llamacpp`, `tauri-plugin-mlx`, `extensions/` | Local chat and embeddings | ✅ (from the base app) |
| **Hardware** | `tauri-plugin-hardware` | GPU/RAM detection for model choice | ✅ |
| **CLI / TUI** | `src-tauri/aliv-cli` | Terminal agent and tools | ✅ (base) |
| **Model Gateway** | Rust core (new crate) | The only path to any model: policy, routing, redaction, provider adapters, audit | R1 |
| **Router** | Gateway | Step type → model and location | R1 |
| **Redactor** | Gateway | Entity detection, placeholders, restore | R1 |
| **Agent runtime** | Rust core | Runs tables, workflows and agents step by step; resumable | R2 |
| **Scheduler** | Rust core | Monitors, watch folders | R3 |
| **Firm Server** | Separate binary reusing the vault crate | Shared vaults, users, roles, SSO | R4 |

## 3. Key flows

### 3.1 Ingestion (R0)
1. The UI calls `ingest_paths(matter, paths)`.
2. The vault walks the folders and hashes each file (sha256). Unchanged files are skipped.
3. Text is extracted (PDF, DOCX, text). Scans and images go to OCR, with low-resolution pages upscaled.
4. Text is split into passages (`chunks`), and the FTS5 index updates via triggers.
5. Fields are extracted (type, parties, dates, amounts, notice, law, penalty) with confidence and a snippet.
6. An extractive summary is written. An audit entry records counts.
7. The UI's meaning indexer then pulls `pending_embeddings`, embeds locally and calls `store_embeddings`.

### 3.2 Ask (R0)
1. `search(matter, query, vector?)` runs FTS5 BM25 and vector cosine, fused by RRF (keyword weight 0.55, k = 60).
2. The context budget picks the top passages that fit the model, and counts those left out.
3. `authorize_model_call` checks the matter policy against the model's location. It allows or refuses and writes an audit entry.
4. The model streams an answer with [n] citations, which are linked to sources.
5. Deadline-style questions can be answered directly from the register, with no model.

### 3.3 Ask via the gateway (R1)
1. The UI sends a `ModelRequest{matter, step_type, sources, instructions}` over IPC.
2. The gateway runs the policy check, routes the request, and redacts it if the route is cloud.
3. It calls the provider adapter, streams the result, restores placeholders, writes `model_calls` and the audit entry, and returns a stream.
4. The web layer never holds keys or calls providers. A CI check enforces this.

### 3.4 Review table run (R2)
1. The agent runtime creates a run with one step per (column, row) cell batch.
2. For each cell it retrieves passages from that document, routes the request (extraction defaults to local), calls the gateway and verifies the citation.
3. It writes the cell with value, confidence and citation. Progress is persisted, so a restart resumes at the first unfinished cell.

### 3.5 File action (R0)
`propose_renames` → user approves (`set_action_status`) → `execute_approved` (no overwrite) → `undo_action`. Every step is audited.

## 4. Non-functional requirements

| Area | Requirement |
|---|---|
| **Privacy** | Documents are never copied or uploaded on import. Embeddings are always local. Local only matters make zero cloud calls (tested) |
| **Integrity** | Audit is an append-only SHA-256 chain; `verify_audit` names the first broken entry |
| **Performance** | Ingest ≥ 20 text pages/s on a 2023 laptop (OCR slower); search < 300 ms on 10k documents; first local token < 3 s (7–8B model) |
| **Scale** | Desktop: 10k documents / 1M passages per vault. Firm Server: 20+ concurrent users (R4) |
| **Reliability** | Long jobs (ingestion, tables, agent runs) are resumable after a crash or restart |
| **Portability** | macOS (Apple silicon, Intel), Windows 10+, Linux (x86_64) |
| **Offline** | Fully functional offline for Local only matters with downloaded models |
| **Observability** | Vault health report (counts and reasons, no text); opt-in crash reports without document data (GA) |
| **Accessibility** | WCAG 2.1 AA (see Design System) |

## 5. Constraints and decisions

- **One SQLite file per installation (R0).** The schema is scoped by `matter_id`. The Firm Server will use the same crate with a server database (see Deployment).
- **Rust owns data and policy; the web layer owns presentation.** Anything that must be enforced lives in Rust.
- **Agents propose, people approve.** No autonomous writes to files or outbound messages.
- **No vendor lock-in for models.** Supports llama.cpp, MLX and any OpenAI-compatible or Anthropic endpoint.

Related: [Hybrid AI Architecture](ALIV_Hybrid-AI-Architecture.md) · [Architecture Stack](ALIV_Architecture-Stack.md) · [Data Model](ALIV_Data-Model.md) · [API Design](ALIV_API-Design.md) · [Security & Compliance](ALIV_Security-Compliance.md)
