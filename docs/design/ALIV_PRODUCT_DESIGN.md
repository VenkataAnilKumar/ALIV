# ALIV Product Design

## 1. Product Experience Goals
1. Make high-risk automation feel controlled and transparent.
2. Give users confidence through evidence, not black-box output.
3. Keep workflows fast for document-heavy operations.
4. Reduce cognitive load with clear status, actions, and rollback.

## 2. Primary User Jobs
1. Find critical information in large local document sets.
2. Extract structured facts with source confidence.
3. Organize chaotic files without losing control.

## 3. Design Principles
1. Evidence first.
2. Safety before automation.
3. Progressive disclosure for advanced controls.
4. Visible system status at all times.
5. Reversible actions by default.

## 4. Information Architecture
Top-level product areas:
1. Workspace.
2. Ingestion.
3. Query (with conversation memory thread).
4. Extraction.
5. Actions.
6. Memory (conversation history + pinned facts).
7. Benchmarks.
8. Logs and diagnostics.

## 5. Core Screens

### Screen 1: Workspace Setup
**Route:** `/workspace`
**Purpose:** Register folders, configure scan policies, set LLM and model profile.

**Components:**
- `FolderPicker` — OS native folder selector, shows registered paths as cards with remove button
- `PatternInput` — Include/exclude glob patterns (`**/*.pdf`, `**/drafts/**`)
- `PolicySelector` — Dropdown: Safe (preview-only) / Standard / Aggressive
- `ModelConfigPanel` — Ollama endpoint URL, active LLM model name, embedding model name, hardware check badge
- `WorkspaceSaveButton` — Validates all fields before saving

**States:**
- `empty` — No folders registered, onboarding prompt
- `configured` — Folders registered, model connected badge shown
- `error/model-offline` — Ollama not running, warning banner with setup link

**Primary Action:** Register workspace → triggers first ingestion scan

---

### Screen 2: Ingestion Dashboard
**Route:** `/ingestion`
**Purpose:** Monitor indexing runs, view file-level status, trigger re-index.

**Components:**
- `RunSummaryBar` — Total files / indexed / failed / skipped counts
- `ProgressRing` — Live progress during active run (% complete, files/sec rate)
- `FileStatusTable` — Columns: filename, type, status (indexed/failed/skipped), parser used, chunks, confidence score
- `FailurePanel` — Collapsible list of failed files with error class tags
- `RunControls` — "Run Full Index" / "Run Incremental" / "Stop" buttons
- `RunHistoryList` — Last 5 runs with timestamp, success rate, duration

**States:**
- `idle` — Last run summary, re-run button
- `running` — Live progress, stop button enabled
- `done/pass` — Success rate badge, view failures link
- `done/fail` — Warning banner if below 98% threshold

---

### Screen 3: Query Workbench
**Route:** `/query`
**Purpose:** Natural-language query over indexed documents with cited evidence.

**Components:**
- `QueryInput` — Multi-line textarea with submit (Cmd+Enter), query history dropdown
- `AnswerPanel` — LLM synthesized answer text with inline citation badges `[1]`, `[2]`
- `EvidenceList` — Ranked chunks: filename, page, confidence score, highlighted text snippet
- `ConfidenceBar` — Overall confidence indicator (High / Medium / Low) based on top chunk score
- `ExportButton` — Export answer + citations as Markdown or JSON
- `LatencyBadge` — Query latency in ms shown after response

**States:**
- `empty` — Placeholder prompt suggestions (3 example queries)
- `loading` — Skeleton animation, cancel button
- `result/high-confidence` — Answer with green confidence badge
- `result/low-confidence` — Answer with amber badge + "Treat with caution" notice
- `no-results` — "No relevant documents found" with suggestion to re-index

---

### Screen 4: Extraction Results
**Route:** `/extraction`
**Purpose:** Structured field extraction from documents with source citations.

**Components:**
- `ExtractionJobInput` — Document selector or query, field template selector
- `FieldTable` — Columns: field name, extracted value, confidence %, source file, page ref
- `CitationPanel` — Inline document viewer for selected row, highlights source text
- `ConfidenceFilter` — Filter by confidence threshold (>90%, >70%, all)
- `ExportPanel` — Export to CSV/Excel, field selection checkboxes
- `TemplateSelector` — Pre-defined templates: "Contract Key Terms", "Invoice Fields", "Custom"

**States:**
- `empty` — Template selector, no results yet
- `running` — Per-field progress indicators
- `complete/pass` — All fields extracted, export enabled
- `complete/partial` — Some fields below threshold, flagged with amber badge
- `error` — Extraction failed, error detail + retry

---

### Screen 5: Action Preview Queue
**Route:** `/actions/preview`
**Purpose:** Review recommended file rename/move operations before execution.

**Components:**
- `QueueSummary` — Total recommendations, risk breakdown (low/medium/high)
- `RecommendationList` — Each row: current filename → proposed filename, folder destination, risk badge
- `DiffView` — Before/after path comparison for selected item
- `RiskBadge` — Color-coded: green (confident) / amber (review) / red (block)
- `BulkSelectControls` — Select all low-risk, deselect high-risk
- `ApproveButton` — Confirm selected items, transitions to execution
- `RejectButton` — Remove selected items from queue

