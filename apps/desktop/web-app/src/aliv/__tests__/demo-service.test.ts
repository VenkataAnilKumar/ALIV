import { describe, expect, it } from 'vitest'
import { DemoVaultService, terms } from '../demo-service'

describe('DemoVaultService', () => {
  it('seeds one local-only matter with the demo corpus and real extracted fields', async () => {
    const v = new DemoVaultService()
    const [m] = await v.listMatters()
    expect(m.policy).toBe('local_only')
    expect(m.documentCount).toBeGreaterThanOrEqual(7)
    const grid = await v.reviewGrid(m.id)
    const lease = grid.find((r) => r.document.fileName === 'lease_acme_2025.txt')!
    expect(lease.fields.find((f) => f.key === 'expiry_date')?.value).toBe('2026-03-01')
  })

  it('never executes a rename that was not approved, and can undo', async () => {
    const v = new DemoVaultService()
    const [m] = await v.listMatters()
    const proposals = await v.proposeRenames(m.id)
    expect(proposals.length).toBeGreaterThan(0)
    expect(await v.executeApproved(m.id)).toEqual([])
    const first = proposals[0]
    await v.setActionStatus(first.id, 'approved')
    const done = await v.executeApproved(m.id)
    expect(done.map((a) => a.id)).toEqual([first.id])
    await expect(v.setActionStatus(first.id, 'approved')).rejects.toThrow()
    const undone = await v.undoAction(first.id)
    expect(undone.status).toBe('undone')
  })

  it('scopes search to the matter', async () => {
    const v = new DemoVaultService()
    const other = await v.createMatter({ name: 'Empty matter' })
    expect(await v.search(other.id, 'penalty')).toEqual([])
    const [seeded] = (await v.listMatters()).filter((m) => m.id !== other.id)
    expect((await v.search(seeded.id, 'penalty')).length).toBeGreaterThan(0)
  })

  it('tokenizes questions without stopwords', () => {
    expect(terms("What's the penalty for late payment?")).toEqual(["what's", 'penalty', 'late', 'payment'])
  })
})

describe('vault health', () => {
  it('reports counts without document text', async () => {
    const vault = new DemoVaultService()
    const d = await vault.diagnostics()
    expect(d.matters).toBeGreaterThan(0)
    expect(d.indexed).toBe(d.documents)
    expect(d.audit.ok).toBe(true)
    expect(JSON.stringify(d)).not.toMatch(/holdover/i)
  })
})
