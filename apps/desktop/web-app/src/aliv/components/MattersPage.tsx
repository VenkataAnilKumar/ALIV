import { useCallback, useEffect, useState } from 'react'
import { Link, useNavigate } from '@tanstack/react-router'
import { Briefcase, CalendarClock, FileText, Info, Landmark, Plus, Scale, ShieldCheck } from 'lucide-react'
import { toast } from 'sonner'
import HeaderPage from '@/containers/HeaderPage'
import { Button } from '@/components/ui/button'
import { deadlines, type Deadline } from '../dates'
import type { Matter, MatterInput } from '../types'
import { errorMessage } from '../useMatter'
import { getVault } from '../vault'
import { MatterDialog } from './MatterDialog'
import { DaysPill, EmptyState, Pill, PolicyPill } from './primitives'

interface MatterCard {
  matter: Matter
  next: Deadline | null
}

export function MattersPage() {
  const navigate = useNavigate()
  const vault = getVault()
  const [cards, setCards] = useState<MatterCard[] | null>(null)
  const [dialogOpen, setDialogOpen] = useState(false)

  const load = useCallback(async () => {
    try {
      const matters = await vault.listMatters()
      const withDeadlines = await Promise.all(
        matters.map(async (matter) => {
          const upcoming = deadlines(await vault.reviewGrid(matter.id)).filter((d) => d.days >= 0)
          return { matter, next: upcoming[0] ?? null }
        })
      )
      setCards(withDeadlines)
    } catch (e) {
      toast.error(errorMessage(e))
      setCards([])
    }
  }, [vault])

  useEffect(() => {
    void load()
  }, [load])

  const create = async (input: MatterInput) => {
    try {
      const m = await vault.createMatter(input)
      setDialogOpen(false)
      navigate({ to: '/matters/$matterId', params: { matterId: m.id } })
    } catch (e) {
      toast.error(errorMessage(e))
    }
  }

  return (
    <div className="flex flex-col h-svh w-full">
      <HeaderPage>
        <div className="flex items-center justify-between w-full pr-2">
          <span className="text-sm text-muted-foreground">ALIV</span>
          <Button size="sm" onClick={() => setDialogOpen(true)}>
            <Plus className="size-4" /> New matter
          </Button>
        </div>
      </HeaderPage>

      <div className="h-full overflow-y-auto px-4 md:px-8 pb-10">
        <div className="mx-auto w-full max-w-5xl flex flex-col gap-6">
          <div className="pt-2">
            <h1 className="text-2xl font-semibold tracking-tight">Matters</h1>
            <p className="text-sm text-muted-foreground mt-1">
              Each matter is a private vault: documents are indexed on this computer, answers cite their sources, and file changes wait for your approval.
            </p>
          </div>

          {vault.mode === 'demo' && (
            <div className="flex items-start gap-3 rounded-xl border border-sky-500/25 bg-sky-500/5 p-3 text-sm">
              <Info className="size-4 mt-0.5 text-sky-600 shrink-0" />
              <div>
                <span className="font-medium">Browser preview.</span>{' '}
                <span className="text-muted-foreground">
                  This runs on the sample documents in <code>demo/corpus</code>, using the fields the desktop extractor produced. File actions are simulated. Open the desktop app to work on real folders.
                </span>
              </div>
            </div>
          )}

          {cards === null ? (
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {[0, 1, 2].map((i) => (
                <div key={i} className="h-40 rounded-xl border bg-muted/30 animate-pulse" />
              ))}
            </div>
          ) : cards.length === 0 ? (
            <div className="rounded-xl border bg-card">
              <EmptyState
                icon={<Briefcase className="size-6" />}
                title="No matters yet"
                body="Create a matter for a client engagement, then add the folder that holds its contracts, leases or invoices."
                action={
                  <Button onClick={() => setDialogOpen(true)}>
                    <Plus className="size-4" /> New matter
                  </Button>
                }
              />
            </div>
          ) : (
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {cards.map(({ matter, next }) => (
                <Link
                  key={matter.id}
                  to="/matters/$matterId"
                  params={{ matterId: matter.id }}
                  className="group rounded-xl border bg-card p-4 flex flex-col gap-3 hover:border-foreground/30 hover:shadow-sm transition-all"
                  data-testid="matter-card"
                >
                  <div className="flex items-start justify-between gap-2">
                    <div className="size-9 rounded-lg bg-muted flex items-center justify-center text-muted-foreground shrink-0">
                      {matter.kind === 'finance' ? <Landmark className="size-4" /> : <Scale className="size-4" />}
                    </div>
                    <PolicyPill policy={matter.policy} />
                  </div>
                  <div className="min-w-0">
                    <div className="font-medium truncate group-hover:underline underline-offset-4">{matter.name}</div>
                    <div className="text-xs text-muted-foreground truncate">{matter.client || 'No client set'}</div>
                  </div>
                  <div className="mt-auto flex flex-wrap items-center gap-x-3 gap-y-1.5 text-xs text-muted-foreground">
                    <span className="inline-flex items-center gap-1">
                      <FileText className="size-3.5" /> {matter.documentCount} documents
                    </span>
                    {matter.pendingActions > 0 && (
                      <Pill tone="warning">
                        <ShieldCheck className="size-3" /> {matter.pendingActions} to review
                      </Pill>
                    )}
                  </div>
                  {next ? (
                    <div className="flex items-center justify-between gap-2 rounded-lg bg-muted/50 px-2.5 py-1.5 text-xs">
                      <span className="inline-flex items-center gap-1.5 min-w-0">
                        <CalendarClock className="size-3.5 shrink-0" />
                        <span className="truncate">
                          {next.kind === 'renewal_deadline' ? 'Renewal' : 'Expiry'} · {next.row.document.fileName}
                        </span>
                      </span>
                      <DaysPill days={next.days} />
                    </div>
                  ) : (
                    <div className="rounded-lg bg-muted/50 px-2.5 py-1.5 text-xs text-muted-foreground">No upcoming deadlines</div>
                  )}
                </Link>
              ))}
            </div>
          )}
        </div>
      </div>

      <MatterDialog open={dialogOpen} onOpenChange={setDialogOpen} onSubmit={create} />
    </div>
  )
}
