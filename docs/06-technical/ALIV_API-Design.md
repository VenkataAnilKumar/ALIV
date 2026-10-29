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
| `set_provider_key` / `delete_provider_key` | `provider, key` | — (stored in the OS keychain, as the base app already does in `provider_secrets.rs`; never returned) |
| `get_matter_settings` / `set_matter_settings` | `matterId, settings` | `MatterSettings` (audited) |

**Step types** (same as RTR-02 and the Hybrid AI Architecture router table): `embed`, `extract`, `classify`, `summarise`, `answer`, `compare`, `draft`, `reason`. Redlines are `draft` steps; research connectors (R4) have their own confirmation rule (RES-04).
Example routing for a Hybrid matter: `extract → local`, `draft → cloud`.

## 4. Agent runtime (R2–R3)

| Command | Purpose |
|---|---|
| `table_create`, `table_add_column`, `table_update_column`, `table_run`, `table_pause`, `table_cells`, `table_edit_cell`, `table_comment`, `table_export` | Review tables |
| `definition_list`, `definition_save`, `definition_publish` | Workflows and agents |
| `run_start`, `run_status`, `run_cancel`, `run_resume`, `run_log` | Runs (events streamed over a channel) |
| `playbook_*`, `clause_*`, `kb_*` | Libraries |
| `monitor_*`, `inbox_list`, `inbox_update` | Monitors (R3; become automation templates) |
| `automation_list`, `automation_save`, `automation_toggle`, `automation_runs`, `automation_retry` | Automation (R3) |
| `artifact_list`, `artifact_get`, `artifact_versions`, `artifact_diff`, `artifact_export`, `artifact_save_to_output` | Artifacts (R2) |

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

## 4b. Workspace, skills, tools and connectors (R1–R3)

| Command | Purpose | Release |
|---|---|---|
| `matter_chat_list`, `matter_chat_send` (stream) | Chats inside a matter, through the gateway | R1 |
| `matter_set_instructions`, `matter_folders_set`, `matter_rescan` | Workspace settings | R1 |
| `skill_list`, `skill_review`, `matter_skills_set` | Skills per matter; review shows SKILL.md and scripts. Wraps the existing `agent_skill_*` commands | R1 |
| `mcp_server_list`, `mcp_server_set_trust`, `matter_tools_set` | Trust levels and per-matter allow-lists. Wraps the existing MCP commands | R1 |
| `tool_call_approve`, `tool_call_reject` | Approvals for side-effecting tools and skill scripts | R1 |
| `web_search` (gateway) | Policy-checked, redacted, audited web search | R1 |
| `global_search` | Across permitted matters and object types | R2 |
| `connector_add`, `connector_link`, `connector_sync`, `connector_status` | Document connectors | R3 |

### ALIV as an MCP server (R4)

Read-only tools exposed to paired clients, each limited to granted matters and audited: `search_matter(matter, query)` → cited passages; `get_register(matter, filter)`; `get_artifact(matter, id)`. No tool writes to the vault or files.

## 5. Local API and CLI

- **Local HTTP API (base app, opt-in):** an OpenAI-compatible server on loopback (`src-tauri/src/core/server/`). It serves models only. Vault data is never exposed over HTTP on the desktop, and in R1 any cloud traffic through it goes through the gateway's policy check.
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
