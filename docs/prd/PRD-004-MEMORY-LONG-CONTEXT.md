# PRD-004: Persistent Memory and Long Context

## 1. Problem Statement

ALIV loses all conversational context when the app restarts. Every new query starts from scratch — the LLM has no memory of prior sessions, previous answers, or facts the user has already established. For legal/finance users running repeated investigations across days or weeks, this means:

- Re-stating the same document corpus context every session
- Re-asking "what vendor was in that invoice?" after the app restarts
- Context window overflow on large document sets — chunks are truncated arbitrarily, not by relevance priority
- No ability to build on previous query results

## 2. Goals

| # | Goal | Metric | Target |
|---|---|---|---|
| 1 | Conversation persistence across sessions | Turns recalled per session | > 90% of prior 10 turns available |
| 2 | Long document context without overflow | Prompt token overflow rate | < 5% of queries exceed model context window |
| 3 | User-pinned facts recalled in future queries | Fact recall rate when relevant | > 80% when cosine similarity > 0.7 |

## 3. Non-Goals
1. Cloud sync of memory — all memory stays local (SQLite on disk).
2. Shared memory across workspaces — memory is scoped per workspace.
3. Fine-tuning or model weight updates — this is retrieval-based memory only.
4. Real-time streaming memory update during inference.

## 4. Core Memory Types

### 4.1 Conversation Memory
Persist multi-turn query/answer threads across app restarts.

- Each session is a `conversation` (id, workspace, title, timestamps)
- Each exchange is a `conversation_turn` (role, content, citations, token_count)
- At query time: load last N turns that fit in token budget (newest-first, stop when budget exceeded)
- Title auto-generated from first user query (first 80 chars)

### 4.2 Document Summary Cache
Pre-computed summaries at multiple granularities to handle long documents.

- **Document-level** (~1,000 tokens): full document summary
- **Section-level** (~400 tokens): per logical section summary
- Summaries are generated once after ingestion and cached in SQLite
- At query time: use summaries when raw chunk count would exceed context budget
- Summaries are invalidated and regenerated when the source document changes (hash mismatch)

### 4.3 User Memory (Pinned Facts)
User-saved facts with citations, recalled in future queries.

- User explicitly saves a fact from any query result ("Remember this")
- Facts are stored with source citation (file_id, chunk_id, span)
- At query time: relevant facts retrieved via keyword/embedding match and injected before retrieved chunks
- Scope: `permanent` (default), `week` (expires in 7 days), `session` (current session only)

## 5. Long Context Assembly (Attunement Enhancement)

Context is assembled in priority order within a token budget:

```
token_budget = model_context_window - reserved_output_tokens - system_prompt_tokens
               default: 8192 - 1024 - 256 = 6912

Allocation:
  [1] Instructions:          256 tokens  (system + task instructions)
  [2] Memory facts:          10% of budget (~691 tokens)
  [3] Conversation history:  20% of budget (~1382 tokens, newest-first)
  [4] Retrieved chunks:      60% of budget (~4147 tokens, ranked by score)
  [5] Overflow reserve:      10% of budget (~691 tokens, document summaries if chunks don't fit)
```

If retrieved chunks exceed their allocation, the lowest-scored chunks are dropped first.
If a document is longer than its chunk allocation, its document-level summary is used instead.

## 6. Functional Requirements

1. App must persist conversation history to SQLite on every turn.
2. App must restore the last active conversation on startup for each workspace.
3. Query command must accept optional `conversation_id` to continue a thread.
4. Context assembler must enforce token budget and never send an oversized prompt.
5. System must generate and cache document summaries after ingestion completes.
6. System must invalidate summaries when source document hash changes.
7. User must be able to save any query result as a memory fact from the UI.
8. Memory facts must be retrieved by semantic similarity at query time.
9. All memory operations must be scoped to the active workspace.
10. Conversation history display must show full thread in Query Workbench screen.

## 7. Non-Functional Requirements

- **Latency**: Context assembly must complete in < 200ms (token counting, fact retrieval, history fetch).
- **Storage**: Memory tables must not exceed 500 MB for 1,000 conversations × 100 turns.
- **Privacy**: No memory data leaves the device. No cloud calls for memory operations.
- **Correctness**: Token estimates must be within 15% of actual tokenizer output (use chars/4 heuristic).
- **Determinism**: Context assembly order must be deterministic for benchmark reproducibility.

## 8. New Database Tables

See `db/migrations/0005_memory_context.sql` for schema.

| Table | Purpose |
|---|---|
| `conversations` | One row per conversation session |
| `conversation_turns` | One row per user/assistant exchange |
| `document_summaries` | Cached summaries at document/section granularity |
| `memory_facts` | User-pinned facts with citations and scope |

## 9. New Commands

| Command | Input | Output |
|---|---|---|
| `start_conversation` | workspace_path | conversation_id, title |
| `list_conversations` | workspace_path, limit | list of conversation stubs |
| `get_conversation` | conversation_id | full turn history |
| `save_memory_fact` | content, citation, scope | fact_id |
| `list_memory_facts` | workspace_path | list of facts |
| `delete_memory_fact` | fact_id | ack |
| `get_document_summary` | file_id, granularity | summary text |
| `build_long_context` | query, conversation_id, token_budget | assembled prompt + metadata |

## 10. New Python Worker

`python/workers/summarization_worker.py`
- Input: `{task: "summarize", text: "...", granularity: "document|section", model: "llama3.2:3b"}`
- Output: `{summary: "...", token_count: N, model_version: "..."}`
- Uses Ollama HTTP API (localhost:11434)
- Called after ingestion completes, per file, for document and section granularities

## 11. Phase Mapping

| Phase | Memory Work |
|---|---|
| Phase 2 | Conversation memory + context assembler (pairs with retrieval) |
| Phase 2 | Document summary cache + summarization worker |
| Phase 3 | User memory facts (pairs with file action review workflow) |
| Phase 4 | Memory regression tests + benchmark reproducibility with context |
| Phase 5 | UI: conversation thread panel, memory fact manager |

## 12. Exit Criteria (Memory Feature Ready)
1. Conversation history loads within 100ms from SQLite on app start.
2. Context assembler never sends a prompt exceeding `model_context_window` tokens.
3. Document summaries generated for 100% of successfully indexed files within 30s post-ingestion.
4. Memory fact recall tested: 5 pinned facts retrieved in 10 queries where cosine sim > 0.7.
5. All memory data verified local-only (no network calls outside Ollama localhost).
