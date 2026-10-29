# Data Model
## Vault schema, current and planned

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Active. §2 is as built (`tauri-plugin-aliv-vault/src/store.rs`); §3 is planned.

---

## 1. Overview

One SQLite file per installation (`ALIV/data/aliv/vault.db` under the OS app-data folder). Every row belongs to a matter, directly or through its document. Deleting a matter cascades to its documents, passages, fields, vectors and actions. The audit log is kept.

```
matters 1─* documents 1─* chunks 1─1 chunk_vectors
   │            │            └─ chunks_fts (FTS5, external content)
   │            └─* fields
   ├─* actions
   └─* audit (matter_id nullable for global events)
```

## 2. Current schema (R0)

### `matters`
| Column | Type | Notes |
|---|---|---|
| `id` | TEXT PK | UUID v4 |
| `name` | TEXT | required |
| `client` | TEXT | default '' |
| `kind` | TEXT | `legal` \| `finance` |
| `policy` | TEXT | `local_only` (default) \| `hybrid` |
| `created_at` | TEXT | RFC 3339 |

### `documents`
| Column | Type | Notes |
|---|---|---|
| `id` | TEXT PK | UUID |
| `matter_id` | TEXT FK → matters | cascade |
| `path`, `file_name`, `ext` | TEXT | original location; file is never copied |
| `size` | INTEGER | bytes |
| `sha256` | TEXT | change detection on re-add |
| `status` | TEXT | `indexed`, `skipped`, `failed` |
| `error` | TEXT | reason when not indexed (e.g. protected, empty, needs OCR) |
| `doc_type` | TEXT | classified type (lease, NDA, MSA, invoice…) |
| `char_count` | INTEGER | |
| `ingested_at` | TEXT | |
| `ocr_confidence` | REAL | mean OCR confidence, null if not OCR'd |
| `summary` | TEXT | extractive summary |
| | | `UNIQUE (matter_id, path)` |

### `chunks` (passages) and `chunks_fts`
| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK | rowid for FTS5 |
| `document_id` | TEXT FK | cascade |
| `matter_id` | TEXT | denormalised for fast filtering |
| `ordinal` | INTEGER | position in document; citations point here |
| `text` | TEXT | |

`chunks_fts` is an FTS5 external-content table (`porter unicode61`), kept in sync by insert/delete triggers.

### `chunk_vectors`
| Column | Type | Notes |
|---|---|---|
| `chunk_id` | INTEGER PK FK → chunks | cascade |
| `dim` | INTEGER | embedding dimension |
| `vector` | BLOB | little-endian f32, unit length |

### `fields` (register)
| Column | Type | Notes |
|---|---|---|
| `document_id` | TEXT FK | PK part |
| `key` | TEXT | PK part: `doc_type`, `parties`, `effective_date`, `expiry_date`, `renewal_notice_deadline`, `amount`, `notice_period`, `governing_law`, `penalty`… |
| `value` | TEXT | normalised (ISO dates) |
| `confidence` | REAL | 0–1 |
| `chunk_ordinal` | INTEGER | source passage |
| `snippet` | TEXT | evidence |

### `actions` (proposals)
`id`, `matter_id`, `document_id`, `kind` (`rename`), `from_path`, `to_path`, `reason`, `risk` (`low|medium|high`), `status` (`proposed|approved|rejected|executed|undone|failed`), `error`, `created_at`, `updated_at`.

### `audit`
| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER PK AUTOINCREMENT | order of the chain |
| `matter_id` | TEXT NULL | |
| `ts` | TEXT | |
| `kind` | TEXT | e.g. `matter_created`, `policy_changed`, `model_call`, `model_call_blocked`, `ingest`, `export`, `actions_proposed`, `action_executed`, `action_undone`, `matter_deleted` |
| `summary` | TEXT | human-readable |
| `detail` | TEXT | JSON; never document text |
| `prev_hash`, `hash` | TEXT | `hash = sha256(prev ‖ matter_id ‖ ts ‖ kind ‖ summary ‖ detail)`, each part followed by a NUL byte |

## 3. Planned schema (R1–R4)

