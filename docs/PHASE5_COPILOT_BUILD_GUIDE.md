# ALIV — Phase 5 Copilot Build Guide
# Frontend: React + TypeScript + Vite + 2026 Modern UI

**Phase**: 5 — UX and Operator Experience
**Status**: Active (started 2026-04-03)
**Goal**: Build the entire React frontend from scratch. `src/` is currently empty.
**Exit Criteria**: All 8 screens functional with real Tauri backend calls, zero mocked data, passes `cargo tauri dev` and `npm run build`.

---

## Part 1: Context and Backend Summary

### What Is Built (Phases 0–4)

The entire Rust + Tauri 2 backend is complete and verified:

| Layer | Status |
|---|---|
| SQLite schema (8 migrations) | Complete |
| Ingestion pipeline (OCR, chunking, embeddings) | Complete |
| RRF hybrid retrieval + extraction templates | Complete |
| Persistent memory system (conversations, facts, summaries) | Complete |
| Safe file actions (rename/move/undo/audit) | Complete |
| E2E benchmark suite (all 4 defects closed) | Complete |

### Key Files to Read Before Starting

```
src-tauri/src/models/contracts.rs     — all Tauri command input/output types
src-tauri/src/commands/               — all available Tauri command handlers
docs/design/ALIV_PRODUCT_DESIGN.md   — 9 screens, components, states
docs/ALIV_CONTEXT_MEMORY.md          — quick backend context
src-tauri/tauri.conf.json            — app config (1400×960 window)
```

### Tauri App Config

```json
{
  "productName": "ALIV",
  "version": "0.1.0",
  "identifier": "com.aliv.desktop",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [{ "title": "ALIV", "width": 1400, "height": 960, "resizable": true }]
  }
}
```

---

## Part 2: Tech Stack

| Layer | Technology |
|---|---|
| Framework | React 18 + TypeScript |
| Build | Vite |
| Styling | Tailwind CSS v3 |
| Routing | React Router v6 |
| State | Zustand |
| Tauri IPC | @tauri-apps/api/core → invoke() |
| Icons | Lucide React |
| Markdown | react-markdown |
| Fonts | @fontsource/inter + @fontsource/jetbrains-mono |
| UI Components | Built from scratch — no component library |

### package.json Dependencies

```json
{
  "dependencies": {
    "react": "^18.3.0",
    "react-dom": "^18.3.0",
    "react-router-dom": "^6.22.0",
    "zustand": "^4.5.0",
    "@tauri-apps/api": "^2.0.0",
    "lucide-react": "^0.370.0",
    "react-markdown": "^9.0.0",
    "@fontsource/inter": "^5.0.0",
    "@fontsource/jetbrains-mono": "^5.0.0"
  },
  "devDependencies": {
    "vite": "^5.2.0",
    "@vitejs/plugin-react": "^4.2.0",
    "typescript": "^5.4.0",
    "tailwindcss": "^3.4.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0"
  },
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
  }
}
```

---

## Part 3: Project Structure

```
src/
├── main.tsx                    — React entry, Tauri init
├── App.tsx                     — Router wrapper + Shell layout
├── router.tsx                  — Route definitions
│
├── components/
│   ├── layout/
│   │   ├── Shell.tsx           — App shell (sidebar + topbar + content)
│   │   ├── Sidebar.tsx         — Nav links, Ollama status dot
│   │   └── TopBar.tsx          — Breadcrumb, workspace selector
│   └── ui/
│       ├── Button.tsx          — Primary, Secondary, Destructive, Ghost variants
│       ├── Badge.tsx           — Risk, Status, Confidence badges
│       ├── Table.tsx           — Reusable table with empty/loading states
│       ├── Modal.tsx           — Backdrop + panel + animation
│       ├── ProgressRing.tsx    — SVG circle progress
│       ├── Skeleton.tsx        — Shimmer loading placeholder
│       └── ConfidenceBar.tsx   — Gradient fill bar
│
├── pages/
│   ├── Workspace.tsx           — Screen 1
│   ├── Ingestion.tsx           — Screen 2
│   ├── Query.tsx               — Screen 3
│   ├── Extraction.tsx          — Screen 4
│   ├── ActionsPreview.tsx      — Screen 5
│   ├── ActionsAudit.tsx        — Screen 6
│   ├── Memory.tsx              — Screen 7
│   └── Benchmarks.tsx          — Screen 8
│
├── stores/
│   ├── workspaceStore.ts       — registered paths, model config
│   ├── ingestionStore.ts       — run state, telemetry, polling
│   ├── queryStore.ts           — query, answer, conversation thread
│   ├── actionsStore.ts         — recommendations, approval gate
│   └── memoryStore.ts          — facts, conversations
│
├── lib/
│   └── tauri.ts                — typed invoke() wrappers for all commands
│
└── types/
    └── contracts.ts            — TypeScript mirror of Rust contracts.rs
```

