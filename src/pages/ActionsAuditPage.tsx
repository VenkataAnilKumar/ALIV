import { useEffect, useMemo, useState } from 'react'
import { History, RotateCcw } from 'lucide-react'
import { Badge } from '@/components/ui/Badge'
import { Button } from '@/components/ui/Button'
import { PageHeader } from '@/components/ui/PageHeader'
import { StatCard } from '@/components/ui/StatCard'
import { Table } from '@/components/ui/Table'
import { downloadTextFile, formatDateTime, toCsv } from '@/lib/utils'
import { useActionsStore } from '@/stores/actionsStore'

export function ActionsAuditPage() {
  const { audit, error, loadAudit, undoBatch } = useActionsStore()
  const [statusFilter, setStatusFilter] = useState<'all' | 'success' | 'failed' | 'rolled_back'>('all')
  const [batchFilter, setBatchFilter] = useState('')

  useEffect(() => {
    void loadAudit()
  }, [loadAudit])

  useEffect(() => {
    const onPrimary = () => {
      void loadAudit()
    }
    window.addEventListener('aliv:primary-action', onPrimary)
    return () => window.removeEventListener('aliv:primary-action', onPrimary)
  }, [loadAudit])

  const uniqueBatches = [...new Set((audit?.entries ?? []).map((entry) => entry.batch_id))]
  const filteredEntries = useMemo(() => (audit?.entries ?? []).filter((entry) => {
    const statusMatches = statusFilter === 'all' || entry.status === statusFilter
    const batchMatches = !batchFilter.trim() || entry.batch_id.toLowerCase().includes(batchFilter.toLowerCase())
    return statusMatches && batchMatches
  }), [audit?.entries, batchFilter, statusFilter])
  const successCount = filteredEntries.filter((entry) => entry.status === 'success').length
  const failedCount = filteredEntries.filter((entry) => entry.status === 'failed').length
  const rollbackCount = filteredEntries.filter((entry) => entry.status === 'rolled_back').length

  return (
    <div className="space-y-6">
      <PageHeader
        eyebrow="Accountability"
        title="Action Audit"
        description="Inspect executed batches, confirm file-level status, and reverse a specific execution batch when necessary."
        actions={
          <>
            <Button variant="secondary" onClick={() => downloadTextFile('action-audit.csv', toCsv(filteredEntries.map((entry) => ({
              batch_id: entry.batch_id,
              source_path: entry.source_path,
              proposed_path: entry.proposed_path,
              recommendation_type: entry.recommendation_type,
              status: entry.status,
              executed_at: entry.executed_at,
            }))), 'text/csv;charset=utf-8')} disabled={!filteredEntries.length}>
              Export CSV
            </Button>
            <Button variant="secondary" onClick={() => void loadAudit()}><History className="h-4 w-4" />Refresh audit</Button>
          </>
        }
      />

      <div className="grid gap-4 md:grid-cols-4">
        <StatCard label="Visible Entries" value={filteredEntries.length} detail="After active filters" />
        <StatCard label="Success" value={successCount} detail="Completed actions" />
        <StatCard label="Failed" value={failedCount} detail="Needs review" />
        <StatCard label="Rolled Back" value={rollbackCount} detail="Undo completed" />
      </div>

      <section className="panel p-5">
        <div className="grid gap-4 md:grid-cols-[180px_1fr]">
          <select className="input-base w-full" title="Audit status filter" aria-label="Audit status filter" value={statusFilter} onChange={(event) => setStatusFilter(event.target.value as typeof statusFilter)}>
            <option value="all">All statuses</option>
            <option value="success">Success</option>
            <option value="failed">Failed</option>
            <option value="rolled_back">Rolled Back</option>
          </select>
          <input className="input-base w-full" value={batchFilter} onChange={(event) => setBatchFilter(event.target.value)} placeholder="Filter by batch id" />
        </div>
      </section>

      <section className="panel p-5">
        <div className="text-sm font-medium text-zinc-100">Undo by batch</div>
        <div className="mt-4 flex flex-wrap gap-3">
          {uniqueBatches.length === 0 ? (
            <div className="text-sm text-zinc-500">No execution batches recorded yet.</div>
          ) : (
            uniqueBatches.map((batchId) => (
              <Button key={batchId} variant="secondary" onClick={() => void undoBatch(batchId)}>
                <RotateCcw className="h-4 w-4" />
                {batchId}
              </Button>
            ))
          )}
        </div>
      </section>

      <Table
        columns={[
          { key: 'batch', header: 'Batch', render: (row) => <span className="font-mono text-[11px] text-zinc-300">{row.batch_id}</span> },
          { key: 'source', header: 'Source', render: (row) => <div className="font-mono text-[11px] text-zinc-300">{row.source_path}</div> },
          { key: 'target', header: 'Target', render: (row) => <div className="font-mono text-[11px] text-zinc-500">{row.proposed_path}</div> },
          { key: 'type', header: 'Type', render: (row) => row.recommendation_type },
          {
            key: 'status',
            header: 'Status',
            render: (row) => <Badge tone={row.status === 'success' ? 'success' : row.status === 'rolled_back' ? 'pending' : 'danger'}>{row.status}</Badge>,
          },
          { key: 'executed', header: 'Executed', render: (row) => formatDateTime(row.executed_at) },
        ]}
        rows={filteredEntries}
        error={error}
        rowKey={(row) => row.execution_id}
        rowClassName={(row) => (row.status === 'failed' ? 'bg-red-500/5' : row.status === 'rolled_back' ? 'bg-zinc-500/5' : 'bg-emerald-500/5')}
        emptyTitle="No audit entries"
        emptyDescription="Executed actions will appear here once a batch runs."
      />
    </div>
  )
}