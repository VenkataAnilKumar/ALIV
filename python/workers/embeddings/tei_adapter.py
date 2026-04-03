def embed_texts(texts: list[str]) -> dict:
    return {
        "count": len(texts),
        "vectors": [],
        "provider": "text-embeddings-inference",
    }
