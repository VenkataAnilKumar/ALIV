import { useMemo, useState } from 'react'
import { Download, Search, TableProperties } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { useServiceHub } from '@/hooks/useServiceHub'
import { cn } from '@/lib/utils'
import { daysUntil } from '../dates'
import { FIELD_LABELS, type Field, type FieldKey, type GridRow, type Matter } from '../types'
import { registerCsv } from '../csv'
import { errorMessage } from '../useMatter'
import { getVault } from '../vault'
import { Confidence, DaysPill, EmptyState, Section, Segmented } from './primitives'

type Filter = 'all' | 'next90' | 'passed' | 'review'

const COLUMNS: FieldKey[] = [
  'doc_type',
  'parties',
  'effective_date',
  'expiry_date',
  'renewal_deadline',
  'amount',
  'notice_period',
  'governing_law',
  'penalty',
]

const NOWRAP = new Set<FieldKey>(['effective_date', 'expiry_date', 'renewal_deadline', 'amount', 'notice_period'])
const LOW_CONFIDENCE = 0.75
/** Fields a reviewer must be able to rely on; penalty text is always a quote to read. */
const KEY_FIELDS = new Set<FieldKey>(['doc_type', 'parties', 'effective_date', 'expiry_date', 'renewal_deadline', 'amount'])
const needsReview = (r: GridRow) => r.fields.some((f) => KEY_FIELDS.has(f.key) && f.confidence < LOW_CONFIDENCE)

function nextDays(row: GridRow): number | null {
  const ds = row.fields
    .filter((f) => f.key === 'expiry_date' || f.key === 'renewal_deadline')
    .map((f) => daysUntil(f.value))
    .filter((d): d is number => d !== null)
  const future = ds.filter((d) => d >= 0)
  if (future.length) return Math.min(...future)
  return ds.length ? Math.max(...ds) : null
}

