import { streamText } from 'ai'
import { BACKGROUND_THREAD_ID, CHAT_SLOT_ID } from '@/constants/models'
import { ModelFactory } from '@/lib/model-factory'
import { daysUntil } from './dates'
import { questionVector, type Embedder } from './semantic'
import type { VaultService } from './service'
import { FIELD_LABELS, type FieldKey, type GridRow, type Matter, type SearchHit } from './types'

/** One numbered source the model may cite as [n]. */
export interface AskSource {
  n: number
  kind: 'passage' | 'register'
  documentId: string
  fileName: string
  text: string
}

export interface AskPlan {
  system: string
  prompt: string
  sources: AskSource[]
  charsSent: number
  /** Sources that matched but were left out to fit the model's context window. */
  omitted: number
}

export type AskStatus = 'answered' | 'computed' | 'blocked' | 'no_model' | 'no_sources' | 'error'

export interface AskResult {
  status: AskStatus
  text: string
  sources: AskSource[]
  model?: string
  provider?: string
  location?: 'local' | 'cloud'
  reason?: string
  /** Whether the passages were found by keyword and meaning, or keyword only. */
  searchMode?: 'hybrid' | 'keyword'
  /** Matching sources left out to fit the model's context window. */
  omitted?: number
}

const LOCAL_PROVIDERS = new Set(['llamacpp', 'mlx', 'ollama', 'lmstudio'])
const MAX_PASSAGES = 8
const MAX_REGISTER_ROWS = 60
/** Assumed when the model's context length is not configured ("auto"). */
export const DEFAULT_CONTEXT_TOKENS = 8192
/** Kept free for the answer. */
const ANSWER_RESERVE_TOKENS = 1024
/** About four characters per token for English prose; close enough to budget by. */
const CHARS_PER_TOKEN = 4
const REGISTER_KEYS: FieldKey[] = [
  'doc_type',
  'parties',
  'effective_date',
  'expiry_date',
  'renewal_deadline',
  'amount',
  'notice_period',
  'governing_law',
  'penalty',
]

/**
 * Where a provider runs. Built-in local engines and custom providers that
 * point at this machine count as local; everything else is cloud.
 */
export function modelLocation(provider: Pick<ModelProvider, 'provider' | 'base_url'>): 'local' | 'cloud' {
  if (LOCAL_PROVIDERS.has(provider.provider.toLowerCase())) return 'local'
  try {
    const host = new URL(provider.base_url ?? '').hostname
    if (host === 'localhost' || host === '127.0.0.1' || host === '::1' || host === '[::1]') return 'local'
  } catch {
    // No or invalid base URL: treat as remote.
  }
  return 'cloud'
}

/**
 * Register rows in the order they are worth sending: documents the matching
 * passages came from, then rows that mention the question's words (a party,
 * a document type), then the soonest expiry. A large matter's register does
 * not fit a small model's context, so what is sent first matters.
 */
export function rankRegisterRows(grid: GridRow[], hits: SearchHit[], question: string): GridRow[] {
  const hitDocs = new Set(hits.map((h) => h.documentId))
  const words = question
    .toLowerCase()
    .split(/[^a-z0-9$]+/)
    .filter((w) => w.length > 3)
  const score = (row: GridRow) => {
    const hay = [row.document.fileName, ...row.fields.map((f) => f.value)].join(' ').toLowerCase()
    return (hitDocs.has(row.document.id) ? 100 : 0) + words.filter((w) => hay.includes(w)).length
  }
  const expiry = (row: GridRow) => fieldValue(row, 'expiry_date') ?? '9999'
  return grid
    .map((row) => ({ row, s: score(row) }))
    .sort((a, b) => b.s - a.s || expiry(a.row).localeCompare(expiry(b.row)))
    .map((x) => x.row)
}

