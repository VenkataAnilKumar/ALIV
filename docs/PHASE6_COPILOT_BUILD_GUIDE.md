# Phase 6: Release Candidate — Copilot Build Guide

## What This Phase Covers

Phase 6 prepares ALIV for controlled external usage. Phases 0–5 are complete:
- Phase 0: Foundation (Tauri 2 skeleton, SQLite WAL, migrations 0001–0008)
- Phase 1: Ingestion pipeline (200/200 files, success_rate=1.0)
- Phase 2: RRF hybrid retrieval, extraction, memory layer (PRD-004)
- Phase 3: Safe file actions (7 handlers, approval gate, undo, audit log)
- Phase 4: E2E hardening (4 defects closed, two consecutive benchmark passes)
- Phase 5: Full React + TypeScript + Vite frontend (8 screens, dark glassmorphism UI)

**Your job in Phase 6**: bundle the app for distribution, add structured logging and a diagnostics export, build a first-run onboarding flow, add a React error boundary, and verify all critical paths pass in a packaged build.

---

## Part 1: Context and Current State

### Stack
| Layer | Technology |
|---|---|
| Frontend | React 18 + TypeScript + Vite — `src/` |
| Backend | Rust + Tauri 2 — `src-tauri/src/` |
| Database | SQLite WAL + sqlite-vec — `db/migrations/` |
| AI/ML | Python subprocess bridge — `python/workers/` |
| Local LLM | Ollama `llama3.2:3b` at `localhost:11434` |

### Key files to read first
- `src-tauri/tauri.conf.json` — currently `bundle.active: false`, needs full bundler config
- `src-tauri/Cargo.toml` — current deps (no `log`/`tracing` crate yet)
- `src-tauri/src/lib.rs` — Tauri builder setup and all registered commands
- `src-tauri/src/state.rs` — AppState struct
- `src/App.tsx` — RouterProvider, needs error boundary wrapper
- `src/router.tsx` — all 8 routes (hash router)
- `src/components/layout/Shell.tsx` — main shell, Ollama health check, keyboard shortcuts
- `src/stores/workspaceStore.ts` — workspace path, ollamaUrl, modelName

### Current tauri.conf.json state
```json
{
  "bundle": { "active": false }
}
```
This must be updated to enable the bundler with proper targets.

### Current icon state
`src-tauri/icons/icon.ico` exists (single Windows ICO).
A proper icon set is needed: 32x32.png, 128x128.png, 128x128@2x.png, icon.icns, icon.ico.

---

## Part 2: Deliverables

1. **Bundler config** — `tauri.conf.json` updated with full bundle targets (NSIS for Windows, DMG for macOS, AppImage for Linux), proper icon references, app metadata.
2. **Structured logging** — Tauri plugin `tauri-plugin-log` wired; logs written to platform app-data dir; log level configurable.
3. **Diagnostics export command** — new Tauri command `get_diagnostics_report` that returns system info, Ollama reachability, DB stats, recent log tail.
4. **Diagnostics UI** — a `DiagnosticsPage` (new route `/diagnostics`) with export-to-file button.
5. **React error boundary** — catches frontend crashes, shows a recovery screen, exports state to diagnostics.
6. **First-run onboarding** — detects fresh install (no workspace registered, no files indexed), shows a modal walkthrough covering: register workspace → check Ollama → run first ingestion.
7. **Release notes** — `CHANGELOG.md` at repo root: version 0.1.0, known limitations, upgrade notes.
8. **Critical path test checklist** — `docs/PHASE6_RELEASE_CHECKLIST.md` with manual verification steps.

---

## Part 3: Step-by-Step Implementation

### Step 1 — Update `tauri.conf.json` for bundling