---

## Part 4: TypeScript Contracts (Mirror of contracts.rs)

Create `src/types/contracts.ts` with these exact types:

```typescript
// Workspace
export interface WorkspaceFolderInput { path: string }
export interface IngestionRequest { workspace_path: string; incremental: boolean }

// Retrieval
export interface RetrievalRequest {
  query: string; top_k: number;
  conversation_id?: string; workspace_path?: string; token_budget?: number
}
export interface RetrievalHit {
  file_id: string; source_path: string; chunk_index: number;
  page_ref?: string; score: number; content_preview: string;
  parser_confidence?: number; warning_count: number
}
export interface RetrievalResponse {
  command: string; status: string; query: string; top_k: number;
  total_hits: number; assembled_prompt?: string;
  prompt_token_estimate?: number; hits: RetrievalHit[]
}

// Extraction
export interface ExtractionRequest { query: string; template: string }

// Actions
export interface ActionRecommendationRequest { workspace_path: string; batch_limit: number }
export interface ActionRiskRequest { recommendation_ids: string[] }
export interface ActionApprovalRequest { recommendation_ids: string[] }
export interface ActionExecutionRequest { batch_id: string }
export interface UndoBatchRequest { execution_batch_id: string }
export interface AuditLogRequest { limit: number }

export interface ActionRecommendationView {
  id: string; file_id: string; recommendation_type: string;
  source_path: string; proposed_path: string; confidence: number;
  evidence: string[]; risk_level: 'low' | 'medium' | 'high';
  risk_flags: string[]; created_at: string
}
export interface ActionPreviewList {
  command: string; status: string; workspace_path: string;
  total: number; items: ActionRecommendationView[]
}
export interface ActionExecutionResult {
  command: string; status: string; batch_id: string;
  total: number; succeeded: number; failed: number; errors: string[]
}
export interface UndoResult {
  command: string; status: string; batch_id: string;
  reversed: number; errors: string[]
}
export interface AuditLogEntry {
  execution_id: string; recommendation_id: string; batch_id: string;
  source_path: string; proposed_path: string;
  recommendation_type: string; status: string; executed_at: string
}
export interface AuditLogResponse {
  command: string; status: string; total: number; entries: AuditLogEntry[]
}

// Memory
export interface StartConversationRequest { workspace_path: string }
export interface ConversationStub { id: string; title: string; turn_count: number; updated_at: string }
export interface ConversationTurnView {
  id: string; role: string; content: string;
  citations_json?: string; token_count?: number; created_at: string
}
export interface GetConversationRequest { conversation_id: string; limit: number }
export interface ListConversationsRequest { workspace_path: string; limit: number }
export interface SaveMemoryFactRequest {
  workspace_path: string; content: string;
  citation_json?: string; scope: 'session' | 'week' | 'permanent'
}
export interface MemoryFactView {
  id: string; content: string; citation_json?: string;
  scope: string; created_at: string; expires_at?: string
}
export interface DeleteMemoryFactRequest { fact_id: string }
export interface BuildLongContextRequest {
  query: string; conversation_id: string;
  workspace_path: string; token_budget?: number
}
export interface LongContextResponse {
  conversation_id: string; assembled_prompt: string;
  total_token_estimate: number; token_budget: number; overflow_dropped: number
}

// Benchmarks
export interface BenchmarkRequest { corpus_manifest: string; labels_version: string }
export interface RunMetricsRequest { run_id: string }

// General
export interface CommandAck { command: string; status: string; detail: string }
export interface RuntimeSnapshot {
  workspace_count: number; command_count: number;
  preview_queue_count: number; benchmark_run_count: number;
  last_ingestion_target?: string
}
```

---

## Part 5: Tauri Invoke Wrappers

