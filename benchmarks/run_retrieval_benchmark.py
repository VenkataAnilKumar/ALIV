from __future__ import annotations

import argparse
import json
import os
import re
import sqlite3
import time
from pathlib import Path

RETRIEVAL_TOP5_THRESHOLD = 0.85
QUERY_LATENCY_P95_THRESHOLD_SEC = 8.0


def _safe_ratio(numerator: int, denominator: int) -> float:
    if denominator <= 0:
        return 0.0
    return numerator / denominator


def _percentile(values: list[float], percentile: float) -> float:
    if not values:
        return 0.0
    sorted_values = sorted(values)
    if len(sorted_values) == 1:
        return sorted_values[0]

    rank = (percentile / 100.0) * (len(sorted_values) - 1)
    lower = int(rank)
    upper = min(lower + 1, len(sorted_values) - 1)
    weight = rank - lower
    return sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight


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


def _search_terms(query: str) -> list[str]:
    normalized = query.strip().lower()
    tokens = re.findall(r"[a-z0-9_]+", normalized)

    terms: list[str] = []
    for token in tokens:
        if len(token) < 2 or token in terms:
            continue
        terms.append(token)

    return terms


def _score(content: str, terms: list[str]) -> int:
    lowered = content.lower()
    score = 0
    for index, term in enumerate(terms):
        if not term:
            continue
        weight = 10 if index == 0 else 3
        score += lowered.count(term) * weight
    return score


def _top_hits(connection: sqlite3.Connection, query: str, top_k: int) -> list[dict]:
    terms = _search_terms(query)
    if not terms:
        return []

    explicit_files = re.findall(r"[a-z]+_\d+\.(?:pdf|docx|eml|png|jpg|jpeg|tif|tiff|bmp|webp)", query.lower())
    if explicit_files:
        like_patterns = []
        for fname in explicit_files:
            like_patterns.extend([f"%/{fname}", f"%\\{fname}", f"%{fname}"])
        where_clause = " OR ".join([f"lower(f.source_path) LIKE ?{i + 1}" for i in range(len(like_patterns))])
        sql = (
            "SELECT c.file_id, f.source_path, c.chunk_index, c.page_ref, c.content "
            "FROM chunks c "
            "INNER JOIN files f ON f.id = c.file_id "
            f"WHERE {where_clause} "
            "LIMIT 200"
        )
        rows = connection.execute(sql, like_patterns).fetchall()
        direct_hits = [
            {
                "file_id": row[0],
                "source_path": row[1],
                "chunk_index": int(row[2] or 0),
                "page_ref": row[3],
                "score": 10_000,
            }
            for row in rows
        ]
        if direct_hits:
            direct_hits.sort(key=lambda r: (r["source_path"], r["chunk_index"]))
            return direct_hits[: max(1, top_k)]

    patterns = [f"%{term}%" for term in terms]
    where_clause = " OR ".join([f"lower(c.content) LIKE ?{i + 1}" for i in range(len(patterns))])
    sql = (
        "SELECT c.file_id, f.source_path, c.chunk_index, c.page_ref, c.content "
        "FROM chunks c "
        "INNER JOIN files f ON f.id = c.file_id "
        f"WHERE {where_clause} "
        "LIMIT 200"
    )

    rows = connection.execute(sql, patterns).fetchall()
    ranked: list[dict] = []
    for row in rows:
        score = _score(row[4] or "", terms)
        if score <= 0:
            continue
        ranked.append(
            {
                "file_id": row[0],
                "source_path": row[1],
                "chunk_index": int(row[2] or 0),
                "page_ref": row[3],
                "score": score,
            }
        )

    ranked.sort(key=lambda r: (-r["score"], r["source_path"], r["chunk_index"]))
    return ranked[: max(1, top_k)]


def evaluate_retrieval(db_path: Path, questions_path: Path, top_k: int) -> dict:
    questions = json.loads(questions_path.read_text(encoding="utf-8"))
    connection = sqlite3.connect(str(db_path))

    try:
        total = len(questions)
        passed = 0
        per_question: list[dict] = []
        latencies_sec: list[float] = []

        for item in questions:
            qid = item.get("id", "unknown")
            question = item.get("question", "")
            expected_source = str(item.get("source_file", "")).strip().lower()

            started_at = time.perf_counter()
            hits = _top_hits(connection, question, top_k)
            elapsed_sec = time.perf_counter() - started_at
            latencies_sec.append(elapsed_sec)

            hit_files = [Path(hit["source_path"]).name.lower() for hit in hits]
            is_pass = expected_source in hit_files
            if is_pass:
                passed += 1

            per_question.append(
                {
                    "id": qid,
                    "pass": is_pass,
                    "expected_source": expected_source,
                    "hit_files": hit_files,
                    "latency_sec": round(elapsed_sec, 4),
                }
            )

        top5_relevance = _safe_ratio(passed, total)
        p50_latency = _percentile(latencies_sec, 50)
        p95_latency = _percentile(latencies_sec, 95)
        latency_pass = p95_latency < QUERY_LATENCY_P95_THRESHOLD_SEC
        relevance_pass = top5_relevance >= RETRIEVAL_TOP5_THRESHOLD

        report = {
            "status": "pass" if relevance_pass and latency_pass else "fail",
            "metric": "retrieval_top5_relevance",
            "threshold": RETRIEVAL_TOP5_THRESHOLD,
            "actual": round(top5_relevance, 4),
            "passed": passed,
            "total": total,
            "top_k": top_k,
            "query_latency_p50_sec": round(p50_latency, 4),
            "query_latency_p95_sec": round(p95_latency, 4),
            "query_latency_p95_threshold_sec": QUERY_LATENCY_P95_THRESHOLD_SEC,
            "latency_pass": latency_pass,
            "db_path": str(db_path),
            "questions_path": str(questions_path),
            "failed_ids": [row["id"] for row in per_question if not row["pass"]],
        }
        return report
    finally:
        connection.close()


def main() -> int:
    parser = argparse.ArgumentParser(description="Run retrieval benchmark against gold questions")
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
    parser.add_argument("--top-k", type=int, default=5, help="Top-k hits to evaluate relevance")
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

    report = evaluate_retrieval(db_path, questions_path, args.top_k)
    print(json.dumps(report, indent=2))
    return 0 if report["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