Replace the `bundle` block with:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "ALIV",
  "version": "0.1.0",
  "identifier": "com.aliv.desktop",
  "build": {
    "beforeDevCommand": "npm run dev -- --host",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "ALIV — Legal Intelligence Workbench",
        "width": 1400,
        "height": 960,
        "minWidth": 1100,
        "minHeight": 720,
        "resizable": true,
        "decorations": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": "default-src 'self'; connect-src 'self' http://localhost:11434 http://127.0.0.1:11434; img-src 'self' data: blob:; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "resources": [],
    "externalBin": [],
    "copyright": "© 2026 ALIV",
    "category": "Productivity",
    "shortDescription": "Local-first document intelligence for legal and finance.",
    "longDescription": "ALIV transforms a folder of local PDFs, Word docs, and contracts into an autonomous knowledge worker — without ever uploading data to the cloud.",
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": "",
      "wix": null,
      "nsis": {
        "installMode": "perMachine",
        "languages": ["English"],
        "displayLanguageSelector": false
      }
    },
    "macOS": {
      "frameworks": [],
      "minimumSystemVersion": "10.15",
      "exceptionDomain": "",
      "signingIdentity": null,
      "providerShortName": null,
      "entitlements": null
    }
  }
}
```

The `csp` field is critical — it explicitly allows `localhost:11434` for the Ollama health check in `Shell.tsx`.

### Step 2 — Generate icon set

Run Tauri's icon generator from the repo root. It needs a single 1024x1024 PNG source:

```bash
# If src-tauri/icons/app-icon.png doesn't exist, create a placeholder first:
# cargo tauri icon src-tauri/icons/app-icon.png
```

For now, create the required PNG files as minimal placeholders so the build doesn't fail:

**Create `src-tauri/icons/32x32.png`** — 32×32 cyan square placeholder
**Create `src-tauri/icons/128x128.png`** — 128×128 cyan square placeholder
**Create `src-tauri/icons/128x128@2x.png`** — 256×256 cyan square placeholder

The existing `icon.ico` can stay. For macOS, `icon.icns` is optional for development builds.

The simplest approach: copy `icon.ico` to satisfy the build, then replace with proper artwork before public release. If `cargo tauri icon` is available, use it with any 1024×1024 PNG source.

### Step 3 — Add structured logging (Rust side)

Add `tauri-plugin-log` to `src-tauri/Cargo.toml`:

```toml
[dependencies]
tauri-plugin-log = "2"
log = "0.4"
```

Wire it in `src-tauri/src/lib.rs`:

```rust
use tauri_plugin_log::{Target, TargetKind};

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: Some("aliv".into()) }),
                ])
                .level(log::LevelFilter::Info)
                .build(),
        )
        // ... rest of setup
}
```

Add `log::info!()` calls at key points in commands:
- `register_workspace_folder` on success
- `run_ingestion` on start and completion
- `execute_actions` on each rename
- `undo_action_batch` on each reversal
- Any error paths that currently only return `Err(string)`

### Step 4 — Add `get_diagnostics_report` Tauri command

Create `src-tauri/src/commands/diagnostics.rs`:

```rust
use serde_json::{json, Value};
use tauri::State;
use crate::infra::db::Database;
use crate::state::AppState;

#[tauri::command]
pub fn get_diagnostics_report(
    db: State<Database>,
    app_state: State<AppState>,
) -> Result<String, String> {
    // 1. DB stats: file count, chunk count, embedding count, memory fact count
    let file_count: i64 = db.conn()
        .query_row("SELECT COUNT(*) FROM files", [], |r| r.get(0))
        .unwrap_or(0);
    let chunk_count: i64 = db.conn()
        .query_row("SELECT COUNT(*) FROM chunks", [], |r| r.get(0))
        .unwrap_or(0);
    let embedding_count: i64 = db.conn()
        .query_row("SELECT COUNT(*) FROM embeddings", [], |r| r.get(0))
        .unwrap_or(0);
    let memory_fact_count: i64 = db.conn()
        .query_row("SELECT COUNT(*) FROM memory_facts", [], |r| r.get(0))
        .unwrap_or(0);
    let recommendation_count: i64 = db.conn()
        .query_row("SELECT COUNT(*) FROM action_recommendations", [], |r| r.get(0))
        .unwrap_or(0);

    // 2. Runtime state
    let snapshot = app_state.snapshot();

    // 3. System info
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    // 4. App version from Cargo
    let version = env!("CARGO_PKG_VERSION");

    let report: Value = json!({
        "aliv_version": version,
        "platform": { "os": os, "arch": arch },
        "db": {
            "files": file_count,
            "chunks": chunk_count,
            "embeddings": embedding_count,
            "memory_facts": memory_fact_count,
            "action_recommendations": recommendation_count
        },
        "runtime": {
            "workspace_count": snapshot.workspace_count,
            "command_count": snapshot.command_count,
            "preview_queue_count": snapshot.preview_queue_count,
            "benchmark_run_count": snapshot.benchmark_run_count
        },
        "generated_at": chrono::Utc::now().to_rfc3339()
    });

    Ok(serde_json::to_string_pretty(&report).unwrap_or_default())
}
```

Register in `lib.rs`:
```rust
use commands::diagnostics::get_diagnostics_report;
// add to invoke_handler: get_diagnostics_report
```

Add to `src/lib/tauri.ts`:
```typescript
export const getDiagnosticsReport = () =>
  invoke<string>('get_diagnostics_report')