Create `src/lib/tauri.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core'
import type * as T from '../types/contracts'

// Workspace
export const registerWorkspace = (req: T.WorkspaceFolderInput) =>
  invoke<T.CommandAck>('register_workspace', { request: req })
export const getRuntimeSnapshot = () =>
  invoke<T.RuntimeSnapshot>('get_runtime_snapshot')

// Ingestion
export const runIngestion = (req: T.IngestionRequest) =>
  invoke<T.CommandAck>('run_ingestion', { request: req })
export const getIngestionTelemetry = () =>
  invoke<T.CommandAck>('get_ingestion_telemetry')

// Retrieval
export const retrieveAndAssemble = (req: T.RetrievalRequest) =>
  invoke<T.RetrievalResponse>('retrieve_and_assemble', { request: req })

// Extraction
export const runExtraction = (req: T.ExtractionRequest) =>
  invoke<T.CommandAck>('run_extraction', { request: req })

// Actions
export const generateActionRecommendations = (req: T.ActionRecommendationRequest) =>
  invoke<T.ActionPreviewList>('generate_action_recommendations', { request: req })
export const evaluateActionRisk = (req: T.ActionRiskRequest) =>
  invoke<T.ActionPreviewList>('evaluate_action_risk', { request: req })
export const approveActions = (req: T.ActionApprovalRequest) =>
  invoke<T.CommandAck>('approve_actions', { request: req })
export const executeActions = (req: T.ActionExecutionRequest) =>
  invoke<T.ActionExecutionResult>('execute_actions', { request: req })
export const undoActionBatch = (req: T.UndoBatchRequest) =>
  invoke<T.UndoResult>('undo_action_batch', { request: req })
export const getActionAuditLog = (req: T.AuditLogRequest) =>
  invoke<T.AuditLogResponse>('get_action_audit_log', { request: req })

// Memory
export const startConversation = (req: T.StartConversationRequest) =>
  invoke<T.CommandAck>('start_conversation', { request: req })
export const listConversations = (req: T.ListConversationsRequest) =>
  invoke<T.ConversationStub[]>('list_conversations', { request: req })
export const getConversation = (req: T.GetConversationRequest) =>
  invoke<T.ConversationTurnView[]>('get_conversation', { request: req })
export const saveMemoryFact = (req: T.SaveMemoryFactRequest) =>
  invoke<T.CommandAck>('save_memory_fact', { request: req })
export const listMemoryFacts = (req: T.SaveMemoryFactRequest) =>
  invoke<T.MemoryFactView[]>('list_memory_facts', { request: req })
export const deleteMemoryFact = (req: T.DeleteMemoryFactRequest) =>
  invoke<T.CommandAck>('delete_memory_fact', { request: req })
export const buildLongContext = (req: T.BuildLongContextRequest) =>
  invoke<T.LongContextResponse>('build_long_context', { request: req })

// Benchmarks
export const runBenchmarkSuite = (req: T.BenchmarkRequest) =>
  invoke<T.CommandAck>('run_benchmark_suite', { request: req })
export const getBenchmarkRunMetrics = (req: T.RunMetricsRequest) =>
  invoke<T.CommandAck>('get_benchmark_run_metrics', { request: req })
```

---

## Part 6: All 8 Screens

### Screen 1 — Workspace Setup (`/workspace`)
**Tauri calls**: `register_workspace`, `get_runtime_snapshot`

| Component | Purpose |
|---|---|
| FolderPicker | OS native folder selector, path cards with remove |
| PatternInput | Include/exclude glob patterns |
| PolicySelector | Safe / Standard / Aggressive dropdown |
| ModelConfigPanel | Ollama endpoint URL, model name, hardware badge |
| WorkspaceSaveButton | Validates all fields before saving |

**States**: `empty` (no folders, onboarding prompt) → `configured` (model badge shown) → `error/model-offline` (warning banner)

---

### Screen 2 — Ingestion Dashboard (`/ingestion`)
**Tauri calls**: `run_ingestion`, `get_ingestion_telemetry` (poll every 1s when running)

| Component | Purpose |
|---|---|
| RunSummaryBar | 4 stat cards: total / indexed / failed / skipped |
| ProgressRing | Live % during active run |
| FileStatusTable | filename, type, status dot, parser, chunks, confidence |
| FailurePanel | Collapsible failed files with error class tags |
| RunControls | Run Full / Run Incremental / Stop |
| RunHistoryList | Last 5 runs: timestamp, success rate, duration |

