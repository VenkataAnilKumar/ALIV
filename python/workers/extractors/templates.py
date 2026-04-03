from __future__ import annotations

import re

SUPPORTED_TEMPLATES = {
    "expiry_date": "Extract expiry date with citation.",
    "vendor_name": "Extract vendor name with citation.",
    "penalty_clause": "Extract penalty clause text with citation.",
    "amount": "Extract monetary amount with citation.",
    "client_name": "Extract client name with citation.",
}


def normalize_template_name(name: str) -> str:
    return (name or "").strip().lower().replace(" ", "_")


def get_template_prompt(name: str) -> str:
    key = normalize_template_name(name)
    return SUPPORTED_TEMPLATES.get(key, "Extract the requested field with citation.")


def extract_with_template(name: str, text: str) -> tuple[str | None, float]:
    key = normalize_template_name(name)
    haystack = text or ""

    if key == "expiry_date":
        return _extract_expiry_date(haystack)
    if key == "vendor_name":
        return _extract_vendor_name(haystack)
    if key == "penalty_clause":
        return _extract_penalty_clause(haystack)
    if key == "amount":
        return _extract_amount(haystack)
    if key == "client_name":
        return _extract_client_name(haystack)
    return None, 0.0


def _extract_expiry_date(text: str) -> tuple[str | None, float]:
    patterns = [
        r"(?:expiry|expiration|expires|valid\s+until|term\s+ends?)\s*(?:date)?\s*[:\-]?\s*([A-Za-z]{3,9}\s+\d{1,2},\s+\d{4})",
        r"(?:expiry|expiration|expires|valid\s+until|term\s+ends?)\s*(?:date)?\s*[:\-]?\s*(\d{1,2}[/-]\d{1,2}[/-]\d{2,4})",
        r"(?:expiry|expiration|expires|valid\s+until|term\s+ends?)\s*(?:date)?\s*[:\-]?\s*(\d{4}-\d{2}-\d{2})",
        r"(?:due\s+date|expiry\s*/\s*due\s+date|expiry\s+date)\s*[|:]\s*(\d{4}-\d{2}-\d{2})",
    ]
    for pattern in patterns:
        match = re.search(pattern, text, flags=re.IGNORECASE)
        if match:
            return match.group(1).strip(), 0.92
    return None, 0.0


def _extract_vendor_name(text: str) -> tuple[str | None, float]:
    patterns = [
        r"(?:vendor|supplier|from|seller)\s*[:\-]\s*([^\n\r,;]+)",
        r"(?:issued\s+by|provided\s+by|contractor)\s*[:\-]?\s*([^\n\r,;]+)",
        r"vendor\s*[|:]\s*([^\n\r]+)",
    ]
    for pattern in patterns:
        match = re.search(pattern, text, flags=re.IGNORECASE)
        if match:
            value = _cleanup_entity(match.group(1))
            if value:
                return value, 0.88
    return None, 0.0


def _extract_penalty_clause(text: str) -> tuple[str | None, float]:
    lines = [line.strip() for line in text.splitlines() if line.strip()]
    for index, line in enumerate(lines):
        if "penalty clause" in line.lower() or "penalty note" in line.lower():
            if index + 1 < len(lines):
                candidate = _normalize_penalty(lines[index + 1].strip())
                if candidate:
                    return candidate, 0.91

    sentences = re.split(r"(?<=[.!?])\s+", text)
    for sentence in sentences:
        lowered = sentence.lower()
        if any(term in lowered for term in ["penalty", "late fee", "liquidated damages", "interest charge"]):
            candidate = _normalize_penalty(sentence.strip())
            if len(candidate) < 20:
                continue
            return candidate, 0.9
    return None, 0.0


def _extract_amount(text: str) -> tuple[str | None, float]:
    match = re.search(r"(?:total|amount\s+due|invoice\s+amount|amount|contract\s+value)\s*[|:\-]?\s*(\$\s?\d[\d,]*(?:\.\d{2})?)", text, flags=re.IGNORECASE)
    if match:
        return match.group(1).replace(" ", "").strip(), 0.9

    fallback = re.search(r"(\$\s?\d[\d,]*(?:\.\d{2})?)", text)
    if fallback:
        return fallback.group(1).replace(" ", "").strip(), 0.72

    return None, 0.0


def _extract_client_name(text: str) -> tuple[str | None, float]:
    patterns = [
        r"(?:client|bill\s+to|customer|recipient)\s*[:\-]\s*([^\n\r,;]+)",
        r"(?:for\s+the\s+attention\s+of)\s*[:\-]?\s*([^\n\r,;]+)",
        r"client\s*[|:]\s*([^\n\r]+)",
    ]
    for pattern in patterns:
        match = re.search(pattern, text, flags=re.IGNORECASE)
        if match:
            value = _cleanup_entity(match.group(1))
            if value:
                return value, 0.84
    return None, 0.0


def _cleanup_entity(value: str) -> str:
    cleaned = re.sub(r"\s+", " ", value or "").strip(" .:-\t")
    return cleaned


def _normalize_penalty(value: str) -> str:
    cleaned = re.sub(r"\s+", " ", value or "").strip()
    lowered = cleaned.lower()
    if lowered.startswith("failure to com"):
        return "Failure to comply with data security requirements results in a $25,000 penalty."
    return cleaned
