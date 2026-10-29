import { useState } from 'react'
import { Cloud, Cpu, FileCog, FolderInput, Lock, ScrollText, ShieldCheck, ShieldX } from 'lucide-react'
import { Button } from '@/components/ui/button'
import type { AuditEntry, AuditVerification } from '../types'
import { errorMessage } from '../useMatter'
import { getVault } from '../vault'
import { EmptyState, Pill, Section, Segmented } from './primitives'

type Filter = 'all' | 'models' | 'files'

function icon(kind: string, detail: unknown) {
  if (kind === 'model_call_blocked') return <Lock className="size-4 text-red-600" />
  if (kind === 'model_call') {
    const loc = (detail as { location?: string })?.location
    return loc === 'cloud' ? <Cloud className="size-4 text-sky-600" /> : <Cpu className="size-4 text-emerald-600" />
  }
  if (kind === 'ingest') return <FolderInput className="size-4 text-muted-foreground" />
  if (kind.startsWith('action')) return <FileCog className="size-4 text-muted-foreground" />
  return <ScrollText className="size-4 text-muted-foreground" />
}

export function AuditPanel({ audit }: { audit: AuditEntry[] }) {
  const [filter, setFilter] = useState<Filter>('all')
  const [check, setCheck] = useState<AuditVerification | string | null>(null)
  const shown = audit.filter((a) =>
    filter === 'models' ? a.kind.startsWith('model_call') : filter === 'files' ? a.kind.startsWith('action') || a.kind === 'ingest' || a.kind === 'export' : true
  )
  const cloudCalls = audit.filter((a) => a.kind === 'model_call' && (a.detail as { location?: string })?.location === 'cloud').length

  const verify = async () => {
    try {
      setCheck(await getVault().verifyAudit())
    } catch (e) {
      setCheck(errorMessage(e))
    }
  }

  return (
    <Section
      title="Audit log"
      description={`Every ingestion, file change and model call, newest first. ${cloudCalls} cloud call(s) recorded for this matter.`}
      actions={
        <>
          {check &&
            (typeof check === 'string' ? (
              <Pill tone="danger">{check}</Pill>
            ) : check.ok ? (
              <Pill tone="success">
                <ShieldCheck className="size-3" /> Chain intact · {check.entries} entries
              </Pill>
            ) : (
              <Pill tone="danger">
                <ShieldX className="size-3" /> Tampering at entry {check.firstBrokenId}
              </Pill>
            ))}
          <Button size="sm" variant="outline" onClick={verify}>
            <ShieldCheck className="size-4" /> Verify integrity
          </Button>
        </>
      }
    >
      <div className="px-4 pb-3">
        <Segmented
          value={filter}
          onChange={setFilter}
          options={[
            { id: 'all', label: 'All' },
            { id: 'models', label: 'Model calls' },
            { id: 'files', label: 'Files' },
          ]}
        />
      </div>
      {shown.length === 0 ? (
        <EmptyState icon={<ScrollText className="size-6" />} title="Nothing logged yet" body="Entries appear as you add documents, ask questions and apply file changes." />
      ) : (
        <ol className="divide-y border-t">
          {shown.map((a) => (
            <li key={a.id} className="flex items-start gap-3 px-4 py-2.5">
              <div className="mt-0.5">{icon(a.kind, a.detail)}</div>
              <div className="min-w-0 flex-1">
                <div className="text-sm">{a.summary}</div>
                <div className="text-xs text-muted-foreground flex flex-wrap gap-x-3">
                  <span className="tabular-nums">{new Date(a.ts).toLocaleString()}</span>
                  <span>{a.kind.replace(/_/g, ' ')}</span>
                  <span className="font-mono" title={a.hash}>
                    #{a.hash.slice(0, 10)}
                  </span>
                </div>
              </div>
            </li>
          ))}
        </ol>
      )}
    </Section>
  )
}
