from __future__ import annotations

from email import message_from_file
from pathlib import Path


def parse_email(file_path: str) -> dict:
    path = Path(file_path)
    if not path.exists():
        raise FileNotFoundError(f"Email file not found: {file_path}")

    with open(path, "r", encoding="utf-8", errors="replace") as f:
        message = message_from_file(f)

    text_blocks = _collect_email_blocks(message)
    warnings: list[str] = []

    if not text_blocks:
        warnings.append("email produced no extractable text")

    pages: list[dict] = []
    page_map: list[dict] = []
    segments = _chunk_blocks(text_blocks, chunk_size=6)
    for segment_index, segment in enumerate(segments):
        text = "\n\n".join(segment).strip()
        block_count = len(segment)
        page_entry = {
            "page_number": segment_index,
            "text": text,
            "block_count": block_count,
            "has_text": bool(text),
        }
        pages.append(page_entry)
        page_map.append(
            {
                "page_number": segment_index,
                "text_length": len(text),
                "block_count": block_count,
                "has_text": bool(text),
            }
        )

    normalized_text = "\n\n".join(text_blocks)
    parser_confidence = _estimate_parser_confidence(text_blocks, normalized_text)

    return {
        "file_path": str(path),
        "parser_name": "email-rfc822",
        "normalized_text": normalized_text,
        "page_map": page_map,
        "pages": pages,
        "parser_confidence": parser_confidence,
        "warnings": warnings,
    }


def _collect_email_blocks(message) -> list[str]:
    blocks: list[str] = []

    subject = message.get("Subject", "").strip()
    if subject:
        blocks.append(f"Subject: {subject}")

    sender = message.get("From", "").strip()
    if sender:
        blocks.append(f"From: {sender}")

    to = message.get("To", "").strip()
    if to:
        blocks.append(f"To: {to}")

    cc = message.get("Cc", "").strip()
    if cc:
        blocks.append(f"Cc: {cc}")

    date = message.get("Date", "").strip()
    if date:
        blocks.append(f"Date: {date}")

    if message.is_multipart():
        for part in message.walk():
            if part.get_content_type() == "text/plain":
                body = part.get_payload(decode=True)
                if body:
                    try:
                        text = body.decode("utf-8", errors="ignore").strip()
                        if text:
                            blocks.append(text)
                    except Exception:
                        pass
    else:
        try:
            body = message.get_payload(decode=True)
            if body:
                text = body.decode("utf-8", errors="ignore").strip()
                if text:
                    blocks.append(text)
        except Exception:
            pass

    return blocks


def _chunk_blocks(blocks: list[str], chunk_size: int) -> list[list[str]]:
    if not blocks:
        return []

    return [blocks[index:index + chunk_size] for index in range(0, len(blocks), chunk_size)]


def _estimate_parser_confidence(text_blocks: list[str], normalized_text: str) -> float:
    if not text_blocks:
        return 0.0

    block_signal = min(len(text_blocks) / 15, 1.0)
    text_signal = min(len(normalized_text) / 3000, 1.0)
    confidence = (0.5 * block_signal) + (0.5 * text_signal)
    return round(confidence, 4)
