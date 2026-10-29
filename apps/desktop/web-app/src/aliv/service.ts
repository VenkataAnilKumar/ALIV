import type {
  AuditEntry,
  AuditVerification,
  GridRow,
  EmbeddingStatus,
  IngestReport,
  Matter,
  MatterInput,
  ModelCallDecision,
  ModelCallRequest,
  PassageEmbedding,
  PendingPassage,
  SearchHit,
  VaultAction,
  VaultDocument,
} from './types'

/**
 * Everything the UI needs from the matter vault. The desktop app talks to the
 * Rust plugin; the browser preview and tests use an in-memory demo vault.
 */
export interface VaultService {
  readonly mode: 'desktop' | 'demo'
  listMatters(): Promise<Matter[]>
  createMatter(input: MatterInput): Promise<Matter>
  updateMatter(id: string, input: MatterInput): Promise<Matter>
  deleteMatter(id: string): Promise<void>
  ingestPaths(matterId: string, paths: string[]): Promise<IngestReport>
  listDocuments(matterId: string): Promise<VaultDocument[]>
  /**
   * Keyword search; with `queryVector` (the question's embedding) it is fused
   * with meaning-based search over the matter's embedded passages.
   */
  search(matterId: string, query: string, limit?: number, queryVector?: number[]): Promise<SearchHit[]>
  pendingEmbeddings(matterId: string, limit?: number): Promise<PendingPassage[]>
  storeEmbeddings(embeddings: PassageEmbedding[]): Promise<number>
  embeddingStatus(matterId: string): Promise<EmbeddingStatus>
  reviewGrid(matterId: string): Promise<GridRow[]>
  proposeRenames(matterId: string): Promise<VaultAction[]>
  listActions(matterId: string): Promise<VaultAction[]>
  setActionStatus(id: string, status: 'approved' | 'rejected' | 'proposed'): Promise<VaultAction>
  executeApproved(matterId: string): Promise<VaultAction[]>
  undoAction(id: string): Promise<VaultAction>
  authorizeModelCall(request: ModelCallRequest): Promise<ModelCallDecision>
  listAudit(matterId?: string, limit?: number): Promise<AuditEntry[]>
  verifyAudit(): Promise<AuditVerification>
  exportRegisterCsv(matterId: string, dest: string): Promise<number>
}
