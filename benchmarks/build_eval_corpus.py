"""
W1-01: Build 200-file local evaluation corpus.

Generates a deterministic synthetic corpus of legal/finance documents:
  - 80 PDF contracts / agreements (mixed text-only and scanned-style)
  - 40 DOCX invoices and statements of work
  - 40 EML email threads with attachments
  - 20 PNG scanned pages (for OCR path)
  - 20 JPEG invoice scans (for OCR path)

Total: 200 files across benchmarks/corpus/

Label manifest written to: benchmarks/labels/corpus_manifest.json
Each file entry carries: filename, doc_type, expected_fields, and seed_id.

Run:
    python benchmarks/build_eval_corpus.py [--out benchmarks/corpus]
"""
from __future__ import annotations

import argparse
import email.mime.multipart
import email.mime.text
import hashlib
import json
import os
import random
import sys
from datetime import date, timedelta
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))

# ── dependencies ─────────────────────────────────────────────────────────────

def _require(module_name: str, pip_name: str):
    try:
        return __import__(module_name)
    except ImportError:
        sys.exit(f"Missing dependency: run  pip install {pip_name}")

fpdf_mod   = _require("fpdf", "fpdf2")
docx_mod   = _require("docx", "python-docx")
PIL_mod    = _require("PIL", "Pillow")

from fpdf import FPDF, XPos, YPos  # type: ignore
from docx import Document  # type: ignore
from PIL import Image, ImageDraw, ImageFont  # type: ignore

# ── seed data ─────────────────────────────────────────────────────────────────

VENDORS   = ["Acme Legal LLC", "Pinnacle Finance Corp", "Briarwood Associates",
             "Sterling Document Services", "Vexlar Consulting Group",
             "Meridian Capital Ltd", "Summit Audit Partners", "Axiom Legal LLP"]

CLIENTS   = ["GlobalTech Inc.", "NovaBio Systems", "Redstone Enterprises",
             "ClearPath Holdings", "Ironwood Investments", "Blue Ridge Partners"]

CONTRACT_CLAUSES = [
    "The parties agree that all disputes shall be resolved through binding arbitration.",
    "Vendor warrants that deliverables conform to the specifications in Schedule A.",
    "Client shall pay invoices within thirty (30) days of receipt.",
    "Either party may terminate this agreement with sixty (60) days written notice.",
    "Late payments shall accrue interest at 1.5% per month on the outstanding balance.",
    "All intellectual property created under this contract vests solely in the Client.",
    "Neither party shall be liable for indirect or consequential damages.",
    "This agreement shall be governed by the laws of the State of Delaware.",
]

PENALTY_CLAUSES = [
    "A penalty of $5,000 per day shall apply for delivery delays exceeding five business days.",
    "Failure to comply with data security requirements results in a $25,000 penalty.",
    "Service level breaches exceeding 99.0% uptime SLA incur a credit of 10% of monthly fees.",
]

SUBJECTS = [
    "Re: Contract Renewal - Q2 2026",
    "Fwd: Invoice #INV-{n} — Action Required",
    "Notice of Expiry — Agreement {n}",
    "Follow-up: Statement of Work #{n}",
    "Urgent: Payment Overdue — Account {n}",
    "Signed: Master Services Agreement v{n}",
]

# ── helpers ───────────────────────────────────────────────────────────────────

def _rand(rng: random.Random, lst: list):
    return rng.choice(lst)

def _date(rng: random.Random, offset_days: int = 0) -> str:
    base = date(2025, 1, 1) + timedelta(days=rng.randint(0, 365))
    return (base + timedelta(days=offset_days)).isoformat()

def _expiry(rng: random.Random) -> str:
    return _date(rng, offset_days=rng.randint(180, 730))

def _amount(rng: random.Random) -> str:
    return f"${rng.randint(5_000, 250_000):,}.00"


# ── PDF generator ─────────────────────────────────────────────────────────────

