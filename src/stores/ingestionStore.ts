import { create } from 'zustand'
import type * as T from '@/types/contracts'
import { getIngestionTelemetry, runIngestion } from '@/lib/tauri'
import { useWorkspaceStore } from './workspaceStore'

interface IngestionState {
  telemetry?: T.IngestionTelemetry
  history: Array<{
    id: string
    mode: 'incremental' | 'full'
    detail: string
    status: string
    timestamp: string
  }>
  isRunning: boolean
  error: string | null
  run: (incremental: boolean) => Promise<void>
  refresh: () => Promise<void>
}

export const useIngestionStore = create<IngestionState>((set) => ({
  history: [],
  isRunning: false,
  error: null,
  run: async (incremental) => {
    const workspace = useWorkspaceStore.getState().activeWorkspace
    if (!workspace) {
      set({ error: 'Register a workspace first.' })
      return
    }
    set({ isRunning: true, error: null })
    try {
      const result = await runIngestion({ workspace_path: workspace, incremental })
      const telemetry = await getIngestionTelemetry()
      set((state) => ({
        isRunning: false,
        telemetry,
        history: [
          {
            id: `${Date.now()}`,
            mode: incremental ? ('incremental' as const) : ('full' as const),
            detail: result.detail,
            status: result.status,
            timestamp: new Date().toISOString(),
          },
          ...state.history,
        ].slice(0, 5),
      }))
    } catch (error) {
      set({ isRunning: false, error: error instanceof Error ? error.message : String(error) })
    }
  },
  refresh: async () => {
    try {
      const telemetry = await getIngestionTelemetry()
      set({ telemetry, error: null })
    } catch (error) {
      set({ error: error instanceof Error ? error.message : String(error) })
    }
  },
}))
