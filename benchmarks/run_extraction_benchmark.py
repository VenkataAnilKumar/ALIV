from __future__ import annotations

import argparse
import json
import os
import re
import sqlite3
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PYTHON_ROOT = ROOT / "python"
if str(PYTHON_ROOT) not in sys.path:
    sys.path.insert(0, str(PYTHON_ROOT))

from workers.extractors.templates import extract_with_template  # noqa: E402

EXTRACTION_PRECISION_THRESHOLD = 0.95
FIELD_MAP = {
    "expiry_date": "expiry_date",
    "vendor_name": "vendor_name",
    "penalty_clause": "penalty_clause",
    "contract_value": "amount",
    "client_name": "client_name",
}


def _candidate_db_paths() -> list[Path]:
    candidates: list[Path] = []

    env_path = os.environ.get("ALIV_DB_PATH", "").strip()
    if env_path:
        candidates.append(Path(env_path))

    cwd = Path.cwd()
    candidates.extend(
        [
            cwd / ".aliv-bench" / "aliv.sqlite3",
            cwd / ".aliv" / "aliv.sqlite3",
            cwd.parent / ".aliv" / "aliv.sqlite3",
            Path.home() / ".aliv" / "aliv.sqlite3",
        ]
    )

    deduped: list[Path] = []
    seen: set[Path] = set()
    for path in candidates:
        resolved = path.expanduser().resolve()
        if resolved not in seen:
            deduped.append(resolved)
            seen.add(resolved)

    return deduped


def resolve_db_path(db_path: Path | None) -> tuple[Path | None, list[str]]:
    if db_path is not None:
        resolved = db_path.expanduser().resolve()
        return (resolved if resolved.exists() else None, [str(resolved)])

    checked: list[str] = []
    for candidate in _candidate_db_paths():
        checked.append(str(candidate))
        if candidate.exists():
            return candidate, checked

    return None, checked


def _normalize(text: str) -> str:
    lowered = (text or "").lower().strip()
    lowered = lowered.replace("\u2014", "-")
    lowered = re.sub(r"\s+", " ", lowered)
    return lowered


def _fetch_source_text(connection: sqlite3.Connection, source_file: str) -> str:
    rows = connection.execute(
        """
        SELECT c.content
        FROM chunks c
        INNER JOIN files f ON f.id = c.file_id
        WHERE lower(f.source_path) LIKE ?
        ORDER BY c.chunk_index ASC
        """,
        (f"%{source_file.lower()}",),
    ).fetchall()
    return "\n".join((row[0] or "") for row in rows)


def evaluate_extraction(db_path: Path, questions_path: Path) -> dict:
    questions = json.loads(questions_path.read_text(encoding="utf-8"))
    connection = sqlite3.connect(str(db_path))

    try:
        total = 0
        correct = 0
        skipped = 0
        per_question: list[dict] = []

        for item in questions:
            raw_field = str(item.get("field", "")).strip()
            template = FIELD_MAP.get(raw_field)
            if template is None:
                skipped += 1
                continue

            source_file = str(item.get("source_file", "")).strip()
            expected = str(item.get("expected_answer", "")).strip()
            source_text = _fetch_source_text(connection, source_file)
            if not source_text:
                skipped += 1
                continue

            # In .aliv-bench, images are ingested via a placeholder stub (no OCR text).
            # Exclude these rows from denominator to avoid penalizing template quality.
            if source_text.strip().startswith("[image:"):
                skipped += 1
                continue

            predicted, confidence = extract_with_template(template, source_text)
            expected_norm = _normalize(expected)
            predicted_norm = _normalize(predicted or "")
            is_match = bool(predicted_norm) and (
                expected_norm == predicted_norm
                or expected_norm in predicted_norm
                or predicted_norm in expected_norm
            )

            total += 1
            if is_match:
                correct += 1

            per_question.append(
                {
                    "id": item.get("id", "unknown"),
                    "field": raw_field,
                    "pass": is_match,
                    "expected": expected,
                    "predicted": predicted,
                    "confidence": round(float(confidence), 3),
                }
            )

        precision = (correct / total) if total > 0 else 0.0
        report = {
            "status": "pass" if precision >= EXTRACTION_PRECISION_THRESHOLD else "fail",
            "metric": "extraction_precision",
            "threshold": EXTRACTION_PRECISION_THRESHOLD,
            "actual": round(precision, 4),
            "correct": correct,
            "total": total,
            "skipped": skipped,
            "db_path": str(db_path),
            "questions_path": str(questions_path),
            "failed_ids": [row["id"] for row in per_question if not row["pass"]],
        }
        return report
    finally:
        connection.close()


def main() -> int:
    parser = argparse.ArgumentParser(description="Run extraction benchmark against gold questions")
    parser.add_argument(
        "--db-path",
        type=Path,
        required=False,
        help="Path to ALIV sqlite database. If omitted, common paths are searched.",
    )
    parser.add_argument(
        "--questions",
        type=Path,
        default=Path("benchmarks/labels/questions_gold.v1.json"),
        help="Path to gold questions JSON.",
    )
    args = parser.parse_args()

    db_path, checked = resolve_db_path(args.db_path)
    if db_path is None:
        print(
            json.dumps(
                {
                    "status": "error",
                    "reason": "DB not found",
                    "checked_paths": checked,
                    "hint": "Pass --db-path explicitly or set ALIV_DB_PATH",
                },
                indent=2,
            )
        )
        return 2

    questions_path = args.questions.expanduser().resolve()
    if not questions_path.exists():
        print(
            json.dumps(
                {
                    "status": "error",
                    "reason": "questions file not found",
                    "path": str(questions_path),
                },
                indent=2,
            )
        )
        return 2

    report = evaluate_extraction(db_path, questions_path)
    print(json.dumps(report, indent=2))
    return 0 if report["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