def _build_pdf(path: Path, meta: dict, rng: random.Random):
    pdf = FPDF()
    pdf.set_auto_page_break(auto=True, margin=15)
    pdf.add_page()
    pdf.set_font("Helvetica", "B", 14)
    pdf.cell(0, 10, meta["title"], new_x=XPos.LMARGIN, new_y=YPos.NEXT, align="C")
    pdf.ln(4)
    pdf.set_font("Helvetica", "", 10)
    pdf.cell(0, 6, f"Vendor: {meta['vendor']}", new_x=XPos.LMARGIN, new_y=YPos.NEXT)
    pdf.cell(0, 6, f"Client: {meta['client']}", new_x=XPos.LMARGIN, new_y=YPos.NEXT)
    pdf.cell(0, 6, f"Effective Date: {meta['effective_date']}", new_x=XPos.LMARGIN, new_y=YPos.NEXT)
    pdf.cell(0, 6, f"Expiry Date: {meta['expiry_date']}", new_x=XPos.LMARGIN, new_y=YPos.NEXT)
    pdf.cell(0, 6, f"Contract Value: {meta['amount']}", new_x=XPos.LMARGIN, new_y=YPos.NEXT)
    pdf.ln(6)
    pdf.set_font("Helvetica", "B", 10)
    pdf.cell(0, 6, "Terms and Conditions", new_x=XPos.LMARGIN, new_y=YPos.NEXT)
    pdf.set_font("Helvetica", "", 9)
    for clause in meta.get("clauses", []):
        pdf.multi_cell(180, 5, clause)
        pdf.ln(2)
    if meta.get("penalty_clause"):
        pdf.set_font("Helvetica", "B", 9)
        pdf.multi_cell(180, 5, "Penalty Clause:")
        pdf.set_font("Helvetica", "", 9)
        pdf.multi_cell(180, 5, meta["penalty_clause"])
    # Add a second page for longer contracts
    if rng.random() > 0.5:
        pdf.add_page()
        pdf.set_font("Helvetica", "B", 10)
        pdf.cell(0, 6, "Schedule A - Deliverables", new_x=XPos.LMARGIN, new_y=YPos.NEXT)
        pdf.set_font("Helvetica", "", 9)
        for i in range(rng.randint(3, 7)):
            pdf.cell(0, 5, f"  {i+1}. Deliverable item #{rng.randint(100, 999)}: due {_date(rng)}", new_x=XPos.LMARGIN, new_y=YPos.NEXT)
    pdf.output(str(path))


# ── DOCX generator ────────────────────────────────────────────────────────────

def _build_docx(path: Path, meta: dict, rng: random.Random):
    doc = Document()
    doc.add_heading(meta["title"], level=1)
    tbl = doc.add_table(rows=1, cols=2)
    tbl.style = "Table Grid"
    hrow = tbl.rows[0].cells
    hrow[0].text = "Field"
    hrow[1].text = "Value"
    for k, v in [
        ("Vendor", meta["vendor"]),
        ("Client", meta["client"]),
        ("Invoice No.", meta.get("invoice_no", "N/A")),
        ("Issue Date", meta["effective_date"]),
        ("Due Date", meta["expiry_date"]),
        ("Amount Due", meta["amount"]),
    ]:
        row = tbl.add_row().cells
        row[0].text = k
        row[1].text = v
    doc.add_paragraph()
    doc.add_heading("Description of Services", level=2)
    for i in range(rng.randint(2, 5)):
        doc.add_paragraph(
            f"Line item {i+1}: {_rand(rng, ['Consulting', 'Development', 'Review', 'Audit', 'Analysis'])} "
            f"services — {rng.randint(8, 120)} hours @ ${rng.randint(100, 400)}/hr",
            style="List Bullet",
        )
    if meta.get("penalty_clause"):
        doc.add_heading("Penalty Clause", level=2)
        doc.add_paragraph(meta["penalty_clause"])
    doc.save(str(path))


# ── EML generator ─────────────────────────────────────────────────────────────

def _build_eml(path: Path, meta: dict, rng: random.Random):
    msg = email.mime.multipart.MIMEMultipart("mixed")
    msg["From"]    = f"{meta['vendor']} <legal@{meta['vendor'].lower().replace(' ', '')}.com>"
    msg["To"]      = f"{meta['client']} <accounts@{meta['client'].lower().replace(' ', '')}.com>"
    msg["Subject"] = meta["subject"]
    msg["Date"]    = meta["effective_date"] + " 09:00:00 +0000"
    body = (
        f"Dear {meta['client']},\n\n"
        f"Please find details for {meta['title']}.\n\n"
        f"Invoice/Contract No.: {meta.get('invoice_no', 'N/A')}\n"
        f"Effective Date: {meta['effective_date']}\n"
        f"Expiry / Due Date: {meta['expiry_date']}\n"
        f"Amount: {meta['amount']}\n\n"
        f"{'Penalty note: ' + meta['penalty_clause'] if meta.get('penalty_clause') else ''}\n\n"
        f"Please confirm receipt.\n\nRegards,\n{meta['vendor']}"
    )
    msg.attach(email.mime.text.MIMEText(body, "plain"))
    path.write_text(msg.as_string(), encoding="utf-8")


# ── Image generator ───────────────────────────────────────────────────────────