**States**: `idle` → `running` (live poll) → `done/pass` → `done/fail`

---

### Screen 3 — Query Workbench (`/query`)
**Tauri calls**: `retrieve_and_assemble`, `start_conversation`, `list_conversations`, `get_conversation`, `save_memory_fact`

| Component | Purpose |
|---|---|
| QueryInput | Multi-line, ⌘Enter submit, history dropdown |
| AnswerPanel | Markdown-rendered answer with inline citation badges [1] [2] |
| EvidenceList | Ranked chunks: file + page + confidence + excerpt |
| ConfidenceBar | High/Medium/Low gradient indicator |
| ExportButton | Export answer + citations as Markdown or JSON |
| LatencyBadge | Query latency ms after response |
| ConversationList | Sidebar: past threads sorted by last activity |
| TurnThread | Chronological user/assistant exchanges |
| PinFactButton | Saves answer content to memory facts |

**Layout**: 65% left (query + answer) / 35% right (conversations sidebar)
**States**: `empty` → `loading` (skeleton) → `result/high-confidence` → `result/low-confidence` (amber border) → `no-results`

---

### Screen 4 — Extraction Results (`/extraction`)
**Tauri calls**: `run_extraction`

| Component | Purpose |
|---|---|
| ExtractionJobInput | Document selector + template selector |
| FieldTable | field, value, confidence %, source, page ref |
| CitationPanel | Highlights source text for selected row |
| ConfidenceFilter | Filter > 90% / > 70% / all |
| ExportPanel | CSV / Excel export |
| TemplateSelector | Contract Key Terms / Invoice Fields / Custom |

**States**: `empty` → `running` (per-field progress) → `complete/pass` → `complete/partial` (amber badges) → `error`

---

### Screen 5 — Action Preview Queue (`/actions/preview`)
**Tauri calls**: `generate_action_recommendations`, `evaluate_action_risk`, `approve_actions`, `execute_actions`

| Component | Purpose |
|---|---|
| QueueSummary | Total, risk breakdown (low/medium/high counts) |
| RecommendationList | Before → after path diff per row |
| RiskBadge | green / amber / red |
| BulkSelectControls | Select all low-risk / deselect high-risk |
| ApproveButton | Disabled until ≥1 item selected |
| RejectButton | Remove from queue |

**Safety rules**:
- High-risk items pre-deselected
- ApproveButton disabled until selection made
- Confirmation modal before execute_actions
- Undo banner after execution

**States**: `empty` → `populated/safe` → `populated/mixed` → `executing` → `blocked`

---

### Screen 6 — Execution Audit (`/actions/audit`)
**Tauri calls**: `get_action_audit_log`, `undo_action_batch`

| Component | Purpose |
|---|---|
| AuditTable | timestamp, type, source, destination, status, batch |
| UndoBatchButton | Per-batch, only for eligible batches |
| FailureDetailModal | Root cause, file state, suggested fix |
| FilterBar | Filter by status / date range / batch ID |
| ExportAuditLog | Export as CSV |

**Row colors**: success → emerald left border / failed → red left border / rolled_back → zinc left border
**States**: `empty` → `populated` → `undo-available` → `undo-expired`

---

### Screen 7 — Memory Facts Manager (`/memory`)
**Tauri calls**: `list_memory_facts`, `save_memory_fact`, `delete_memory_fact`

| Component | Purpose |
|---|---|
| FactList | fact, citation badge, scope label, date, delete |
| ScopeFilter | All / Permanent / Week / Session toggle |
| SearchInput | Filter by keyword |
| FactDetailPanel | Full citation: filename + page + excerpt |
| AddFactManually | Text input to pin fact without a query |
| BulkDeleteButton | Delete all expired or session-scoped |

**States**: `empty` → `populated` → `expired_notice` (banner for expired week-scope facts)

---

### Screen 8 — Benchmark Console (`/benchmarks`)
**Tauri calls**: `run_benchmark_suite`, `get_benchmark_run_metrics`

| Component | Purpose |
|---|---|
| RunBenchmarkButton | Triggers full benchmark suite |
| ThresholdScorecard | 5 metric cards: ingestion / retrieval / extraction / harmful action / p95 latency |
| RunHistoryTable | Date, corpus, model, pass/fail per metric |
| RunComparisonView | Side-by-side diff of two selected runs |
| PhaseGateBadge | "Phase 4 Gate: PASS ✓" prominent pill |
| ExportReport | PDF / Markdown export |

