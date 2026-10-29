import { demoDocuments, type FixtureDoc } from './demo/fixture'
import type { VaultService } from './service'
import type {
  AuditEntry,
  Field,
  GridRow,
  Matter,
  MatterInput,
  SearchHit,
  VaultAction,
  VaultDocument,
} from './types'

/**
 * In-memory vault for the browser preview and tests. It is seeded with the
 * repository's demo corpus and the fields the Rust extractor produced for it
 * (see apps/desktop/scripts/aliv-demo-fixture.py). File actions are simulated: nothing on
 * disk changes, but the same approval and policy rules apply.
 */

interface Passage {
  id: number
  documentId: string
  ordinal: number
  text: string
}

const DEMO_FOLDER = '/Demo/Redstone Enterprises'
const STOPWORDS = new Set(
  'a an and are as at be by do does for from has have how in is it its me of on or show that the their there this to was what when where which who will with all any find list tell give can you our we i my please about'.split(
    ' '
  )
)

export function terms(text: string): string[] {
  return [
    ...new Set(
      text
        .toLowerCase()
        .split(/[^\p{L}\p{N}']+/u)
        .map((t) => t.replace(/^'+|'+$/g, ''))
        .filter((t) => t.length >= 2 && !STOPWORDS.has(t))
    ),
  ]
}

const stem = (t: string) => t.replace(/(ing|ed|es|s)$/, '')

export class DemoVaultService implements VaultService {
  readonly mode = 'demo' as const
  private matters: Matter[] = []
  private docs: VaultDocument[] = []
  private fields = new Map<string, Field[]>()
  private passages: Passage[] = []
  private actions: VaultAction[] = []
  private audit: AuditEntry[] = []
  private seq = 0

  constructor(seed = true) {
    if (seed) this.seed()
  }

  private id(prefix: string) {
    this.seq += 1
    return `${prefix}-${this.seq}`
  }

  private now() {
    return new Date().toISOString().replace(/\.\d{3}Z$/, 'Z')
  }

  private log(matterId: string | null, kind: string, summary: string, detail: unknown = {}) {
    const prev = this.audit[0]?.hash ?? ''
    const entry: AuditEntry = {
      id: (this.audit[0]?.id ?? 0) + 1,
      matterId,
      ts: this.now(),
      kind,
      summary,
      detail,
      hash: demoHash(prev + kind + summary),
    }
    this.audit.unshift(entry)
    return entry.id
  }

  private seed() {
    const m = this.makeMatter({
      name: 'Redstone portfolio review',
      client: 'Redstone Enterprises Inc.',
      kind: 'legal',
      policy: 'local_only',
    })
    this.addFixtureDocs(m.id)
  }

  private addFixtureDocs(matterId: string) {
    for (const d of demoDocuments as FixtureDoc[]) {
      const id = this.id('doc')
      const paragraphs = d.text.split(/\n\s*\n/).map((p) => p.trim()).filter(Boolean)
      paragraphs.forEach((text, ordinal) =>
        this.passages.push({ id: this.passages.length + 1, documentId: id, ordinal, text })
      )
      this.docs.push({
        id,
        matterId,
        path: `${DEMO_FOLDER}/${d.fileName}`,
        fileName: d.fileName,
        ext: d.ext,
        size: d.size,
        status: 'indexed',
        error: null,
        docType: d.docType,
        charCount: d.text.length,
        chunkCount: paragraphs.length,
        ingestedAt: this.now(),
        ocrConfidence: null,
      })
      this.fields.set(id, d.fields)
    }
    this.log(matterId, 'ingest', `Indexed ${demoDocuments.length} file(s) from the demo corpus`)
  }

  private makeMatter(input: MatterInput): Matter {
    const m: Matter = {
      id: this.id('matter'),
      name: input.name.trim(),
      client: input.client?.trim() ?? '',
      kind: input.kind ?? 'legal',
      policy: input.policy ?? 'local_only',
      createdAt: this.now(),
      documentCount: 0,
      pendingActions: 0,
    }
    this.matters.unshift(m)
    this.log(m.id, 'matter_created', `Matter "${m.name}" created (${m.policy})`)
    return m
  }

  private withCounts(m: Matter): Matter {
    return {
      ...m,
      documentCount: this.docs.filter((d) => d.matterId === m.id && d.status === 'indexed').length,
      pendingActions: this.actions.filter(
        (a) => a.matterId === m.id && (a.status === 'proposed' || a.status === 'approved')
      ).length,
    }
  }

  private matter(id: string): Matter {
    const m = this.matters.find((x) => x.id === id)
    if (!m) throw new Error(`not found: matter ${id}`)
    return m
  }

  async listMatters() {
    return this.matters.map((m) => this.withCounts(m))
  }

  async createMatter(input: MatterInput) {
    if (!input.name.trim()) throw new Error('Matter name is required')
    return this.withCounts(this.makeMatter(input))
  }

  async updateMatter(id: string, input: MatterInput) {
    const m = this.matter(id)
    const before = m.policy
    if (input.name.trim()) m.name = input.name.trim()
    if (input.client !== undefined) m.client = input.client.trim()
    if (input.kind) m.kind = input.kind
    if (input.policy) m.policy = input.policy
    if (m.policy !== before) {
      this.log(id, 'policy_changed', `Model policy changed from ${before} to ${m.policy}`)
    }
    return this.withCounts(m)
  }

  async deleteMatter(id: string) {
    const m = this.matter(id)
    this.matters = this.matters.filter((x) => x.id !== id)
    const docIds = new Set(this.docs.filter((d) => d.matterId === id).map((d) => d.id))
    this.docs = this.docs.filter((d) => d.matterId !== id)
    this.passages = this.passages.filter((p) => !docIds.has(p.documentId))
    this.actions = this.actions.filter((a) => a.matterId !== id)
    this.log(null, 'matter_deleted', `Matter "${m.name}" deleted (source files untouched)`)
  }

  async ingestPaths(matterId: string) {
    this.matter(matterId)
    const already = this.docs.some((d) => d.matterId === matterId)
    if (!already) this.addFixtureDocs(matterId)
    const n = demoDocuments.length
    return {
      scanned: n,
      indexed: already ? 0 : n,
      unchanged: already ? n : 0,
      skipped: 0,
      failed: 0,
      ocr: 0,
      errors: [],
    }
  }

  async listDocuments(matterId: string) {
    return this.docs
      .filter((d) => d.matterId === matterId)
      .sort((a, b) => a.fileName.localeCompare(b.fileName))
  }

  async search(matterId: string, query: string, limit = 8): Promise<SearchHit[]> {
    const q = terms(query).map(stem)
    if (!q.length) return []
    const docs = new Map(this.docs.filter((d) => d.matterId === matterId).map((d) => [d.id, d]))
    const scored = this.passages
      .filter((p) => docs.has(p.documentId))
      .map((p) => {
        const words = terms(p.text).map(stem)
        const score = q.reduce((s, t) => s + (words.includes(t) ? 1 : 0), 0)
        return { p, score: score / Math.sqrt(words.length + 1) }
      })
      .filter((x) => x.score > 0)
      .sort((a, b) => b.score - a.score)
      .slice(0, limit)
    return scored.map(({ p, score }) => {
      const d = docs.get(p.documentId)!
      return {
        chunkId: p.id,
        documentId: d.id,
        fileName: d.fileName,
        path: d.path,
        ordinal: p.ordinal,
        text: p.text,
        snippet: p.text.length > 220 ? `${p.text.slice(0, 219)}…` : p.text,
        score,
      }
    })
  }

  async reviewGrid(matterId: string): Promise<GridRow[]> {
    const docs = await this.listDocuments(matterId)
    return docs
      .filter((d) => d.status === 'indexed')
      .map((document) => ({ document, fields: this.fields.get(document.id) ?? [] }))
  }

  async proposeRenames(matterId: string) {
    this.actions = this.actions.filter(
      (a) => !(a.matterId === matterId && (a.status === 'proposed' || a.status === 'approved'))
    )
    const out: VaultAction[] = []
    const taken = new Set<string>()
    for (const row of await this.reviewGrid(matterId)) {
      const f = (k: string) => row.fields.find((x) => x.key === k)
      const type = f('doc_type')?.value ?? 'Document'
      const party = f('parties')?.value.split(' · ')[0]
      const date = (f('effective_date') ?? f('expiry_date'))?.value
      const stemName = [slug(type), party ? slug(party) : '', date ?? ''].filter(Boolean).join('_')
      const dir = row.document.path.slice(0, row.document.path.lastIndexOf('/'))
      let target = `${dir}/${stemName}.${row.document.ext}`
      for (let n = 2; taken.has(target); n++) target = `${dir}/${stemName}_${n}.${row.document.ext}`
      if (target === row.document.path) continue
      taken.add(target)
      const weak = (k: string) => (f(k)?.confidence ?? 0) < 0.75
      const risk: VaultAction['risk'] =
        type === 'Document' || !party ? 'high' : !date || weak('doc_type') || weak('parties') ? 'medium' : 'low'
      const ts = this.now()
      const action: VaultAction = {
        id: this.id('action'),
        matterId,
        documentId: row.document.id,
        kind: 'rename',
        fromPath: row.document.path,
        toPath: target,
        reason: [`Type ${type}`, party && `party ${party}`, date && `dated ${date}`].filter(Boolean).join(', '),
        risk,
        status: 'proposed',
        error: null,
        createdAt: ts,
        updatedAt: ts,
      }
      this.actions.push(action)
      out.push(action)
    }
    this.log(matterId, 'actions_proposed', `Proposed ${out.length} rename(s) for review`)
    return out
  }

  async listActions(matterId: string) {
    return this.actions.filter((a) => a.matterId === matterId)
  }

  private action(id: string) {
    const a = this.actions.find((x) => x.id === id)
    if (!a) throw new Error(`not found: action ${id}`)
    return a
  }

  async setActionStatus(id: string, status: 'approved' | 'rejected' | 'proposed') {
    const a = this.action(id)
    const allowed: Record<string, string[]> = {
      proposed: ['approved', 'rejected'],
      approved: ['proposed', 'rejected'],
      rejected: ['proposed'],
    }
    if (!allowed[a.status]?.includes(status)) {
      throw new Error(`cannot change a ${a.status} action to ${status}`)
    }
    a.status = status
    a.updatedAt = this.now()
    this.log(a.matterId, `action_${status}`, `Rename ${base(a.fromPath)} → ${base(a.toPath)} marked ${status}`)
    return { ...a }
  }

  async executeApproved(matterId: string) {
    const done: VaultAction[] = []
    for (const a of this.actions.filter((x) => x.matterId === matterId && x.status === 'approved')) {
      a.status = 'executed'
      a.updatedAt = this.now()
      const d = this.docs.find((x) => x.id === a.documentId)
      if (d) {
        d.path = a.toPath
        d.fileName = base(a.toPath)
      }
      this.log(matterId, 'action_executed', `Rename ${base(a.fromPath)} → ${base(a.toPath)} (simulated)`)
      done.push({ ...a })
    }
    return done
  }

  async undoAction(id: string) {
    const a = this.action(id)
    if (a.status !== 'executed') {
      throw new Error(`only executed actions can be undone (this one is ${a.status})`)
    }
    a.status = 'undone'
    a.updatedAt = this.now()
    const d = this.docs.find((x) => x.id === a.documentId)
    if (d) {
      d.path = a.fromPath
      d.fileName = base(a.fromPath)
    }
    this.log(a.matterId, 'action_undone', `Undid rename; restored ${base(a.fromPath)}`)
    return { ...a }
  }

  async authorizeModelCall(req: Parameters<VaultService['authorizeModelCall']>[0]) {
    const m = this.matter(req.matterId)
    const allowed = !(req.location === 'cloud' && m.policy === 'local_only')
    const auditId = this.log(
      req.matterId,
      allowed ? 'model_call' : 'model_call_blocked',
      allowed
        ? `Sent ${req.charsSent} chars from ${req.sourceCount} passage(s) to ${req.model} (${req.provider}, ${req.location})`
        : `Blocked call to ${req.model} (${req.provider}): matter is local only`,
      req
    )
    return {
      allowed,
      reason: allowed
        ? `${req.location} model allowed by ${m.policy} policy`
        : 'This matter is Local only. Choose a local model, or change the matter policy to Hybrid.',
      auditId,
    }
  }

  async listAudit(matterId?: string, limit = 200) {
    return this.audit.filter((a) => !matterId || a.matterId === matterId).slice(0, limit)
  }

  async verifyAudit() {
    return { ok: true, entries: this.audit.length, firstBrokenId: null }
  }

  async exportRegisterCsv(): Promise<number> {
    throw new Error('Export writes a file, which needs the desktop app.')
  }
}

function slug(s: string) {
  const drop = new Set(['llc', 'inc', 'ltd', 'llp', 'corp', 'co', 'the', 'plc', 'gmbh'])
  return s
    .split(/[^\p{L}\p{N}]+/u)
    .filter((w) => w && !drop.has(w.toLowerCase()))
    .slice(0, 4)
    .map((w) => w[0].toUpperCase() + w.slice(1))
    .join('')
    .slice(0, 40)
}

function base(path: string) {
  return path.slice(path.lastIndexOf('/') + 1)
}

/** Not cryptographic: the demo only needs a stable-looking chain id. */
function demoHash(s: string) {
  let h = 0x811c9dc5
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i)
    h = Math.imul(h, 0x01000193)
  }
  return (h >>> 0).toString(16).padStart(8, '0').repeat(8)
}
