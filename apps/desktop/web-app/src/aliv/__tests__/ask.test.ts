import { describe, expect, it, vi } from 'vitest'
import { ask, buildAskPlan, deadlineAnswer, modelLocation, parseCitations, stripThinking } from '../ask'
import { DemoVaultService } from '../demo-service'

const provider = (name: string, base_url?: string) =>
  ({ provider: name, base_url, active: true, settings: [], models: [] }) as unknown as ModelProvider

function fakeStream(chunks: string[]) {
  return vi.fn(() => ({
    textStream: (async function* () {
      for (const c of chunks) yield c
    })(),
  })) as never
}

describe('modelLocation', () => {
  it('treats built-in engines and loopback endpoints as local', () => {
    expect(modelLocation(provider('llamacpp'))).toBe('local')
    expect(modelLocation(provider('custom', 'http://127.0.0.1:11434/v1'))).toBe('local')
    expect(modelLocation(provider('custom', 'http://localhost:1234/v1'))).toBe('local')
    expect(modelLocation(provider('anthropic', 'https://api.anthropic.com'))).toBe('cloud')
    expect(modelLocation(provider('openai'))).toBe('cloud')
  })
})

describe('parseCitations', () => {
  it('splits text and in-range citations', () => {
    expect(parseCitations('Rent is $8,400 [2][3]. Penalty [1, 4].', 4)).toEqual([
      { type: 'text', text: 'Rent is $8,400 ' },
      { type: 'cite', n: 2 },
      { type: 'cite', n: 3 },
      { type: 'text', text: '. Penalty ' },
      { type: 'cite', n: 1 },
      { type: 'cite', n: 4 },
      { type: 'text', text: '.' },
    ])
  })

  it('leaves out-of-range brackets as text', () => {
    expect(parseCitations('See [9].', 3)).toEqual([{ type: 'text', text: 'See [9].' }])
  })

  it('strips reasoning blocks', () => {
    expect(stripThinking('<think>hmm</think>\nAnswer [1]')).toBe('Answer [1]')
  })
})

describe('ask', () => {
  it('numbers passages before register rows and tells the model the date', async () => {
    const vault = new DemoVaultService()
    const [matter] = await vault.listMatters()
    const hits = await vault.search(matter.id, 'holdover penalty')
    const plan = buildAskPlan('What is the holdover penalty?', hits, await vault.reviewGrid(matter.id), new Date('2026-10-29T12:00:00Z'))
    expect(plan.sources[0].kind).toBe('passage')
    expect(plan.sources[0].text).toMatch(/holdover/i)
    expect(plan.sources.some((s) => s.kind === 'register')).toBe(true)
    expect(plan.system).toContain('Today is 2026-10-29')
    expect(plan.prompt).toContain('[1] (passage · ')
  })

  it('blocks cloud models on local-only matters and logs it', async () => {
    const vault = new DemoVaultService()
    const [matter] = await vault.listMatters()
    expect(matter.policy).toBe('local_only')
    const stream = fakeStream(['should not run'])
    const res = await ask({ vault, matter, question: 'penalty', provider: provider('anthropic'), modelId: 'claude', stream, createModel: vi.fn() as never })
    expect(res.status).toBe('blocked')
    expect(stream).not.toHaveBeenCalled()
    const audit = await vault.listAudit(matter.id)
    expect(audit[0].kind).toBe('model_call_blocked')
  })

  it('streams a local answer and records the call', async () => {
    const vault = new DemoVaultService()
    const [matter] = await vault.listMatters()
    const seen: string[] = []
    const res = await ask({
      vault,
      matter,
      question: 'What is the holdover penalty?',
      provider: provider('llamacpp'),
      modelId: 'qwen',
      createModel: vi.fn(async () => ({}) as never),
      stream: fakeStream(['$2,800 ', 'per day [1].']),
      onText: (t) => seen.push(t),
    })
    expect(res.status).toBe('answered')
    expect(res.text).toBe('$2,800 per day [1].')
    expect(res.location).toBe('local')
    expect(seen).toEqual(['$2,800 ', '$2,800 per day [1].'])
    const audit = await vault.listAudit(matter.id)
    expect(audit[0].kind).toBe('model_call')
  })

  it('returns evidence only when no model is selected', async () => {
    const vault = new DemoVaultService()
    const [matter] = await vault.listMatters()
    const res = await ask({ vault, matter, question: 'governing law' })
    expect(res.status).toBe('no_model')
    expect(res.sources.length).toBeGreaterThan(0)
  })
})

describe('deadlineAnswer', () => {
  it('lists dates inside the asked window with register citations, soonest first', async () => {
    const vault = new DemoVaultService()
    const [matter] = await vault.listMatters()
    const grid = await vault.reviewGrid(matter.id)
    const today = new Date(2026, 9, 29)
    const plan = buildAskPlan('renewals', [], grid, today)
    const text = deadlineAnswer('Which agreements expire or need a renewal notice in the next 90 days?', plan.sources, grid, today)!
    const lines = text.split('\n').filter((l) => l.startsWith('- '))
    expect(lines).toHaveLength(4)
    expect(lines[0]).toContain('msa_northwind_logistics_2025.txt')
    expect(lines[0]).toContain('renewal notice due 2026-11-15, in 17 days')
    expect(lines[0]).toMatch(/\[\d+\]$/)
    expect(deadlineAnswer('due in 2 weeks', plan.sources, grid, today)).toMatch(/^No expiry or renewal dates/)
    expect(deadlineAnswer('who are the parties?', plan.sources, grid, today)).toBeNull()
  })

  it('is what ask returns for a deadline question without a model', async () => {
    const vault = new DemoVaultService()
    const [matter] = await vault.listMatters()
    const res = await ask({ vault, matter, question: 'What expires in the next 400 days?' })
    expect(res.status).toBe('computed')
    expect(res.text).toContain('[')
  })
})
