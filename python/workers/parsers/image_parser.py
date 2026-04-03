from __future__ import annotations

from pathlib import Path

from workers.ocr.tesseract_adapter import run_tesseract


def parse_image(file_path: str) -> dict:
    path = Path(file_path)
    if not path.exists():
        raise FileNotFoundError(f"Image file not found: {file_path}")

    ocr_result = run_tesseract(file_path)
    recognized_text = str(ocr_result.get("recognized_text", "")).strip()
    confidence_map = ocr_result.get("confidence_map", {})
    warnings = list(ocr_result.get("quality_flags", []))
    mean_confidence = float(confidence_map.get("mean_confidence", 0.0) or 0.0)

    page_map = [
        {
            "page_number": 0,
            "text_length": len(recognized_text),
            "block_count": 1 if recognized_text else 0,
            "has_text": bool(recognized_text),
            "ocr_mean_confidence": mean_confidence,
        }
    ]
    pages = [
        {
            "page_number": 0,
            "text": recognized_text,
            "block_count": 1 if recognized_text else 0,
            "has_text": bool(recognized_text),
        }
    ]

    if not recognized_text:
        warnings.append("image produced no extractable text")

    parser_confidence = round(min(max(mean_confidence / 100.0, 0.0), 1.0), 4)

    return {
        "file_path": str(path),
        "parser_name": "tesseract-image",
        "normalized_text": recognized_text,
        "page_map": page_map,
        "pages": pages,
        "parser_confidence": parser_confidence,
        "warnings": warnings,
    }
