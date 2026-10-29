/**
 * Mirrors the Rust types in tauri-plugin-aliv-vault/src/model.rs.
 */

export type MatterKind = 'legal' | 'finance'
export type MatterPolicy = 'local_only' | 'hybrid'

export interface Matter {
  id: string
  name: string
  client: string
  kind: MatterKind
  policy: MatterPolicy
  createdAt: string
  documentCount: number
  pendingActions: number
}

export interface MatterInput {
  name: string
  client?: string
  kind?: MatterKind
  policy?: MatterPolicy
}

export interface VaultDocument {
  id: string
  matterId: string
  path: string
  fileName: string
  ext: string
  size: number
  status: 'indexed' | 'skipped' | 'failed'
  error: string | null
  docType: string | null
  charCount: number
  chunkCount: number
  ingestedAt: string
  /** Set when the text was read by OCR: mean word confidence, 0 to 1. */
  ocrConfidence: number | null
  /** Overview from the extracted fields and the opening words; no model involved. */
  summary: string | null
}

export type FieldKey =
  | 'doc_type'
  | 'parties'
  | 'effective_date'
  | 'expiry_date'
  | 'renewal_deadline'
  | 'amount'
  | 'governing_law'
  | 'notice_period'
  | 'penalty'

export interface Field {
  key: FieldKey
  value: string
  confidence: number
  chunkOrdinal: number | null
  snippet: string
}

export interface GridRow {
  document: VaultDocument
  fields: Field[]
}

export interface SearchHit {
  chunkId: number
  documentId: string
  fileName: string
  path: string
  ordinal: number
  text: string
  snippet: string
  score: number
}

/** A passage the vault has no embedding for yet. */
export interface PendingPassage {
  chunkId: number
  text: string
}

export interface PassageEmbedding {
  chunkId: number
  vector: number[]
}

/** How much of a matter is covered by meaning-based search. */
export interface EmbeddingStatus {
  passages: number
  embedded: number
}

export interface IngestReport {
  scanned: number
  indexed: number
  unchanged: number
  skipped: number
  failed: number
  /** Of `indexed`, how many were read by OCR. */
  ocr: number
  errors: string[]
}

export type ActionStatus =
  | 'proposed'
  | 'approved'
  | 'rejected'
  | 'executed'
  | 'undone'
  | 'failed'

export interface VaultAction {
  id: string
  matterId: string
  documentId: string
  kind: 'rename'
  fromPath: string
  toPath: string
  reason: string
  risk: 'low' | 'medium' | 'high'
  status: ActionStatus
  error: string | null
  createdAt: string
  updatedAt: string
}

export interface AuditEntry {
  id: number
  matterId: string | null
  ts: string
  kind: string
  summary: string
  detail: unknown
  hash: string
}

export interface ModelCallRequest {
  matterId: string
  provider: string
  model: string
  location: 'local' | 'cloud'
  charsSent: number
  sourceCount: number
  purpose: string
}

export interface ModelCallDecision {
  allowed: boolean
  reason: string
  auditId: number
}

export interface AuditVerification {
  ok: boolean
  entries: number
  firstBrokenId: number | null
}

export const FIELD_LABELS: Record<FieldKey, string> = {
  doc_type: 'Type',
  parties: 'Parties',
  effective_date: 'Effective',
  expiry_date: 'Expiry / due',
  renewal_deadline: 'Renewal deadline',
  amount: 'Amount',
  governing_law: 'Governing law',
  notice_period: 'Notice',
  penalty: 'Penalty / late charge',
}