**States**: `no-runs` → `running` → `pass` → `fail` → `partial`

---

## Part 7: 2026 Modern UI Design System

### Visual Identity

**Aesthetic**: Dark glassmorphism + sharp utility. Linear meets Vercel meets Raycast.
Dense, precise, trustworthy. Not colorful or playful — built for legal and finance professionals.

### Color Tokens

```typescript
// tailwind.config.ts custom colors
surface: {
  app:     '#09090b',   // near-black app background
  base:    '#111113',   // elevated panels
  raised:  '#18181b',   // cards, sidebar
  overlay: '#1c1c1f',   // modals, dropdowns
}

// Semantic
border: {
  default: '#27272a',   // zinc-800
  subtle:  '#1f1f22',
}
accent: {
  primary: '#6366f1',   // indigo-500
  hover:   '#818cf8',   // indigo-400
  glow:    'rgba(99,102,241,0.15)',
}
```

| Role | Value | Usage |
|---|---|---|
| text.primary | #fafafa | Headings, labels |
| text.secondary | #a1a1aa | Body text |
| text.muted | #52525b | Placeholders, hints |
| success | #10b981 | Pass, low risk |
| warning | #f59e0b | Medium risk, caution |
| danger | #ef4444 | High risk, fail, error |
| info | #3b82f6 | Info states |

### Typography

```css
Font stack:
  sans: Inter, system-ui, sans-serif
  mono: JetBrains Mono, Fira Code, monospace

Scale:
  Display : 24px / 700 / tracking -0.02em  — page titles
  Heading : 18px / 600 / tracking -0.01em  — section titles
  Body    : 14px / 400                      — all content
  Small   : 12px / 400 / tracking +0.01em  — labels, badges
  Mono    : 13px JetBrains Mono             — file paths, IDs, hashes, timestamps
```

**Rule**: All file paths, chunk IDs, batch IDs, timestamps → JetBrains Mono always.

### Layout

```
┌─────────────────────────────────────────────────┐
│  SIDEBAR (220px)  │  TOP BAR (48px)              │
│  fixed left       │  breadcrumb + workspace sel  │
│                   ├──────────────────────────────│
│  logo + version   │                              │
│  ─────────────    │  CONTENT AREA                │
│  nav items        │  padding: 24px               │
│  (icon + label)   │  max-width: 1100px           │
│  ─────────────    │                              │
│  Ollama status    │                              │
└───────────────────┴──────────────────────────────┘
```

### Component Specs

**Cards / Panels**:
```css
background: surface.raised (#18181b)
border: 1px solid #27272a
border-radius: 8px
padding: 16px or 20px
box-shadow: 0 1px 3px rgba(0,0,0,0.3)
hover (interactive): border-color → zinc-700
```

**Buttons**:

| Variant | Background | Border | Text | Hover |
|---|---|---|---|---|
| Primary | indigo-500 | — | white | indigo-400 + glow shadow |
| Secondary | transparent | zinc-800 | zinc-400 | border zinc-600, text white |
| Destructive | transparent | red-500 | red-400 | bg red-500/10 |
| Ghost | transparent | — | zinc-400 | bg surface.overlay |

All buttons: `transition: all 150ms ease`, focus ring `indigo-500/50`, `border-radius: 6px`

**Badges**:

| Type | Background | Text | Border |
|---|---|---|---|
| low / pass / success | emerald-500/10 | emerald-400 | emerald-500/20 |
| medium / warning | amber-500/10 | amber-400 | amber-500/20 |
| high / fail / danger | red-500/10 | red-400 | red-500/20 |
| pending | zinc-800 | zinc-400 | zinc-700 |
| running | indigo-500/10 | indigo-400 | indigo-500/20 + pulse dot |

All badges: `px-2 py-0.5 rounded-full text-[11px] font-medium font-mono`

**Input Fields**:
```css
background: surface.base
border: 1px solid border.default
focus-border: accent.primary
focus-ring: 0 0 0 2px accent.glow
border-radius: 6px
padding: 8px 12px
transition: border-color 150ms
```

**Tables**:
```
Header: 11px uppercase, letter-spacing 0.08em, text.muted, border-bottom
Rows: 36px height, border-bottom border.subtle, hover → surface.overlay 100ms
Selected: accent.glow background + 2px accent.primary left border
```

