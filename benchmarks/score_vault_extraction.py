"""Score the ALIV vault's rule-based extraction against the labelled corpus.

Usage (from apps/desktop/src-tauri/plugins/tauri-plugin-aliv-vault):
  cargo run -q --example scan --no-default-features -- ../../../../../benchmarks/corpus --json > /tmp/scan.json
  python3 ../../../../../benchmarks/score_vault_extraction.py /tmp/scan.json
"""
import json
import re
import sys
from collections import Counter
from pathlib import Path

TYPE_MAP = {
    "invoice": "Invoice", "lease": "Lease", "nda": "NDA", "email_thread": "Email",
    "service_agreement": "Service Agreement", "statement_of_work": "Statement of Work",
    "settlement": "Settlement",
}

scan = json.load(open(sys.argv[1]))
manifest = {m["filename"]: m for m in json.load(open(Path(__file__).parent / "labels" / "corpus_manifest.json"))}
digits = lambda s: re.sub(r"[^0-9.]", "", s or "")
c = Counter()
for row in scan["grid"]:
    m = manifest.get(row["document"]["fileName"])
    if not m:
        continue
    f = {x["key"]: x["value"] for x in row["fields"]}
    checks = {
        "doc_type": (TYPE_MAP.get(m["doc_type"]), f.get("doc_type")),
        "effective_date": (m.get("effective_date"), f.get("effective_date")),
        "expiry_date": (m.get("expiry_date"), f.get("expiry_date")),
        "amount": (digits(m.get("amount")) or None, digits(f.get("amount")) or None),
        "vendor": (m.get("vendor"), m["vendor"] if m.get("vendor") and m["vendor"].lower() in (f.get("parties") or "").lower() else f.get("parties")),
        "penalty": (bool(m.get("penalty_clause")) or None, bool(f.get("penalty")) or None),
    }
    for key, (want, got) in checks.items():
        if want is None:
            if got:
                c[f"{key}.extra"] += 1
            continue
        c[f"{key}.labelled"] += 1
        if got is None:
            c[f"{key}.missing"] += 1
        elif got == want:
            c[f"{key}.correct"] += 1
        else:
            c[f"{key}.wrong"] += 1

r = scan["report"]
print(f"files: {r['scanned']} scanned, {r['indexed']} indexed, {r['skipped']} skipped (no OCR), {r['failed']} failed")
print(f"{'field':<15}{'labelled':>9}{'correct':>9}{'wrong':>7}{'missing':>9}{'extra':>7}{'precision':>11}{'recall':>8}")
for key in ["doc_type", "effective_date", "expiry_date", "amount", "vendor", "penalty"]:
    n, ok, bad, miss, extra = (c[f"{key}.{s}"] for s in ["labelled", "correct", "wrong", "missing", "extra"])
    prec = ok / (ok + bad + extra) if ok + bad + extra else 0
    rec = ok / n if n else 0
    print(f"{key:<15}{n:>9}{ok:>9}{bad:>7}{miss:>9}{extra:>7}{prec:>10.1%}{rec:>8.1%}")