| Table | Release | Key columns |
|---|---|---|
| `model_calls` | R1 | `id`, `matter_id`, `audit_id`, `step_type`, `route` (`local|cloud`), `provider`, `model`, `redaction_level`, `prompt_sha256`, `tokens_in`, `tokens_out`, `cost_usd`, `status`, `ts` |
| `entities` | R1 | `id`, `matter_id`, `kind` (`party|person|email|phone|account|amount`), `value`, `placeholder`, `source` (`register|ner|user`) |
| `matter_settings` | R1 | `matter_id`, `redaction_level`, `monthly_cloud_limit_usd`, `routes` (JSON step_type → route) |
| `review_tables` | R2 | `id`, `matter_id`, `name`, `template`, `created_at` |
| `review_columns` | R2 | `id`, `table_id`, `ordinal`, `label`, `question`, `answer_type`, `options` (JSON), `condition` (JSON), `manual` |
| `review_cells` | R2 | `table_id`, `column_id`, `document_id`, `state`, `value`, `confidence`, `chunk_ordinal`, `snippet`, `original_value`, `edited_by`, `run_id`, `updated_at` |
| `cell_comments` | R2 | `id`, `table_id`, `column_id`, `document_id`, `author`, `text`, `ts` |
| `definitions` | R2 | `id`, `kind` (`workflow|agent`), `name`, `version`, `spec` (JSON steps), `published_at`, `change_note` |
| `runs` | R2 | `id`, `matter_id`, `definition_id`, `version`, `status`, `inputs` (JSON), `started_at`, `finished_at` |
| `run_steps` | R2 | `run_id`, `ordinal`, `step_type`, `route`, `model_call_id`, `status`, `output_ref`, `skipped` |
| `outputs` | R2 | `id`, `matter_id`, `run_id`, `kind` (`docx|xlsx|pdf|md`), `path`, `created_at` |
| `playbooks`, `positions` | R2 | playbook: `id`, `name`, `contract_type`, `version`; position: `topic`, `preferred`, `fallbacks` (JSON), `unacceptable`, `sample_language` |
| `clauses` | R2 | `id`, `title`, `text`, `tags` (JSON), `source_document_id` |
| `knowledge_bases`, `kb_documents` | R3 | KBs are read-only collections; their passages live in `chunks` with `matter_id = 'kb:<id>'` |
| `monitors`, `inbox_items` | R3 | monitor: `kind`, `schedule`, `config` (JSON), `enabled`; item: `monitor_id`, `matter_id`, `payload` (JSON), `state` |
| `field_overrides` | R2 | user corrections kept across re-ingestion: `document_id`, `key`, `value`, `edited_by`, `ts` |
| `matter_instructions` | R1 | `matter_id`, `text`, `updated_by`, `updated_at` (FS-27) |
| `matter_folders` | R1 | `matter_id`, `kind` (`source|output`), `path` or `connector_source_id`, `last_scan_at`, `watch` (FS-27, FS-23) |
| `matter_chats`, `chat_messages` | R1 | chat: `id`, `matter_id`, `title`, `created_at`; message: `chat_id`, `role`, `text`, `model_call_id`, `skills` (JSON), `citations` (JSON) |
| `skills_enabled` | R1 | `matter_id`, `skill_name`, `version`, `trust` (`builtin|firm|personal|third_party`), `enabled_by`, `ts` (FS-28) |
| `mcp_servers` | R1 | `id`, `name`, `transport`, `trust` (`local|firm|third_party`), `firm_status` (`allowed|blocked`) (FS-24) |
| `matter_tools` | R1 | `matter_id`, `mcp_server_id`, `allowed_tools` (JSON), `web_search` (bool) |
| `tool_calls` | R1 | `id`, `matter_id`, `kind` (`mcp|web_search|skill_script`), `target`, `payload_sha256`, `redaction_level`, `approved_by`, `status`, `audit_id` |
| `artifacts`, `artifact_versions` | R2 | artifact: `id`, `matter_id`, `type`, `title`, `pinned`; version: `artifact_id`, `version`, `run_id`, `content_ref`, `citations` (JSON), `skills` (JSON), `route`, `created_by`, `created_at` (FS-25) |
| `connector_sources` | R3 | `id`, `kind` (`imanage|netdocuments|sharepoint|gdrive|box|outlook|gmail`), `config` (JSON, no secrets), `matter_id`, `last_sync_at`, `status` (FS-23) |
| `automations`, `automation_runs` | R3 | automation: `id`, `matter_id`, `trigger` (JSON), `action` (JSON), `enabled`; run: `automation_id`, `run_id`, `trigger_payload` (JSON), `status`, `started_at`, `finished_at` (FS-26; replaces `monitors`) |
| `users`, `groups`, `matter_access` | R4 | Firm Server identities, roles (`admin|member|viewer`), ethical walls |

## 4. Rules

- **Migrations** are additive and versioned (`PRAGMA user_version`). Never drop user data.
- **No document text in `audit` or `model_calls`**: hashes and sizes only.
- **Every citation** resolves to `(document_id, chunk_ordinal)`.
- **User edits** (`field_overrides`, `review_cells.original_value`) survive re-ingestion.
- **Encryption at rest (GA):** SQLCipher or page-level encryption with the key in the OS keychain.
