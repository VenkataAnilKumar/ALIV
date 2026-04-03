from __future__ import annotations

import json
import sqlite3
import sys
import time
from hashlib import sha256
from pathlib import Path

import pymupdf
from docx import Document

ROOT = Path(__file__).resolve().parents[1]
PYTHON_ROOT = ROOT / "python"
if str(PYTHON_ROOT) not in sys.path:
    sys.path.insert(0, str(PYTHON_ROOT))

from workers.parsers.docx_parser import parse_docx
from workers.parsers.email_parser import parse_email
from workers.parsers.pdf_parser import parse_pdf

DB_PATH = ROOT / ".aliv" / "aliv.sqlite3"
CORPUS_DIR = ROOT / "benchmarks" / "sample_corpus"
MIGRATIONS_DIR = ROOT / "db" / "migrations"
SUPPORTED_TYPES = {"pdf", "docx", "eml"}


def stable_hex(value: str) -> str:
    return sha256(value.encode("utf-8")).hexdigest()


def now_ts() -> str:
    return str(int(time.time()))


def content_hash(path: Path) -> str:
    hasher = sha256()
    with path.open("rb") as handle:
        while True:
            chunk = handle.read(8192)
            if not chunk:
                break
            hasher.update(chunk)
    return hasher.hexdigest()


def apply_migrations(connection: sqlite3.Connection) -> None:
    migration_files = sorted(MIGRATIONS_DIR.glob("*.sql"))
    for migration_file in migration_files:
        connection.executescript(migration_file.read_text(encoding="utf-8"))


def ensure_sample_corpus() -> None:
    CORPUS_DIR.mkdir(parents=True, exist_ok=True)

    pdf_path = CORPUS_DIR / "sample.pdf"
    if not pdf_path.exists():
        document = pymupdf.open()
        page = document.new_page()
        page.insert_text((72, 72), "Sample invoice text for ingestion gate run")
        document.save(pdf_path)
        document.close()

    docx_path = CORPUS_DIR / "sample.docx"
    if not docx_path.exists():
        document = Document()
        document.add_paragraph("Sample contract renewal note")
        document.save(docx_path)

    eml_path = CORPUS_DIR / "sample.eml"
    if not eml_path.exists():
        eml_path.write_text(
            "Subject: Sample Mail\\n"
            "From: founder@example.com\\n"
            "To: team@example.com\\n"
            "\\n"
            "Body for ingestion benchmark baseline.",
            encoding="utf-8",
        )


def discover_supported_files(workspace_path: Path) -> list[dict]:
    discovered: list[dict] = []
    for path in sorted(workspace_path.rglob("*")):
        if not path.is_file():
            continue
        file_type = path.suffix.lstrip(".").lower()
        if file_type not in SUPPORTED_TYPES:
            continue

        source_path = str(path.resolve())
        discovered.append(
            {
                "id": stable_hex(f"file:{source_path}"),
                "workspace_path": str(workspace_path.resolve()),
                "source_path": source_path,
                "file_hash": content_hash(path),
                "file_type": file_type,
                "modified_at": str(int(path.stat().st_mtime)),
                "inferred_doc_type": file_type,
            }
        )

    return discovered


def parse_by_type(file_path: str, file_type: str) -> dict:
    if file_type == "pdf":
        return parse_pdf(file_path)
    if file_type == "docx":
        return parse_docx(file_path)
    if file_type == "eml":
        return parse_email(file_path)
    raise ValueError(f"unsupported file type: {file_type}")


def upsert_scan_files(
    connection: sqlite3.Connection,
    run_id: str,
    workspace_path: str,
    mode: str,
    discovered: list[dict],
    indexed_count: int,
    skipped_count: int,
    created_at: str,
) -> None:
    connection.execute(
        """
        INSERT INTO ingestion_runs
          (id, workspace_path, mode, discovered_count, indexed_count, skipped_count, parsed_count, chunk_count, embedding_count, failed_count, created_at)
        VALUES
          (?, ?, ?, ?, ?, ?, 0, 0, 0, 0, ?)
        """,
        (run_id, workspace_path, mode, len(discovered), indexed_count, skipped_count, created_at),
    )

    for file in discovered:
        connection.execute(
            """
            INSERT INTO files (id, workspace_path, source_path, file_hash, file_type, modified_at, inferred_doc_type, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
              workspace_path = excluded.workspace_path,
              source_path = excluded.source_path,
              file_hash = excluded.file_hash,
              file_type = excluded.file_type,
              modified_at = excluded.modified_at,
              inferred_doc_type = excluded.inferred_doc_type
            """,
            (
                file["id"],
                file["workspace_path"],
                file["source_path"],
                file["file_hash"],
                file["file_type"],
                file["modified_at"],
                file["inferred_doc_type"],
                created_at,
            ),
        )


