import type {
  AuditEntry,
  AuditVerification,
  GridRow,
  IngestReport,
  Matter,
  MatterInput,
  ModelCallDecision,
  ModelCallRequest,
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
  search(matterId: string, query: string, limit?: number): Promise<SearchHit[]>
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
