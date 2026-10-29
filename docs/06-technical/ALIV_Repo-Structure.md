# Repo Structure
## Where everything lives

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active

---

## 1. Top level

```
ALIV/
├─ README.md                 Project overview and quick start
├─ .github/workflows/        CI (desktop.yml: web + Rust jobs)
├─ apps/desktop/             The ALIV desktop app (Tauri 2 + React)
├─ benchmarks/               Extraction benchmark corpus and scorer
├─ demo/                     Demo material
└─ docs/                     Product documents, guides, background, archive
```

## 2. `apps/desktop`

```
apps/desktop/
├─ web-app/                  React 19 + Vite UI
│  └─ src/
│     ├─ aliv/               ALIV feature code
│     │  ├─ components/      MattersPage, MatterPage, Ask/Register/Documents/
│     │  │                   Approvals/Audit/Overview panels, HealthDialog,
│     │  │                   MatterDialog, primitives.tsx
│     │  ├─ service.ts       Service interface; tauri-service.ts / demo-service.ts
│     │  ├─ ask.ts           Ask orchestration (search, budget, policy, citations)
│     │  ├─ semantic.ts      Meaning index (local embeddings)
│     │  ├─ csv.ts, dates.ts, types.ts, vault.ts, use*.ts
│     │  ├─ demo/fixture.ts  Demo data for the browser build
│     │  └─ __tests__/       Vitest tests
│     ├─ components/ui/      Shared shadcn-style components
│     ├─ routes/             TanStack Router routes
│     ├─ index.css           Design tokens (light/dark)
│     └─ …                   Base app: chat, models, settings, providers, hooks
├─ src-tauri/                Rust core
│  ├─ src/                   App entry (main.rs, lib.rs) and core modules
│  ├─ plugins/
│  │  ├─ tauri-plugin-aliv-vault/   ALIV vault: store, ingest, extract, OCR,
│  │  │                             chunking, search, register, actions, audit
│  │  ├─ tauri-plugin-llamacpp/     Local llama.cpp engine
│  │  ├─ tauri-plugin-mlx/          MLX engine (Apple silicon)
│  │  ├─ tauri-plugin-hardware/     Hardware detection
│  │  ├─ tauri-plugin-rag/          Document parsing used by the vault
│  │  ├─ tauri-plugin-vector-db/    Vector store (base app)
│  │  ├─ tauri-plugin-agent-tools/  Agent tool plumbing (base app)
│  │  └─ tauri-plugin-websearch/    Web search tool (base app)
│  ├─ aliv-cli/              Terminal agent / TUI
│  ├─ capabilities/          Tauri permission grants
│  ├─ icons*/                App icons (ALIV mark)
│  └─ tauri*.conf.json       Bundle config per platform
├─ core/                     @aliv/core: shared TS (engines, embeddings)
├─ extensions/               Engine and assistant extensions (llamacpp, mlx, rag…)
├─ packages/                 adk, streamdown (committed dist), remend
├─ adk/, examples/           Agent development kit and examples
├─ protocol/                 Protocol schemas
├─ tests/, e2e/              Integration and end-to-end tests
├─ scripts/                  Build helpers (e.g. stub-tauri-resources.mjs)
├─ Makefile                  dev, build, lint, test targets
└─ package.json, yarn.lock   Yarn 4 workspace
```

## 3. `docs`

```
docs/
├─ README.md                 Index of product docs and guides
├─ 01-brand/                 Brand Identity
├─ 02-product/               Product Overview, Product Definition
├─ 03-market/                Competitive Analysis
├─ 04-specs/                 Feature Spec, User Journeys, User Stories
├─ 05-design/                Design System, UI Spec, Wireframes
├─ 06-technical/             System Architecture, Hybrid AI Architecture, Stack,
│                            Data Model, API Design, Security & Compliance,
│                            Deployment, Repo Structure
├─ 07-implementation/        Implementation Phases, Sprint Plan, Checklist
├─ guides/                  User guide, security and privacy, developer guide
├─ assets/                   Screenshots
└─ archive/                  History: superseded docs and the legacy app
```

## 4. Where new code goes (R1–R4)

| Feature | Rust | Web |
|---|---|---|
| Model gateway, router, redaction (R1) | `src-tauri/src/core/gateway/`, building on the existing `core/server/proxy.rs` and `provider_secrets.rs`; split into a crate once the Firm Server needs it | `web-app/src/aliv/gateway.ts` (thin client); RoutePill, RedactionPreview in `aliv/components/` |
| Agent runtime, tables, workflows (R2) | `src-tauri/crates/aliv-agents` using the vault crate | `aliv/tables/`, `aliv/workflows/`, `aliv/library/` |
| Document writers (R2) | `src-tauri/crates/aliv-docgen` (DOCX, XLSX, PDF) | — |
| Monitors and automation (R3) | `aliv-agents::scheduler` | `aliv/inbox/`, `aliv/automations/` |
| Matter workspace, chats (R1) | vault plugin (new tables) + gateway | `aliv/workspace/`, `aliv/chats/` (replaces base-app `routes/project` for matter work) |
| Skills, MCP and web-search governance (R1) | `core/agent/skills.rs`, `core/mcp`, `tauri-plugin-websearch`, called only via `core/gateway/` | `aliv/settings/skills-tools/` |
| Artifacts (R2) | vault plugin + `aliv-docgen` | `aliv/artifacts/` (reuses base-app `HtmlArtifact` in a sandbox) |
| Connectors (R3) | `src-tauri/src/core/connectors/` | `aliv/connections/` |
| Firm Server (R4) | `apps/server/` (axum) reusing the vault and agents crates | Desktop gains a server-backed `service` implementation |
| Word add-in (R4) | — | `apps/word-addin/` (Office.js) |

**Rules:**
- feature code stays under `aliv/`, not mixed into the base app's folders;
- each new crate has unit tests plus a CI matrix entry;
- each new IPC command is added to permissions and to `tests/ipc.rs`.