export function RegisterPanel({ matter, grid }: { matter: Matter; grid: GridRow[] }) {
  const vault = getVault()
  const serviceHub = useServiceHub()
  const [filter, setFilter] = useState<Filter>('all')
  const [query, setQuery] = useState('')

  const counts = useMemo(() => {
    const c = { all: grid.length, next90: 0, passed: 0, review: 0 }
    for (const r of grid) {
      const d = nextDays(r)
      if (d !== null && d >= 0 && d <= 90) c.next90++
      if (d !== null && d < 0) c.passed++
      if (needsReview(r)) c.review++
    }
    return c
  }, [grid])

  const rows = useMemo(() => {
    const q = query.trim().toLowerCase()
    return grid
      .filter((r) => {
        const d = nextDays(r)
        if (filter === 'next90' && !(d !== null && d >= 0 && d <= 90)) return false
        if (filter === 'passed' && !(d !== null && d < 0)) return false
        if (filter === 'review' && !needsReview(r)) return false
        if (!q) return true
        return (r.document.fileName + ' ' + r.fields.map((f) => f.value).join(' ')).toLowerCase().includes(q)
      })
      .sort((a, b) => (nextDays(a) ?? 1e9) - (nextDays(b) ?? 1e9))
  }, [grid, filter, query])

  const exportCsv = async () => {
    const name = `${matter.name.replace(/[^\p{L}\p{N}]+/gu, '_')}_register.csv`
    try {
      if (vault.mode === 'desktop') {
        const dest = await serviceHub.dialog().save({ defaultPath: name, filters: [{ name: 'CSV', extensions: ['csv'] }] })
        if (!dest) return
        const n = await vault.exportRegisterCsv(matter.id, dest)
        toast.success(`Exported ${n} rows`, { description: dest })
      } else {
        const url = URL.createObjectURL(new Blob([registerCsv(rows)], { type: 'text/csv' }))
        const a = Object.assign(document.createElement('a'), { href: url, download: name })
        a.click()
        URL.revokeObjectURL(url)
      }
    } catch (e) {
      toast.error(errorMessage(e))
    }
  }

  if (!grid.length) {
    return (
      <Section>
        <EmptyState icon={<TableProperties className="size-6" />} title="The register fills itself" body="Once documents are indexed, their type, parties, key dates, amounts, notice periods and penalty clauses appear here, each linked to the passage it was read from." />
      </Section>
    )
  }

  return (
    <Section
      title="Register"
      description="Fields read from each document by rules, with the passage they came from. Click a value to check it."
      actions={
        <>
          <div className="relative">
            <Search className="size-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" />
            <Input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Filter…" className="h-8 w-44 pl-8 text-sm" />
          </div>
          <Button size="sm" variant="outline" onClick={exportCsv}>
            <Download className="size-4" /> Export CSV
          </Button>
        </>
      }
    >
      <div className="px-4 pb-3">
        <Segmented
          value={filter}
          onChange={setFilter}
          options={[
            { id: 'all', label: 'All', count: counts.all },
            { id: 'next90', label: 'Due in 90 days', count: counts.next90 },
            { id: 'passed', label: 'Passed', count: counts.passed },
            { id: 'review', label: 'Needs review', count: counts.review },
          ]}
        />
      </div>
      <div className="overflow-x-auto border-t">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-xs text-muted-foreground bg-muted/30">
              <th className="px-4 py-2 font-medium sticky left-0 bg-card min-w-52">Document</th>
              {COLUMNS.map((k) => (
                <th key={k} className={cn('px-3 py-2 font-medium whitespace-nowrap', k === 'penalty' && 'min-w-72', k === 'parties' && 'min-w-56')}>
                  {FIELD_LABELS[k]}
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="divide-y">
            {rows.map((r) => (
              <tr key={r.document.id} className="align-top hover:bg-muted/20">
                <td className="px-4 py-2.5 sticky left-0 bg-card">
                  <div className="font-medium truncate max-w-64" title={r.document.path}>
                    {r.document.fileName}
                  </div>
                  <DaysPill days={nextDays(r)} className="mt-1" />
                </td>
                {COLUMNS.map((k) => (
                  <td key={k} className={cn('px-3 py-2.5', NOWRAP.has(k) && 'whitespace-nowrap')}>
                    <FieldCell field={r.fields.find((f) => f.key === k)} fileName={r.document.fileName} isDate={k === 'expiry_date' || k === 'renewal_deadline'} />
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
        {rows.length === 0 && <p className="px-4 py-6 text-sm text-muted-foreground">No documents match this filter.</p>}
      </div>
    </Section>
  )
}

function FieldCell({ field, fileName, isDate }: { field?: Field; fileName: string; isDate: boolean }) {
  if (!field) return <span className="text-muted-foreground/50">—</span>
  const low = KEY_FIELDS.has(field.key) && field.confidence < LOW_CONFIDENCE
  const days = isDate ? daysUntil(field.value) : null
  return (
    <Popover>
      <PopoverTrigger asChild>
        <button
          className={cn(
            'text-left rounded-md -mx-1 px-1 py-0.5 hover:bg-accent cursor-pointer',
            field.key === 'penalty' && 'line-clamp-2',
            low && 'underline decoration-amber-500 decoration-dotted underline-offset-4',
            days !== null && days < 0 && 'text-muted-foreground',
            days !== null && days >= 0 && days <= 30 && 'text-amber-700 dark:text-amber-300 font-medium'
          )}
          title={low ? 'Low confidence: check the source' : undefined}
        >
          {field.value}
        </button>
      </PopoverTrigger>
      <PopoverContent className="w-96" align="start">
        <div className="flex items-center justify-between gap-2 mb-2">
          <div className="text-xs font-medium">{FIELD_LABELS[field.key]}</div>
          <Confidence value={field.confidence} />
        </div>
        <div className="text-sm font-medium mb-2">{field.value}</div>
        <div className="text-xs text-muted-foreground mb-1">
          Source · {fileName}
          {field.chunkOrdinal !== null && ` · passage ${field.chunkOrdinal + 1}`}
        </div>
        <blockquote className="text-xs leading-relaxed border-l-2 pl-2 text-foreground/80">{field.snippet}</blockquote>
      </PopoverContent>
    </Popover>
  )
}
