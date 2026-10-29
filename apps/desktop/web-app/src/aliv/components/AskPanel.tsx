import { Fragment, useEffect, useRef, useState } from 'react'
import { BookOpenText, Cloud, Cpu, FileText, Loader2, Lock, SendHorizontal, ShieldAlert, Square, TableProperties } from 'lucide-react'
import DropdownModelProvider from '@/containers/DropdownModelProvider'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { useModelProvider } from '@/hooks/useModelProvider'
import { cn } from '@/lib/utils'
import { ask, modelLocation, parseCitations, type AskResult, type AskSource } from '../ask'
import type { Matter } from '../types'
import { localEmbedder } from '../semantic'
import { errorMessage } from '../useMatter'
import { getVault } from '../vault'
import { EmptyState, Pill, Section } from './primitives'

interface Turn {
  id: number
  question: string
  result: AskResult | null
  streaming: string
}

export function AskPanel({
  matter,
  initialQuestion,
  onAsked,
  hasDocuments,
}: {
  matter: Matter
  initialQuestion: string
  onAsked: () => void
  hasDocuments: boolean
}) {
  const vault = getVault()
  const selectedModel = useModelProvider((s) => s.selectedModel)
  const provider = useModelProvider((s) => s.getProviderByName(s.selectedProvider))
  const [input, setInput] = useState('')
  const [turns, setTurns] = useState<Turn[]>([])
  const [busy, setBusy] = useState(false)
  const [active, setActive] = useState<{ turn: number; n: number } | null>(null)
  const abortRef = useRef<AbortController | null>(null)
  const askedInitial = useRef<string>('')

  const location = provider ? modelLocation(provider) : null
  const blocked = location === 'cloud' && matter.policy === 'local_only'

  const run = async (question: string) => {
    const q = question.trim()
    if (!q || busy) return
    const id = Date.now()
    setTurns((t) => [{ id, question: q, result: null, streaming: '' }, ...t])
    setInput('')
    setBusy(true)
    const controller = new AbortController()
    abortRef.current = controller
    try {
      const result = await ask({
        vault,
        matter,
        question: q,
        provider: provider ?? undefined,
        modelId: selectedModel?.id,
        signal: controller.signal,
        // Embeddings come from the local engine, so they are safe for every
        // matter; the browser preview has none and searches by keyword.
        embed: vault.mode === 'desktop' ? localEmbedder : undefined,
        contextTokens: contextTokens(selectedModel),
        onText: (text) => setTurns((t) => t.map((x) => (x.id === id ? { ...x, streaming: text } : x))),
      })
      setTurns((t) => t.map((x) => (x.id === id ? { ...x, result } : x)))
      // Open the first source the answer actually cites.
      const firstCite = parseCitations(result.text, result.sources.length).find((p) => p.type === 'cite')
      const n = firstCite?.type === 'cite' ? firstCite.n : 1
      setActive(result.sources.length ? { turn: id, n } : null)
    } catch (e) {
      setTurns((t) => t.map((x) => (x.id === id ? { ...x, result: { status: 'error', text: errorMessage(e), sources: [] } } : x)))
    } finally {
      setBusy(false)
      abortRef.current = null
      onAsked()
    }
  }

  useEffect(() => {
    if (initialQuestion && askedInitial.current !== initialQuestion) {
      askedInitial.current = initialQuestion
      void run(initialQuestion)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [initialQuestion])

  const activeTurn = turns.find((t) => t.id === active?.turn) ?? turns[0]
  const sources = activeTurn?.result?.sources ?? []

  return (
    <div className="grid gap-5 lg:grid-cols-5">
      <div className="lg:col-span-3 flex flex-col gap-4 min-w-0">
        <Section>
          <div className="p-3 flex flex-col gap-3">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <DropdownModelProvider />
              <RoutingPill location={location} blocked={blocked} hasModel={!!selectedModel} />
            </div>
            <form
              onSubmit={(e) => {
                e.preventDefault()
                void run(input)
              }}
              className="flex items-end gap-2"
            >
              <Textarea
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault()
                    void run(input)
                  }
                }}
                rows={2}
                placeholder={hasDocuments ? 'Ask a question about this matter. Answers cite the documents.' : 'Add documents to this matter first.'}
                className="min-h-[56px] resize-none"
                disabled={!hasDocuments}
              />
              {busy ? (
                <Button type="button" size="icon" variant="outline" onClick={() => abortRef.current?.abort()} aria-label="Stop">
                  <Square className="size-4" />
                </Button>
              ) : (
                <Button type="submit" size="icon" disabled={!input.trim() || !hasDocuments} aria-label="Ask">
                  <SendHorizontal className="size-4" />
                </Button>
              )}
            </form>
            {blocked && (
              <div className="flex items-start gap-2 rounded-lg bg-red-500/5 border border-red-500/20 p-2.5 text-xs">
                <ShieldAlert className="size-4 text-red-600 shrink-0" />
                <span>
                  <span className="font-medium">This matter is Local only.</span> The selected model runs in the cloud, so ALIV will refuse to send it any passages. Pick a local model, or switch the matter to Hybrid in its settings.
                </span>
              </div>
            )}
          </div>
        </Section>

        {turns.length === 0 ? (
          <Section>
            <EmptyState
              icon={<BookOpenText className="size-6" />}
              title="Every answer shows its sources"
              body="ALIV searches this matter's passages and its extracted register, then asks the model to answer only from them and cite each statement. Click a citation to read the passage."
            />
          </Section>
        ) : (
          turns.map((t) => (
            <TurnView
              key={t.id}
              turn={t}
              activeN={active?.turn === t.id ? active.n : null}
              onCite={(n) => setActive({ turn: t.id, n })}
            />
          ))
        )}
      </div>

      <div className="lg:col-span-2 min-w-0">
        <Section
          className="lg:sticky lg:top-0"
          title="Sources"
          description={sources.length ? 'Passages and register entries given to the model, numbered as cited.' : 'Sources for the selected answer appear here.'}
        >
          <ol className="flex flex-col gap-2 px-3 pb-3 max-h-[70vh] overflow-y-auto">
            {sources.map((s) => (
              <SourceCard
                key={s.n}
                source={s}
                active={active?.turn === activeTurn?.id && active?.n === s.n}
                onClick={() => activeTurn && setActive({ turn: activeTurn.id, n: s.n })}
              />
            ))}
          </ol>
        </Section>
      </div>
    </div>
  )
}

