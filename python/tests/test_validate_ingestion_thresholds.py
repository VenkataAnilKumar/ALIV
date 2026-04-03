from __future__ import annotations

import importlib.util
import sqlite3
from pathlib import Path

import pytest


def _load_validator_module():
    project_root = Path(__file__).resolve().parents[2]
    module_path = project_root / "benchmarks" / "validate_ingestion_thresholds.py"
    spec = importlib.util.spec_from_file_location("validate_ingestion_thresholds", module_path)
    if spec is None or spec.loader is None:
        raise RuntimeError("Unable to load validate_ingestion_thresholds module")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _create_ingestion_runs_table(db_path: Path) -> None:
    with sqlite3.connect(db_path) as connection:
        connection.execute(
            """
            CREATE TABLE ingestion_runs (
              id TEXT PRIMARY KEY,
              workspace_path TEXT NOT NULL,
              mode TEXT NOT NULL,
              discovered_count INTEGER NOT NULL,
              indexed_count INTEGER NOT NULL,
              skipped_count INTEGER NOT NULL DEFAULT 0,
              parsed_count INTEGER NOT NULL DEFAULT 0,
              chunk_count INTEGER NOT NULL DEFAULT 0,
              embedding_count INTEGER NOT NULL DEFAULT 0,
              failed_count INTEGER NOT NULL DEFAULT 0,
              created_at TEXT NOT NULL
            )
            """
        )


def test_validator_reports_insufficient_data(tmp_path: Path) -> None:
    module = _load_validator_module()
    db_path = tmp_path / "aliv.sqlite3"
    _create_ingestion_runs_table(db_path)

    report = module.evaluate_ingestion_thresholds(db_path)

    assert report["status"] == "insufficient-data"
    assert report["checks"] == []


def test_validator_fails_when_below_thresholds(tmp_path: Path) -> None:
    module = _load_validator_module()
    db_path = tmp_path / "aliv.sqlite3"
    _create_ingestion_runs_table(db_path)

    with sqlite3.connect(db_path) as connection:
        connection.execute(
            """
            INSERT INTO ingestion_runs
              (id, workspace_path, mode, discovered_count, indexed_count, skipped_count, parsed_count, chunk_count, embedding_count, failed_count, created_at)
            VALUES
              (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            ("run-full-1", "workspace", "full", 100, 100, 0, 90, 200, 200, 10, "100"),
        )
        connection.execute(
            """
            INSERT INTO ingestion_runs
              (id, workspace_path, mode, discovered_count, indexed_count, skipped_count, parsed_count, chunk_count, embedding_count, failed_count, created_at)
            VALUES
              (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            ("run-inc-1", "workspace", "incremental", 100, 10, 90, 10, 20, 20, 0, "101"),
        )

    report = module.evaluate_ingestion_thresholds(db_path)

    assert report["status"] == "fail"
    check_names = {check["name"] for check in report["checks"]}
    assert check_names == {"full_ingestion_success_rate", "incremental_noop_ratio"}
    full_check = next(check for check in report["checks"] if check["name"] == "full_ingestion_success_rate")
    assert full_check["actual"] == pytest.approx(0.9)
    assert full_check["pass"] is False


def test_validator_passes_when_thresholds_met(tmp_path: Path) -> None:
    module = _load_validator_module()
    db_path = tmp_path / "aliv.sqlite3"
    _create_ingestion_runs_table(db_path)

    with sqlite3.connect(db_path) as connection:
        connection.execute(
            """
            INSERT INTO ingestion_runs
              (id, workspace_path, mode, discovered_count, indexed_count, skipped_count, parsed_count, chunk_count, embedding_count, failed_count, created_at)
            VALUES
              (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            ("run-full-2", "workspace", "full", 200, 200, 0, 198, 300, 300, 2, "200"),
        )
        connection.execute(
            """
            INSERT INTO ingestion_runs
              (id, workspace_path, mode, discovered_count, indexed_count, skipped_count, parsed_count, chunk_count, embedding_count, failed_count, created_at)
            VALUES
              (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            ("run-inc-2", "workspace", "incremental", 200, 6, 194, 6, 12, 12, 0, "201"),
        )

    report = module.evaluate_ingestion_thresholds(db_path)

    assert report["status"] == "pass"
    assert all(check["pass"] for check in report["checks"])
