import { create } from 'zustand'
import type * as T from '@/types/contracts'
import { getConversation, listConversations, retrieveAndAssemble, saveMemoryFact, startConversation } from '@/lib/tauri'
import { useWorkspaceStore } from './workspaceStore'

interface QueryState {
  query: string
  response?: T.RetrievalResponse
  conversations: T.ConversationStub[]
  turns: T.ConversationTurnView[]
  conversationId?: string
  isLoading: boolean
  error: string | null
  latencyMs?: number
  setQuery: (query: string) => void
  ensureConversation: () => Promise<string | undefined>
  loadConversations: () => Promise<void>
  loadConversation: (conversationId: string) => Promise<void>
  runQuery: () => Promise<void>
  pinCurrentAnswer: () => Promise<void>
}

export const useQueryStore = create<QueryState>((set, get) => ({
  query: '',
  conversations: [],
  turns: [],
  isLoading: false,
  error: null,
  setQuery: (query) => set({ query }),
  ensureConversation: async () => {
    const workspace = useWorkspaceStore.getState().activeWorkspace
    if (!workspace) {
      set({ error: 'Register a workspace first.' })
      return undefined
    }
    if (get().conversationId) return get().conversationId
    const conversation = await startConversation({ workspace_path: workspace })
    set({ conversationId: conversation.id })
    return conversation.id
  },
  loadConversations: async () => {
    const workspace = useWorkspaceStore.getState().activeWorkspace
    if (!workspace) return
    const conversations = await listConversations({ workspace_path: workspace, limit: 20 })
    set({ conversations })
  },
  loadConversation: async (conversationId) => {
    const turns = await getConversation({ conversation_id: conversationId, limit: 50 })
    set({ conversationId, turns })
  },
  runQuery: async () => {
    const workspace = useWorkspaceStore.getState().activeWorkspace
    if (!workspace || !get().query.trim()) {
      set({ error: 'Provide a workspace and a query.' })
      return
    }
    set({ isLoading: true, error: null })
    try {
      const conversationId = await get().ensureConversation()
      if (!conversationId) return
      const start = performance.now()
      const response = await retrieveAndAssemble({
        query: get().query,
        top_k: 5,
        workspace_path: workspace,
        conversation_id: conversationId,
        token_budget: 8192,
      })
      set({
        response,
        isLoading: false,
        latencyMs: Math.round(performance.now() - start),
      })
      await get().loadConversations()
      await get().loadConversation(conversationId)
    } catch (error) {
      set({ isLoading: false, error: error instanceof Error ? error.message : String(error) })
    }
  },
  pinCurrentAnswer: async () => {
    const workspace = useWorkspaceStore.getState().activeWorkspace
    const answer = get().response?.assembled_prompt
    if (!workspace || !answer) return
    await saveMemoryFact({ workspace_path: workspace, content: answer, scope: 'session' })
  },
}))
