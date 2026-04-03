"""Summarization worker: generates document and section summaries via local Ollama LLM.

IPC contract (JSON-lines over stdin/stdout, same as all ALIV workers):
  Request payload fields:
    text          str   — document or section text to summarize
    granularity   str   — "document" | "section"
    model         str   — Ollama model name (e.g. "llama3.2:3b")
    ollama_url    str   — Ollama base URL (default "http://localhost:11434")

  Response result fields:
    summary       str   — generated summary text
    token_count   int   — estimated token count of summary (len/4)
    model_version str   — model name used
    granularity   str   — echoed from request

Summarization is not called during ingestion blocking path. It is triggered
as a post-ingestion background task per file via the Rust command surface.
"""
from __future__ import annotations

import json

import urllib.request
import urllib.error

from workers.protocol import WorkerRequest, WorkerResponse
from workers.runner import run

_DEFAULT_OLLAMA_URL = "http://localhost:11434"

_PROMPTS = {
    "document": (
        "You are a document analysis assistant for legal and financial documents. "
        "Write a concise factual summary of the following document in 150-200 words. "
        "Include: document type, key parties, main dates, and critical obligations or amounts. "
        "Do not speculate. Use only information present in the text.\n\nDOCUMENT:\n{text}\n\nSUMMARY:"
    ),
    "section": (
        "Summarize the following document section in 60-80 words. "
        "Focus on facts: parties, amounts, dates, obligations. "
        "Do not speculate.\n\nSECTION:\n{text}\n\nSUMMARY:"
    ),
}

_MAX_INPUT_CHARS = 12000  # ~3000 tokens — keeps Ollama call fast


def _call_ollama(prompt: str, model: str, ollama_url: str) -> str:
    """POST to Ollama /api/generate and return the response text."""
    payload = json.dumps({
        "model": model,
        "prompt": prompt,
        "stream": False,
        "options": {
            "temperature": 0.1,   # low temperature for factual summary
            "num_predict": 300,   # max output tokens
        },
    }).encode("utf-8")

    url = ollama_url.rstrip("/") + "/api/generate"
    req = urllib.request.Request(
        url,
        data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            body = json.loads(resp.read().decode("utf-8"))
            return body.get("response", "").strip()
    except urllib.error.URLError as exc:
        raise RuntimeError(f"Ollama unreachable at {ollama_url}: {exc}") from exc


def handle(request: WorkerRequest) -> WorkerResponse:
    text: str = request.payload.get("text", "").strip()
    granularity: str = request.payload.get("granularity", "document")
    model: str = request.payload.get("model", "llama3.2:3b")
    ollama_url: str = request.payload.get("ollama_url", _DEFAULT_OLLAMA_URL)

    if not text:
        return WorkerResponse(
            request_id=request.request_id,
            status="error",
            error_code="missing_text",
            error_detail="summarization_worker requires payload.text",
        )

    if granularity not in _PROMPTS:
        return WorkerResponse(
            request_id=request.request_id,
            status="error",
            error_code="invalid_granularity",
            error_detail=f"granularity must be 'document' or 'section', got '{granularity}'",
        )

    # Truncate input to avoid exceeding Ollama context window
    truncated_text = text[:_MAX_INPUT_CHARS]
    if len(text) > _MAX_INPUT_CHARS:
        truncated_text += "\n[... content truncated for summarization ...]"

    prompt = _PROMPTS[granularity].format(text=truncated_text)

    try:
        summary = _call_ollama(prompt, model, ollama_url)
    except RuntimeError as exc:
        return WorkerResponse(
            request_id=request.request_id,
            status="error",
            error_code="ollama_error",
            error_detail=str(exc),
        )

    if not summary:
        return WorkerResponse(
            request_id=request.request_id,
            status="error",
            error_code="empty_summary",
            error_detail="Ollama returned an empty response",
        )

    token_count = max(1, len(summary) // 4)

    return WorkerResponse(
        request_id=request.request_id,
        status="ok",
        result={
            "summary": summary,
            "token_count": token_count,
            "model_version": model,
            "granularity": granularity,
        },
    )


if __name__ == "__main__":
    run(handle)