export function buildAskPlan(
  question: string,
  hits: SearchHit[],
  grid: GridRow[],
  today: Date = new Date(),
  contextTokens: number = DEFAULT_CONTEXT_TOKENS
): AskPlan {
  const iso = today.toISOString().slice(0, 10)
  const system = [
    'You are ALIV, a careful assistant for legal and finance professionals.',
    'Answer only from the numbered sources provided. Never use outside knowledge about the documents.',
    'Cite every factual statement with the source numbers in square brackets, e.g. [2] or [1][4].',
    'Passages are verbatim excerpts. Register entries are fields extracted by rules and may be incomplete; prefer a passage when both cover a point.',
    'If the sources do not answer the question, say so plainly and suggest what document might.',
    `Today is ${iso}. When a question involves time ("expiring in 90 days", "overdue"), compute it against today and show the dates.`,
    'Be concise. Use short paragraphs or bullet lists. Do not invent clause numbers, amounts or dates.',
  ].join('\n')

  // Sources are added best first while they fit what the model can read,
  // after the instructions, the question and room for the answer.
  const fixed = system.length + question.length + 40
  let room = Math.max(0, (contextTokens - ANSWER_RESERVE_TOKENS) * CHARS_PER_TOKEN - fixed)
  const sources: AskSource[] = []
  let omitted = 0
  const add = (s: Omit<AskSource, 'n'>) => {
    const cost = s.text.length + s.fileName.length + 30
    if (cost > room) {
      omitted++
      return
    }
    room -= cost
    sources.push({ ...s, n: sources.length + 1 })
  }
  for (const h of hits.slice(0, MAX_PASSAGES)) {
    add({ kind: 'passage', documentId: h.documentId, fileName: h.fileName, text: h.text })
  }
  for (const row of rankRegisterRows(grid, hits, question).slice(0, MAX_REGISTER_ROWS)) {
    const parts = REGISTER_KEYS.map((k) => {
      const v = fieldValue(row, k)
      return v ? `${FIELD_LABELS[k]}: ${v}` : null
    }).filter(Boolean)
    if (!parts.length) continue
    add({ kind: 'register', documentId: row.document.id, fileName: row.document.fileName, text: parts.join('; ') })
  }

  const body = sources
    .map((s) => `[${s.n}] (${s.kind === 'passage' ? 'passage' : 'register'} · ${s.fileName})\n${s.text}`)
    .join('\n\n')
  const prompt = `SOURCES\n\n${body}\n\nQUESTION\n${question.trim()}`
  return { system, prompt, sources, charsSent: system.length + prompt.length, omitted }
}

function fieldValue(row: GridRow, key: FieldKey) {
  return row.fields.find((f) => f.key === key)?.value
}

export interface AskOptions {
  vault: VaultService
  matter: Matter
  question: string
  provider?: ModelProvider
  modelId?: string
  onText?: (text: string) => void
  signal?: AbortSignal
  /** The local embedder; without one, passages are found by keyword only. */
  embed?: Embedder
  /** The model's context window in tokens, when configured. */
  contextTokens?: number
  /** Injected in tests. */
  createModel?: typeof ModelFactory.createModel
  stream?: typeof streamText
}

