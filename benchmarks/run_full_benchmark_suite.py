from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PYTHON_ROOT = ROOT / "python"
if str(PYTHON_ROOT) not in sys.path:
    sys.path.insert(0, str(PYTHON_ROOT))

if str(ROOT / "benchmarks") not in sys.path:
    sys.path.insert(0, str(ROOT / "benchmarks"))

from run_extraction_benchmark import evaluate_extraction
from run_ingestion_benchmark import run_benchmark
from run_retrieval_benchmark import evaluate_retrieval
from validate_ingestion_thresholds import evaluate_ingestion_thresholds


RESULTS_DIR = ROOT / "benchmarks" / "results"
DEFECT_BACKLOG = ROOT / "docs" / "PHASE4_DEFECT_BACKLOG.json"


def _timestamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def _find_check(checks: list[dict], name: str) -> dict | None:
    for check in checks:
        if check.get("name") == name:
            return check
    return None


def _read_open_critical_defects() -> int:
    if not DEFECT_BACKLOG.exists():
        return 0
    try:
        defects = json.loads(DEFECT_BACKLOG.read_text(encoding="utf-8"))
        return sum(
            1 for d in defects
            if d.get("status") == "open" and d.get("severity") == "critical"
        )
    except (json.JSONDecodeError, TypeError):
        return 0


def _suite_status(ingestion_gate: dict, retrieval: dict, extraction: dict, open_critical: int) -> dict:
    full_ingestion = _find_check(ingestion_gate.get("checks", []), "full_ingestion_success_rate")
    incremental_noop = _find_check(ingestion_gate.get("checks", []), "incremental_noop_ratio")

    checks = {
        "ingestion": bool(full_ingestion and full_ingestion.get("pass")) and bool(incremental_noop and incremental_noop.get("pass")),
        "retrieval": retrieval.get("status") == "pass",
        "extraction": extraction.get("status") == "pass",
        "no_open_critical_safety_defects": open_critical == 0,
    }

    return {
        "checks": checks,
        "status": "pass" if all(checks.values()) else "fail",
    }


def run_single_suite(corpus_dir: Path, db_path: Path, questions_path: Path) -> dict:
    if db_path.exists():
        db_path.unlink()

    full_run = run_benchmark(corpus_dir, db_path, incremental=False)
    incremental_run = run_benchmark(corpus_dir, db_path, incremental=True)

    ingestion_gate = evaluate_ingestion_thresholds(db_path)
    retrieval = evaluate_retrieval(db_path, questions_path, top_k=5)
    extraction = evaluate_extraction(db_path, questions_path)

    open_critical = _read_open_critical_defects()
    suite_status = _suite_status(ingestion_gate, retrieval, extraction, open_critical)

    return {
        "timestamp_utc": _timestamp(),
        "runs": {
            "ingestion_full": full_run,
            "ingestion_incremental": incremental_run,
        },
        "metrics": {
            "ingestion": ingestion_gate,
            "retrieval": retrieval,
            "extraction": extraction,
        },
        "defects": {
            "open_critical_count": open_critical,
        },
        "suite": suite_status,
    }


def _trend(previous: dict | None, current: dict) -> dict:
    if previous is None:
        return {"note": "no previous run available"}

    prev_ret = previous["metrics"]["retrieval"].get("actual", 0.0)
    cur_ret = current["metrics"]["retrieval"].get("actual", 0.0)

    prev_ext = previous["metrics"]["extraction"].get("actual", 0.0)
    cur_ext = current["metrics"]["extraction"].get("actual", 0.0)

    prev_p95 = previous["metrics"]["retrieval"].get("query_latency_p95_sec", 0.0)
    cur_p95 = current["metrics"]["retrieval"].get("query_latency_p95_sec", 0.0)

    return {
        "retrieval_top5_delta": round(cur_ret - prev_ret, 4),
        "extraction_precision_delta": round(cur_ext - prev_ext, 4),
        "query_latency_p95_delta_sec": round(cur_p95 - prev_p95, 4),
    }


def _phase4_exit(runs: list[dict]) -> dict:
    if len(runs) < 2:
        return {
            "status": "fail",
            "reason": "need at least two consecutive full-suite runs",
            "pass": False,
        }

    run_1 = runs[-2]
    run_2 = runs[-1]

    both_pass = run_1["suite"]["status"] == "pass" and run_2["suite"]["status"] == "pass"

    return {
        "status": "pass" if both_pass else "fail",
        "reason": "two consecutive full-suite runs pass all phase checks" if both_pass else "one or more suite runs failed",
        "pass": both_pass,
    }


def _write_report(report: dict) -> tuple[Path, Path]:
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    ts = _timestamp()

    latest_path = RESULTS_DIR / "full_suite_latest.json"
    stamped_path = RESULTS_DIR / f"full_suite_{ts}.json"

    payload = json.dumps(report, indent=2)
    latest_path.write_text(payload, encoding="utf-8")
    stamped_path.write_text(payload, encoding="utf-8")

    return latest_path, stamped_path


def main() -> int:
    parser = argparse.ArgumentParser(description="Run ALIV full benchmark suite and evaluate Phase 4 exit")
    parser.add_argument("--corpus", type=Path, default=Path("benchmarks/corpus"))
    parser.add_argument("--db", type=Path, default=Path(".aliv-bench/aliv.sqlite3"))
    parser.add_argument("--questions", type=Path, default=Path("benchmarks/labels/questions_gold.v1.json"))
    parser.add_argument("--runs", type=int, default=2, help="Number of consecutive suite runs")
    args = parser.parse_args()

    corpus_dir = (ROOT / args.corpus).resolve()
    db_path = (ROOT / args.db).resolve()
    questions_path = (ROOT / args.questions).resolve()

    if not corpus_dir.exists():
        print(json.dumps({"status": "error", "reason": f"corpus not found: {corpus_dir}"}, indent=2))
        return 2
    if not questions_path.exists():
        print(json.dumps({"status": "error", "reason": f"questions file not found: {questions_path}"}, indent=2))
        return 2

    suite_runs: list[dict] = []
    previous: dict | None = None

    for index in range(max(1, args.runs)):
        current = run_single_suite(corpus_dir, db_path, questions_path)
        current["trend"] = _trend(previous, current)
        current["run_index"] = index + 1
        suite_runs.append(current)
        previous = current

    report = {
        "status": "pass" if all(run["suite"]["status"] == "pass" for run in suite_runs) else "fail",
        "runs": suite_runs,
        "phase4_exit": _phase4_exit(suite_runs),
    }

    latest_path, stamped_path = _write_report(report)
    report["result_files"] = {
        "latest": str(latest_path),
        "stamped": str(stamped_path),
    }

    print(json.dumps(report, indent=2))

    # Phase 4 exit requires two consecutive passes.
    return 0 if report["phase4_exit"].get("pass") else 1


if __name__ == "__main__":
    raise SystemExit(main())
