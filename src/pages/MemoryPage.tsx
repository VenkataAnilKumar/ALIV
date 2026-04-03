import { useEffect, useMemo, useState } from 'react'
import { BrainCircuit, Plus, Trash2 } from 'lucide-react'
import { Badge } from '@/components/ui/Badge'
import { Button } from '@/components/ui/Button'
import { PageHeader } from '@/components/ui/PageHeader'
import { Table } from '@/components/ui/Table'
import { downloadTextFile, formatDateTime, toCsv } from '@/lib/utils'
import { useMemoryStore } from '@/stores/memoryStore'

const formatCitationJson = (citationJson?: string) => {
  if (!citationJson) {
    return 'No citation payload recorded for this fact.'
  }

  try {
    return JSON.stringify(JSON.parse(citationJson), null, 2)
  } catch {
    return citationJson
  }
}

export function MemoryPage() {
  const { facts, filter, search, loading, error, setFilter, setSearch, loadFacts, addFact, removeFact } = useMemoryStore()
  const [draft, setDraft] = useState('')
  const [scope, setScope] = useState<'session' | 'week' | 'permanent'>('session')

  const visibleFacts = useMemo(() => facts.filter((fact) => {
    const matchesScope = filter === 'all' || fact.scope === filter
    const matchesSearch = !search.trim() || fact.content.toLowerCase().includes(search.toLowerCase())
    return matchesScope && matchesSearch
  }), [facts, filter, search])
  const [selectedFactId, setSelectedFactId] = useState<string | null>(null)
  const selectedFact = visibleFacts.find((fact) => fact.id === selectedFactId) ?? visibleFacts[0]
  const expiredFacts = facts.filter((fact) => fact.expires_at && new Date(fact.expires_at).getTime() <= Date.now())

  useEffect(() => {
    void loadFacts()
  }, [loadFacts])

  useEffect(() => {
    setSelectedFactId(visibleFacts[0]?.id ?? null)
  }, [visibleFacts])

  useEffect(() => {
    const onPrimary = () => {
      if (!draft.trim()) return
      void addFact(draft, scope)
      setDraft('')
    }
    window.addEventListener('aliv:primary-action', onPrimary)
    return () => window.removeEventListener('aliv:primary-action', onPrimary)
  }, [addFact, draft, scope])

  const bulkDelete = async (mode: 'expired' | 'session') => {
    const targetIds = facts
      .filter((fact) => mode === 'expired'
        ? Boolean(fact.expires_at && new Date(fact.expires_at).getTime() <= Date.now())
        : fact.scope === 'session')
      .map((fact) => fact.id)

    for (const factId of targetIds) {
      await removeFact(factId)
    }
  }

  return (
    <div className="space-y-6">
      <PageHeader
        eyebrow="Context"
        title="Memory"
        description="Persist reusable findings across the current matter, filter them by retention scope, and prune stale or weak facts."
        actions={
          <>
            <Button variant="secondary" onClick={() => downloadTextFile('memory-facts.csv', toCsv(visibleFacts.map((fact) => ({
              id: fact.id,
              content: fact.content,
              scope: fact.scope,
              created_at: fact.created_at,
              expires_at: fact.expires_at ?? '',
            }))), 'text/csv;charset=utf-8')} disabled={!visibleFacts.length}>
              Export CSV
            </Button>
            <Button variant="secondary" onClick={() => void loadFacts()}><BrainCircuit className="h-4 w-4" />Refresh memory</Button>
          </>
        }
      />

      {expiredFacts.length ? (
        <div className="flex items-center justify-between rounded-xl border border-amber-500/20 bg-amber-500/8 px-4 py-3 text-sm text-amber-200">
          <span>{expiredFacts.length} facts are expired and can be removed from the memory layer.</span>
          <Button variant="ghost" onClick={() => void bulkDelete('expired')}>Delete expired</Button>
        </div>
      ) : null}

      <div className="grid gap-6 xl:grid-cols-[420px_1fr]">
        <section className="panel p-5">
          <div className="text-sm font-medium text-zinc-100">Save fact</div>
          <div className="mt-4 space-y-4">
            <textarea
              className="input-base min-h-[220px] w-full resize-none leading-6"
              placeholder="Pin a grounded legal finding, clause summary, or workflow note."
              value={draft}
              onChange={(event) => setDraft(event.target.value)}
            />
            <div className="grid gap-4 md:grid-cols-2">
              <select className="input-base w-full" title="Memory scope" aria-label="Memory scope" value={scope} onChange={(event) => setScope(event.target.value as typeof scope)}>
                <option value="session">Session</option>
                <option value="week">Week</option>
                <option value="permanent">Permanent</option>
              </select>
              <Button variant="primary" onClick={() => { void addFact(draft, scope); setDraft('') }} disabled={!draft.trim()}>
                <Plus className="h-4 w-4" />
                Save fact
              </Button>
            </div>
            <div className="flex justify-end">
              <Button variant="ghost" onClick={() => void bulkDelete('session')} disabled={!facts.some((fact) => fact.scope === 'session')}>
                Clear session facts
              </Button>
            </div>
          </div>
        </section>

        <section className="space-y-4">
          <div className="panel p-5">
            <div className="grid gap-4 md:grid-cols-[180px_1fr]">
              <select className="input-base w-full" title="Memory scope filter" aria-label="Memory scope filter" value={filter} onChange={(event) => setFilter(event.target.value as typeof filter)}>
                <option value="all">All scopes</option>
                <option value="session">Session</option>
                <option value="week">Week</option>
                <option value="permanent">Permanent</option>
              </select>
              <input className="input-base w-full" value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Search saved facts" />
            </div>
          </div>

          <div className="grid gap-6 lg:grid-cols-[1.15fr_0.85fr]">
            <Table
              columns={[
                {
                  key: 'content',
                  header: 'Fact',
                  render: (row) => (
                    <button type="button" className="max-w-[680px] text-left text-sm leading-6 text-zinc-300" onClick={() => setSelectedFactId(row.id)}>
                      {row.content}
                    </button>
                  ),
                },
                { key: 'scope', header: 'Scope', render: (row) => <Badge tone="info">{row.scope}</Badge> },
                { key: 'created', header: 'Created', render: (row) => <span className="font-mono text-[11px] text-zinc-500">{formatDateTime(row.created_at)}</span> },
                {
                  key: 'remove',
                  header: '',
                  className: 'w-16',
                  render: (row) => (
                    <Button variant="ghost" className="px-2 py-1" onClick={() => void removeFact(row.id)}>
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  ),
                },
              ]}
              rows={visibleFacts}
              loading={loading}
              error={error}
              rowKey={(row) => row.id}
              rowClassName={(row) => (row.id === selectedFact?.id ? 'bg-indigo-500/8' : undefined)}
              emptyTitle="No memory facts"
              emptyDescription="Save a reusable fact from retrieval or manual review to keep the context layer warm."
            />

            <section className="panel p-5">
              <div className="text-sm font-medium text-zinc-100">Fact detail</div>
              {selectedFact ? (
                <>
                  <div className="mt-4 space-y-3 text-sm text-zinc-300">
                    <div>{selectedFact.content}</div>
                    <div className="flex flex-wrap gap-2">
                      <Badge tone="info">{selectedFact.scope}</Badge>
                      {selectedFact.expires_at ? <Badge tone="warning">expires {formatDateTime(selectedFact.expires_at)}</Badge> : null}
                    </div>
                  </div>
                  <div className="mt-4 text-sm font-medium text-zinc-100">Citation</div>
                  <pre className="mt-3 overflow-x-auto rounded-xl border border-zinc-800 bg-zinc-950/70 p-4 font-mono text-xs leading-6 text-zinc-300">
                    {formatCitationJson(selectedFact.citation_json)}
                  </pre>
                </>
              ) : (
                <div className="mt-4 rounded-xl border border-dashed border-zinc-800 px-4 py-8 text-sm text-zinc-500">
                  Select a memory fact to inspect its citation payload and retention details.
                </div>
              )}
            </section>
          </div>
        </section>
      </div>
    </div>
  )
}