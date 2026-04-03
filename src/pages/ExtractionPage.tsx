import { useEffect, useMemo, useState } from 'react'
import { Download, Filter, Sparkles, WandSparkles } from 'lucide-react'
import { Badge } from '@/components/ui/Badge'
import { Button } from '@/components/ui/Button'
import { PageHeader } from '@/components/ui/PageHeader'
import { Table } from '@/components/ui/Table'
import { downloadTextFile, toCsv } from '@/lib/utils'
import { parseExtractionPayload, parseExtractionRows } from '@/lib/parsers'
import { runExtraction } from '@/lib/tauri'
import type { CommandAck, ExtractionFieldRow } from '@/types/contracts'

const TEMPLATE_OPTIONS = [
  'Lease summary',
  'Contract Key Terms',
  'Invoice Fields',
  'Custom',
]

export function ExtractionPage() {
  const [query, setQuery] = useState('Extract counterparty name, effective date, and renewal mechanics.')
  const [template, setTemplate] = useState('Lease summary')
  const [rows, setRows] = useState<ExtractionFieldRow[]>([])
  const [raw, setRaw] = useState<CommandAck | null>(null)
  const [selectedField, setSelectedField] = useState<string | null>(null)
  const [confidenceFilter, setConfidenceFilter] = useState<'all' | '90' | '70'>('all')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const payload = useMemo(() => parseExtractionPayload(raw?.detail ?? ''), [raw?.detail])
  const filteredRows = useMemo(() => {
    if (confidenceFilter === 'all') return rows
    const threshold = confidenceFilter === '90' ? 0.9 : 0.7
    return rows.filter((row) => (row.confidence ?? 0) >= threshold)
  }, [confidenceFilter, rows])
  const activeRow = filteredRows.find((row) => row.field === selectedField) ?? filteredRows[0]

  const execute = async () => {
    if (!query.trim() || !template.trim()) {
      setError('Provide both a natural language extraction request and a template.')
      return
    }
    setLoading(true)
    setError(null)
    try {
      const ack = await runExtraction({ query, template })
      setRaw(ack)
      const parsedRows = parseExtractionRows(ack.detail)
      setRows(parsedRows)
      setSelectedField(parsedRows[0]?.field ?? null)
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : String(nextError))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    const onPrimary = () => {
      void execute()
    }
    window.addEventListener('aliv:primary-action', onPrimary)
    return () => window.removeEventListener('aliv:primary-action', onPrimary)
  }, [query, template])

  return (
    <div className="space-y-6">
      <PageHeader
        eyebrow="Structured Output"
        title="Extraction"
        description="Run a templated extraction pass and inspect the returned structured payload before persisting or acting on it."
        actions={
          <>
            <Button
              variant="secondary"
              onClick={() => downloadTextFile('extraction-result.json', JSON.stringify(payload ?? {}, null, 2), 'application/json;charset=utf-8')}
              disabled={!raw}
            >
              <Download className="h-4 w-4" />
              Export JSON
            </Button>
            <Button
              variant="secondary"
              onClick={() => downloadTextFile('extraction-fields.csv', toCsv(filteredRows.map((row) => ({
                field: row.field,
                value: row.value,
                confidence: row.confidence == null ? '' : row.confidence,
              }))), 'text/csv;charset=utf-8')}
              disabled={!filteredRows.length}
            >
              <Download className="h-4 w-4" />
              Export CSV
            </Button>
            <Button variant="primary" onClick={() => void execute()} disabled={loading}>
              <WandSparkles className="h-4 w-4" />
              {loading ? 'Extracting...' : 'Run extraction'}
            </Button>
          </>
        }
      />

      <div className="grid gap-6 xl:grid-cols-[420px_1fr]">
        <section className="panel p-5">
          <div className="text-sm font-medium text-zinc-100">Extraction inputs</div>
          <div className="mt-4 space-y-4">
            <div>
              <label className="mb-2 block text-sm text-zinc-400">Instruction</label>
              <textarea className="input-base min-h-[200px] w-full resize-none leading-6" title="Extraction instruction" aria-label="Extraction instruction" placeholder="Extract the key obligations, dates, and parties from the selected documents." value={query} onChange={(event) => setQuery(event.target.value)} />
            </div>
            <div>
              <label className="mb-2 block text-sm text-zinc-400">Template</label>
              <select className="input-base w-full" title="Extraction template" aria-label="Extraction template" value={TEMPLATE_OPTIONS.includes(template) ? template : 'Custom'} onChange={(event) => setTemplate(event.target.value)}>
                {TEMPLATE_OPTIONS.map((option) => (
                  <option key={option} value={option}>{option}</option>
                ))}
              </select>
            </div>
            {(template === 'Custom' || !TEMPLATE_OPTIONS.includes(template)) && (
              <div>
                <label className="mb-2 block text-sm text-zinc-400">Custom template label</label>
                <input className="input-base w-full" title="Custom template label" aria-label="Custom template label" placeholder="Custom clause extraction" value={template === 'Custom' ? '' : template} onChange={(event) => setTemplate(event.target.value)} />
              </div>
            )}
            {error ? <div className="rounded-lg border border-red-500/20 bg-red-500/5 px-4 py-3 text-sm text-red-300">{error}</div> : null}
          </div>
        </section>

        <section className="space-y-6">
          <div className="panel p-5">
            <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
              <div className="flex items-center gap-2 text-sm font-medium text-zinc-100">
                <Sparkles className="h-4 w-4 text-indigo-300" />
                Parsed fields
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <Badge tone="info">hits {payload?.retrieval_hits ?? 0}</Badge>
                <div className="inline-flex items-center gap-2 rounded-lg border border-zinc-800 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-400">
                  <Filter className="h-3.5 w-3.5" />
                  <select className="bg-transparent outline-none" title="Confidence filter" aria-label="Confidence filter" value={confidenceFilter} onChange={(event) => setConfidenceFilter(event.target.value as typeof confidenceFilter)}>
                    <option value="all">All</option>
                    <option value="90">Above 90%</option>
                    <option value="70">Above 70%</option>
                  </select>
                </div>
              </div>
            </div>
            <Table
              columns={[
                {
                  key: 'field',
                  header: 'Field',
                  render: (row) => (
                    <button type="button" className="text-left font-medium text-zinc-100" onClick={() => setSelectedField(row.field)}>
                      {row.field}
                    </button>
                  ),
                },
                { key: 'value', header: 'Value', render: (row) => <span className="text-zinc-300">{row.value}</span> },
                {
                  key: 'confidence',
                  header: 'Confidence',
                  render: (row) => (row.confidence == null ? 'n/a' : `${Math.round(row.confidence * 100)}%`),
                },
                { key: 'citations', header: 'Citations', render: (row) => row.citations.length },
              ]}
              rows={filteredRows}
              loading={loading}
              error={error}
              rowKey={(row) => row.field}
              rowClassName={(row) => (row.field === activeRow?.field ? 'bg-indigo-500/8' : undefined)}
              emptyTitle="No extraction rows yet"
              emptyDescription="Run extraction to parse the returned JSON detail into field rows."
            />
          </div>

          <div className="grid gap-6 lg:grid-cols-[1.05fr_0.95fr]">
            <div className="panel p-5">
              <div className="text-sm font-medium text-zinc-100">Citation panel</div>
              <div className="mt-4 space-y-3">
                {activeRow?.citations.length ? (
                  activeRow.citations.map((citation, index) => (
                    <pre key={index} className="overflow-x-auto rounded-xl border border-zinc-800 bg-zinc-950/70 p-4 font-mono text-xs leading-6 text-zinc-300">
                      {JSON.stringify(citation, null, 2)}
                    </pre>
                  ))
                ) : (
                  <div className="rounded-xl border border-dashed border-zinc-800 px-4 py-8 text-sm text-zinc-500">
                    Select a parsed field to inspect the associated citations.
                  </div>
                )}
              </div>
            </div>

            <div className="panel p-5">
              <div className="text-sm font-medium text-zinc-100">Raw backend detail</div>
              <pre className="mt-4 overflow-x-auto rounded-xl border border-zinc-800 bg-zinc-950/70 p-4 font-mono text-xs leading-6 text-zinc-300">
                {raw?.detail ?? 'No extraction has been run yet.'}
              </pre>
            </div>
          </div>
        </section>
      </div>
    </div>
  )
}