**Modals**:
```css
backdrop: rgba(0,0,0,0.7) blur(4px)
panel: surface.overlay, border border.default, border-radius 12px, shadow-2xl
width: 480px default / 600px wide
animation: scale 0.95→1 + opacity 0→1, 150ms ease-out
close: scale 1→0.95 + opacity, 100ms ease-in
```

**Skeleton Shimmer**:
```css
background: linear-gradient(90deg, zinc-800 25%, zinc-700 50%, zinc-800 75%)
background-size: 200% 100%
animation: shimmer 1.5s linear infinite
```

**ProgressRing**:
```
SVG circle, 80px diameter, stroke indigo-500, track zinc-800
% text centered in Inter 14px/600
Animated: CSS stroke-dashoffset transition 600ms ease-out
```

**ConfidenceBar**:
```
Track: zinc-800, height 4px, border-radius 2px
Fill: emerald→amber→red gradient based on score
Animation: width 0 → final, 400ms ease-out on mount
```

**Diff View** (Action Preview):
```
Two columns with thin divider
Before: path in mono, red-400/20 background
After:  path in mono, emerald-400/20 background
Arrow → in accent.primary between columns
```

### Animation System

| Interaction | Animation | Duration |
|---|---|---|
| Page transition | opacity 0→1, translateY 4px→0 | 150ms ease-out |
| Card entry | opacity 0→1, staggered 40ms | 200ms ease-out |
| Modal open | scale 0.95→1 + opacity | 150ms ease-out |
| Modal close | scale 1→0.95 + opacity | 100ms ease-in |
| Button hover | color + shadow only | 150ms ease |
| Table row hover | background | 100ms |
| Skeleton shimmer | continuous | 1.5s linear |
| Progress ring | stroke-dashoffset | 600ms ease-out |
| Undo banner | slide down from top | 200ms ease-out |
| Status dot pulse | running state only | 2s ease-in-out |

### Tailwind Config

```typescript
// tailwind.config.ts
import type { Config } from 'tailwindcss'

export default {
  content: ['./src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
      },
      colors: {
        surface: {
          app:     '#09090b',
          base:    '#111113',
          raised:  '#18181b',
          overlay: '#1c1c1f',
        },
      },
      keyframes: {
        shimmer: {
          '0%':   { backgroundPosition: '-200% 0' },
          '100%': { backgroundPosition: '200% 0' },
        },
        'fade-in': {
          '0%':   { opacity: '0', transform: 'translateY(4px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        'slide-down': {
          '0%':   { opacity: '0', transform: 'translateY(-8px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
      },
      animation: {
        shimmer:      'shimmer 1.5s linear infinite',
        'fade-in':    'fade-in 150ms ease-out',
        'slide-down': 'slide-down 200ms ease-out',
      },
    },
  },
} satisfies Config
```

### Screen-Specific Design Notes

**Workspace Setup**: Folder cards show icon + mono path + hover-reveal X button. ModelConfigPanel has live Ollama connection dot. Empty state: 64px FolderOpen icon + headline + CTA.

**Ingestion Dashboard**: RunSummaryBar = 4 stat cards (large number 24px/700 + muted label). File status column uses colored inline dots (emerald/red/zinc).

**Query Workbench**: Citation badges `[1]` `[2]` as small indigo pills inline in answer text. Answer rendered as markdown. Low-confidence → amber left border on AnswerPanel + warning icon.

**Action Preview Queue**: Each recommendation is a full-width card with diff view. Bulk action bar sticky at bottom. ApproveButton becomes "Execute [N] Actions" in emerald after approval.

**Benchmark Console**: ThresholdScorecard 5 cards in a row, each with thin top border (emerald=pass, red=fail). PhaseGateBadge is a large prominent pill at top of screen.

---

## Part 8: Safety Rules (Non-Negotiable)

1. **Never call `execute_actions` without prior `approve_actions`**
   — `actionsStore` must have `approved = true` flag
   — ExecuteButton disabled until approved

2. **Confirmation modal before `execute_actions`**
   — Text: *"This will rename [N] files. This action can be undone."*
   — Two buttons: "Cancel" (secondary) + "Execute [N] Files" (primary)

3. **Undo banner after `execute_actions`**
   — Slides in from top, stays until dismissed or navigation
   — Shows: "Renamed [N] files. [Undo]" with batch_id