function RoutingPill({ location, blocked, hasModel }: { location: 'local' | 'cloud' | null; blocked: boolean; hasModel: boolean }) {
  if (!hasModel || !location) return <Pill tone="neutral">No model selected · evidence only</Pill>
  if (blocked) {
    return (
      <Pill tone="danger">
        <ShieldAlert className="size-3" /> Cloud model blocked by policy
      </Pill>
    )
  }
  return location === 'local' ? (
    <Pill tone="success">
      <Cpu className="size-3" /> Runs on this computer
    </Pill>
  ) : (
    <Pill tone="info">
      <Cloud className="size-3" /> Cloud · matching passages only, logged
    </Pill>
  )
}

function TurnView({ turn, activeN, onCite }: { turn: Turn; activeN: number | null; onCite: (n: number) => void }) {
  const r = turn.result
  const text = r?.text ?? turn.streaming
  const maxN = r?.sources.length ?? 0
  return (
    <Section>
      <div className="px-4 pt-4 pb-2 text-sm font-medium">{turn.question}</div>
      <div className="px-4 pb-4">
        {!r && !turn.streaming ? (
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <Loader2 className="size-4 animate-spin" /> Searching the matter…
          </div>
        ) : r?.status === 'blocked' ? (
          <div className="flex items-start gap-2 text-sm text-red-700 dark:text-red-300">
            <Lock className="size-4 mt-0.5 shrink-0" /> {r.text}
          </div>
        ) : r && r.status !== 'answered' && r.status !== 'computed' ? (
          <p className="text-sm text-muted-foreground">{r.text}</p>
        ) : (
          <AnswerText text={text} maxSource={maxN || 99} activeN={activeN} onCite={onCite} />
        )}
        {r?.status === 'computed' && (
          <div className="mt-3 flex items-center gap-2 text-xs text-muted-foreground">
            <TableProperties className="size-3.5" /> Computed from the extracted register, no model used. Choose a model for a written answer.
          </div>
        )}
        {r?.model && (
          <div className="mt-3 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            {r.location === 'local' ? <Cpu className="size-3.5" /> : <Cloud className="size-3.5" />}
            {r.model} · {r.provider} · {r.location}
            {r.status === 'answered' && ` · ${r.sources.length} sources`}
          </div>
        )}
        {r?.searchMode && r.sources.length > 0 && (
          <div className="mt-1 text-xs text-muted-foreground">
            {r.searchMode === 'hybrid'
              ? 'Passages found by keyword and by meaning.'
              : 'Passages found by keyword. Meaning-based search needs the local embedding model.'}
            {!!r.omitted && ` ${r.omitted} more matching ${r.omitted === 1 ? 'source was' : 'sources were'} left out to fit the model's context window.`}
          </div>
        )}
      </div>
    </Section>
  )
}

