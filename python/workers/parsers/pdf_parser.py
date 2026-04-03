from __future__ import annotations

import tempfile
from pathlib import Path

import pymupdf

from workers.ocr.tesseract_adapter import run_tesseract


def parse_pdf(file_path: str) -> dict:
    path = Path(file_path)
    if not path.exists():
        raise FileNotFoundError(f"PDF file not found: {file_path}")

    page_map: list[dict] = []
    pages: list[dict] = []
    page_texts: list[str] = []
    warnings: list[str] = []

    with pymupdf.open(path) as document:
        for page in document:
            text = page.get_text("text", sort=True).strip()
            blocks = page.get_text("blocks")
            page_warnings: list[str] = []

            if not text:
                text, ocr_warnings = _ocr_page_text(page)
                page_warnings.extend(ocr_warnings)

            page_entry = {
                "page_number": page.number,
                "text": text,
                "block_count": len(blocks),
                "has_text": bool(text),
            }
            page_map.append(
                {
                    "page_number": page_entry["page_number"],
                    "text_length": len(text),
                    "block_count": page_entry["block_count"],
                    "has_text": page_entry["has_text"],
                }
            )
            pages.append(page_entry)
            if text:
                page_texts.append(text)
            else:
                warnings.append(f"page {page.number} produced no extractable text")
            warnings.extend(f"page {page.number}: {warning}" for warning in page_warnings)

    normalized_text = "\n\f\n".join(page_texts)
    parser_confidence = _estimate_parser_confidence(page_map, normalized_text)

    return {
        "file_path": str(path),
        "parser_name": "pymupdf",
        "normalized_text": normalized_text,
        "page_map": page_map,
        "pages": pages,
        "parser_confidence": parser_confidence,
        "warnings": warnings,
    }


def _estimate_parser_confidence(page_map: list[dict], normalized_text: str) -> float:
    if not page_map:
        return 0.0

    populated_pages = sum(1 for page in page_map if page["has_text"])
    populated_ratio = populated_pages / len(page_map)
    text_signal = min(len(normalized_text) / 5000, 1.0)
    confidence = (0.7 * populated_ratio) + (0.3 * text_signal)
    return round(confidence, 4)


def _ocr_page_text(page: pymupdf.Page) -> tuple[str, list[str]]:
    pixmap = page.get_pixmap(matrix=pymupdf.Matrix(2, 2), alpha=False)
    with tempfile.NamedTemporaryFile(suffix=".png", delete=True) as temp_image:
        pixmap.save(temp_image.name)
        ocr_result = run_tesseract(temp_image.name)

    text = str(ocr_result.get("recognized_text", "")).strip()
    warnings = [str(flag) for flag in ocr_result.get("quality_flags", [])]
    if not text:
        warnings.append("ocr fallback produced no text")
    return text, warnings
