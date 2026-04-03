from __future__ import annotations

from pathlib import Path

from workers.parse_worker import handle
from workers.protocol import WorkerRequest


def _request(file_path: str = "", file_type: str = "") -> WorkerRequest:
    return WorkerRequest(
        request_id="req-1",
        worker_type="parse",
        payload={"file_path": file_path, "file_type": file_type},
    )


def test_parse_worker_missing_file_path_returns_error() -> None:
    response = handle(_request())

    assert response.status == "error"
    assert response.error_code == "missing_file_path"


def test_parse_worker_unsupported_file_type_returns_error(tmp_path: Path) -> None:
    unsupported = tmp_path / "data.xyz"
    unsupported.write_text("x", encoding="utf-8")

    response = handle(_request(str(unsupported), "xyz"))

    assert response.status == "error"
    assert response.error_code == "unsupported_file_type"


def test_parse_worker_routes_docx(tmp_path: Path, monkeypatch) -> None:
    sample = tmp_path / "doc.docx"
    sample.write_bytes(b"dummy")

    monkeypatch.setattr(
        "workers.parse_worker.parse_docx",
        lambda _: {
            "file_path": str(sample),
            "parser_name": "python-docx",
            "normalized_text": "ok",
            "page_map": [],
            "pages": [],
            "parser_confidence": 1.0,
            "warnings": [],
        },
    )

    response = handle(_request(str(sample), "docx"))

    assert response.status == "ok"
    assert response.result["parser_name"] == "python-docx"


def test_parse_worker_routes_image_type(tmp_path: Path, monkeypatch) -> None:
    sample = tmp_path / "scan.png"
    sample.write_bytes(b"dummy")

    monkeypatch.setattr(
        "workers.parse_worker.parse_image",
        lambda _: {
            "file_path": str(sample),
            "parser_name": "tesseract-image",
            "normalized_text": "scanned",
            "page_map": [],
            "pages": [],
            "parser_confidence": 0.8,
            "warnings": [],
        },
    )

    response = handle(_request(str(sample), "png"))

    assert response.status == "ok"
    assert response.result["parser_name"] == "tesseract-image"
