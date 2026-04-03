from __future__ import annotations

from typing import Any


def bind_citations(extracted_value: str | list[str], candidates: list[dict[str, Any]] | None = None, max_citations: int = 3) -> list[dict[str, Any]]:
    # Backward-compatible call path: bind_citations(["chunk-a", "chunk-b"])
    if isinstance(extracted_value, list):
        return [{"chunk_id": chunk_id, "confidence": 0.0} for chunk_id in extracted_value]

    target = (extracted_value or "").strip().lower()
    if not target or not candidates:
        return []

    scored: list[tuple[float, dict[str, Any]]] = []
    for idx, candidate in enumerate(candidates):
        text = str(candidate.get("content") or candidate.get("content_preview") or "")
        lowered = text.lower()

        score = 0.0
        if target and target in lowered:
            score += 0.7

        retrieval_score = candidate.get("score")
        if isinstance(retrieval_score, (int, float)):
            score += min(float(retrieval_score) / 1_000_000.0, 0.25)

        score += max(0.0, 0.05 - (idx * 0.01))
        if score <= 0:
            continue

        scored.append((score, candidate))

    scored.sort(key=lambda item: item[0], reverse=True)

    bound: list[dict[str, Any]] = []
    for score, candidate in scored[:max(1, max_citations)]:
        bound.append(
            {
                "file_id": candidate.get("file_id"),
                "chunk_id": candidate.get("chunk_id"),
                "page_ref": candidate.get("page_ref"),
                "confidence": round(min(score, 1.0), 3),
            }
        )

    return bound