/** Paragraphs, bullet lists, **bold** and clickable [n] citations. */
export function AnswerText({
  text,
  maxSource,
  activeN,
  onCite,
}: {
  text: string
  maxSource: number
  activeN?: number | null
  onCite: (n: number) => void
}) {
  const blocks = text.split(/\n{2,}/)
  return (
    <div className="flex flex-col gap-2 text-sm leading-relaxed">
      {blocks.map((block, bi) => {
        const lines = block.split('\n').filter((l) => l.trim())
        const isList = lines.length > 0 && lines.every((l) => /^\s*(?:[-*•]|\d+[.)])\s+/.test(l))
        if (isList) {
          return (
            <ul key={bi} className="flex flex-col gap-1 pl-4 list-disc">
              {lines.map((l, li) => (
                <li key={li}>
                  <Inline text={l.replace(/^\s*(?:[-*•]|\d+[.)])\s+/, '')} maxSource={maxSource} activeN={activeN} onCite={onCite} />
                </li>
              ))}
            </ul>
          )
        }
        return (
          <p key={bi}>
            {lines.map((l, li) => (
              <Fragment key={li}>
                {li > 0 && <br />}
                <Inline text={l} maxSource={maxSource} activeN={activeN} onCite={onCite} />
              </Fragment>
            ))}
          </p>
        )
      })}
    </div>
  )
}

function Inline({ text, maxSource, activeN, onCite }: { text: string; maxSource: number; activeN?: number | null; onCite: (n: number) => void }) {
  return (
    <>
      {parseCitations(text, maxSource).map((p, i) =>
        p.type === 'cite' ? (
          <button
            key={i}
            onClick={() => onCite(p.n)}
            className={cn(
              'mx-0.5 inline-flex items-center justify-center rounded-md border px-1.5 text-[11px] font-medium tabular-nums align-text-top cursor-pointer transition-colors',
              activeN === p.n ? 'bg-foreground text-background border-foreground' : 'bg-muted hover:bg-accent'
            )}
            aria-label={`Source ${p.n}`}
          >
            {p.n}
          </button>
        ) : (
          <Fragment key={i}>
            {p.text.split(/(\*\*[^*]+\*\*)/g).map((s, j) =>
              s.startsWith('**') && s.endsWith('**') ? <strong key={j}>{s.slice(2, -2)}</strong> : <Fragment key={j}>{s}</Fragment>
            )}
          </Fragment>
        )
      )}
    </>
  )
}

function SourceCard({ source, active, onClick }: { source: AskSource; active: boolean; onClick: () => void }) {
  const ref = useRef<HTMLLIElement>(null)
  useEffect(() => {
    if (active) ref.current?.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
  }, [active])
  return (
    <li
      ref={ref}
      onClick={onClick}
      className={cn('rounded-lg border p-2.5 cursor-pointer transition-colors', active ? 'border-foreground/50 bg-accent' : 'hover:bg-accent/50')}
    >
      <div className="flex items-center gap-2 text-xs text-muted-foreground mb-1">
        <span className="rounded-md border bg-background px-1.5 font-medium text-foreground tabular-nums">{source.n}</span>
        {source.kind === 'passage' ? <FileText className="size-3.5" /> : <TableProperties className="size-3.5" />}
        <span className="truncate">{source.fileName}</span>
        <span className="ml-auto shrink-0">{source.kind === 'passage' ? 'passage' : 'register'}</span>
      </div>
      <p className={cn('text-xs leading-relaxed whitespace-pre-line', !active && 'line-clamp-3')}>{source.text}</p>
    </li>
  )
}

/** The model's configured context length in tokens; undefined when "auto". */
function contextTokens(model: { settings?: Record<string, { controller_props?: { value?: unknown } }> } | null | undefined) {
  const value = Number(model?.settings?.ctx_len?.controller_props?.value)
  return Number.isFinite(value) && value > 0 ? value : undefined
}