```

Add to `src/types/contracts.ts`:
```typescript
export interface DiagnosticsReport {
  aliv_version: string
  platform: { os: string; arch: string }
  db: { files: number; chunks: number; embeddings: number; memory_facts: number; action_recommendations: number }
  runtime: { workspace_count: number; command_count: number; preview_queue_count: number; benchmark_run_count: number }
  generated_at: string
}
```

### Step 5 — Add DiagnosticsPage

Create `src/pages/DiagnosticsPage.tsx`:

A single-panel page with:
- A "Generate Report" button that calls `getDiagnosticsReport()` and parses the JSON
- StatCards for: ALIV version, platform, file count, chunk count, embedding count, memory facts
- A raw JSON pre block showing the full report
- An "Export to file" button using `downloadTextFile('aliv-diagnostics.json', raw, 'application/json')`
- An "Open log folder" button using Tauri shell plugin to open the OS log directory

Add to router:
```typescript
{ path: '/diagnostics', element: <DiagnosticsPage /> }
```

Add to Sidebar nav:
```typescript
{ to: '/diagnostics', label: 'Diagnostics', icon: ClipboardList }
```

Add to Shell quick-switch palette routes array.

### Step 6 — Add React error boundary

Create `src/components/ErrorBoundary.tsx`:

```typescript
import { Component, ErrorInfo, ReactNode } from 'react'
import { downloadTextFile } from '@/lib/utils'

interface State {
  error: Error | null
}

export class ErrorBoundary extends Component<{ children: ReactNode }, State> {
  state: State = { error: null }

  static getDerivedStateFromError(error: Error): State {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('[ALIV] Uncaught frontend error:', error, info)
  }

  render() {
    if (!this.state.error) return this.props.children

    return (
      <div className="flex min-h-screen flex-col items-center justify-center bg-surface-app p-8">
        <div className="w-full max-w-xl rounded-2xl border border-red-500/20 bg-red-500/5 p-8">
          <div className="font-mono text-xs uppercase tracking-[0.18em] text-red-400">Unrecoverable Error</div>
          <h1 className="mt-4 text-2xl font-bold text-slate-50">ALIV encountered a problem</h1>
          <p className="mt-3 text-sm leading-6 text-slate-300">
            A frontend error stopped the app from rendering. Your data is safe — this is a UI-only crash.
          </p>
          <pre className="mt-6 overflow-x-auto rounded-xl border border-zinc-800 bg-zinc-950/70 p-4 font-mono text-xs leading-6 text-red-300">
            {this.state.error.message}
            {'\n\n'}
            {this.state.error.stack}
          </pre>
          <div className="mt-6 flex gap-3">
            <button
              type="button"
              className="rounded-xl bg-gradient-to-r from-cyan-300 to-teal-300 px-4 py-2.5 text-sm font-medium text-slate-950"
              onClick={() => this.setState({ error: null })}
            >
              Try to recover
            </button>
            <button
              type="button"
              className="rounded-xl border border-white/10 bg-slate-950/35 px-4 py-2.5 text-sm font-medium text-slate-100"
              onClick={() => downloadTextFile('aliv-crash.txt', `${this.state.error?.message}\n\n${this.state.error?.stack}`, 'text/plain')}
            >
              Export crash log
            </button>
          </div>
        </div>
      </div>
    )
  }
}
```

Wrap `App.tsx`:
```typescript
import { ErrorBoundary } from './components/ErrorBoundary'

