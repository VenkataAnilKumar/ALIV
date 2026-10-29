import { useState } from 'react'
import { ArrowRight, Check, Loader2, Play, RotateCcw, ShieldCheck, Sparkles, X } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import type { Matter, VaultAction } from '../types'
import { errorMessage } from '../useMatter'
import { getVault } from '../vault'
import { ActionStatusPill, EmptyState, fileBase, RiskPill, Section } from './primitives'

export function ApprovalsPanel({ matter, actions, onChanged }: { matter: Matter; actions: VaultAction[]; onChanged: () => Promise<void> }) {
  const vault = getVault()
  const [busy, setBusy] = useState<string | null>(null)

  const run = async (key: string, fn: () => Promise<unknown>, success?: string) => {
    setBusy(key)
    try {
      await fn()
      if (success) toast.success(success)
      await onChanged()
    } catch (e) {
      toast.error(errorMessage(e))
    } finally {
      setBusy(null)
    }
  }

  const open = actions.filter((a) => a.status === 'proposed' || a.status === 'approved')
  const approved = open.filter((a) => a.status === 'approved')
  const history = actions.filter((a) => a.status !== 'proposed' && a.status !== 'approved')

  const propose = () =>
    run('propose', async () => {
      const proposed = await vault.proposeRenames(matter.id)
      toast.success(proposed.length ? `${proposed.length} rename(s) proposed. Nothing changes until you approve.` : 'All files already follow the naming convention.')
    })

  const approveLowRisk = () =>
    run('approve-low', () => Promise.all(open.filter((a) => a.status === 'proposed' && a.risk === 'low').map((a) => vault.setActionStatus(a.id, 'approved'))))

  const execute = () => {
    if (!window.confirm(`Rename ${approved.length} file(s) on disk now? Each one can be undone from the history below.`)) return
    void run('execute', async () => {
      const done = await vault.executeApproved(matter.id)
      const failed = done.filter((a) => a.status === 'failed')
      if (failed.length) toast.error(`${failed.length} rename(s) failed`, { description: failed[0].error ?? undefined })
      else toast.success(`Renamed ${done.length} file(s)`)
    })
  }

  return (
    <div className="flex flex-col gap-5">
      <Section
        title="Proposed file changes"
        description="ALIV suggests names in the form Type_Counterparty_Date. Review each one: nothing on disk changes until you approve it and apply."
        actions={
          <>
            <Button size="sm" variant="outline" onClick={propose} disabled={busy !== null}>
              {busy === 'propose' ? <Loader2 className="size-4 animate-spin" /> : <Sparkles className="size-4" />}
              Suggest file names
            </Button>
            {open.some((a) => a.status === 'proposed' && a.risk === 'low') && (
              <Button size="sm" variant="outline" onClick={approveLowRisk} disabled={busy !== null}>
                <Check className="size-4" /> Approve all low risk
              </Button>
            )}
            <Button size="sm" onClick={execute} disabled={!approved.length || busy !== null}>
              {busy === 'execute' ? <Loader2 className="size-4 animate-spin" /> : <Play className="size-4" />}
              Apply {approved.length || ''} approved
            </Button>
          </>
        }
      >
        {open.length === 0 ? (
          <EmptyState icon={<ShieldCheck className="size-6" />} title="Nothing waiting for approval" body="Ask for suggestions to tidy file names. Medium and high risk proposals are those with missing or uncertain fields." />
        ) : (
          <ul className="divide-y border-t">
            {open.map((a) => (
              <li key={a.id} className="flex flex-wrap items-center gap-3 px-4 py-3">
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-2 text-sm">
                    <span className="truncate text-muted-foreground line-through decoration-muted-foreground/40">{fileBase(a.fromPath)}</span>
                    <ArrowRight className="size-3.5 text-muted-foreground shrink-0" />
                    <span className="truncate font-medium">{fileBase(a.toPath)}</span>
                  </div>
                  <div className="text-xs text-muted-foreground mt-0.5">{a.reason}</div>
                </div>
                <RiskPill risk={a.risk} />
                {a.status === 'approved' ? (
                  <>
                    <ActionStatusPill status="approved" />
                    <Button size="xs" variant="ghost" onClick={() => run(a.id, () => vault.setActionStatus(a.id, 'proposed'))} disabled={busy !== null}>
                      Unapprove
                    </Button>
                  </>
                ) : (
                  <div className="flex gap-1">
                    <Button size="xs" variant="outline" onClick={() => run(a.id, () => vault.setActionStatus(a.id, 'approved'))} disabled={busy !== null}>
                      <Check className="size-3" /> Approve
                    </Button>
                    <Button size="xs" variant="ghost" onClick={() => run(a.id, () => vault.setActionStatus(a.id, 'rejected'))} disabled={busy !== null}>
                      <X className="size-3" /> Reject
                    </Button>
                  </div>
                )}
              </li>
            ))}
          </ul>
        )}
      </Section>

      {history.length > 0 && (
        <Section title="History" description="Applied changes can be undone as long as the file has not been moved since.">
          <ul className="divide-y border-t">
            {history.map((a) => (
              <li key={a.id} className="flex flex-wrap items-center gap-3 px-4 py-2.5 text-sm">
                <div className="min-w-0 flex-1 flex flex-wrap items-center gap-2">
                  <span className="truncate text-muted-foreground">{fileBase(a.fromPath)}</span>
                  <ArrowRight className="size-3.5 text-muted-foreground shrink-0" />
                  <span className="truncate">{fileBase(a.toPath)}</span>
                  {a.error && <span className="text-xs text-red-600">{a.error}</span>}
                </div>
                <ActionStatusPill status={a.status} />
                {a.status === 'executed' && (
                  <Button size="xs" variant="ghost" onClick={() => run(a.id, () => vault.undoAction(a.id), 'Rename undone')} disabled={busy !== null}>
                    <RotateCcw className="size-3" /> Undo
                  </Button>
                )}
              </li>
            ))}
          </ul>
        </Section>
      )}
    </div>
  )
}
