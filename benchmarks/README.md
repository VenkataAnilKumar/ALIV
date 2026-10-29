# Benchmarks

A labelled corpus of 200 synthetic legal and finance documents (PDF, DOCX, email, and 40 scanned PNG/JPEG images) and a script that scores the vault's field extraction against it.

| Path | What |
|---|---|
| `corpus/` | The documents |
| `labels/corpus_manifest.json` | Expected type, parties, dates, amount and penalty clause per document |
| `labels/questions_gold.v1.json` | 50 questions with their source document and expected answer |
| `build_eval_corpus.py` | Regenerates the corpus and labels |
| `score_vault_extraction.py` | Scores a vault scan against the labels |

## Score the vault's extraction

From `apps/desktop/src-tauri/plugins/tauri-plugin-aliv-vault`, with Tesseract and Poppler installed so the scans are read:

```bash
cargo run -q --example scan --no-default-features -- ../../../../../benchmarks/corpus --json > /tmp/scan.json
python3 ../../../../../benchmarks/score_vault_extraction.py /tmp/scan.json
```

The corpus is generated from templates, so expect lower numbers on real documents.
