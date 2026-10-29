import { useEffect, useRef, useState } from 'react'
import { indexMatter, localEmbedder, type Embedder } from './semantic'
import type { EmbeddingStatus } from './types'
import { getVault } from './vault'

export interface MeaningIndex extends EmbeddingStatus {
  /** Embedding is in progress. */
  running: boolean
  /** False once the embedder has returned nothing: no local embedding model. */
  available: boolean
}

/**
 * Keeps a matter's meaning index current: embeds passages that have no
 * vector yet when the matter opens and whenever `version` changes (after an
 * import). Runs in the background and resumes where it stopped.
 */
export function useMeaningIndex(matterId: string, version: unknown, embed: Embedder | undefined = defaultEmbedder()): MeaningIndex {
  const [state, setState] = useState<MeaningIndex>({ passages: 0, embedded: 0, running: false, available: !!embed })
  const run = useRef(0)

  useEffect(() => {
    const vault = getVault()
    const controller = new AbortController()
    const id = ++run.current
    const live = () => run.current === id && !controller.signal.aborted
    void (async () => {
      try {
        const status = await vault.embeddingStatus(matterId)
        if (!live()) return
        if (!embed || status.embedded >= status.passages) {
          setState({ ...status, running: false, available: !!embed })
          return
        }
        setState({ ...status, running: true, available: true })
        const result = await indexMatter(vault, matterId, embed, {
          signal: controller.signal,
          onProgress: (p) => live() && setState((s) => ({ ...s, ...p })),
        })
        const final = await vault.embeddingStatus(matterId)
        if (live()) setState({ ...final, running: false, available: result.available })
      } catch (e) {
        console.debug('Meaning index not updated', e)
        if (live()) setState((s) => ({ ...s, running: false }))
      }
    })()
    return () => controller.abort()
  }, [matterId, version, embed])

  return state
}

function defaultEmbedder(): Embedder | undefined {
  return getVault().mode === 'desktop' ? localEmbedder : undefined
}