**States:**
- `empty` — No recommendations, "Run analysis to generate suggestions"
- `populated/safe` — All recommendations low-risk, bulk approve available
- `populated/mixed` — Mix of risk levels, high-risk items pre-deselected
- `executing` — Progress bar per action, real-time status
- `blocked` — Safety threshold not yet passed, approve button disabled with reason

---

### Screen 6: Execution Audit
**Route:** `/actions/audit`
**Purpose:** Review all executed operations, undo batches, inspect failure reasons.

**Components:**
- `AuditTable` — Columns: timestamp, operation type, source path, destination path, status, batch ID
- `UndoBatchButton` — Per-batch undo button (only for reversible operations within undo window)
- `FailureDetailModal` — Root cause, file state at failure, suggested fix
- `FilterBar` — Filter by status (success/failed/undone), date range, batch ID
- `ExportAuditLog` — Export full audit log as CSV

**States:**
- `empty` — No executions yet
- `populated` — Table with all operations
- `undo-available` — Undo button enabled for eligible batches
- `undo-expired` — Batch too old for undo, grayed out with explanation

---

### Screen 7A: Conversation History Panel
**Location:** Sidebar within Query Workbench (`/query`)
**Purpose:** View and switch between past conversation threads.

**Components:**
- `ConversationList` — List of past conversations, sorted by last activity; shows title + relative time
- `NewConversationButton` — Starts a fresh thread, clears active context
- `ConversationTitle` — Editable inline title (auto-generated from first query)
- `TurnThread` — Chronological list of user/assistant exchanges with citation badges
- `PinFactButton` — On any assistant answer, saves content to memory facts

**States:**
- `empty` — No conversations yet, "Ask your first question" prompt
- `active` — Current thread shown, streaming answer appended at bottom
- `history` — Past thread selected, read-only view with "Continue thread" option

---

### Screen 7B: Memory Facts Manager
**Route:** `/memory`
**Purpose:** View, manage, and delete pinned memory facts.

**Components:**
- `FactList` — Each row: fact content, source citation badge, scope label, creation date, delete button
- `ScopeFilter` — Toggle: All / Permanent / Week / Session
- `SearchInput` — Filter facts by keyword
- `FactDetailPanel` — Expand row to show full citation: filename, page, excerpt
- `AddFactManually` — Text input to pin a fact manually (without a query)
- `BulkDeleteButton` — Delete all expired or all session-scoped facts

**States:**
- `empty` — No facts saved, "Pin answers using the 'Remember this' button in Query Workbench"
- `populated` — Fact list with scope badges
- `expired_notice` — Banner if any week-scoped facts expired this session

---

### Screen 9: Benchmark Console
**Route:** `/benchmarks`
**Purpose:** Run benchmark suite, compare runs, check phase gate pass/fail.

**Components:**
- `RunBenchmarkButton` — Triggers full benchmark suite
- `ThresholdScorecard` — Five metric cards: ingestion rate, retrieval relevance, extraction precision, harmful action rate, p95 latency — each showing current vs target
- `RunHistoryTable` — Past runs with date, corpus version, model version, pass/fail per metric
- `RunComparisonView` — Side-by-side diff for two selected runs
- `PhaseGateBadge` — "Phase 1 Gate: PASS / FAIL / PENDING"
- `ExportReport` — Export benchmark report as PDF/Markdown

**States:**
- `no-runs` — Run button prominent, threshold targets shown
- `running` — Per-metric progress, estimated time remaining
- `pass` — Green phase gate badge, export report enabled
- `fail` — Red badge, failing metrics highlighted, suggested fixes linked
- `partial` — Some metrics pass, failing ones highlighted with diff from threshold

## 6. Interaction Model
### 6.1 Query and Extraction
1. User enters query.
2. System returns answer plus evidence set.
3. User inspects citations and confidence.
4. User exports structured output when confident.

### 6.2 File Action Flow
1. System generates recommendations.
2. User reviews recommendations in preview list.
3. User approves selected items.
4. System executes with progress updates.
5. User can undo batch if needed.

## 7. UX Safety Patterns
1. Explicit risk labels for low-confidence and high-impact operations.
2. Confirmation modal for irreversible or high-risk actions.
3. Inline citation badges for every extracted field.
4. Undo banner after execution with clear time window.

## 8. Visual Design Direction
1. Modern utility-focused interface with strong hierarchy.
2. Dense but readable data tables for document-heavy workflows.
3. Neutral palette with semantic color for confidence and risk states.
4. Keyboard-first interactions for power usage.

## 9. Accessibility Requirements
1. Keyboard navigation across all primary actions.
2. Clear focus states and shortcut hints.
3. Contrast-compliant text and status indicators.
4. Screen-reader labels for action and confidence controls.

## 10. Product Metrics
1. Time to first indexed result.
2. Query to cited-answer completion time.
3. Approval-to-execution completion rate.
4. Undo usage and post-undo correction rate.
5. Benchmark pass trend by run.

## 11. Design Validation Plan
1. Validate task completion on the three core jobs.
2. Track confusion points in action preview flow.
3. Measure confidence comprehension using citations and risk labels.

## 12. Definition of Product Design Ready
Ready when:
1. all core flows have low-fidelity and high-fidelity specs.
2. safety states are defined for every action.
3. benchmark and diagnostics surfaces are mapped.
