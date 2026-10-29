import type { VaultService } from './service'

/** Embeds texts with the app's local embedding model; `[]` when none is available. */
export type Embedder = (texts: string[]) => Promise<number[][]>

/** Passages sent to the embedder per call. */
export const EMBED_BATCH = 32

export interface IndexProgress {
  embedded: number
  passages: number
}

export interface IndexResult {
  /** Passages embedded in this run. */
  embedded: number
  /** False when the embedder returned nothing: no embedding model available. */
  available: boolean
}

/**
 * Embeds a matter's passages that have no vector yet, a batch at a time, so
 * search can match by meaning. Stops early when the embedder is unavailable,
 * returns the wrong number of vectors, or `signal` aborts; whatever was
 * stored stays, and the next run resumes from the remaining passages.
 */
export async function indexMatter(
  vault: VaultService,
  matterId: string,
  embed: Embedder,
  opts: { signal?: AbortSignal; onProgress?: (p: IndexProgress) => void } = {}
): Promise<IndexResult> {
  let embedded = 0
  for (;;) {
    if (opts.signal?.aborted) return { embedded, available: true }
    const pending = await vault.pendingEmbeddings(matterId, EMBED_BATCH)
    if (!pending.length) return { embedded, available: true }
    const vectors = await embed(pending.map((p) => p.text))
    if (vectors.length !== pending.length) return { embedded, available: vectors.length > 0 }
    const stored = await vault.storeEmbeddings(
      pending.map((p, i) => ({ chunkId: p.chunkId, vector: vectors[i] }))
    )
    // Nothing stored means the vectors were unusable (empty or non-finite);
    // asking again would loop on the same passages.
    if (stored === 0) return { embedded, available: false }
    embedded += stored
    if (opts.onProgress) opts.onProgress(await vault.embeddingStatus(matterId))
  }
}

/** The question's embedding, or undefined when no embedder is available. */
export async function questionVector(embed: Embedder | undefined, question: string): Promise<number[] | undefined> {
  if (!embed || !question.trim()) return undefined
  try {
    const [vector] = await embed([question.trim()])
    return vector?.length ? vector : undefined
  } catch {
    return undefined
  }
}

/**
 * The app's local embedder: the llama.cpp engine's embedding model, on this
 * computer. Never a cloud provider, so it is safe for Local only matters.
 * Resolves to `[]` when the engine or its embedding model is unavailable.
 */
export const localEmbedder: Embedder = async (texts) => {
  if (!texts.length) return []
  try {
    const { embedTexts } = await import('@aliv/core')
    return await embedTexts(texts)
  } catch (e) {
    console.debug('Local embedding unavailable', e)
    return []
  }
}