def write_parse_and_chunks(connection: sqlite3.Connection, file: dict, parsed: dict, created_at: str) -> int:
    warnings_json = json.dumps(parsed.get("warnings", []))
    summary_json = json.dumps(
        {
            "page_count": len(parsed.get("pages", [])),
            "page_map_count": len(parsed.get("page_map", [])),
            "normalized_text_length": len(parsed.get("normalized_text", "")),
        }
    )

    connection.execute(
        """
        INSERT INTO parse_results (file_id, parser_name, parser_confidence, warning_count, warnings_json, summary_json, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(file_id) DO UPDATE SET
          parser_name = excluded.parser_name,
          parser_confidence = excluded.parser_confidence,
          warning_count = excluded.warning_count,
          warnings_json = excluded.warnings_json,
          summary_json = excluded.summary_json,
          updated_at = excluded.updated_at
        """,
        (
            file["id"],
            parsed.get("parser_name", "unknown"),
            float(parsed.get("parser_confidence", 0.0)),
            len(parsed.get("warnings", [])),
            warnings_json,
            summary_json,
            created_at,
        ),
    )

    connection.execute("DELETE FROM chunks WHERE file_id = ?", (file["id"],))

    chunk_count = 0
    for page in parsed.get("pages", []):
        text = str(page.get("text", ""))
        has_text = bool(page.get("has_text", False))
        if not has_text or not text.strip():
            continue
        content_hash_value = stable_hex(text)
        page_number = int(page.get("page_number", chunk_count))
        chunk_id = stable_hex(f"chunk:{file['id']}:{page_number}:{content_hash_value}")
        page_ref = f"page:{page_number}"

        connection.execute(
            """
            INSERT INTO chunks (id, file_id, chunk_index, page_ref, content, content_hash, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            """,
            (chunk_id, file["id"], page_number, page_ref, text, content_hash_value, created_at),
        )
        chunk_count += 1

    return chunk_count


def finalize_run(
    connection: sqlite3.Connection,
    run_id: str,
    parsed_count: int,
    chunk_count: int,
    failed_count: int,
) -> None:
    connection.execute(
        """
        UPDATE ingestion_runs
        SET parsed_count = ?, chunk_count = ?, embedding_count = 0, failed_count = ?
        WHERE id = ?
        """,
        (parsed_count, chunk_count, failed_count, run_id),
    )


def execute_run(connection: sqlite3.Connection, workspace_path: Path, incremental: bool) -> str:
    created_at = now_ts()
    mode = "incremental" if incremental else "full"
    run_id = stable_hex(f"ingestion-run:{workspace_path}:{created_at}:{mode}")

    discovered = discover_supported_files(workspace_path)
    existing_hashes = {
        row[0]: row[1]
        for row in connection.execute(
            "SELECT source_path, file_hash FROM files WHERE workspace_path = ?",
            (str(workspace_path.resolve()),),
        ).fetchall()
    }

    should_parse = []
    indexed_count = 0
    skipped_count = 0
    for file in discovered:
        parse_flag = (not incremental) or (existing_hashes.get(file["source_path"]) != file["file_hash"])
        should_parse.append(parse_flag)
        if parse_flag:
            indexed_count += 1
        else:
            skipped_count += 1

    upsert_scan_files(
        connection,
        run_id,
        str(workspace_path.resolve()),
        mode,
        discovered,
        indexed_count,
        skipped_count,
        created_at,
    )

    parsed_count = 0
    chunk_total = 0
    failed_count = 0

    for file, parse_flag in zip(discovered, should_parse):
        if not parse_flag:
            continue

        try:
            parsed = parse_by_type(file["source_path"], file["file_type"])
            chunk_total += write_parse_and_chunks(connection, file, parsed, created_at)
            parsed_count += 1
        except Exception:
            failed_count += 1

    finalize_run(connection, run_id, parsed_count, chunk_total, failed_count)
    return run_id


def main() -> int:
    DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    ensure_sample_corpus()

    with sqlite3.connect(DB_PATH) as connection:
        apply_migrations(connection)
        connection.execute(
            "INSERT OR IGNORE INTO workspace_folders (path, created_at) VALUES (?, ?)",
            (str(CORPUS_DIR.resolve()), now_ts()),
        )

        full_run_id = execute_run(connection, CORPUS_DIR, incremental=False)
        incremental_run_id = execute_run(connection, CORPUS_DIR, incremental=True)
        connection.commit()

    print(json.dumps({
        "status": "ok",
        "db_path": str(DB_PATH),
        "workspace_path": str(CORPUS_DIR.resolve()),
        "full_run_id": full_run_id,
        "incremental_run_id": incremental_run_id,
    }, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
