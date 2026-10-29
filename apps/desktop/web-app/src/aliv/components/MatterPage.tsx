import { useMemo, useState } from 'react'
import { Link, useNavigate } from '@tanstack/react-router'
import { ChevronLeft, FolderPlus, Landmark, Loader2, MoreHorizontal, Scale, Settings2, Trash2 } from 'lucide-react'
import { toast } from 'sonner'
import HeaderPage from '@/containers/HeaderPage'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useServiceHub } from '@/hooks/useServiceHub'
import type { MatterInput } from '../types'
import { errorMessage, useMatter } from '../useMatter'
import { getVault } from '../vault'
import { ApprovalsPanel } from './ApprovalsPanel'
import { AskPanel } from './AskPanel'
import { AuditPanel } from './AuditPanel'
import { DocumentsPanel } from './DocumentsPanel'
import { MatterDialog } from './MatterDialog'
import { OverviewPanel } from './OverviewPanel'
import { PolicyPill, Tabs, type TabDef } from './primitives'
import { RegisterPanel } from './RegisterPanel'

export type MatterTab = 'overview' | 'ask' | 'register' | 'documents' | 'approvals' | 'audit'

export function MatterPage({ matterId }: { matterId: string }) {
  const data = useMatter(matterId)
  const { matter, grid, documents, actions, audit, reload } = data
  const vault = getVault()
  const serviceHub = useServiceHub()
  const navigate = useNavigate()
  const [tab, setTab] = useState<MatterTab>('overview')
  const [question, setQuestion] = useState('')
  const [ingesting, setIngesting] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)

  const pending = actions.filter((a) => a.status === 'proposed' || a.status === 'approved').length
  const tabs: TabDef<MatterTab>[] = useMemo(
    () => [
      { id: 'overview', label: 'Overview' },
      { id: 'ask', label: 'Ask' },
      { id: 'register', label: 'Register' },
      { id: 'documents', label: 'Documents' },
      { id: 'approvals', label: 'Approvals', badge: pending },
      { id: 'audit', label: 'Audit log' },
    ],
    [pending]
  )

  const addDocuments = async () => {
    let paths: string[] = []
    if (vault.mode === 'desktop') {
      const picked = await serviceHub.dialog().open({ directory: true, multiple: true })
      if (!picked) return
      paths = Array.isArray(picked) ? picked : [picked]
    }
    setIngesting(true)
    const id = toast.loading('Indexing documents on this computer…')
    try {
      const r = await vault.ingestPaths(matterId, paths)
      const parts = [r.ocr ? `${r.indexed} indexed (${r.ocr} read by OCR)` : `${r.indexed} indexed`]
      if (r.unchanged) parts.push(`${r.unchanged} unchanged`)
      if (r.skipped) parts.push(`${r.skipped} skipped`)
      if (r.failed) parts.push(`${r.failed} failed`)
      toast.success(`Done: ${parts.join(', ')}`, { id, description: r.skipped ? 'Skipped files are listed under Documents with the reason.' : undefined })
      await reload()
    } catch (e) {
      toast.error(errorMessage(e), { id })
    } finally {
      setIngesting(false)
    }
  }

  const saveSettings = async (input: MatterInput) => {
    try {
      await vault.updateMatter(matterId, input)
      setSettingsOpen(false)
      await reload()
      toast.success('Matter updated')
    } catch (e) {
      toast.error(errorMessage(e))
    }
  }

  const remove = async () => {
    if (!matter) return
    if (!window.confirm(`Delete "${matter.name}"? The index and history are removed; your files on disk are not touched.`)) return
    try {
      await vault.deleteMatter(matterId)
      navigate({ to: '/matters' })
    } catch (e) {
      toast.error(errorMessage(e))
    }
  }

  const askAbout = (q: string) => {
    setQuestion(q)
    setTab('ask')
  }

  return (
    <div className="flex flex-col h-svh w-full">
      <HeaderPage>
        <div className="flex items-center justify-between w-full pr-2 gap-2">
          <Link to="/matters" className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground">
            <ChevronLeft className="size-4" /> Matters
          </Link>
          {matter && (
            <div className="flex items-center gap-2">
              <Button size="sm" variant="outline" onClick={addDocuments} disabled={ingesting}>
                {ingesting ? <Loader2 className="size-4 animate-spin" /> : <FolderPlus className="size-4" />}
                {vault.mode === 'demo' ? 'Load demo documents' : 'Add folder'}
              </Button>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="ghost" size="icon-sm" aria-label="Matter options">
                    <MoreHorizontal className="size-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem onSelect={() => setSettingsOpen(true)}>
                    <Settings2 className="size-4" /> Matter settings
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem variant="destructive" onSelect={remove}>
                    <Trash2 className="size-4" /> Delete matter
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          )}
        </div>
      </HeaderPage>

      <div className="h-full overflow-y-auto px-4 md:px-8 pb-10">
        <div className="mx-auto w-full max-w-6xl flex flex-col gap-5">
          {data.loading && !matter ? (
            <div className="h-24 rounded-xl border bg-muted/30 animate-pulse mt-2" />
          ) : !matter ? (
            <div className="pt-8 text-sm text-muted-foreground">{data.error}</div>
          ) : (
            <>
              <div className="flex flex-wrap items-end justify-between gap-3 pt-2">
                <div className="min-w-0">
                  <div className="flex items-center gap-2 text-xs text-muted-foreground mb-1">
                    {matter.kind === 'finance' ? <Landmark className="size-3.5" /> : <Scale className="size-3.5" />}
                    {matter.kind === 'finance' ? 'Finance' : 'Legal'} matter
                    {matter.client && <span>· {matter.client}</span>}
                  </div>
                  <h1 className="text-2xl font-semibold tracking-tight truncate">{matter.name}</h1>
                </div>
                <button onClick={() => setSettingsOpen(true)} className="cursor-pointer" title="Change model policy">
                  <PolicyPill policy={matter.policy} />
                </button>
              </div>

              <Tabs tabs={tabs} value={tab} onChange={setTab} />

              {tab === 'overview' && (
                <OverviewPanel
                  matter={matter}
                  grid={grid}
                  documents={documents}
                  actions={actions}
                  onAsk={askAbout}
                  onOpenTab={setTab}
                  onAddDocuments={addDocuments}
                />
              )}
              {tab === 'ask' && <AskPanel matter={matter} initialQuestion={question} onAsked={reload} hasDocuments={documents.length > 0} />}
              {tab === 'register' && <RegisterPanel matter={matter} grid={grid} />}
              {tab === 'documents' && <DocumentsPanel documents={documents} onAddDocuments={addDocuments} />}
              {tab === 'approvals' && <ApprovalsPanel matter={matter} actions={actions} onChanged={reload} />}
              {tab === 'audit' && <AuditPanel audit={audit} />}
            </>
          )}
        </div>
      </div>

      <MatterDialog open={settingsOpen} onOpenChange={setSettingsOpen} initial={matter} onSubmit={saveSettings} />
    </div>
  )
}
