import { create } from 'zustand'
import { persist, createJSONStorage } from 'zustand/middleware'
import { localStorageKey, CACHE_EXPIRY_MS } from '@/constants/localStorage'
import { getServiceHub } from '@/hooks/useServiceHub'
import type { CatalogModel } from '@/services/models/types'
import { backendStorage } from '@/lib/backendStorage'

type RecommendedModelState = {
  model: CatalogModel | null
  lastFetchedAt: number | null
  loading: boolean
  error: boolean
  fetchRecommendedModel: (force?: boolean) => Promise<void>
}

export const useRecommendedModel = create<RecommendedModelState>()(
  persist(
    (set, get) => ({
      model: null,
      lastFetchedAt: null,
      loading: false,
      error: false,
      fetchRecommendedModel: async (force = false) => {
        const { lastFetchedAt, loading } = get()

        if (loading) return

        if (
          !force &&
          lastFetchedAt &&
          Date.now() - lastFetchedAt < CACHE_EXPIRY_MS
        ) {
          return
        }

        set({ loading: true, error: false })

        try {
          const result = await getServiceHub()
            .models()
            .fetchRecommendedModel()

          if (result) {
            set({
              model: result,
              lastFetchedAt: Date.now(),
              loading: false,
            })
          } else {
            set({ error: true, loading: false })
          }
        } catch {
          set({ error: true, loading: false })
        }
      },
    }),
    {
      name: localStorageKey.recommendedModel,
      storage: createJSONStorage(() => backendStorage),
      skipHydration: true,
      partialize: (state) => ({
        model: state.model,
        lastFetchedAt: state.lastFetchedAt,
      }),
    }
  )
)