4. **High-risk recommendations pre-deselected**
   — `risk_level === 'high'` → checkbox unchecked by default
   — User must explicitly check to include

5. **Never auto-execute** — all file operations require explicit approval

---

## Part 9: Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| ⌘/Ctrl + Enter | Submit query (Screen 3) |
| ⌘/Ctrl + K | Focus query input from anywhere |
| Escape | Close any open modal |

---

## Part 10: Build Order

Do these in sequence. Verify each step renders before moving on.

```
Step 1:  package.json + vite.config.ts + tailwind.config.ts + tsconfig.json
Step 2:  src/types/contracts.ts
Step 3:  src/lib/tauri.ts
Step 4:  src/components/ui/  (Button, Badge, Table, Modal, ProgressRing, Skeleton)
Step 5:  src/components/layout/ (Shell, Sidebar, TopBar)
Step 6:  src/router.tsx + src/App.tsx + src/main.tsx
Step 7:  Screen 1 — Workspace Setup
Step 8:  Screen 2 — Ingestion Dashboard
Step 9:  Screen 3 — Query Workbench
Step 10: Screen 4 — Extraction Results
Step 11: Screen 5 — Action Preview Queue
Step 12: Screen 6 — Execution Audit
Step 13: Screen 7 — Memory Facts Manager
Step 14: Screen 8 — Benchmark Console
Step 15: Global keyboard shortcuts + page transitions
Step 16: Ollama status poll in Sidebar (every 10s)
```

---

## Part 11: Phase 5 Exit Checklist

```
[ ] All 8 screens render at 1400×960 with correct layout
[ ] Dark theme consistent — no white backgrounds anywhere
[ ] All file paths, IDs, timestamps use JetBrains Mono
[ ] All tables: empty state + loading skeleton + error banner
[ ] All buttons: hover + active + disabled + focus states
[ ] Risk badges correct across all screens
[ ] Confidence badges threshold-based (>=0.85 green, >=0.60 amber, <0.60 red)
[ ] Confirmation modal appears before execute_actions
[ ] Undo banner appears after execute_actions with dismiss
[ ] High-risk recommendations pre-deselected in queue
[ ] Ollama status dot live in sidebar (poll every 10s)
[ ] Page transitions animate on route change (fade-in 150ms)
[ ] Keyboard shortcuts work: ⌘K, ⌘Enter, Escape
[ ] Zero mocked data — all screens use real Tauri invoke() calls
[ ] npm run build — zero TypeScript errors
[ ] cargo tauri dev — all screens navigable, no console errors
```

---

## Reference Products (Design Inspiration)

| Product | What to Study |
|---|---|
| Linear.app | Dense dark UI, precise typography, fast transitions |
| Vercel Dashboard | Neutral dark palette, clean data surfaces |
| Raycast | Command palette feel, keyboard-first interaction |
| Fig | Developer tool aesthetics, mono + sans font pairing |

ALIV belongs in this tier. Users are legal and finance professionals.
The product must feel **trustworthy, precise, and fast** — not colorful or playful.

---

---

## Part 12: Web Search and Reference Code Policy

**You are explicitly allowed and encouraged to search the web and copy
reference code to build faster. Do not reinvent solved problems.**

### When To Search

Search before writing any of these from scratch:

| Component | What To Search For |
|---|---|
| ProgressRing | "React SVG circular progress ring Tailwind TypeScript" |
| Skeleton shimmer | "Tailwind CSS skeleton shimmer animation React component" |
| Markdown renderer | "react-markdown tailwind prose dark theme code blocks" |
| File path breadcrumb | "React breadcrumb component Tailwind dark UI" |
| Diff view (before/after) | "React file diff before after component Tailwind" |
| Modal with backdrop blur | "React modal backdrop blur Tailwind animation TypeScript" |
| Zustand store pattern | "Zustand TypeScript store pattern async actions" |
| Tauri invoke with error handling | "Tauri v2 invoke TypeScript error handling React" |
| Poll with useEffect | "React useEffect polling interval cleanup TypeScript" |
| Table with sort and filter | "React table component Tailwind dark sort filter TypeScript" |
| Keyboard shortcut hook | "React useKeyboardShortcut hook TypeScript Cmd Ctrl" |
| Toast / banner notification | "React toast notification slide animation Tailwind" |
| Sidebar nav active state | "React Router v6 NavLink active style Tailwind sidebar" |

### Trusted Reference Sources (search these first)

