from __future__ import annotations

from workers.ocr.tesseract_adapter import run_tesseract
from workers.protocol import WorkerRequest, WorkerResponse
from workers.runner import run


def handle(request: WorkerRequest) -> WorkerResponse:
    file_path = str(request.payload.get("file_path", "")).strip()
    if not file_path:
        return WorkerResponse(
            request_id=request.request_id,
            status="error",
            error_code="missing_file_path",
            error_detail="ocr worker requires payload.file_path",
        )

    try:
        result = run_tesseract(file_path)
        return WorkerResponse(
            request_id=request.request_id,
            status="ok",
            result=result,
        )
    except Exception as exc:
        return WorkerResponse(
            request_id=request.request_id,
            status="error",
            error_code="ocr_failure",
            error_detail=str(exc),
        )


if __name__ == "__main__":
    run(handle)
