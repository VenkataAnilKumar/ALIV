import { useState } from 'react'
import { ActivitySquare, HardDriveDownload, RefreshCcw, ShieldCheck } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { PageHeader } from '@/components/ui/PageHeader'
import { StatCard } from '@/components/ui/StatCard'
import { parseDiagnosticsReport } from '@/lib/parsers'
import { getDiagnosticsReport, openLogDirectory } from '@/lib/tauri'
import { downloadTextFile, formatDateTime } from '@/lib/utils'
import type { DiagnosticsReport } from '@/types/contracts'

export function DiagnosticsPage() {
  const [raw, setRaw] = useState('')
  const [report, setReport] = useState<DiagnosticsReport | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const generateReport = async () => {
    setLoading(true)
    setError(null)
    try {
      const payload = await getDiagnosticsReport()
      const parsed = parseDiagnosticsReport(payload)
      if (!parsed) {
        throw new Error('Diagnostics payload was not valid JSON for the current UI contract.')
      }
      setRaw(payload)
      setReport(parsed)
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : String(nextError))
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="space-y-6">
      <PageHeader
        eyebrow="Runtime"
        title="Diagnostics"
        description="Generate a release-support bundle snapshot with runtime counters, database inventory, Ollama reachability, and the latest local log tail."
        actions={
          <>
            <Button variant="secondary" onClick={() => downloadTextFile('aliv-diagnostics.json', raw, 'application/json;charset=utf-8')} disabled={!raw}>
              <HardDriveDownload className="h-4 w-4" />
              Export to file
            </Button>
            <Button
              variant="secondary"
              onClick={() => void openLogDirectory().catch((nextError) => {
                setError(nextError instanceof Error ? nextError.message : String(nextError))
              })}
            >
              Open log folder
            </Button>
            <Button variant="primary" onClick={() => void generateReport()} disabled={loading}>
              <RefreshCcw className="h-4 w-4" />
              {loading ? 'Generating...' : 'Generate Report'}
            </Button>
          </>
        }
      />

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
        <StatCard label="ALIV Version" value={report?.aliv_version ?? '0.1.0'} detail={report ? formatDateTime(report.generated_at) : 'Generate a fresh report'} />
        <StatCard label="Platform" value={report ? `${report.platform.os}/${report.platform.arch}` : 'unknown'} detail="Captured from the bundled runtime" />
        <StatCard label="Files" value={report?.db.files ?? 0} detail={`workspaces ${report?.db.workspace_folders ?? 0}`} />
        <StatCard label="Chunks" value={report?.db.chunks ?? 0} detail={`embeddings ${report?.db.embeddings ?? 0}`} />
        <StatCard label="Memory Facts" value={report?.db.memory_facts ?? 0} detail={`recommendations ${report?.db.action_recommendations ?? 0}`} />
      </div>

      <div className="grid gap-6 xl:grid-cols-[360px_1fr]">
        <section className="panel p-5">
          <div className="flex items-center gap-2 text-sm font-medium text-slate-100">
            <ShieldCheck className="h-4 w-4 text-cyan-200" />
            Health summary
          </div>
          <div className="mt-4 space-y-3 text-sm text-slate-300/82">
            <div className="flex items-center justify-between rounded-2xl border border-white/8 bg-slate-950/45 px-4 py-3">
              <span>Ollama</span>
              <span className={report?.ollama.reachable ? 'text-emerald-200' : 'text-orange-200'}>
                {report ? (report.ollama.reachable ? 'Reachable' : 'Unavailable') : 'Unknown'}
              </span>
            </div>
            <div className="flex items-center justify-between rounded-2xl border border-white/8 bg-slate-950/45 px-4 py-3">
              <span>Commands observed</span>
              <span>{report?.runtime.command_count ?? 0}</span>
            </div>
            <div className="flex items-center justify-between rounded-2xl border border-white/8 bg-slate-950/45 px-4 py-3">
              <span>Preview queue</span>
              <span>{report?.runtime.preview_queue_count ?? 0}</span>
            </div>
            <div className="flex items-center justify-between rounded-2xl border border-white/8 bg-slate-950/45 px-4 py-3">
              <span>Benchmark runs</span>
              <span>{report?.db.benchmark_runs ?? 0}</span>
            </div>
            <div className="rounded-2xl border border-white/8 bg-slate-950/45 px-4 py-4">
              <div className="text-xs uppercase tracking-[0.14em] text-slate-400">Log directory</div>
              <div className="mt-2 break-all text-sm text-slate-200">{report?.logs.directory ?? 'Not generated yet.'}</div>
              <div className="mt-3 text-xs text-slate-400">Active file: {report?.logs.active_file ?? 'No log file yet'}</div>
            </div>
          </div>
          {error ? <div className="mt-4 rounded-2xl border border-red-400/20 bg-red-500/8 px-4 py-3 text-sm text-red-200">{error}</div> : null}
        </section>

        <section className="space-y-6">
          <div className="panel p-5">
            <div className="flex items-center gap-2 text-sm font-medium text-slate-100">
              <ActivitySquare className="h-4 w-4 text-orange-200" />
              Recent log tail
            </div>
            <pre className="mt-4 max-h-[280px] overflow-auto rounded-2xl border border-white/8 bg-slate-950/70 p-4 font-mono text-xs leading-6 text-slate-200">
              {report?.logs.tail ?? 'Generate a diagnostics report to inspect the latest local log lines.'}
            </pre>
          </div>

          <div className="panel p-5">
            <div className="text-sm font-medium text-slate-100">Raw JSON</div>
            <pre className="mt-4 max-h-[420px] overflow-auto rounded-2xl border border-white/8 bg-slate-950/70 p-4 font-mono text-xs leading-6 text-slate-200">
              {raw || 'No diagnostics report generated yet.'}
            </pre>
          </div>
        </section>
      </div>
    </div>
  )
}