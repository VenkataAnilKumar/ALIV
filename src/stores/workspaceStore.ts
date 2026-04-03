import { create } from 'zustand'
import type * as T from '@/types/contracts'
import { getRuntimeSnapshot, registerWorkspace } from '@/lib/tauri'

interface WorkspaceState {
  folders: string[]
  activeWorkspace?: string
  runtime?: T.RuntimeSnapshot
  policy: 'Safe' | 'Standard' | 'Aggressive'
  ollamaUrl: string
  modelName: string
  embeddingModel: string
  isSaving: boolean
  error: string | null
  addFolder: (path: string) => void
  removeFolder: (path: string) => void
  setActiveWorkspace: (path: string) => void
  setPolicy: (value: WorkspaceState['policy']) => void
  setOllamaUrl: (value: string) => void
  setModelName: (value: string) => void
  setEmbeddingModel: (value: string) => void
  refreshRuntime: () => Promise<void>
  saveWorkspace: () => Promise<void>
}

export const useWorkspaceStore = create<WorkspaceState>((set, get) => ({
  folders: [],
  policy: 'Safe',
  ollamaUrl: 'http://127.0.0.1:11434',
  modelName: 'llama3.2:3b',
  embeddingModel: 'all-MiniLM-L6-v2',
  isSaving: false,
  error: null,
  addFolder: (path) =>
    set((state) => ({
      folders: state.folders.includes(path) ? state.folders : [...state.folders, path],
      activeWorkspace: state.activeWorkspace ?? path,
    })),
  removeFolder: (path) =>
    set((state) => {
      const folders = state.folders.filter((item) => item !== path)
      return {
        folders,
        activeWorkspace: state.activeWorkspace === path ? folders[0] : state.activeWorkspace,
      }
    }),
  setActiveWorkspace: (path) => set({ activeWorkspace: path }),
  setPolicy: (policy) => set({ policy }),
  setOllamaUrl: (ollamaUrl) => set({ ollamaUrl }),
  setModelName: (modelName) => set({ modelName }),
  setEmbeddingModel: (embeddingModel) => set({ embeddingModel }),
  refreshRuntime: async () => {
    const runtime = await getRuntimeSnapshot()
    set({ runtime })
  },
  saveWorkspace: async () => {
    const { activeWorkspace } = get()
    if (!activeWorkspace) {
      set({ error: 'Choose a workspace path before saving.' })
      return
    }
    set({ isSaving: true, error: null })
    try {
      await registerWorkspace({ path: activeWorkspace })
      const runtime = await getRuntimeSnapshot()
      set({ runtime, isSaving: false })
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      set({ error: message, isSaving: false })
      throw new Error(message)
    }
  },
}))
