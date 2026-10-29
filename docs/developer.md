# Developer guide

How ALIV is put together, and how to build and test it. The app lives in [`apps/desktop`](../apps/desktop); its [README](../apps/desktop/README.md) has the platform prerequisites and the quick-start commands.

## Architecture

```
web-app (React, TanStack Router, Zustand)
  └─ src/aliv/                Matters UI and its logic
       ├─ service.ts            VaultService: the UI's contract with the vault
       ├─ tauri-service.ts      desktop: invokes plugin:aliv-vault|<command>
       ├─ demo-service.ts       browser preview and tests: in-memory, seeded from demo/corpus
       ├─ ask.ts                retrieval → context budget → policy check → streamed answer
       ├─ semantic.ts           embeds passages and questions with the local engine
       └─ useMeaningIndex.ts    keeps a matter's embeddings current in the background
src-tauri (Rust, Tauri 2)
  ├─ src/                     app shell, model engines, local API server, agent runtime
  ├─ aliv-cli/                the headless `aliv` agent
  └─ plugins/tauri-plugin-aliv-vault/   the matter vault
       ├─ store.rs              SQLite: matters, documents, chunks (FTS5), chunk_vectors,
       │                        fields, actions, hash-chained audit
       ├─ vault.rs              ingestion, rename proposals, policy check, diagnostics
       ├─ extract.rs            rule-based field extraction and summaries
       ├─ ocr.rs                Tesseract / pdftoppm OCR
       ├─ semantic.rs           vector normalization and reciprocal-rank fusion
       ├─ chunk.rs              passage splitting
       └─ commands.rs           the Tauri command surface
```

The vault runs no language model. Extraction, summaries and OCR are deterministic. Embeddings come from the app's local llama.cpp engine (`@aliv/core`'s `embedTexts`), and the vault only stores vectors and ranks with them.

### How a question is answered

1. **Search.** `ask()` embeds the question (if the local embedder is available) and calls `search` with the text and the vector. The vault ranks passages by FTS5 keyword match and, when a vector is given, by cosine similarity. It fuses the two rankings with reciprocal-rank fusion: keyword weight 0.55, k = 60.
2. **Plan.** `buildAskPlan` adds passages, then register rows ranked by relevance, while they fit the model's context window less 1,024 tokens for the answer.
3. **Policy check.** `authorize_model_call` checks the matter's policy against where the model runs, writes an audit entry, and allows or refuses the call.
4. **Answer.** The model is called with instructions to answer only from the numbered sources and cite them. With no model, deadline questions are answered from the register.

### Ingestion

`Vault::ingest_paths` walks the folders and hashes each file, skipping unchanged ones. For each changed file it:

1. parses the file, falling back to OCR for images and for PDFs with no text layer;
2. splits the text into passages;
3. extracts fields and a summary;
4. writes everything in one transaction.

Re-ingesting a file replaces its passages, and the passages' embeddings go with them through a cascade delete.

### Vault commands

`list_matters`, `create_matter`, `update_matter`, `delete_matter`, `ingest_paths`, `list_documents`, `search`, `pending_embeddings`, `store_embeddings`, `embedding_status`, `review_grid`, `propose_renames`, `list_actions`, `set_action_status`, `execute_approved`, `undo_action`, `authorize_model_call`, `list_audit`, `verify_audit`, `vault_diagnostics`, `export_register_csv`.

To add a command, change all of these:
- `commands.rs`
- `lib.rs` (the invoke handler)
- `build.rs` (`COMMANDS`)
- `permissions/default.toml`
- `web-app/src/aliv/service.ts`, `tauri-service.ts` and `demo-service.ts`
- `tests/ipc.rs`, which sends the web app's exact payloads through Tauri's IPC and permission layer

### Schema changes

`store.rs` creates tables with `CREATE TABLE IF NOT EXISTS`, which leaves existing tables as they are. To add a column, add it to `SCHEMA` and to `migrate()`, and extend the `a_vault_from_before_ocr_is_migrated` test.

## Build and test

From `apps/desktop`:

```bash
corepack enable && yarn install
yarn build:tauri:plugin:api && yarn build:core && yarn build:extensions
yarn lint && (cd web-app && yarn exec tsc -b) && yarn test:all        # web
node scripts/stub-tauri-resources.mjs                                  # once, before Rust tests
(cd src-tauri && cargo test --lib -- --test-threads=1)                 # desktop
(cd src-tauri && cargo test --no-default-features --features cli --lib -- --test-threads=1)  # CLI
(cd src-tauri/aliv-cli && cargo test --features cli)                   # CLI binary and integration tests
(cd src-tauri/plugins/tauri-plugin-aliv-vault && cargo test --target-dir target)   # vault
```

Notes:
- **One test thread.** The agent and CLI tests share one process-wide test home (user skills, config), so run them with `--test-threads=1`.
- **OCR tests.** The vault's real-OCR test runs only when Tesseract is installed. CI installs `tesseract-ocr` and `poppler-utils`.
- **Disk space.** Rust debug builds are large. `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` cuts their size several-fold, and CI uses that.
- **Extraction accuracy.** Measure it with [`benchmarks/`](../benchmarks/README.md).

## CI

[`.github/workflows/desktop.yml`](../.github/workflows/desktop.yml) runs on every pull request and push to `main` that touches `apps/desktop`. It has four jobs:
- **Web:** lint, type check and tests.
- **Rust desktop:** the desktop library tests.
- **Rust CLI:** the CLI library, binary and integration tests.
- **Vault plugin:** the vault tests.

## Conventions

- **Matter data stays on the machine** unless a Hybrid matter's Ask call sends matching passages to a cloud model. New code that sends data anywhere must go through `authorize_model_call` and be audited.
- **Vault changes come with tests** in the same module. A new command also needs the IPC contract test.
- **Licence.** `apps/desktop` builds on Apache-2.0 open-source software. Keep `LICENSE` and `THIRD_PARTY_NOTICES.md` intact.
