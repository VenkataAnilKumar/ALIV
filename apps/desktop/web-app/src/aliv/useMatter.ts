import { useCallback, useEffect, useState } from 'react'
import type { AuditEntry, GridRow, Matter, VaultAction, VaultDocument } from './types'
import { getVault } from './vault'

export interface MatterData {
  matter: Matter | null
  grid: GridRow[]
  documents: VaultDocument[]
  actions: VaultAction[]
  audit: AuditEntry[]
  loading: boolean
  error: string | null
  reload: () => Promise<void>
}

/** Loads everything a matter page shows, and reloads it after any change. */
export function useMatter(matterId: string): MatterData {
  const [state, setState] = useState<Omit<MatterData, 'reload'>>({
    matter: null,
    grid: [],
    documents: [],
    actions: [],
    audit: [],
    loading: true,
    error: null,
  })

  const reload = useCallback(async () => {
    const vault = getVault()
    try {
      const [matters, grid, documents, actions, audit] = await Promise.all([
        vault.listMatters(),
        vault.reviewGrid(matterId),
        vault.listDocuments(matterId),
        vault.listActions(matterId),
        vault.listAudit(matterId, 300),
      ])
      const matter = matters.find((m) => m.id === matterId) ?? null
      setState({
        matter,
        grid,
        documents,
        actions,
        audit,
        loading: false,
        error: matter ? null : 'This matter no longer exists.',
      })
    } catch (e) {
      setState((s) => ({ ...s, loading: false, error: String(e) }))
    }
  }, [matterId])

  useEffect(() => {
    setState((s) => ({ ...s, loading: true }))
    void reload()
  }, [reload])

  return { ...state, reload }
}

export function errorMessage(e: unknown): string {
  if (typeof e === 'string') return e
  if (e instanceof Error) return e.message
  return String(e)
}
