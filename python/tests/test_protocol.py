from __future__ import annotations

import pytest
from pydantic import ValidationError

from workers.protocol import WorkerRequest, WorkerResponse


class TestWorkerRequest:
    def test_defaults(self) -> None:
        req = WorkerRequest(request_id="r1", worker_type="parse")
        assert req.payload == {}
        assert req.timeout_ms == 30000
        assert req.worker_version == "0.1.0"

    def test_custom_fields(self) -> None:
        req = WorkerRequest(
            request_id="r2",
            worker_type="embed",
            payload={"file": "doc.pdf"},
            timeout_ms=5000,
            worker_version="0.2.0",
        )
        assert req.payload == {"file": "doc.pdf"}
        assert req.timeout_ms == 5000
        assert req.worker_version == "0.2.0"

    def test_missing_required_fields_raises(self) -> None:
        with pytest.raises(ValidationError):
            WorkerRequest()  # type: ignore[call-arg]

    def test_roundtrip_json(self) -> None:
        req = WorkerRequest(request_id="r3", worker_type="ocr")
        restored = WorkerRequest.model_validate_json(req.model_dump_json())
        assert restored == req


class TestWorkerResponse:
    def test_ok_response(self) -> None:
        resp = WorkerResponse(request_id="r1", status="ok", result={"pages": 3})
        assert resp.status == "ok"
        assert resp.error_code is None
        assert resp.error_detail is None

    def test_error_response(self) -> None:
        resp = WorkerResponse(
            request_id="r2",
            status="error",
            error_code="FILE_NOT_FOUND",
            error_detail="path does not exist",
        )
        assert resp.status == "error"
        assert resp.result == {}
        assert resp.error_code == "FILE_NOT_FOUND"

    def test_invalid_status_raises(self) -> None:
        with pytest.raises(ValidationError):
            WorkerResponse(request_id="r3", status="unknown")  # type: ignore[arg-type]

    def test_roundtrip_json(self) -> None:
        resp = WorkerResponse(request_id="r4", status="ok")
        restored = WorkerResponse.model_validate_json(resp.model_dump_json())
        assert restored == resp
