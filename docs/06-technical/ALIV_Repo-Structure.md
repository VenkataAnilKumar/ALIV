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

## 4. Target structure (from Sprint 1)

The current layout works for one desktop app. Before R4 adds a server, an admin console and a Word add-in, shared code moves out of `apps/desktop` so every app can use it:

```
ALIV/
├─ Cargo.toml                 Rust workspace: all crates and app shells, shared dependency versions
├─ package.json               Yarn workspace: all apps and packages
├─ crates/                    shared Rust domain, used by desktop and server
│  ├─ aliv-vault/             store, ingest, search, register, audit       (moved from the plugin, S2)
│  ├─ aliv-gateway/           policy, routing, redaction, adapters, tool governance   (new, S1)
│  ├─ aliv-agents/            runtime, tables, workflows, automation        (R2–R3)
│  ├─ aliv-docgen/            DOCX, XLSX, PDF                               (R2)
│  ├─ aliv-mcp/               trust levels, allow-lists, ALIV MCP server    (R1 / R4)
│  └─ aliv-connectors/        DMS, SharePoint, mail                         (R3)
├─ apps/
│  ├─ desktop/                Tauri shell + web-app; plugins are thin wrappers over crates/
│  ├─ server/                 Firm Server (R4, axum)
│  ├─ admin-console/          web admin (R4)
│  └─ word-addin/             Office.js (R4)
├─ packages/
│  ├─ ui/                     design system: tokens, pills, ReviewGrid…    (R2)
│  └─ types/                  TypeScript types generated from Rust (specta / ts-rs)
├─ evals/                     labelled sets and the eval harness (FS-29)
├─ benchmarks/  demo/  docs/
└─ .github/                   CI per area (path filters) and CODEOWNERS
```

**Migration, in small PRs:**
1. **S1:** root Cargo workspace; the gateway is created directly as `crates/aliv-gateway`.
2. **S2:** move the vault plugin's domain code to `crates/aliv-vault`; `tauri-plugin-aliv-vault` keeps only the Tauri commands (the vault crate already has `tauri` as an optional dependency). The vault's tests and CI job move with it.
3. **R2:** `packages/ui` and generated `packages/types` once a second UI exists; one lockfile per language at the root.
4. **R4:** `apps/server`, `apps/admin-console` and `apps/word-addin` slot in without restructuring.

**Base-app code** (mobile configs, `mlx-server`, `adk`, `examples`, `extensions`) stays in `apps/desktop` and is marked upstream-owned in `CODEOWNERS`, so ALIV code and upstream code stay easy to tell apart.

## 5. Where new code goes (R1–R4)

| Feature | Rust | Web |
|---|---|---|
| Model gateway, router, redaction (R1) | `crates/aliv-gateway`, building on the existing `core/server/proxy.rs` and `provider_secrets.rs` | `web-app/src/aliv/gateway.ts` (thin client); RoutePill, RedactionPreview in `aliv/components/` |
| Agent runtime, tables, workflows (R2) | `crates/aliv-agents` using `aliv-vault` | `aliv/tables/`, `aliv/workflows/`, `aliv/library/` |
| Document writers (R2) | `crates/aliv-docgen` (DOCX, XLSX, PDF) | — |
| Monitors and automation (R3) | `aliv-agents::scheduler` | `aliv/inbox/`, `aliv/automations/` |
| Matter workspace, chats (R1) | `aliv-vault` (new tables) + gateway | `aliv/workspace/`, `aliv/chats/` (replaces base-app `routes/project` for matter work) |
| Skills, MCP and web-search governance (R1) | `crates/aliv-mcp` and `aliv-gateway`, wrapping `core/agent/skills.rs`, `core/mcp` and `tauri-plugin-websearch` | `aliv/settings/skills-tools/` |
| Artifacts (R2) | `aliv-vault` + `aliv-docgen` | `aliv/artifacts/` (reuses base-app `HtmlArtifact` in a sandbox) |
| Connectors (R3) | `crates/aliv-connectors` | `aliv/connections/` |
| Evaluation (R1–R2) | `evals/` harness (Rust runner + labelled sets) | — |
| Firm Server (R4) | `apps/server/` (axum) using the same crates | Desktop gains a server-backed `service` implementation |
| Word add-in (R4) | — | `apps/word-addin/` (Office.js), shares `packages/ui` and `packages/types` |

**Rules:**
- feature code stays under `aliv/`, not mixed into the base app's folders;
- domain logic lives in `crates/`; app shells only wire it up;
- each new crate has unit tests plus a CI matrix entry;
- TypeScript types for IPC are generated from Rust, never hand-copied;
- each new IPC command is added to permissions and to `tests/ipc.rs`.
