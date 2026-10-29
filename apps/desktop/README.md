# ALIV Desktop

Local-first AI workspace for legal and finance teams. Client documents are indexed on your own computer. Answers cite their sources, file changes wait for approval, and every model call is recorded in a tamper-evident audit log.

Built with Tauri 2 (Rust) and React. Runs local models through llama.cpp or MLX, and can use cloud models when a matter's policy allows it.

## Features

- **Matters.** One vault per client engagement, typed Legal or Finance, with a model policy:
  - **Local only** (default): passages never go to a cloud model.
  - **Hybrid**: cloud models are allowed, but only the matching passages are sent, and every call is logged.
- **Ingestion.** Point a matter at folders. PDF, DOCX, XLSX/XLS, PPTX, HTML, TXT/MD, CSV, RTF and EML files are parsed on the device, split into passages and indexed with SQLite FTS5. Unchanged files are skipped by content hash. Unreadable files stay listed with the reason.
- **OCR.** Scanned images (PNG, JPEG, TIFF) and PDFs with no text layer are read with [Tesseract](https://github.com/tesseract-ocr/tesseract), on the device. Low-resolution scans are enlarged first, and PDF pages are rendered at 300 dpi (up to 50 pages). Each OCR'd document shows its OCR confidence, and its extracted values are scaled by it, so a poor scan lands in *Needs review*. OCR needs Tesseract, plus Poppler for scanned PDFs: `brew install tesseract poppler` on macOS, `sudo apt install tesseract-ocr poppler-utils` on Debian/Ubuntu, or the [UB Mannheim installer](https://github.com/UB-Mannheim/tesseract/wiki) on Windows. Set `ALIV_TESSERACT` / `ALIV_PDFTOPPM` to use binaries outside the usual locations. Without Tesseract, scans stay listed as skipped with that reason.
- **Register.** Rules extract type, parties, effective, expiry/due and renewal-notice dates, amount, notice period, governing law and the penalty clause. Every value links to the passage it was read from, with a confidence score. Filters: due in 90 days, passed, needs review. The register exports to CSV, with formula injection neutralized.
- **Hybrid search.** Passages are found by keyword (SQLite FTS5) and by meaning, fused with reciprocal-rank fusion. The app embeds passages in the background with the local llama.cpp embedding model, never a cloud provider, so it is safe for Local only matters; the Documents tab shows the coverage. Without the embedding model, search is keyword-only.
- **Summaries.** Each document gets a one-paragraph overview at ingestion (type, parties, key dates, amount, opening words), built from the extracted fields with no model, and shown under Documents.
- **Ask.** Retrieves the top passages plus the register, sends them as numbered sources, and asks the model to cite each statement. Sources are fitted to the selected model's context window (8,192 tokens when it is set to auto), passages first, then register rows for the matching documents and the question's parties, then the nearest deadlines; the answer says how many were left out. Answers stream, and each `[n]` opens its source. Without a model, deadline questions are answered directly from the register ("computed, no model used"), and other questions return the matching passages.
- **Policy gate.** Every model call made by Ask goes through `authorize_model_call` in Rust first (routing all calls, web search and MCP tools through one Rust gateway is the next release; see [Security and privacy](../../docs/guides/security.md)). A cloud model on a Local only matter is refused and the refusal is logged. Providers count as local when they are built-in engines (llama.cpp, MLX) or point at loopback.
- **Approvals.** ALIV proposes `Type_Counterparty_Date` renames with a risk level. Nothing on disk changes until a proposal is approved and applied. Applying refuses to overwrite files, and each rename can be undone.
- **Vault health.** Matters → Vault health shows what the vault holds, why any file could not be read, whether Tesseract and Poppler were found, meaning-index coverage and whether the audit chain verifies. "Copy report" copies it as JSON for support: counts and reasons only, never document text. App logs are under Settings → General.
- **Audit log.** Ingestion, policy changes, model calls (allowed and blocked), proposals and file changes are recorded in a SHA-256 hash chain, with a one-click integrity check.
- **Models.** Local models download straight from Hugging Face, with no mirror in between. A model catalog ships with the app (`web-app/public/catalog/`). The first-run setup recommends `unsloth/Qwen3.5-4B-GGUF`.

## Layout

| Where | What |
|---|---|
| `src-tauri/` | Rust app: model engines, MCP, agent runtime, local API server |
| `src-tauri/plugins/tauri-plugin-aliv-vault/` | The matter vault: ingestion, passage search, field extraction, file actions, policy gate, audit log |
| `src-tauri/aliv-cli/` | Headless `aliv` command-line agent |
| `web-app/` | React UI; Matters workspace in `web-app/src/aliv/` |
| `core/`, `extensions/`, `packages/` | Extension runtime, built-in extensions, SDK packages |
| `scripts/aliv-demo-fixture.py` | Builds the browser-preview fixture from `demo/corpus` using the real extractor |

## Build and run

Requires Node 22 and Rust (the version is pinned by `rust-toolchain.toml`). On Linux you also need `libwebkit2gtk-4.1-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`.

```bash
cd apps/desktop
corepack enable                    # uses yarn 4.5.3 from package.json
# if repo.yarnpkg.com is blocked: export COREPACK_NPM_REGISTRY=https://registry.npmjs.org
yarn install
yarn build:tauri:plugin:api && yarn build:core && yarn build:extensions
yarn dev                           # desktop app
yarn dev:web                       # browser preview at /matters/ (demo data, simulated file actions)
```

## Tests

```bash
yarn test:all                                                   # all JS/TS tests
yarn download:bin && node scripts/stub-tauri-resources.mjs
(cd src-tauri && cargo test --lib)                              # Rust app
(cd src-tauri/plugins/tauri-plugin-aliv-vault && cargo test)    # vault, incl. IPC contract test
python3 ../../benchmarks/score_vault_extraction.py <scan.json>  # extraction accuracy (see script header)
```

`tests/ipc.rs` sends the exact payloads the web app sends through Tauri's IPC and permission layer, so a renamed command or argument fails in CI rather than in the app.

Accuracy on the labelled corpus in `benchmarks/` (200 documents, including 40 scanned images read by OCR): type, effective date, expiry date, amount and vendor are 100% correct. Penalty clauses: 100% recall, 88% precision, because late-payment interest clauses are counted as extra. That corpus is generated from templates, so expect lower numbers on real-world documents. The register therefore always shows the source passage and flags low-confidence values.

## Release infrastructure (not set up yet)

- **Auto-update** is off. Set `AUTO_UPDATER_DISABLED=false` once releases are signed with an ALIV key: put the key in `tauri.conf.json` under `plugins.updater.pubkey`, and add endpoints.
- **CLI and agent update manifests** point at `https://github.com/VenkataAnilKumar/ALIV/releases/download/<channel>/<file>`. They work once releases with those tags and assets are published.

## Roadmap

The product roadmap is in [Implementation Phases](../../docs/07-implementation/ALIV_Implementation-Phases.md) and the [Sprint Plan](../../docs/07-implementation/ALIV_Sprint-Plan.md). In short:

1. **R1 Hybrid foundation (next).** A model gateway in the Rust core, built on the existing provider proxy (`src-tauri/src/core/server/proxy.rs`) and keychain storage (`provider_secrets.rs`), so every model call is policy-checked and audited and can't be bypassed. Then per-step routing and redaction before cloud calls.
2. **R2 Work product.** Review tables, workflows, playbooks with redlines, drafting, compare.
3. **R3 Agents.** Agent builder, monitors, knowledge bases, finance pack.
4. **R4 Scale.** Firm Server, SSO, Word add-in, research connectors.
5. **Release (alongside R2–R3).** Signed builds, update feed, vault encryption at rest, onboarding.

## License

Apache-2.0. See [LICENSE](LICENSE) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
