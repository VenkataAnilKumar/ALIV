# Security and privacy

What ALIV stores, what can leave the computer, and the controls around it. This is written for a firm's IT or risk team.

## Data flow

```
Your folders ──read──▶ Vault (vault.db, on this computer)
                          │  passages, extracted fields, embeddings, audit log
                          ▼
              Question ──▶ Search (on this computer) ──▶ matching passages
                                                            │
                               policy check (Rust) ─────────┤
                                                            ▼
                               Local model (llama.cpp / MLX)   or   Cloud model (Hybrid matters only)
```

## What stays on the computer

- **Documents:** they are read in place and never copied, uploaded or modified on import.
- **The vault:** passages, extracted fields, summaries, embeddings and the audit log are kept in one SQLite file. By default it is `ALIV/data/aliv/vault.db` under the operating system's application-data folder.
- **Processing:** parsing, OCR (Tesseract), search, field extraction and summaries all run on the computer, with no model and no network.
- **Embeddings:** these are what let search match by meaning. They are computed by the local llama.cpp engine's embedding model and never by a cloud provider, so they are produced even for Local only matters without anything leaving the machine.

## What can leave the computer

**Only the text of an Ask request sent to a cloud model, and only for a matter set to Hybrid.** That request contains:
- the question;
- the matching passages and register rows, trimmed to the model's context window;
- the instructions.

Whole documents are never sent. Each such call is recorded in the audit log with the provider, model, size and number of sources.

Everything else that touches the network is user-initiated and carries no matter data:
- downloading models from Hugging Face;
- the model providers you configure;
- MCP servers you add.

## Controls

| Control | How it works |
|---|---|
| **Matter policy** | Each matter is *Local only* (default) or *Hybrid*. Before Ask calls a model, the app classifies where it runs, and `authorize_model_call` in the vault (Rust) decides and logs the call. llama.cpp, MLX, Ollama, LM Studio and any provider pointing at a loopback address count as local; everything else is cloud. A cloud model on a Local only matter is refused, and the refusal is logged. |
| **Audit log** | Ingestion, policy changes, model calls (allowed and refused), file proposals and file changes are appended to a SHA-256 hash chain. **Verify integrity** recomputes the chain and reports the first entry that does not match. |
| **File actions** | Renames are proposals until approved, never overwrite an existing file, and can be undone. |
| **CSV export** | Cells that start with `=`, `+`, `-` or `@` are neutralized, so an exported register cannot run formulas in a spreadsheet. |
| **Vault health report** | Counts and reasons only, never document text, so it can be shared with support. |

## Limits to know about

- **The vault is not encrypted by ALIV.** Protect it with full-disk encryption (FileVault, BitLocker, LUKS), as you would the documents themselves.
- **The audit log is tamper-evident, not tamper-proof.** Verification detects an altered or reordered entry. It cannot prevent someone with access to the file from deleting the whole vault.
- **The policy check guards the Ask flow.** The cloud request itself is sent from the app after the check approves it. Routing cloud calls through the Rust proxy, so that no code path can skip the check, and pseudonymizing names and amounts before a cloud call, are on the roadmap. Until then, keep matters that must never reach a cloud provider on *Local only*, and use only local models for them.
- **Cloud providers' own handling** of the text sent to them (retention, training) is governed by your agreement with that provider.

## Reporting a vulnerability

See [`apps/desktop/SECURITY.md`](../apps/desktop/SECURITY.md).
