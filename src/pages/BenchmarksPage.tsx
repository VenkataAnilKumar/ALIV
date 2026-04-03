import { useEffect, useState } from 'react'
import { ActivitySquare, FlaskConical, Gauge } from 'lucide-react'
import { Badge } from '@/components/ui/Badge'
import { Button } from '@/components/ui/Button'
import { PageHeader } from '@/components/ui/PageHeader'
import { StatCard } from '@/components/ui/StatCard'
import { Table } from '@/components/ui/Table'
import { parseBenchmarkReport } from '@/lib/parsers'
import { getBenchmarkRunMetrics, runBenchmarkSuite } from '@/lib/tauri'
import { downloadTextFile, formatDateTime } from '@/lib/utils'
import type { BenchmarkMetricSummary, BenchmarkSuiteReport, CommandAck } from '@/types/contracts'

export function BenchmarksPage() {
  const [corpusManifest, setCorpusManifest] = useState('benchmarks/labels/corpus_manifest.json')
  const [labelsVersion, setLabelsVersion] = useState('questions_gold.v1')
  const [runId, setRunId] = useState('latest')
  const [runAck, setRunAck] = useState<CommandAck | null>(null)
  const [metrics, setMetrics] = useState<BenchmarkMetricSummary | null>(null)
  const [report, setReport] = useState<BenchmarkSuiteReport | null>(null)
  const [loadingRun, setLoadingRun] = useState(false)
  const [loadingMetrics, setLoadingMetrics] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const latestRun = report && report.runs.length > 0 ? report.runs[report.runs.length - 1] : null
  const previousRun = report && report.runs.length > 1 ? report.runs[report.runs.length - 2] : null

  const execute = async () => {
    setLoadingRun(true)
    setError(null)
    try {
      const ack = await runBenchmarkSuite({ corpus_manifest: corpusManifest, labels_version: labelsVersion })
      setRunAck(ack)
      const parsed = parseBenchmarkReport(ack.detail)
      if (parsed) {
        setReport(parsed)
        setRunId(parsed.result_files?.stamped ?? parsed.result_files?.latest ?? 'latest')
        setMetrics({
          run_id: parsed.result_files?.stamped ?? 'latest',
          status: ack.status,
          detail: ack.detail,
        })
      }
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : String(nextError))
    } finally {
      setLoadingRun(false)
    }
  }

  const fetchMetrics = async () => {
    if (!runId.trim()) return
    setLoadingMetrics(true)
    setError(null)
    try {
      const ack = await getBenchmarkRunMetrics({ run_id: runId })
      setMetrics({ run_id: runId, status: ack.status, detail: ack.detail })
      const parsed = parseBenchmarkReport(ack.detail)
      if (parsed) setReport(parsed)
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : String(nextError))
    } finally {
      setLoadingMetrics(false)
    }
  }

  useEffect(() => {
    const onPrimary = () => {
      void execute()
    }
    window.addEventListener('aliv:primary-action', onPrimary)
    return () => window.removeEventListener('aliv:primary-action', onPrimary)
  }, [corpusManifest, labelsVersion])

  return (
    <div className="space-y-6">
      <PageHeader
        eyebrow="Verification"
        title="Benchmarks"
        description="Kick off corpus-level benchmark suites and inspect the stored run detail from the desktop control plane."
        actions={
          <>
            <Button variant="secondary" onClick={() => downloadTextFile('benchmark-report.json', JSON.stringify(report ?? {}, null, 2), 'application/json;charset=utf-8')} disabled={!report}>
              Export JSON
            </Button>
            <Button variant="secondary" onClick={() => void fetchMetrics()} disabled={!runId.trim() || loadingMetrics}>
              <Gauge className="h-4 w-4" />
              Load metrics
            </Button>
            <Button variant="primary" onClick={() => void execute()} disabled={loadingRun}>
              <FlaskConical className="h-4 w-4" />
              {loadingRun ? 'Running suite...' : 'Run suite'}
            </Button>
          </>
        }
      />

      <div className="grid gap-4 md:grid-cols-3">
        <StatCard label="Run Status" value={runAck?.status ?? 'idle'} detail={runAck?.command ?? 'run_benchmark_suite'} />
        <StatCard label="Metrics Status" value={metrics?.status ?? 'unloaded'} detail={loadingMetrics ? 'Fetching...' : 'Ready when a run id is available'} />
        <StatCard label="Run ID" value={runId || 'n/a'} detail="Latest report path or alias" />
      </div>

      {report ? (
        <div className="flex items-center justify-between rounded-xl border border-zinc-800 bg-zinc-950/50 px-4 py-3">
          <div className="text-sm text-zinc-300">{report.phase4_exit.reason}</div>
          <Badge tone={report.phase4_exit.pass ? 'success' : 'danger'}>
            Phase 4 Gate: {report.phase4_exit.pass ? 'PASS' : 'FAIL'}
          </Badge>
        </div>
      ) : null}

      {latestRun ? (
        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
          <StatCard label="Ingestion" value={latestRun.metrics.ingestion.status.toUpperCase()} detail={`checks ${latestRun.metrics.ingestion.checks.length}`} />
          <StatCard label="Retrieval" value={`${Math.round(latestRun.metrics.retrieval.actual * 100)}%`} detail={`threshold ${Math.round(latestRun.metrics.retrieval.threshold * 100)}%`} />
          <StatCard label="Extraction" value={`${Math.round(latestRun.metrics.extraction.actual * 100)}%`} detail={`threshold ${Math.round(latestRun.metrics.extraction.threshold * 100)}%`} />
          <StatCard label="P95 Latency" value={`${latestRun.metrics.retrieval.query_latency_p95_sec.toFixed(4)}s`} detail={`limit ${latestRun.metrics.retrieval.query_latency_p95_threshold_sec.toFixed(1)}s`} />
          <StatCard label="Safety Defects" value={latestRun.suite.checks.no_open_critical_safety_defects ? '0' : 'open'} detail={latestRun.suite.status} />
        </div>
      ) : null}

      <div className="grid gap-6 xl:grid-cols-[420px_1fr]">
        <section className="panel p-5">
          <div className="flex items-center gap-2 text-sm font-medium text-zinc-100">
            <ActivitySquare className="h-4 w-4 text-indigo-300" />
            Benchmark inputs
          </div>
          <div className="mt-4 space-y-4">
            <div>
              <label className="mb-2 block text-sm text-zinc-400">Corpus manifest</label>
              <input className="input-base w-full" title="Corpus manifest path" aria-label="Corpus manifest path" placeholder="benchmarks/labels/corpus_manifest.json" value={corpusManifest} onChange={(event) => setCorpusManifest(event.target.value)} />
            </div>
            <div>
              <label className="mb-2 block text-sm text-zinc-400">Labels version</label>
              <input className="input-base w-full" title="Labels version" aria-label="Labels version" placeholder="questions_gold.v1" value={labelsVersion} onChange={(event) => setLabelsVersion(event.target.value)} />
            </div>
            <div>
              <label className="mb-2 block text-sm text-zinc-400">Run id</label>
              <input className="input-base w-full" value={runId} onChange={(event) => setRunId(event.target.value)} placeholder="Paste an existing benchmark run id" />
            </div>
            {error ? <div className="rounded-lg border border-red-500/20 bg-red-500/5 px-4 py-3 text-sm text-red-300">{error}</div> : null}
          </div>
        </section>

        <section className="space-y-6">
          <div className="panel p-5">
            <div className="mb-4 flex items-center justify-between gap-3">
              <div className="text-sm font-medium text-zinc-100">Run history</div>
              <Badge tone={report?.status === 'pass' ? 'success' : report ? 'danger' : 'pending'}>{report?.status ?? 'unloaded'}</Badge>
            </div>
            <Table
              columns={[
                { key: 'run_index', header: 'Run', render: (row) => row.run_index },
                { key: 'timestamp', header: 'Timestamp', render: (row) => formatDateTime(row.timestamp_utc) },
                { key: 'ingestion', header: 'Ingestion', render: (row) => <Badge tone={row.metrics.ingestion.status === 'pass' ? 'success' : 'danger'}>{row.metrics.ingestion.status}</Badge> },
                { key: 'retrieval', header: 'Retrieval', render: (row) => `${Math.round(row.metrics.retrieval.actual * 100)}%` },
                { key: 'extraction', header: 'Extraction', render: (row) => `${Math.round(row.metrics.extraction.actual * 100)}%` },
                { key: 'suite', header: 'Suite', render: (row) => <Badge tone={row.suite.status === 'pass' ? 'success' : 'danger'}>{row.suite.status}</Badge> },
              ]}
              rows={report?.runs ?? []}
              rowKey={(row) => `${row.run_index}-${row.timestamp_utc}`}
              emptyTitle="No benchmark report loaded"
              emptyDescription="Run the suite or load the latest stored metrics report."
            />
          </div>

          <div className="grid gap-6 lg:grid-cols-2">
            <div className="panel p-5">
              <div className="text-sm font-medium text-zinc-100">Latest run comparison</div>
              {latestRun ? (
                <div className="mt-4 space-y-3 text-sm text-zinc-300">
                  <div className="flex items-center justify-between rounded-xl border border-zinc-800 bg-zinc-950/50 px-4 py-3">
                    <span>Retrieval top-5 delta</span>
                    <span className="font-mono">{latestRun.trend?.retrieval_top5_delta ?? 0}</span>
                  </div>
                  <div className="flex items-center justify-between rounded-xl border border-zinc-800 bg-zinc-950/50 px-4 py-3">
                    <span>Extraction precision delta</span>
                    <span className="font-mono">{latestRun.trend?.extraction_precision_delta ?? 0}</span>
                  </div>
                  <div className="flex items-center justify-between rounded-xl border border-zinc-800 bg-zinc-950/50 px-4 py-3">
                    <span>P95 latency delta</span>
                    <span className="font-mono">{latestRun.trend?.query_latency_p95_delta_sec ?? 0}s</span>
                  </div>
                  {previousRun ? (
                    <div className="rounded-xl border border-zinc-800 bg-zinc-950/50 px-4 py-3 text-xs text-zinc-500">
                      Comparing run {latestRun.run_index} against run {previousRun.run_index}.
                    </div>
                  ) : null}
                </div>
              ) : (
                <div className="mt-4 rounded-xl border border-dashed border-zinc-800 px-4 py-8 text-sm text-zinc-500">
                  Benchmark trend deltas appear after loading a stored report.
                </div>
              )}
            </div>

            <div className="panel p-5">
              <div className="text-sm font-medium text-zinc-100">Run acknowledgement</div>
              <pre className="mt-4 overflow-x-auto rounded-xl border border-zinc-800 bg-zinc-950/70 p-4 font-mono text-xs leading-6 text-zinc-300">
                {runAck ? JSON.stringify(runAck, null, 2) : 'No benchmark suite launched from the UI yet.'}
              </pre>
            </div>
          </div>

          <div className="panel p-5">
            <div className="text-sm font-medium text-zinc-100">Metrics detail</div>
            <pre className="mt-4 overflow-x-auto rounded-xl border border-zinc-800 bg-zinc-950/70 p-4 font-mono text-xs leading-6 text-zinc-300">
              {metrics ? JSON.stringify(metrics, null, 2) : 'No metrics loaded yet.'}
            </pre>
          </div>
        </section>
      </div>
    </div>
  )
}