export async function ask(opts: AskOptions): Promise<AskResult> {
  const { vault, matter, question, provider, modelId } = opts
  const vector = await questionVector(opts.embed, question)
  const searchMode: AskResult['searchMode'] = vector ? 'hybrid' : 'keyword'
  const [hits, grid] = await Promise.all([
    vault.search(matter.id, question, MAX_PASSAGES, vector),
    vault.reviewGrid(matter.id),
  ])
  const plan = buildAskPlan(question, hits, grid, new Date(), opts.contextTokens || DEFAULT_CONTEXT_TOKENS)
  if (!plan.sources.length) {
    return {
      status: 'no_sources',
      text: 'No documents in this matter match the question yet. Add documents, or try different words.',
      sources: [],
      searchMode,
    }
  }
  if (!provider || !modelId) {
    const computed = deadlineAnswer(question, plan.sources, grid)
    if (computed) return { status: 'computed', text: computed, sources: plan.sources, searchMode, omitted: plan.omitted }
    return {
      status: 'no_model',
      text: 'Choose a model to get a written answer. Until then, here are the passages that match.',
      sources: plan.sources,
      searchMode,
      omitted: plan.omitted,
    }
  }

  const location = modelLocation(provider)
  const decision = await vault.authorizeModelCall({
    matterId: matter.id,
    provider: provider.provider,
    model: modelId,
    location,
    charsSent: plan.charsSent,
    sourceCount: plan.sources.length,
    purpose: 'answer',
  })
  const meta = { model: modelId, provider: provider.provider, location, searchMode, omitted: plan.omitted }
  if (!decision.allowed) {
    return { status: 'blocked', text: decision.reason, sources: plan.sources, reason: decision.reason, ...meta }
  }

  try {
    const params: Record<string, unknown> = {}
    if (provider.provider === 'llamacpp') {
      // Same slot discipline as other background calls: do not clobber the
      // open chat's cached prefix.
      params.id_slot = CHAT_SLOT_ID
      params.thread_id = BACKGROUND_THREAD_ID
    }
    const model = await (opts.createModel ?? ModelFactory.createModel.bind(ModelFactory))(modelId, provider, params)
    const result = (opts.stream ?? streamText)({
      model,
      system: plan.system,
      prompt: plan.prompt,
      temperature: 0.1,
      abortSignal: opts.signal,
    })
    let text = ''
    for await (const delta of result.textStream) {
      text += delta
      opts.onText?.(text)
    }
    return { status: 'answered', text: stripThinking(text), sources: plan.sources, ...meta }
  } catch (err) {
    if ((err as Error)?.name === 'AbortError') {
      return { status: 'error', text: 'Stopped.', sources: plan.sources, ...meta }
    }
    return {
      status: 'error',
      text: `The model call failed: ${(err as Error)?.message ?? String(err)}`,
      sources: plan.sources,
      ...meta,
    }
  }
}

/** Some local models emit <think> blocks; keep only the answer. */
export function stripThinking(text: string): string {
  return text.replace(/<think>[\s\S]*?<\/think>/g, '').trim()
}

export type AnswerPart = { type: 'text'; text: string } | { type: 'cite'; n: number }

/** Splits "Rent is $8,400 [2][5]." into text and citation parts. */
export function parseCitations(text: string, maxSource: number): AnswerPart[] {
  const parts: AnswerPart[] = []
  const re = /\[(\d{1,3}(?:\s*,\s*\d{1,3})*)\]/g
  let last = 0
  for (const m of text.matchAll(re)) {
    const nums = m[1].split(',').map((x) => Number(x.trim()))
    if (!nums.every((n) => n >= 1 && n <= maxSource)) continue
    if (m.index! > last) parts.push({ type: 'text', text: text.slice(last, m.index) })
    for (const n of nums) parts.push({ type: 'cite', n })
    last = m.index! + m[0].length
  }
  if (last < text.length) parts.push({ type: 'text', text: text.slice(last) })
  return parts
}

/**
 * Answers deadline questions ("what expires in the next 90 days?") straight
 * from the register, with no model involved. Returns null for anything else.
 */
export function deadlineAnswer(question: string, sources: AskSource[], grid: GridRow[], today: Date = new Date()): string | null {
  const q = question.toLowerCase()
  if (!/expir|renew|deadline|\bdue\b|lapse|end date|terminat/.test(q)) return null
  const m = /(\d{1,4})\s*(day|week|month)/.exec(q)
  const window = m ? Number(m[1]) * (m[2] === 'week' ? 7 : m[2] === 'month' ? 30 : 1) : 90
  const items: { days: number; line: string }[] = []
  for (const row of grid) {
    const n = sources.find((s) => s.kind === 'register' && s.documentId === row.document.id)?.n
    const parties = fieldValue(row, 'parties')
    for (const key of ['renewal_deadline', 'expiry_date'] as const) {
      const value = fieldValue(row, key)
      const days = value ? daysUntil(value, today) : null
      if (days === null || days < 0 || days > window) continue
      const what = key === 'renewal_deadline' ? 'renewal notice due' : 'expires / due'
      items.push({
        days,
        line: `- **${row.document.fileName}**${parties ? ` (${parties})` : ''}: ${what} ${value}, in ${days} days${n ? ` [${n}]` : ''}`,
      })
    }
  }
  if (!items.length) return `No expiry or renewal dates fall in the next ${window} days, based on the extracted register.`
  items.sort((a, b) => a.days - b.days)
  return `${items.length} date(s) fall in the next ${window} days:\n\n${items.map((i) => i.line).join('\n')}`
}
