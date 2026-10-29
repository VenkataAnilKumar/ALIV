import { useState } from 'react'
import { AlertTriangle, ArrowRight, CalendarClock, FileText, FolderPlus, MessageSquareText, ShieldCheck } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { deadlines } from '../dates'
import type { GridRow, Matter, VaultAction, VaultDocument } from '../types'
import type { MatterTab } from './MatterPage'
import { DaysPill, EmptyState, Pill, Section, StatCard } from './primitives'

const SUGGESTIONS: Record<Matter['kind'], string[]> = {
  legal: [
    'Which agreements expire or need a renewal notice in the next 90 days?',
    'Summarize the penalty and liquidated damages clauses.',
    'Which contracts can be terminated on notice, and how much notice?',
    'What governing law applies to each agreement?',
  ],
  finance: [
    'Which invoices are due in the next 30 days, and for how much?',
    'What late payment interest or fees apply?',
    'List every amount above $10,000 with its counterparty.',
    'Which agreements renew automatically?',
  ],
}

export function OverviewPanel({
  matter,
  grid,
  documents,
  actions,
  onAsk,
  onOpenTab,
  onAddDocuments,
}: {
  matter: Matter
  grid: GridRow[]
  documents: VaultDocument[]
  actions: VaultAction[]
  onAsk: (q: string) => void
  onOpenTab: (t: MatterTab) => void
  onAddDocuments: () => void
}) {
  const [q, setQ] = useState('')
  const all = deadlines(grid)
  const window90 = all.filter((d) => d.days >= 0 && d.days <= 90)
  const overdue = all.filter((d) => d.days < 0 && d.days >= -60)
  const skipped = documents.filter((d) => d.status !== 'indexed')
  const pending = actions.filter((a) => a.status === 'proposed' || a.status === 'approved').length

  if (!documents.length) {
    return (
      <Section>
        <EmptyState
          icon={<FolderPlus className="size-6" />}
          title="Add the matter's documents"
          body="Pick a folder of contracts, leases, invoices or emails. ALIV reads them on this computer, extracts key dates and amounts, and builds a searchable index. Nothing is uploaded."
          action={
            <Button onClick={onAddDocuments}>
              <FolderPlus className="size-4" /> Add folder
            </Button>
          }
        />
      </Section>
    )
  }

  return (
    <div className="flex flex-col gap-5">
      <div className="grid gap-3 grid-cols-2 lg:grid-cols-4">
        <StatCard label="Documents indexed" value={documents.length - skipped.length} hint={skipped.length ? `${skipped.length} need attention` : 'All readable'} icon={<FileText className="size-4" />} />
        <StatCard label="Deadlines in 90 days" value={window90.length} hint={window90[0] ? `Next: ${window90[0].date}` : 'Nothing coming up'} icon={<CalendarClock className="size-4" />} tone={window90.some((d) => d.days <= 30) ? 'warning' : undefined} />
        <StatCard label="Passed in last 60 days" value={overdue.length} hint="Expiry or renewal dates" icon={<AlertTriangle className="size-4" />} tone={overdue.length ? 'danger' : undefined} />
        <StatCard label="Awaiting approval" value={pending} hint="Proposed file changes" icon={<ShieldCheck className="size-4" />} />
      </div>

      <div className="grid gap-5 lg:grid-cols-5">
        <Section
          className="lg:col-span-3"
          title="Upcoming deadlines"
          description="Expiry, due and renewal-notice dates found in the documents, soonest first."
          actions={
            <Button size="xs" variant="ghost" onClick={() => onOpenTab('register')}>
              Open register <ArrowRight className="size-3" />
            </Button>
          }
        >
          {window90.length + overdue.length === 0 ? (
            <p className="px-4 pb-4 text-sm text-muted-foreground">No expiry or renewal dates in the last 60 or next 90 days.</p>
          ) : (
            <ul className="divide-y">
              {[...overdue, ...window90].map((d) => (
                <li key={`${d.row.document.id}-${d.kind}`} className="flex items-center gap-3 px-4 py-2.5">
                  <div className="w-24 shrink-0 text-xs tabular-nums text-muted-foreground">{d.date}</div>
                  <div className="min-w-0 flex-1">
                    <div className="text-sm truncate">{d.row.document.fileName}</div>
                    <div className="text-xs text-muted-foreground truncate">
                      {d.kind === 'renewal_deadline' ? 'Renewal notice deadline' : 'Expiry / due date'}
                      {(() => {
                        const parties = d.row.fields.find((f) => f.key === 'parties')?.value
                        return parties ? ` · ${parties}` : ''
                      })()}
                    </div>
                  </div>
                  <DaysPill days={d.days} />
                </li>
              ))}
            </ul>
          )}
        </Section>

        <Section className="lg:col-span-2" title="Ask this matter" description={`Answers cite passages. ${matter.policy === 'local_only' ? 'Local models only.' : 'Local or cloud models.'}`}>
          <form
            className="px-4 flex gap-2"
            onSubmit={(e) => {
              e.preventDefault()
              if (q.trim()) onAsk(q.trim())
            }}
          >
            <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Ask about these documents…" />
            <Button type="submit" size="icon" aria-label="Ask">
              <MessageSquareText className="size-4" />
            </Button>
          </form>
          <div className="p-4 flex flex-col gap-2">
            {SUGGESTIONS[matter.kind].map((s) => (
              <button
                key={s}
                onClick={() => onAsk(s)}
                className="text-left text-sm rounded-lg border px-3 py-2 hover:bg-accent transition-colors cursor-pointer"
              >
                {s}
              </button>
            ))}
          </div>
        </Section>
      </div>

      {skipped.length > 0 && (
        <Section title="Needs attention" description="These files could not be read and are not searchable yet.">
          <ul className="divide-y">
            {skipped.slice(0, 5).map((d) => (
              <li key={d.id} className="flex items-center justify-between gap-3 px-4 py-2 text-sm">
                <span className="truncate">{d.fileName}</span>
                <Pill tone="warning">{d.error ?? d.status}</Pill>
              </li>
            ))}
          </ul>
        </Section>
      )}
    </div>
  )
}
