from __future__ import annotations

from workers.extractors.citation_binding import bind_citations
from workers.extractors.templates import extract_with_template, get_template_prompt, normalize_template_name
from workers.protocol import WorkerRequest, WorkerResponse
from workers.runner import run


def handle(request: WorkerRequest) -> WorkerResponse:
    payload = request.payload or {}
    template = normalize_template_name(str(payload.get("template") or payload.get("field") or ""))
    query = str(payload.get("query") or "").strip()

    chunks = payload.get("chunks") or []
    if not isinstance(chunks, list):
        chunks = []

    text = _build_text(payload, chunks)
    value, confidence = extract_with_template(template, text)

    citations = bind_citations(value or "", _normalize_candidates(chunks), max_citations=3)
    if value and not citations:
        confidence = min(confidence, 0.49)

    field_name = template or "value"

    return WorkerResponse(
        request_id=request.request_id,
        status="ok",
        result={
            "field_value_map": {field_name: value},
            "field_confidence_map": {field_name: round(confidence, 3)},
            "citations": citations,
            "template": field_name,
            "prompt": get_template_prompt(field_name),
            "query": query,
        },
    )


def _build_text(payload: dict, chunks: list) -> str:
    direct_text = str(payload.get("text") or payload.get("context") or "").strip()
    if direct_text:
        return direct_text

    parts: list[str] = []
    for chunk in chunks:
        if isinstance(chunk, dict):
            content = chunk.get("content") or chunk.get("content_preview") or ""
            if content:
                parts.append(str(content))
        elif isinstance(chunk, str):
            parts.append(chunk)

    return "\n".join(parts).strip()


def _normalize_candidates(chunks: list) -> list[dict]:
    normalized: list[dict] = []
    for index, chunk in enumerate(chunks):
        if isinstance(chunk, dict):
            item = dict(chunk)
            if "chunk_id" not in item:
                item["chunk_id"] = f"chunk-{index}"
            normalized.append(item)
        elif isinstance(chunk, str):
            normalized.append(
                {
                    "chunk_id": f"chunk-{index}",
                    "content": chunk,
                    "score": 0,
                }
            )
    return normalized


if __name__ == "__main__":
    run(handle)
