import { create } from 'zustand'
import type * as T from '@/types/contracts'
import { deleteMemoryFact, listMemoryFacts, saveMemoryFact } from '@/lib/tauri'
import { useWorkspaceStore } from './workspaceStore'

interface MemoryState {
  facts: T.MemoryFactView[]
  filter: 'all' | 'permanent' | 'week' | 'session'
  search: string
  loading: boolean
  error: string | null
  setFilter: (filter: MemoryState['filter']) => void
  setSearch: (search: string) => void
  loadFacts: () => Promise<void>
  addFact: (content: string, scope: 'session' | 'week' | 'permanent') => Promise<void>
  removeFact: (factId: string) => Promise<void>
}

export const useMemoryStore = create<MemoryState>((set, get) => ({
  facts: [],
  filter: 'all',
  search: '',
  loading: false,
  error: null,
  setFilter: (filter) => set({ filter }),
  setSearch: (search) => set({ search }),
  loadFacts: async () => {
    const workspace = useWorkspaceStore.getState().activeWorkspace
    if (!workspace) return
    set({ loading: true, error: null })
    try {
      const facts = await listMemoryFacts({ workspace_path: workspace, limit: 100 })
      set({ facts, loading: false })
    } catch (error) {
      set({ loading: false, error: error instanceof Error ? error.message : String(error) })
    }
  },
  addFact: async (content, scope) => {
    const workspace = useWorkspaceStore.getState().activeWorkspace
    if (!workspace) return
    await saveMemoryFact({ workspace_path: workspace, content, scope })
    await get().loadFacts()
  },
  removeFact: async (factId) => {
    await deleteMemoryFact({ fact_id: factId })
    await get().loadFacts()
  },
}))
