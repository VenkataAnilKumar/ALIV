import { create } from 'zustand'
import type * as T from '@/types/contracts'
import {
  approveActions,
  evaluateActionRisk,
  executeActions,
  generateActionRecommendations,
  getActionAuditLog,
  undoActionBatch,
} from '@/lib/tauri'
import { useWorkspaceStore } from './workspaceStore'

interface UndoBanner {
  batchId: string
  count: number
}

interface ActionsState {
  preview?: T.ActionPreviewList
  audit?: T.AuditLogResponse
  selectedIds: string[]
  approved: boolean
  execution?: T.ActionExecutionResult
  undoBanner?: UndoBanner
  error: string | null
  loading: boolean
  loadRecommendations: () => Promise<void>
  evaluateRisk: () => Promise<void>
  toggleSelected: (id: string) => void
  selectLowRisk: () => void
  approveSelection: () => Promise<void>
  executeApproved: () => Promise<void>
  loadAudit: () => Promise<void>
  undoBatch: (batchId: string) => Promise<void>
  dismissUndoBanner: () => void
}

export const useActionsStore = create<ActionsState>((set, get) => ({
  selectedIds: [],
  approved: false,
  error: null,
  loading: false,
  loadRecommendations: async () => {
    const workspace = useWorkspaceStore.getState().activeWorkspace
    if (!workspace) {
      set({ error: 'Register a workspace first.' })
      return
    }
    set({ loading: true, error: null })
    try {
      const preview = await generateActionRecommendations({ workspace_path: workspace, batch_limit: 50 })
      const selectedIds = preview.items
        .filter((item) => item.risk_level !== 'high')
        .map((item) => item.id)
      set({ preview, selectedIds, approved: false, loading: false })
    } catch (error) {
      set({ error: error instanceof Error ? error.message : String(error), loading: false })
    }
  },
  evaluateRisk: async () => {
    const ids = get().preview?.items.map((item) => item.id) ?? []
    if (ids.length === 0) return
    const preview = await evaluateActionRisk({ recommendation_ids: ids })
    set({
      preview,
      selectedIds: preview.items.filter((item) => item.risk_level !== 'high').map((item) => item.id),
      approved: false,
    })
  },
  toggleSelected: (id) =>
    set((state) => ({
      selectedIds: state.selectedIds.includes(id)
        ? state.selectedIds.filter((item) => item !== id)
        : [...state.selectedIds, id],
      approved: false,
    })),
  selectLowRisk: () =>
    set((state) => ({
      selectedIds: state.preview?.items.filter((item) => item.risk_level !== 'high').map((item) => item.id) ?? [],
      approved: false,
    })),
  approveSelection: async () => {
    if (get().selectedIds.length === 0) return
    await approveActions({ recommendation_ids: get().selectedIds })
    set({ approved: true })
  },
  executeApproved: async () => {
    if (!get().approved) {
      set({ error: 'Approve actions before execution.' })
      return
    }
    const result = await executeActions({ batch_id: crypto.randomUUID() })
    set({
      execution: result,
      undoBanner: { batchId: result.batch_id, count: result.succeeded },
      approved: false,
      selectedIds: [],
    })
    await get().loadAudit()
  },
  loadAudit: async () => {
    const audit = await getActionAuditLog({ limit: 100 })
    set({ audit })
  },
  undoBatch: async (batchId) => {
    await undoActionBatch({ execution_batch_id: batchId })
    await get().loadAudit()
  },
  dismissUndoBanner: () => set({ undoBanner: undefined }),
}))
