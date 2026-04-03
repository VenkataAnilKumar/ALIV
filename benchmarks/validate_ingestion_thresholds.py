from __future__ import annotations

import argparse
import json
import os
import sqlite3
from pathlib import Path

INGESTION_SUCCESS_THRESHOLD = 0.98
INCREMENTAL_NOOP_THRESHOLD = 0.95


def _safe_ratio(numerator: int, denominator: int) -> float:
    if denominator <= 0:
        return 0.0
    return numerator / denominator


def _load_latest_run(connection: sqlite3.Connection, mode: str) -> dict | None:
    row = connection.execute(
        """
        SELECT id, workspace_path, mode, discovered_count, indexed_count, skipped_count, parsed_count, failed_count, created_at
        FROM ingestion_runs
        WHERE mode = ?
        ORDER BY CAST(created_at AS INTEGER) DESC
        LIMIT 1
        """,
        (mode,),
    ).fetchone()

    if not row:
        return None

    return {
        "id": row[0],
        "workspace_path": row[1],
        "mode": row[2],
        "discovered_count": int(row[3]),
        "indexed_count": int(row[4]),
        "skipped_count": int(row[5]),
        "parsed_count": int(row[6]),
        "failed_count": int(row[7]),
        "created_at": row[8],
    }


def evaluate_ingestion_thresholds(db_path: Path) -> dict:
    connection = sqlite3.connect(db_path)
    try:
        latest_full = _load_latest_run(connection, "full")
        latest_incremental = _load_latest_run(connection, "incremental")

        if latest_full is None and latest_incremental is None:
            return {
                "status": "insufficient-data",
                "reason": "No ingestion runs found in ingestion_runs",
                "checks": [],
            }

        checks: list[dict] = []

        if latest_full is not None:
            full_success_rate = _safe_ratio(
                latest_full["parsed_count"], latest_full["indexed_count"]
            )
            checks.append(
                {
                    "name": "full_ingestion_success_rate",
                    "threshold": INGESTION_SUCCESS_THRESHOLD,
                    "actual": round(full_success_rate, 4),
                    "pass": full_success_rate >= INGESTION_SUCCESS_THRESHOLD,
                    "run_id": latest_full["id"],
                }
            )

        if latest_incremental is not None:
            noop_ratio = _safe_ratio(
                latest_incremental["discovered_count"] - latest_incremental["indexed_count"],
                latest_incremental["discovered_count"],
            )
            checks.append(
                {
                    "name": "incremental_noop_ratio",
                    "threshold": INCREMENTAL_NOOP_THRESHOLD,
                    "actual": round(noop_ratio, 4),
                    "pass": noop_ratio >= INCREMENTAL_NOOP_THRESHOLD,
                    "run_id": latest_incremental["id"],
                }
            )

        overall_pass = bool(checks) and all(check["pass"] for check in checks)
        return {
            "status": "pass" if overall_pass else "fail",
            "latest_full": latest_full,
            "latest_incremental": latest_incremental,
            "checks": checks,
        }
    finally:
        connection.close()


def _candidate_db_paths() -> list[Path]:
    candidates: list[Path] = []

    env_path = os.environ.get("ALIV_DB_PATH", "").strip()
    if env_path:
        candidates.append(Path(env_path))

    cwd = Path.cwd()
    candidates.extend(
        [
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


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate ingestion thresholds from ALIV SQLite DB")
    parser.add_argument(
        "--db-path",
        type=Path,
        required=False,
        help="Path to ALIV sqlite database (.aliv/aliv.sqlite3). If omitted, common locations are searched.",
    )
    args = parser.parse_args()

    db_path, checked_paths = resolve_db_path(args.db_path)
    if db_path is None:
        print(
            json.dumps(
                {
                    "status": "error",
                    "reason": "DB not found",
                    "checked_paths": checked_paths,
                    "hint": "Pass --db-path explicitly or set ALIV_DB_PATH",
                },
                indent=2,
            )
        )
        return 2

    report = evaluate_ingestion_thresholds(db_path)
    report["db_path"] = str(db_path)
    print(json.dumps(report, indent=2))
    return 0 if report["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
