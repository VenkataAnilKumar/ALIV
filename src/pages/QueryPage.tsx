import { useEffect } from 'react'
import ReactMarkdown from 'react-markdown'
import { MessageSquareQuote, Pin, Search } from 'lucide-react'
import { Badge } from '@/components/ui/Badge'
import { Button } from '@/components/ui/Button'
import { ConfidenceBar } from '@/components/ui/ConfidenceBar'
import { PageHeader } from '@/components/ui/PageHeader'
import { StatCard } from '@/components/ui/StatCard'
import { formatDateTime, formatNumber } from '@/lib/utils'
import { useQueryStore } from '@/stores/queryStore'

export function QueryPage() {
  const {
    query,
    response,
    conversations,
    turns,
    conversationId,
    isLoading,
    error,
    latencyMs,
    setQuery,
    loadConversations,
    loadConversation,
    runQuery,
    pinCurrentAnswer,
  } = useQueryStore()

  useEffect(() => {
    void loadConversations()
  }, [loadConversations])

  useEffect(() => {
    const onPrimary = () => {
      void runQuery()
    }
    window.addEventListener('aliv:primary-action', onPrimary)
    return () => window.removeEventListener('aliv:primary-action', onPrimary)
  }, [runQuery])

  return (
    <div className="space-y-6">
      <PageHeader
        eyebrow="Research"
        title="Query & Assemble"
        description="Run retrieval over the active workspace, inspect ranked evidence, and pin useful answers into memory for later sessions."
        actions={
          <>
            <Button variant="secondary" onClick={() => void pinCurrentAnswer()} disabled={!response?.assembled_prompt}>
              <Pin className="h-4 w-4" />
              Pin answer
            </Button>
            <Button variant="primary" onClick={() => void runQuery()} disabled={isLoading || !query.trim()}>
              <Search className="h-4 w-4" />
              {isLoading ? 'Running...' : 'Run query'}
            </Button>
          </>
        }
      />

      <div className="grid gap-4 md:grid-cols-3">
        <StatCard label="Total Hits" value={formatNumber(response?.total_hits ?? 0)} detail={`Top K ${response?.top_k ?? 5}`} />
        <StatCard label="Prompt Tokens" value={formatNumber(response?.prompt_token_estimate ?? 0)} detail={`Latency ${latencyMs ?? 0} ms`} />
        <StatCard label="Conversation" value={conversationId ?? 'unstarted'} detail={`Turns ${turns.length}`} />
      </div>

      <div className="grid gap-6 xl:grid-cols-[320px_1fr]">
        <section className="space-y-6">
          <div className="panel p-5">
            <div className="text-sm font-medium text-zinc-100">Prompt</div>
            <textarea
              className="input-base mt-4 min-h-[220px] w-full resize-none leading-6"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="What lease clauses mention unilateral renewal rights and notice periods?"
            />
            <div className="mt-3 text-xs text-zinc-500">Ctrl/Cmd+Enter runs the current query.</div>
            {error ? <div className="mt-4 rounded-lg border border-red-500/20 bg-red-500/5 px-4 py-3 text-sm text-red-300">{error}</div> : null}
          </div>

          <div className="panel p-5">
            <div className="flex items-center gap-2 text-sm font-medium text-zinc-100">
              <MessageSquareQuote className="h-4 w-4 text-indigo-300" />
              Conversations
            </div>
            <div className="mt-4 space-y-2">
              {conversations.length === 0 ? (
                <div className="rounded-xl border border-dashed border-zinc-800 px-4 py-8 text-sm text-zinc-500">
                  No saved conversations yet.
                </div>
              ) : (
                conversations.map((conversation) => (
                  <button
                    key={conversation.id}
                    type="button"
                    className="w-full rounded-xl border border-zinc-800 bg-zinc-950/50 px-4 py-3 text-left transition hover:border-zinc-700"
                    onClick={() => void loadConversation(conversation.id)}
                  >
                    <div className="text-sm text-zinc-100">{conversation.title}</div>
                    <div className="mt-1 flex items-center justify-between font-mono text-[11px] text-zinc-500">
                      <span>{conversation.turn_count} turns</span>
                      <span>{formatDateTime(conversation.updated_at)}</span>
                    </div>
                  </button>
                ))
              )}
            </div>
          </div>
        </section>

        <section className="space-y-6">
          <div className="panel p-5">
            <div className="text-sm font-medium text-zinc-100">Assembled answer</div>
            <div className="prose prose-invert mt-4 max-w-none text-sm leading-7 prose-headings:text-zinc-100 prose-p:text-zinc-300 prose-strong:text-zinc-100 prose-code:text-indigo-300">
              <ReactMarkdown>{response?.assembled_prompt ?? 'Run a query to assemble a grounded answer.'}</ReactMarkdown>
            </div>
          </div>

          <div className="grid gap-6 lg:grid-cols-[1.1fr_0.9fr]">
            <div className="panel p-5">
              <div className="text-sm font-medium text-zinc-100">Retrieved evidence</div>
              <div className="mt-4 space-y-3">
                {response?.hits.length ? (
                  response.hits.map((hit) => (
                    <div key={`${hit.file_id}:${hit.chunk_index}`} className="rounded-xl border border-zinc-800 bg-zinc-950/50 p-4">
                      <div className="flex flex-wrap items-center gap-2">
                        <Badge tone="info">score {hit.score.toFixed(3)}</Badge>
                        <Badge tone={hit.warning_count > 0 ? 'warning' : 'success'}>warnings {hit.warning_count}</Badge>
                        <span className="font-mono text-[11px] text-zinc-500">{hit.source_path}</span>
                      </div>
                      <div className="mt-3 text-sm leading-6 text-zinc-300">{hit.content_preview}</div>
                      <div className="mt-3">
                        <ConfidenceBar value={hit.parser_confidence} />
                      </div>
                    </div>
                  ))
                ) : (
                  <div className="rounded-xl border border-dashed border-zinc-800 px-4 py-8 text-sm text-zinc-500">
                    Retrieved hits will appear here after a query runs.
                  </div>
                )}
              </div>
            </div>

            <div className="panel p-5">
              <div className="text-sm font-medium text-zinc-100">Conversation turns</div>
              <div className="mt-4 space-y-3">
                {turns.length === 0 ? (
                  <div className="rounded-xl border border-dashed border-zinc-800 px-4 py-8 text-sm text-zinc-500">
                    Load a saved conversation to inspect turns.
                  </div>
                ) : (
                  turns.map((turn) => (
                    <div key={turn.id} className="rounded-xl border border-zinc-800 bg-zinc-950/50 p-4">
                      <div className="flex items-center justify-between">
                        <Badge tone={turn.role === 'assistant' ? 'info' : 'pending'}>{turn.role}</Badge>
                        <span className="font-mono text-[11px] text-zinc-500">{formatDateTime(turn.created_at)}</span>
                      </div>
                      <div className="mt-3 whitespace-pre-wrap text-sm leading-6 text-zinc-300">{turn.content}</div>
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  )
}