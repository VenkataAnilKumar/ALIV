import { describe, expect, it, vi } from 'vitest'
import { ask } from '../ask'
import { DemoVaultService } from '../demo-service'
import { EMBED_BATCH, indexMatter, questionVector } from '../semantic'
import type { VaultService } from '../service'
import type { PassageEmbedding, PendingPassage } from '../types'

/** A vault holding `n` passages, recording the embeddings it is given. */
function fakeVault(n: number) {
  const stored = new Map<number, number[]>()
  const passages: PendingPassage[] = Array.from({ length: n }, (_, i) => ({ chunkId: i + 1, text: `passage ${i + 1}` }))
  const vault = {
    pendingEmbeddings: vi.fn(async (_m: string, limit = 64) => passages.filter((p) => !stored.has(p.chunkId)).slice(0, limit)),
    storeEmbeddings: vi.fn(async (e: PassageEmbedding[]) => {
      const ok = e.filter((x) => x.vector.length > 0)
      ok.forEach((x) => stored.set(x.chunkId, x.vector))
      return ok.length
    }),
    embeddingStatus: vi.fn(async () => ({ passages: n, embedded: stored.size })),
  } as unknown as VaultService
  return { vault, stored }
}

describe('indexMatter', () => {
  it('embeds every pending passage in batches and reports progress', async () => {
    const { vault, stored } = fakeVault(EMBED_BATCH * 2 + 5)
    const embed = vi.fn(async (texts: string[]) => texts.map(() => [1, 0]))
    const progress: number[] = []
    const r = await indexMatter(vault, 'm', embed, { onProgress: (p) => progress.push(p.embedded) })
    expect(r).toEqual({ embedded: EMBED_BATCH * 2 + 5, available: true })
    expect(stored.size).toBe(EMBED_BATCH * 2 + 5)
    expect(embed).toHaveBeenCalledTimes(3)
    expect(progress).toEqual([EMBED_BATCH, EMBED_BATCH * 2, EMBED_BATCH * 2 + 5])
  })

  it('stops, and says so, when no embedding model is available', async () => {
    const { vault, stored } = fakeVault(10)
    const r = await indexMatter(vault, 'm', async () => [])
    expect(r).toEqual({ embedded: 0, available: false })
    expect(stored.size).toBe(0)
  })

  it('does not loop when the vectors are unusable', async () => {
    const { vault } = fakeVault(10)
    const embed = vi.fn(async (texts: string[]) => texts.map(() => [] as number[]))
    const r = await indexMatter(vault, 'm', embed)
    expect(r.available).toBe(false)
    expect(embed).toHaveBeenCalledTimes(1)
  })

  it('stops when aborted and keeps what was stored', async () => {
    const { vault, stored } = fakeVault(EMBED_BATCH * 3)
    const controller = new AbortController()
    const embed = async (texts: string[]) => {
      controller.abort()
      return texts.map(() => [1])
    }
    const r = await indexMatter(vault, 'm', embed, { signal: controller.signal })
    expect(r.embedded).toBe(EMBED_BATCH)
    expect(stored.size).toBe(EMBED_BATCH)
  })
})

describe('questionVector', () => {
  it('is undefined without an embedder or when it fails', async () => {
    expect(await questionVector(undefined, 'q')).toBeUndefined()
    expect(await questionVector(async () => [], 'q')).toBeUndefined()
    expect(await questionVector(async () => { throw new Error('down') }, 'q')).toBeUndefined()
    expect(await questionVector(async () => [[0.5, 0.5]], ' q ')).toEqual([0.5, 0.5])
  })
})

describe('ask search mode', () => {
  it('sends the question vector to the vault and reports hybrid search', async () => {
    const vault = new DemoVaultService()
    const [matter] = await vault.listMatters()
    const search = vi.spyOn(vault, 'search')
    const r = await ask({ vault, matter, question: 'holdover penalty', embed: async () => [[1, 0, 0]] })
    expect(search).toHaveBeenCalledWith(matter.id, 'holdover penalty', 8, [1, 0, 0])
    expect(r.searchMode).toBe('hybrid')
  })

  it('falls back to keyword search without an embedder', async () => {
    const vault = new DemoVaultService()
    const [matter] = await vault.listMatters()
    const r = await ask({ vault, matter, question: 'holdover penalty' })
    expect(r.searchMode).toBe('keyword')
    expect(r.sources.length).toBeGreaterThan(0)
  })
})