```
GitHub searches:
  site:github.com "tauri" "react" "typescript" "invoke"
  site:github.com "tailwind" "dark" "table" component
  site:github.com "zustand" "react" typescript example

Community resources:
  https://ui.shadcn.com/         — copy component patterns (adapt to our tokens)
  https://tailwindui.com/        — layout and component reference
  https://headlessui.com/        — accessible modal, dropdown, transition patterns
  https://www.radix-ui.com/      — unstyled primitives (Modal, Dialog, Select)
  https://lucide.dev/icons/      — find correct Lucide icon names
  https://tauri.app/v2/          — Tauri v2 invoke, events, file dialog API
  https://zustand.docs.pmnd.rs/  — Zustand patterns, middleware, devtools
```

### Rules When Using Reference Code

1. **Adapt to our design tokens** — change all colors to surface.*/accent.*/zinc-* tokens
2. **Adapt to our type system** — use types from `src/types/contracts.ts`
3. **Remove all mock/dummy data** — replace with real Tauri invoke() calls
4. **Keep the font** — always apply `font-mono` (JetBrains Mono) to paths, IDs, timestamps
5. **Match border-radius** — use `rounded` (4px) for small elements, `rounded-md` (6px) for inputs/buttons, `rounded-lg` (8px) for cards
6. **Never copy license-incompatible code** — MIT and Apache-2.0 only
7. **Cite the source** — add a comment with the URL when copying a non-trivial pattern:
   ```typescript
   // Adapted from: https://github.com/example/repo/blob/main/src/component.tsx
   ```

### Quick Copy Patterns (Use These Directly)

**Tauri file dialog** (for FolderPicker in Screen 1):
```typescript
import { open } from '@tauri-apps/plugin-dialog'

const pickFolder = async () => {
  const selected = await open({ directory: true, multiple: false })
  if (selected) setPath(selected as string)
}
```

**Polling hook** (for Ingestion Dashboard live updates):
```typescript
function usePolling(fn: () => void, intervalMs: number, active: boolean) {
  useEffect(() => {
    if (!active) return
    const id = setInterval(fn, intervalMs)
    return () => clearInterval(id)
  }, [fn, intervalMs, active])
}
```

**Global keyboard shortcut hook**:
```typescript
function useKeyboardShortcut(key: string, meta: boolean, fn: () => void) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((meta ? e.metaKey || e.ctrlKey : true) && e.key === key) {
        e.preventDefault()
        fn()
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [key, meta, fn])
}
```

**Error boundary for Tauri calls**:
```typescript
const safeInvoke = async <T>(fn: () => Promise<T>): Promise<[T | null, string | null]> => {
  try {
    const result = await fn()
    return [result, null]
  } catch (err) {
    return [null, err instanceof Error ? err.message : String(err)]
  }
}
// Usage: const [data, error] = await safeInvoke(() => listConversations(...))
```

**Zustand store boilerplate**:
```typescript
import { create } from 'zustand'

interface QueryState {
  query: string
  isLoading: boolean
  error: string | null
  setQuery: (q: string) => void
  setLoading: (v: boolean) => void
  setError: (e: string | null) => void
}

export const useQueryStore = create<QueryState>((set) => ({
  query: '',
  isLoading: false,
  error: null,
  setQuery: (query) => set({ query }),
  setLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error }),
}))
```

**Shadcn Badge pattern** (adapt colors to our tokens):
```typescript
// Copy the variant pattern from shadcn/ui Badge, replace colors with:
// low/pass   → 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
// medium     → 'bg-amber-500/10 text-amber-400 border-amber-500/20'
// high/fail  → 'bg-red-500/10 text-red-400 border-red-500/20'
// pending    → 'bg-zinc-800 text-zinc-400 border-zinc-700'
```

### Vite Config for Tauri

```typescript
// vite.config.ts
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    strictPort: true,
    port: 1420,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: ['es2021', 'chrome105', 'safari13'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
})
```

### tsconfig.json for Tauri + React

```json
{
  "compilerOptions": {
    "target": "ES2021",
    "useDefineForClassFields": true,
    "lib": ["ES2021", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "paths": { "@/*": ["./src/*"] }
  },
  "include": ["src"]
}
```

---

*Phase 5 is complete when all 8 screens are fully functional with real Tauri backend calls, the UI matches the design system above, and both `npm run build` and `cargo tauri dev` pass cleanly.*
