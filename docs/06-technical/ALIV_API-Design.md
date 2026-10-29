# API Design
## Vault commands, gateway, agent tools and integrations

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active. §2 is as built; §3 onwards is planned.

---

## 1. Conventions

- **Transport:** Tauri IPC `invoke('plugin:aliv-vault|<command>', args)`. Arguments are camelCase in JS and snake_case in Rust (serde).
- **Errors:** a rejected promise with a string message from `VaultError`. The UI shows it as is, so messages are written for users.
- **Types:** mirrored in `web-app/src/aliv/types.ts`. `tests/ipc.rs` checks the command list and permissions against the Tauri mock runtime.
- **Permissions:** each command is listed in the plugin's `permissions/` and granted in `src-tauri/capabilities`.
- **Audit:** commands that change state or call models write an audit entry.

## 2. Vault commands (R0, shipped)

| Command | Args | Returns | Audited |
|---|---|---|---|
| `list_matters` | — | `Matter[]` | |
| `create_matter` | `input: {name, client, kind, policy}` | `Matter` | ✓ |
| `update_matter` | `id, input` | `Matter` | ✓ (policy changes with old → new) |
| `delete_matter` | `id` | — | ✓ |
| `ingest_paths` | `matterId, paths[]` | `IngestReport{scanned, indexed, unchanged, skipped, failed, ocr, errors[]}` | ✓ |
| `list_documents` | `matterId` | `Document[]` | |
| `search` | `matterId, query, limit?, queryVector?` | `SearchHit[]{chunkId, documentId, fileName, path, ordinal, text, snippet, score}` | |
| `pending_embeddings` | `matterId, limit?` | `PendingPassage[]` | |
| `store_embeddings` | `embeddings: {chunkId, vector}[]` | count | |
| `embedding_status` | `matterId` | `{passages, embedded}` | |
| `vault_diagnostics` | — | `VaultDiagnostics` (counts, reasons, audit status; no text) | |
| `review_grid` | `matterId` | `GridRow[]` (register rows with fields) | |
| `propose_renames` | `matterId` | `Action[]` | ✓ |
| `list_actions` | `matterId` | `Action[]` | |
| `set_action_status` | `id, status` | `Action` | ✓ |
| `execute_approved` | `matterId` | `Action[]` | ✓ |
| `undo_action` | `id` | `Action` | ✓ |
| `authorize_model_call` | `request: {matterId, provider, model, location, charsSent, sourceCount, purpose}` | `{allowed, reason, auditId}` | ✓ (allowed and refused) |
| `list_audit` | `matterId?, limit?` | `AuditEntry[]` | |
| `verify_audit` | — | `{ok, entries, firstBrokenId?}` | |
| `export_register_csv` | `matterId, dest` | rows written | ✓ |

## 3. Model gateway (R1)

| Command | Args | Returns |
|---|---|---|
| `gateway_plan` | `ModelRequest{matterId, stepType, sources[], instructions}` | `RoutePlan{route, provider, model, redactionLevel, estimatedCost}` (no call made) |
| `gateway_preview` | `ModelRequest` | `{redactedPrompt, replacements[]}` (exactly what would be sent) |
| `gateway_call` | `ModelRequest, streamChannel` | stream of `{delta}` then `{done, modelCallId, citations[]}` |
| `gateway_cancel` | `callId` | — |
| `set_provider_key` / `delete_provider_key` | `provider, key` | — (stored in the OS keychain; never returned) |
| `get_matter_settings` / `set_matter_settings` | `matterId, settings` | `MatterSettings` (audited) |

**Step types:** `extract`, `classify`, `answer`, `summarise`, `draft`, `compare`, `redline`, `research`.
Example routing for a Hybrid matter: `extract → local`, `draft → cloud`.

## 4. Agent runtime (R2–R3)

| Command | Purpose |
|---|---|
| `table_create`, `table_add_column`, `table_update_column`, `table_run`, `table_pause`, `table_cells`, `table_edit_cell`, `table_comment`, `table_export` | Review tables |
| `definition_list`, `definition_save`, `definition_publish` | Workflows and agents |
| `run_start`, `run_status`, `run_cancel`, `run_resume`, `run_log` | Runs (events streamed over a channel) |
| `playbook_*`, `clause_*`, `kb_*` | Libraries |
| `monitor_*`, `inbox_list`, `inbox_update` | Monitors (R3) |

### Agent tools (what a step may call)

| Tool | Effect | Needs approval |
|---|---|---|
| `vault.search(query, scope)` | Read passages | no |
| `vault.fields(document)` | Read register | no |
| `table.read(table, filter)` | Read cells | no |
| `kb.search(kb, query)` | Read firm knowledge | no |
| `doc.write(kind, content)` | Create an output file in the matter's output folder | no (new files only) |
| `file.propose_rename/move` | Proposal | **yes** |
| `message.propose(to, body)` | Draft an outbound message (R3) | **yes** |
| `research.search(source, query)` | External research (R4) | Hybrid + confirmation |

Tools that change existing files or leave the machine only ever create proposals.

## 5. Local API and CLI

- **Local HTTP API (base app, opt-in):** OpenAI-compatible `/v1/chat/completions` served on loopback for local models. In R1 it is limited to models only. Vault data is never exposed over HTTP on the desktop.
- **CLI (`aliv-cli`):** terminal agent from the base app. Planned vault subcommands: `aliv vault ingest <matter> <path>`, `aliv vault search`, `aliv vault audit verify`, `aliv run <workflow>` (R2) for scripted, scheduled jobs.

## 6. Firm Server API (R4)

- REST + JSON over HTTPS with OIDC bearer tokens. Each desktop command has a server endpoint (`/api/v1/matters/{id}/search`, …) with the same types.
- Server-sent events for streams and run progress.
- Every request is checked against `matter_access` (ethical walls) and audited server-side.

## 7. Word add-in and connectors (R4)

- **Word add-in** talks to the local desktop app via a loopback HTTPS endpoint (paired with a one-time code), or to the Firm Server: `playbook_run(document)`, `redlines_apply`.
- **Research connectors:** `Connector{id, search(query) → results[{title, url, snippet, citation}]}`. Licensed sources are added through partners.

## 8. Versioning

- IPC commands are versioned with the app. Breaking changes need a migration in the web layer in the same PR.
- Firm Server API: `/api/v1`, with additive changes only within a version.
