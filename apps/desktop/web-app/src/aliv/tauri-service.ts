import { invoke } from '@tauri-apps/api/core'
import type { VaultService } from './service'

const cmd = <T>(name: string, args?: Record<string, unknown>) =>
  invoke<T>(`plugin:aliv-vault|${name}`, args)

export class TauriVaultService implements VaultService {
  readonly mode = 'desktop' as const

  listMatters: VaultService['listMatters'] = () => cmd('list_matters')
  createMatter: VaultService['createMatter'] = (input) => cmd('create_matter', { input })
  updateMatter: VaultService['updateMatter'] = (id, input) => cmd('update_matter', { id, input })
  deleteMatter: VaultService['deleteMatter'] = (id) => cmd('delete_matter', { id })
  ingestPaths: VaultService['ingestPaths'] = (matterId, paths) =>
    cmd('ingest_paths', { matterId, paths })
  listDocuments: VaultService['listDocuments'] = (matterId) => cmd('list_documents', { matterId })
  search: VaultService['search'] = (matterId, query, limit) =>
    cmd('search', { matterId, query, limit })
  reviewGrid: VaultService['reviewGrid'] = (matterId) => cmd('review_grid', { matterId })
  proposeRenames: VaultService['proposeRenames'] = (matterId) => cmd('propose_renames', { matterId })
  listActions: VaultService['listActions'] = (matterId) => cmd('list_actions', { matterId })
  setActionStatus: VaultService['setActionStatus'] = (id, status) =>
    cmd('set_action_status', { id, status })
  executeApproved: VaultService['executeApproved'] = (matterId) =>
    cmd('execute_approved', { matterId })
  undoAction: VaultService['undoAction'] = (id) => cmd('undo_action', { id })
  authorizeModelCall: VaultService['authorizeModelCall'] = (request) =>
    cmd('authorize_model_call', { request })
  listAudit: VaultService['listAudit'] = (matterId, limit) =>
    cmd('list_audit', { matterId: matterId ?? null, limit })
  verifyAudit: VaultService['verifyAudit'] = () => cmd('verify_audit')
  exportRegisterCsv: VaultService['exportRegisterCsv'] = (matterId, dest) =>
    cmd('export_register_csv', { matterId, dest })
}