export default function App() {
  return (
    <ErrorBoundary>
      <RouterProvider router={router} />
    </ErrorBoundary>
  )
}
```

### Step 7 — First-run onboarding modal

Create `src/components/OnboardingModal.tsx`:

A multi-step modal (3 steps) shown once when:
- `activeWorkspace` is undefined AND
- `runtime.workspace_count === 0`

The modal should NOT be dismissible via Escape — the user must complete or explicitly skip.

**Step 1 — Welcome**
- Title: "Welcome to ALIV"
- Body: explain local-first, no cloud, legal/finance use case
- Primary: "Get started →"

**Step 2 — Register workspace**
- Title: "Set your workspace"
- Body: text input for folder path
- Hint: "This is the folder ALIV will watch, index, and act on. Use a dedicated legal matter folder."
- Primary: "Register & continue →" (calls `registerWorkspace` + `getRuntimeSnapshot`)
- Skip link: "Skip for now"

**Step 3 — Check Ollama**
- Title: "Confirm Ollama is running"
- Body: shows Ollama status pill (same logic as Shell.tsx health check)
- Shows: `ollama pull llama3.2:3b` command in a code block
- Primary: "Open ALIV →" (closes modal, saves `onboarded: true` to localStorage)

Wire in `Shell.tsx`:
```typescript
const [showOnboarding, setShowOnboarding] = useState(() => {
  return !localStorage.getItem('aliv_onboarded') && !activeWorkspace
})
// render: {showOnboarding && <OnboardingModal onComplete={() => { localStorage.setItem('aliv_onboarded', '1'); setShowOnboarding(false) }} />}
```

### Step 8 — Sidebar update for Diagnostics

In `src/components/layout/Sidebar.tsx`, add the Diagnostics entry at the end of the `items` array:
```typescript
{ to: '/diagnostics', label: 'Diagnostics', icon: ClipboardList }
```

Import `ClipboardList` from `lucide-react`.

---

## Part 4: Files to Create or Modify

| File | Action |
|---|---|
| `src-tauri/tauri.conf.json` | Update bundle config, CSP, window min-size |
| `src-tauri/Cargo.toml` | Add `tauri-plugin-log`, `log` |
| `src-tauri/src/lib.rs` | Wire log plugin, register `get_diagnostics_report` |
| `src-tauri/src/commands/diagnostics.rs` | **Create** — diagnostics report command |
| `src-tauri/src/commands/mod.rs` | Add `pub mod diagnostics` |
| `src/types/contracts.ts` | Add `DiagnosticsReport` interface |
| `src/lib/tauri.ts` | Add `getDiagnosticsReport` wrapper |
| `src/pages/DiagnosticsPage.tsx` | **Create** — diagnostics UI |
| `src/components/ErrorBoundary.tsx` | **Create** — React error boundary |
| `src/components/OnboardingModal.tsx` | **Create** — first-run flow |
| `src/components/layout/Sidebar.tsx` | Add Diagnostics nav entry |
| `src/components/layout/Shell.tsx` | Wire onboarding modal |
| `src/App.tsx` | Wrap with ErrorBoundary |
| `src/router.tsx` | Add `/diagnostics` route |
| `CHANGELOG.md` | **Create** — version 0.1.0 release notes |
| `docs/PHASE6_RELEASE_CHECKLIST.md` | **Create** — manual verification checklist |

---

## Part 5: Rust Side — `state.rs` snapshot method

`get_diagnostics_report` calls `app_state.snapshot()` which must return `RuntimeSnapshot`. Verify that method exists in `src-tauri/src/state.rs`. If it is not a public method, read the file and expose the relevant fields. If `snapshot()` does not exist, use the individual state fields directly in the command.

---

## Part 6: CHANGELOG.md Format

```markdown
# Changelog

## [0.1.0] — 2026-04-03

### What's new
- Full local-first document intelligence for legal and finance teams
- Ingestion pipeline: PDF, Word, email parsing with OCR fallback (Tesseract 5)
- RRF hybrid retrieval (55% lexical + 45% semantic) with citation binding
- Structured extraction for 5 field templates (lease, contract, invoice, etc.)
- Safe file actions: preview-first rename with approval gate, undo, audit log
- Persistent memory layer: session/week/permanent facts with token-budget context assembly
- Benchmark suite with pass/fail gates across ingestion, retrieval, and extraction

### Known limitations
- Ollama must be running locally before starting ALIV (`ollama serve`)
- First ingestion of large folders (>500 files) may take several minutes
- sqlite-vec ANN search requires Python sqlite-vec wheel (auto-installed from requirements.txt)
- macOS code signing not configured for this release — Gatekeeper bypass required
- Windows NSIS installer not code-signed — SmartScreen warning expected

### Upgrade notes
- This is the initial release — no migration required
- DB is stored in platform app-data dir; uninstall does not delete user data

### System requirements
- OS: Windows 10+, macOS 11+, Ubuntu 22+
- RAM: 8 GB minimum, 16 GB recommended (for LLM inference)
- Disk: 2 GB for app + model storage
- Ollama: v0.3+ with `llama3.2:3b` pulled
```

---

## Part 7: `docs/PHASE6_RELEASE_CHECKLIST.md` Format

Create this file with the following manual test steps. Each must be checked before marking Phase 6 complete:

```markdown
# Phase 6 Release Checklist

