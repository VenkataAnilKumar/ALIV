import { useEffect } from 'react'
import { AlertTriangle, DatabaseZap, LoaderCircle, Play, RefreshCw } from 'lucide-react'
import { Badge } from '@/components/ui/Badge'
import { Button } from '@/components/ui/Button'
import { PageHeader } from '@/components/ui/PageHeader'
import { ProgressRing } from '@/components/ui/ProgressRing'
import { StatCard } from '@/components/ui/StatCard'
import { Table } from '@/components/ui/Table'
import { formatDateTime, formatNumber } from '@/lib/utils'
import { useIngestionStore } from '@/stores/ingestionStore'

export function IngestionPage() {
  const { telemetry, history, isRunning, error, run, refresh } = useIngestionStore()

  const progressValue = telemetry?.discovered_total
    ? (telemetry.indexed_total / Math.max(telemetry.discovered_total, 1)) * 100
    : 0
  const parserConfidencePercent = Math.round((telemetry?.avg_parser_confidence ?? 0) * 100)

  useEffect(() => {
    void refresh()
  }, [refresh])

  useEffect(() => {
    const onPrimary = () => {
      void run(true)
    }
    window.addEventListener('aliv:primary-action', onPrimary)
    return () => window.removeEventListener('aliv:primary-action', onPrimary)
  }, [run])

  return (
    <div className="space-y-6">
      <PageHeader
        eyebrow="Pipeline"
        title="Ingestion"
        description="Run incremental or full ingestion, then inspect parse, chunk, and embedding telemetry without leaving the desktop flow."
        actions={
          <>
            <Button variant="secondary" onClick={() => void refresh()}>
              <RefreshCw className="h-4 w-4" />
              Refresh telemetry
            </Button>
            <Button variant="secondary" onClick={() => void run(false)} disabled={isRunning}>
              <DatabaseZap className="h-4 w-4" />
              Full rebuild
            </Button>
            <Button variant="primary" onClick={() => void run(true)} disabled={isRunning}>
              {isRunning ? <LoaderCircle className="h-4 w-4 animate-spin" /> : <Play className="h-4 w-4" />}
              Incremental ingest
            </Button>
          </>
        }
      />

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
        <StatCard label="Runs" value={formatNumber(telemetry?.runs ?? 0)} detail={`Last mode ${telemetry?.last_mode ?? 'n/a'}`} />
        <StatCard label="Discovered" value={formatNumber(telemetry?.discovered_total ?? 0)} detail={`Indexed ${formatNumber(telemetry?.indexed_total ?? 0)}`} />
        <StatCard label="Parsed" value={formatNumber(telemetry?.parsed_total ?? 0)} detail={`Chunks ${formatNumber(telemetry?.chunks ?? 0)}`} />
        <StatCard label="Embeddings" value={formatNumber(telemetry?.embeddings ?? 0)} detail={`Skipped ${formatNumber(telemetry?.skipped_total ?? 0)}`} />
        <StatCard label="Failures" value={formatNumber(telemetry?.failed_total ?? 0)} detail={`Parser avg ${parserConfidencePercent}%`} />
      </div>

      {isRunning ? (
        <div className="rounded-xl border border-indigo-500/20 bg-indigo-500/8 px-4 py-3 text-sm text-indigo-200">
          Ingestion is running. The UI will refresh telemetry when the current command completes.
        </div>
      ) : null}

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <StatCard label="Current Workspace" value={telemetry?.last_workspace ?? 'n/a'} detail="Most recent ingestion target" className="xl:col-span-2" />
        <StatCard label="Last Run" value={telemetry?.last_run ?? 'n/a'} detail={formatDateTime(telemetry?.last_created_at)} />
        <StatCard label="Parser Confidence" value={`${parserConfidencePercent}%`} detail="Average confidence across parse results" />
      </div>

      <div className="grid gap-6 xl:grid-cols-[320px_1fr]">
        <section className="panel flex flex-col items-center justify-center p-6">
          <ProgressRing value={progressValue} label="Indexed coverage" />
          <div className="mt-4 text-center text-sm text-zinc-400">
            <div>
              Workspace <span className="font-mono text-zinc-200">{telemetry?.last_workspace ?? 'n/a'}</span>
            </div>
            <div className="mt-2">
              {formatNumber(telemetry?.indexed_total ?? 0)} indexed of {formatNumber(telemetry?.discovered_total ?? 0)} discovered files
            </div>
          </div>
        </section>

        <section className="panel p-5">
          <div className="mb-4 flex items-center justify-between gap-3">
            <div>
              <div className="text-sm font-medium text-zinc-100">Run summary</div>
              <div className="mt-1 text-sm text-zinc-500">Recent local runs captured by the desktop UI.</div>
            </div>
            <Badge tone={telemetry?.failed_total ? 'warning' : 'success'}>
              {telemetry?.failed_total ? 'attention needed' : 'healthy'}
            </Badge>
          </div>

          <Table
            columns={[
              {
                key: 'mode',
                header: 'Mode',
                render: (row) => <Badge tone={row.mode === 'full' ? 'info' : 'pending'}>{row.mode}</Badge>,
              },
              {
                key: 'status',
                header: 'Status',
                render: (row) => <Badge tone={row.status === 'ok' ? 'success' : 'warning'}>{row.status}</Badge>,
              },
              { key: 'detail', header: 'Detail', render: (row) => <span className="text-sm text-zinc-300">{row.detail}</span> },
              { key: 'timestamp', header: 'Started', render: (row) => <span className="font-mono text-[11px] text-zinc-500">{formatDateTime(row.timestamp)}</span> },
            ]}
            rows={history}
            rowKey={(row) => row.id}
            emptyTitle="No UI-triggered runs yet"
            emptyDescription="Run a full or incremental ingestion pass to populate local run history."
          />
        </section>
      </div>

      <section className="panel p-5">
        <div className="mb-4 flex items-center gap-2 text-sm font-medium text-zinc-100">
          <AlertTriangle className="h-4 w-4 text-amber-300" />
          Failure and skip overview
        </div>

        <div className="grid gap-4 md:grid-cols-3">
          <div className="rounded-xl border border-zinc-800 bg-zinc-950/50 p-4">
            <div className="text-[11px] uppercase tracking-[0.08em] text-zinc-500">Skipped Files</div>
            <div className="mt-2 text-xl font-semibold text-zinc-100">{formatNumber(telemetry?.skipped_total ?? 0)}</div>
            <div className="mt-2 text-sm text-zinc-500">Files unchanged during incremental scans.</div>
          </div>
          <div className="rounded-xl border border-zinc-800 bg-zinc-950/50 p-4">
            <div className="text-[11px] uppercase tracking-[0.08em] text-zinc-500">Failed Files</div>
            <div className="mt-2 text-xl font-semibold text-zinc-100">{formatNumber(telemetry?.failed_total ?? 0)}</div>
            <div className="mt-2 text-sm text-zinc-500">Files that did not parse or embed successfully in recorded runs.</div>
          </div>
          <div className="rounded-xl border border-zinc-800 bg-zinc-950/50 p-4">
            <div className="text-[11px] uppercase tracking-[0.08em] text-zinc-500">Parsed Records</div>
            <div className="mt-2 text-xl font-semibold text-zinc-100">{formatNumber(telemetry?.parsed ?? 0)}</div>
            <div className="mt-2 text-sm text-zinc-500">Current parse results stored in the local database.</div>
          </div>
        </div>
      </section>

      {error ? <div className="rounded-lg border border-red-500/20 bg-red-500/5 px-4 py-3 text-sm text-red-300">{error}</div> : null}
    </div>
  )
}