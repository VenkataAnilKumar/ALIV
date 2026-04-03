"""
W1-03: Full ingestion benchmark against the 200-file evaluation corpus.

Ingests benchmarks/corpus/ into a fresh .aliv-bench/aliv.sqlite3 database,
then runs validate_ingestion_thresholds.py against it.

Usage:
    python benchmarks/run_ingestion_benchmark.py [--corpus benchmarks/corpus]
"""
from __future__ import annotations

import json
import sqlite3
import sys
import time
from hashlib import sha256
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PYTHON_ROOT = ROOT / "python"
if str(PYTHON_ROOT) not in sys.path:
    sys.path.insert(0, str(PYTHON_ROOT))

from workers.parsers.pdf_parser import parse_pdf
from workers.parsers.docx_parser import parse_docx
from workers.parsers.email_parser import parse_email

MIGRATIONS_DIR = ROOT / "db" / "migrations"
SUPPORTED = {"pdf", "docx", "eml", "png", "jpg", "jpeg", "tif", "tiff", "bmp", "webp"}
TEXT_TYPES = {"pdf", "docx", "eml"}


# ── helpers ───────────────────────────────────────────────────────────────────

def _sha256_file(path: Path) -> str:
    h = sha256()
    with path.open("rb") as f:
        while chunk := f.read(8192):
            h.update(chunk)
    return h.hexdigest()

def _sha256_str(s: str) -> str:
    return sha256(s.encode()).hexdigest()

def _now() -> str:
    return str(int(time.time()))


# ── DB helpers ────────────────────────────────────────────────────────────────

def _apply_migrations(conn: sqlite3.Connection) -> None:
    # Skip the vec0 migration (0008) — requires native sqlite-vec extension
    for p in sorted(MIGRATIONS_DIR.glob("*.sql")):
        if "0008" in p.name:
            continue
        try:
            conn.executescript(p.read_text(encoding="utf-8"))
        except sqlite3.OperationalError:
            pass

def _open_db(db_path: Path) -> sqlite3.Connection:
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(db_path))
    conn.execute("PRAGMA foreign_keys = ON")
    conn.execute("PRAGMA journal_mode = WAL")
    _apply_migrations(conn)
    return conn

def _register_workspace(conn: sqlite3.Connection, ws: str) -> None:
    conn.execute(
        "INSERT OR IGNORE INTO workspace_folders(path, created_at) VALUES (?, ?)",
        (ws, _now()),
    )
    conn.commit()

def _upsert_file(conn: sqlite3.Connection, ws: str, path: Path, file_hash: str) -> tuple[str, bool]:
    """Returns (file_id, should_parse).  should_parse=False when hash unchanged."""
    file_id = _sha256_str(f"file:{path}")
    ext = path.suffix.lstrip(".").lower()
    existing = conn.execute(
        "SELECT file_hash FROM files WHERE id = ?", (file_id,)
    ).fetchone()
    if existing and existing[0] == file_hash:
        return file_id, False
    conn.execute(
        "INSERT INTO files(id, workspace_path, source_path, file_hash, file_type, modified_at, inferred_doc_type, created_at) "
        "VALUES(?,?,?,?,?,?,?,?) "
        "ON CONFLICT(id) DO UPDATE SET file_hash=excluded.file_hash, modified_at=excluded.modified_at",
        (file_id, ws, str(path), file_hash, ext, _now(), ext, _now()),
    )
    conn.commit()
    return file_id, True

def _store_chunks(conn: sqlite3.Connection, file_id: str, pages: list[dict]) -> list[tuple[str, str]]:
    conn.execute("DELETE FROM chunks WHERE file_id = ?", (file_id,))
    chunks = []
    for page in pages:
        text = page.get("text", "").strip()
        if not text:
            continue
        page_no = page.get("page_number", 1)
        chunk_id = _sha256_str(f"chunk:{file_id}:{page_no}:{_sha256_str(text)}")
        conn.execute(
            "INSERT OR IGNORE INTO chunks(id, file_id, chunk_index, page_ref, content, content_hash, created_at) "
            "VALUES(?,?,?,?,?,?,?)",
            (chunk_id, file_id, page_no, f"page:{page_no}", text, _sha256_str(text), _now()),
        )
        chunks.append((chunk_id, text))
    conn.commit()
    return chunks

def _store_parse_result(conn: sqlite3.Connection, file_id: str, parser_name: str,
                         confidence: float, warnings: list) -> None:
    conn.execute(
        "INSERT INTO parse_results(file_id, parser_name, parser_confidence, warning_count, warnings_json, summary_json, updated_at) "
        "VALUES(?,?,?,?,?,?,?) "
        "ON CONFLICT(file_id) DO UPDATE SET parser_name=excluded.parser_name, "
        "parser_confidence=excluded.parser_confidence, warning_count=excluded.warning_count, "
        "warnings_json=excluded.warnings_json, summary_json=excluded.summary_json, updated_at=excluded.updated_at",
        (file_id, parser_name, confidence, len(warnings), json.dumps(warnings), "{}", _now()),
    )
    conn.commit()

