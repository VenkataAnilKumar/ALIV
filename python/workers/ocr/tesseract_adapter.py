from __future__ import annotations

from pathlib import Path


def run_tesseract(file_path: str) -> dict:
    path = Path(file_path)
    if not path.exists():
        raise FileNotFoundError(f"OCR input file not found: {file_path}")

    quality_flags: list[str] = []

    try:
        import pytesseract
        from PIL import Image
    except Exception:
        quality_flags.append("ocr dependencies missing: install pytesseract and pillow")
        return {
            "file_path": str(path),
            "recognized_text": "",
            "confidence_map": {"mean_confidence": 0.0, "word_count": 0},
            "quality_flags": quality_flags,
        }

    image = Image.open(path)
    data = pytesseract.image_to_data(image, output_type=pytesseract.Output.DICT)

    words: list[str] = []
    confidences: list[float] = []
    for text, confidence in zip(data.get("text", []), data.get("conf", [])):
        cleaned = str(text).strip()
        if not cleaned:
            continue

        words.append(cleaned)
        try:
            confidence_value = float(confidence)
            if confidence_value >= 0:
                confidences.append(confidence_value)
        except (TypeError, ValueError):
            continue

    recognized_text = " ".join(words).strip()
    mean_confidence = sum(confidences) / len(confidences) if confidences else 0.0

    if not recognized_text:
        quality_flags.append("ocr produced no text")
    if mean_confidence > 0 and mean_confidence < 60:
        quality_flags.append("low_ocr_confidence")

    return {
        "file_path": str(path),
        "recognized_text": recognized_text,
        "confidence_map": {
            "mean_confidence": round(mean_confidence, 2),
            "word_count": len(words),
        },
        "quality_flags": quality_flags,
    }