## Build and Install
- [ ] `cargo tauri build` completes without error on Windows
- [ ] Installer runs cleanly on a clean Windows machine
- [ ] App launches after install
- [ ] App version shows "0.1.0" in Diagnostics page

## Critical Paths
- [ ] Register workspace → runtime shows workspace_count = 1
- [ ] Run full ingestion → success_rate >= 0.98
- [ ] Run query → returns hits with citations
- [ ] Run extraction → returns field rows
- [ ] Generate action recommendations → preview table populates
- [ ] Approve + execute actions → undo banner appears
- [ ] Undo batch → audit shows rolled_back entries
- [ ] Save memory fact → appears in Memory page
- [ ] Run benchmark suite → ingestion, retrieval, extraction all pass

## Onboarding
- [ ] Fresh install (no localStorage) shows onboarding modal
- [ ] Completing onboarding sets workspace and dismisses modal
- [ ] Onboarding does not reappear after reload

## Error Handling
- [ ] Ollama offline → TopBar shows "Ollama offline" badge (red)
- [ ] Execute actions without approve → store returns "Approve actions before execution"
- [ ] Invalid workspace path → error banner visible in Workspace page

## Diagnostics
- [ ] Generate Report returns valid JSON with db.files > 0 after ingestion
- [ ] Export to file downloads aliv-diagnostics.json

## Logging
- [ ] Log file created in platform app-data dir after first launch
- [ ] `register_workspace_folder` success appears in logs

## Performance
- [ ] Query latency p95 < 8 seconds on reference machine
- [ ] App startup (cold) < 5 seconds
```

---

## Part 8: Build Order

1. Update `tauri.conf.json` (bundle config + CSP)
2. Add `tauri-plugin-log` to Cargo.toml and wire in `lib.rs`
3. Create `commands/diagnostics.rs` + register command
4. Update TypeScript contracts + `lib/tauri.ts`
5. Create `DiagnosticsPage.tsx`
6. Add Diagnostics route + sidebar entry
7. Create `ErrorBoundary.tsx` + wrap `App.tsx`
8. Create `OnboardingModal.tsx` + wire in `Shell.tsx`
9. Run `cargo tauri build` — fix any compile errors
10. Create `CHANGELOG.md`
11. Create `docs/PHASE6_RELEASE_CHECKLIST.md`
12. Manually work through the release checklist — record pass/fail

---

## Part 9: Exit Criteria (Gate)

All of the following must be true before Phase 6 is marked complete:

| Criterion | How to verify |
|---|---|
| `cargo tauri build` succeeds | No compile errors, installer artifact exists in `src-tauri/target/release/bundle/` |
| DiagnosticsPage renders and exports JSON | Manual test in running app |
| ErrorBoundary catches and shows recovery screen | Temporarily throw in a component, confirm boundary catches it |
| Onboarding modal appears on fresh localStorage | Clear `aliv_onboarded` from devtools, reload |
| All critical path checklist items pass | Walk through `PHASE6_RELEASE_CHECKLIST.md` |
| No open critical defects | Zero items with `severity: critical` and `status: open` |
| CHANGELOG.md committed at repo root | `git log --oneline` shows it |

---

## Part 10: Web Search and Reference Permissions

You are allowed to search the web and copy reference code during this phase. Follow these rules:

| Need | Search query |
|---|---|
| Tauri plugin-log setup | `tauri-plugin-log v2 setup Cargo.toml` |
| Tauri bundle config options | `tauri.conf.json bundle targets nsis dmg 2024` |
| React error boundary class component | `react error boundary class component typescript 2024` |
| Tauri CSP for localhost fetch | `tauri v2 csp allow localhost fetch` |
| React multi-step modal wizard | `react multi step modal wizard typescript` |

**Trusted sources:**
- `tauri.app/v2/` — official Tauri v2 docs
- `github.com/tauri-apps/plugins-workspace` — official plugin source
- `react.dev` — React official docs
- `github.com/shadcn-ui/ui` — component reference

**Rules when copying:**
1. Adapt to ALIV's design tokens (surface.*/accent.*)
2. Keep TypeScript strict — no `any`
3. Add `// source: <url>` comment on the first line of the copied block
4. Do not copy license-incompatible code (MIT/Apache only)