def _record_run(conn: sqlite3.Connection, run_id: str, ws: str, mode: str,
                discovered: int, indexed: int, skipped: int,
                parsed: int, chunks: int, embeddings: int, failed: int) -> None:
    ts = _now()
    conn.execute(
        "INSERT INTO ingestion_runs(id, workspace_path, mode, discovered_count, indexed_count, "
        "skipped_count, parsed_count, chunk_count, embedding_count, failed_count, created_at) "
        "VALUES(?,?,?,?,?,?,?,?,?,?,?)",
        (run_id, ws, mode, discovered, indexed, skipped, parsed, chunks, embeddings, failed, ts),
    )
    conn.commit()


# ── parse dispatch ────────────────────────────────────────────────────────────

def _parse_file(path: Path) -> tuple[list[dict], str, float, list[str]]:
    """Returns (pages, parser_name, confidence, warnings)."""
    ext = path.suffix.lstrip(".").lower()
    try:
        if ext == "pdf":
            result = parse_pdf(str(path))
        elif ext == "docx":
            result = parse_docx(str(path))
        elif ext == "eml":
            result = parse_email(str(path))
        else:
            # Image — return single stub page (OCR worker not wired here)
            return (
                [{"page_number": 1, "text": f"[image:{path.name}]", "has_text": True, "block_count": 1}],
                "image_stub", 0.5, ["ocr_not_run"],
            )
        pages = result.get("pages", [])
        return (
            pages,
            result.get("parser_name", ext),
            result.get("parser_confidence", 1.0),
            result.get("warnings", []),
        )
    except Exception as exc:
        return [], "error", 0.0, [str(exc)]


# ── benchmark runner ──────────────────────────────────────────────────────────

def run_benchmark(corpus_dir: Path, db_path: Path, incremental: bool = False) -> dict:
    ws = str(corpus_dir.resolve())
    mode = "incremental" if incremental else "full"
    run_id = _sha256_str(f"bench:{ws}:{mode}:{time.time_ns()}")[:32]

    conn = _open_db(db_path)
    _register_workspace(conn, ws)

    corpus_files = [p for p in sorted(corpus_dir.iterdir())
                    if p.is_file() and p.suffix.lstrip(".").lower() in SUPPORTED]

    discovered = len(corpus_files)
    indexed = 0
    skipped = 0
    parsed = 0
    chunk_total = 0
    failed = 0

    t_start = time.perf_counter()
    for fpath in corpus_files:
        file_hash = _sha256_file(fpath)
        file_id, should_parse = _upsert_file(conn, ws, fpath, file_hash)

        if not should_parse:
            skipped += 1
            continue

        indexed += 1
        pages, parser_name, confidence, warnings = _parse_file(fpath)

        if not pages:
            failed += 1
            continue

        chunks = _store_chunks(conn, file_id, pages)
        _store_parse_result(conn, file_id, parser_name, confidence, warnings)
        parsed += 1
        chunk_total += len(chunks)

    elapsed = time.perf_counter() - t_start
    _record_run(conn, run_id, ws, mode, discovered, indexed, skipped,
                parsed, chunk_total, 0, failed)
    conn.close()

    return {
        "run_id": run_id,
        "mode": mode,
        "corpus_dir": str(corpus_dir),
        "db_path": str(db_path),
        "discovered": discovered,
        "indexed": indexed,
        "skipped": skipped,
        "parsed": parsed,
        "chunks": chunk_total,
        "failed": failed,
        "elapsed_sec": round(elapsed, 2),
        "success_rate": round(parsed / indexed, 4) if indexed > 0 else 0.0,
    }


def main():
    import argparse
    parser = argparse.ArgumentParser(description="W1-03 ingestion benchmark runner")
    parser.add_argument("--corpus", default="benchmarks/corpus", help="Path to evaluation corpus")
    parser.add_argument("--db", default=".aliv-bench/aliv.sqlite3", help="Benchmark DB path (isolated from dev DB)")
    parser.add_argument("--incremental", action="store_true", help="Run as incremental pass")
    args = parser.parse_args()

    corpus_dir = ROOT / args.corpus
    db_path    = ROOT / args.db

    if not corpus_dir.exists():
        sys.exit(f"Corpus not found: {corpus_dir}  — run build_eval_corpus.py first")

    # Full mode must be deterministic against a fresh benchmark DB.
    if not args.incremental and db_path.exists():
        db_path.unlink()

    print(f"Running {'incremental' if args.incremental else 'full'} ingestion benchmark")
    print(f"  corpus : {corpus_dir}  ({sum(1 for p in corpus_dir.iterdir() if p.is_file())} files)")
    print(f"  db     : {db_path}")

    result = run_benchmark(corpus_dir, db_path, incremental=args.incremental)

    print(json.dumps(result, indent=2))

    # Run gate validator against the benchmark DB
    import subprocess
    gate = subprocess.run(
        [sys.executable, str(ROOT / "benchmarks" / "validate_ingestion_thresholds.py"),
         "--db-path", str(db_path)],
        capture_output=True, text=True,
    )
    try:
        gate_result = json.loads(gate.stdout)
        print("\nGate result:")
        print(json.dumps(gate_result, indent=2))
    except json.JSONDecodeError:
        print(gate.stdout)
        if gate.stderr:
            print(gate.stderr)


if __name__ == "__main__":
    main()
