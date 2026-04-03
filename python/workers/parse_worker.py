from __future__ import annotations

from pathlib import Path

from workers.parsers.docx_parser import parse_docx
from workers.parsers.email_parser import parse_email
from workers.parsers.image_parser import parse_image
from workers.parsers.pdf_parser import parse_pdf
from workers.protocol import WorkerRequest, WorkerResponse
from workers.runner import run


def handle(request: WorkerRequest) -> WorkerResponse:
    file_path = str(request.payload.get("file_path", "")).strip()
    file_type = str(request.payload.get("file_type", "")).strip().lower()

    if not file_path:
        return WorkerResponse(
            request_id=request.request_id,
            status="error",
            error_code="missing_file_path",
            error_detail="parse worker requires payload.file_path",
        )

    resolved_type = file_type or Path(file_path).suffix.lstrip(".").lower()

    try:
        if resolved_type == "pdf":
            result = parse_pdf(file_path)
        elif resolved_type == "docx":
            result = parse_docx(file_path)
        elif resolved_type == "eml":
            result = parse_email(file_path)
        elif resolved_type in {"png", "jpg", "jpeg", "tif", "tiff", "bmp", "webp"}:
            result = parse_image(file_path)
        else:
            return WorkerResponse(
                request_id=request.request_id,
                status="error",
                error_code="unsupported_file_type",
                error_detail=f"parse worker does not yet support '{resolved_type}'",
            )

        return WorkerResponse(
            request_id=request.request_id,
            status="ok",
            result=result,
        )
    except Exception as exc:
        return WorkerResponse(
            request_id=request.request_id,
            status="error",
            error_code="parse_failure",
            error_detail=str(exc),
        )


if __name__ == "__main__":
    run(handle)
