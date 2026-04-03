from __future__ import annotations

import json

from sentence_transformers import SentenceTransformer

from workers.embeddings.model_registry import MODEL_REGISTRY
from workers.protocol import WorkerRequest, WorkerResponse
from workers.runner import run

# Load embedding model once at worker startup
EMBEDDING_MODEL = SentenceTransformer(MODEL_REGISTRY["default_embedding_model"])


def handle(request: WorkerRequest) -> WorkerResponse:
    chunks = request.payload.get("chunks", [])
    if not chunks:
        return WorkerResponse(
            request_id=request.request_id,
            status="error",
            error_code="missing_chunks",
            error_detail="embedding worker requires payload.chunks (list of strings)",
        )

    try:
        embeddings = EMBEDDING_MODEL.encode(
            chunks,
            convert_to_numpy=False,
            show_progress_bar=False,
        )

        vectors = [embedding.tolist() if hasattr(embedding, "tolist") else list(embedding) for embedding in embeddings]

        return WorkerResponse(
            request_id=request.request_id,
            status="ok",
            result={
                "vectors": vectors,
                "embedding_model": MODEL_REGISTRY["default_embedding_model"],
                "embedding_dim": MODEL_REGISTRY["default_embedding_dim"],
                "chunk_count": len(chunks),
            },
        )
    except Exception as exc:
        return WorkerResponse(
            request_id=request.request_id,
            status="error",
            error_code="embedding_failure",
            error_detail=str(exc),
        )


if __name__ == "__main__":
    run(handle)