def _build_image(path: Path, meta: dict, fmt: str):
    w, h = 794, 1123  # A4 at 96 dpi
    img = Image.new("RGB", (w, h), color=(255, 255, 255))
    draw = ImageDraw.Draw(img)
    try:
        font_b = ImageFont.truetype("arial.ttf", 18)
        font_n = ImageFont.truetype("arial.ttf", 13)
    except OSError:
        font_b = ImageFont.load_default()
        font_n = font_b

    draw.text((w // 2 - 150, 40), meta["title"], fill=(0, 0, 0), font=font_b)
    y = 100
    for label, value in [
        ("Vendor:", meta["vendor"]),
        ("Client:", meta["client"]),
        ("Date:", meta["effective_date"]),
        ("Expiry:", meta["expiry_date"]),
        ("Amount:", meta["amount"]),
    ]:
        draw.text((60, y), label, fill=(80, 80, 80), font=font_n)
        draw.text((200, y), value, fill=(0, 0, 0), font=font_n)
        y += 28
    if meta.get("penalty_clause"):
        draw.text((60, y + 10), "Penalty:", fill=(160, 0, 0), font=font_n)
        draw.text((60, y + 30), meta["penalty_clause"][:90], fill=(0, 0, 0), font=font_n)
    img.save(str(path), format=fmt)


# ── Corpus builder ────────────────────────────────────────────────────────────

def build_corpus(out_dir: Path) -> list[dict]:
    rng = random.Random(42)  # deterministic — do not change seed
    manifest: list[dict] = []

    def _meta(seed_id: int, doc_type: str, title: str, extra: dict | None = None) -> dict:
        vendor = _rand(rng, VENDORS)
        client = _rand(rng, CLIENTS)
        has_penalty = rng.random() > 0.5
        m = {
            "seed_id": seed_id,
            "doc_type": doc_type,
            "title": title,
            "vendor": vendor,
            "client": client,
            "invoice_no": f"INV-{seed_id:04d}",
            "effective_date": _date(rng),
            "expiry_date": _expiry(rng),
            "amount": _amount(rng),
            "clauses": rng.sample(CONTRACT_CLAUSES, k=rng.randint(2, 4)),
            "penalty_clause": _rand(rng, PENALTY_CLAUSES) if has_penalty else None,
            "subject": _rand(rng, SUBJECTS).format(n=seed_id),
        }
        if extra:
            m.update(extra)
        return m

    # ── PDFs (80) ─────────────────────────────────────────────────────────────
    types_pdf = (
        ["service_agreement"] * 25
        + ["nda"] * 15
        + ["invoice"] * 20
        + ["lease"] * 10
        + ["settlement"] * 10
    )
    rng.shuffle(types_pdf)
    for i, dt in enumerate(types_pdf, start=1):
        fname = f"contract_{i:03d}.pdf"
        fpath = out_dir / fname
        titles = {
            "service_agreement": f"Master Service Agreement #{i}",
            "nda": f"Non-Disclosure Agreement #{i}",
            "invoice": f"Invoice #INV-{i:04d}",
            "lease": f"Commercial Lease Agreement #{i}",
            "settlement": f"Settlement and Release Agreement #{i}",
        }
        meta = _meta(i, dt, titles[dt])
        _build_pdf(fpath, meta, rng)
        manifest.append({**meta, "filename": fname, "ext": "pdf"})

    # ── DOCX (40) ─────────────────────────────────────────────────────────────
    types_docx = ["invoice"] * 20 + ["statement_of_work"] * 20
    rng.shuffle(types_docx)
    for i, dt in enumerate(types_docx, start=81):
        fname = f"document_{i:03d}.docx"
        fpath = out_dir / fname
        title = (
            f"Invoice #INV-{i:04d}"
            if dt == "invoice"
            else f"Statement of Work #{i}"
        )
        meta = _meta(i, dt, title)
        _build_docx(fpath, meta, rng)
        manifest.append({**meta, "filename": fname, "ext": "docx"})

    # ── EML (40) ──────────────────────────────────────────────────────────────
    for i in range(121, 161):
        fname = f"email_{i:03d}.eml"
        fpath = out_dir / fname
        meta = _meta(i, "email_thread", f"Email Thread #{i}")
        _build_eml(fpath, meta, rng)
        manifest.append({**meta, "filename": fname, "ext": "eml"})

    # ── PNG scans (20) ────────────────────────────────────────────────────────
    for i in range(161, 181):
        fname = f"scan_{i:03d}.png"
        fpath = out_dir / fname
        meta = _meta(i, "scanned_contract", f"Scanned Contract #{i}")
        _build_image(fpath, meta, "PNG")
        manifest.append({**meta, "filename": fname, "ext": "png"})

    # ── JPEG invoice scans (20) ───────────────────────────────────────────────
    for i in range(181, 201):
        fname = f"scan_{i:03d}.jpg"
        fpath = out_dir / fname
        meta = _meta(i, "scanned_invoice", f"Scanned Invoice #{i}")
        _build_image(fpath, meta, "JPEG")
        manifest.append({**meta, "filename": fname, "ext": "jpg"})

    return manifest


# ── Gold questions generator ──────────────────────────────────────────────────

def build_gold_questions(manifest: list[dict], labels_dir: Path) -> list[dict]:
    """
    Produce 50 benchmark Q&A pairs from the manifest.
    Each entry: question, expected_answer, source_file, field, doc_type
    """
    rng = random.Random(99)  # separate seed for question selection
    questions: list[dict] = []

    # Candidate documents with known fields
    with_expiry   = [m for m in manifest if m.get("expiry_date")]
    with_vendor   = [m for m in manifest if m.get("vendor")]
    with_penalty  = [m for m in manifest if m.get("penalty_clause")]
    with_amount   = [m for m in manifest if m.get("amount")]
    with_client   = [m for m in manifest if m.get("client")]

    templates = [
        # expiry extraction (15 questions)
        *[
            {
                "question": f"What is the expiry date in {m['filename']}?",
                "expected_answer": m["expiry_date"],
                "source_file": m["filename"],
                "field": "expiry_date",
                "doc_type": m["doc_type"],
            }
            for m in rng.sample(with_expiry, k=min(15, len(with_expiry)))
        ],
        # vendor extraction (10 questions)
        *[
            {
                "question": f"Who is the vendor named in {m['filename']}?",
                "expected_answer": m["vendor"],
                "source_file": m["filename"],
                "field": "vendor_name",
                "doc_type": m["doc_type"],
            }
            for m in rng.sample(with_vendor, k=min(10, len(with_vendor)))
        ],
        # penalty clause (10 questions)
        *[
            {
                "question": f"Does {m['filename']} contain a penalty clause? If so, what does it say?",
                "expected_answer": m["penalty_clause"],
                "source_file": m["filename"],
                "field": "penalty_clause",
                "doc_type": m["doc_type"],
            }
            for m in rng.sample(with_penalty, k=min(10, len(with_penalty)))
        ],
        # contract value (10 questions)
        *[
            {
                "question": f"What is the total contract value or amount due in {m['filename']}?",
                "expected_answer": m["amount"],
                "source_file": m["filename"],
                "field": "contract_value",
                "doc_type": m["doc_type"],
            }
            for m in rng.sample(with_amount, k=min(10, len(with_amount)))
        ],
        # client name (5 questions)
        *[
            {
                "question": f"Who is the client or counterparty in {m['filename']}?",
                "expected_answer": m["client"],
                "source_file": m["filename"],
                "field": "client_name",
                "doc_type": m["doc_type"],
            }
            for m in rng.sample(with_client, k=min(5, len(with_client)))
        ],
    ]

    # Trim to exactly 50
    rng.shuffle(templates)
    questions = templates[:50]
    for idx, q in enumerate(questions, start=1):
        q["id"] = f"Q{idx:03d}"

    return questions


# ── Entry point ───────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Build ALIV 200-file evaluation corpus")
    parser.add_argument("--out", default="benchmarks/corpus", help="Output directory for corpus files")
    parser.add_argument("--labels", default="benchmarks/labels", help="Output directory for label files")
    args = parser.parse_args()

    out_dir    = ROOT / args.out
    labels_dir = ROOT / args.labels
    out_dir.mkdir(parents=True, exist_ok=True)
    labels_dir.mkdir(parents=True, exist_ok=True)

    print(f"Building corpus in: {out_dir}")
    manifest = build_corpus(out_dir)

    manifest_path = labels_dir / "corpus_manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    print(f"Manifest written: {manifest_path}  ({len(manifest)} entries)")

    print("Building 50 gold benchmark questions ...")
    questions = build_gold_questions(manifest, labels_dir)

    questions_path = labels_dir / "questions_gold.v1.json"
    questions_path.write_text(json.dumps(questions, indent=2), encoding="utf-8")
    print(f"Gold questions written: {questions_path}  ({len(questions)} questions)")

    # Summary
    from collections import Counter
    ext_counts = Counter(m["ext"] for m in manifest)
    dt_counts  = Counter(m["doc_type"] for m in manifest)
    print("\nCorpus summary:")
    for ext, n in sorted(ext_counts.items()):
        print(f"  .{ext}: {n} files")
    print("\nDoc types:")
    for dt, n in sorted(dt_counts.items()):
        print(f"  {dt}: {n}")
    print(f"\nTotal: {len(manifest)} files  |  Gold questions: {len(questions)}")
    print("\nStatus: W1-01 DONE  W1-02 DONE")


if __name__ == "__main__":
    main()
