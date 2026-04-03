import { useEffect, useMemo, useState } from 'react'
import { AlertTriangle, CheckCheck, Play, RefreshCw, ShieldAlert } from 'lucide-react'
import { Badge } from '@/components/ui/Badge'
import { Button } from '@/components/ui/Button'
import { Modal } from '@/components/ui/Modal'
import { PageHeader } from '@/components/ui/PageHeader'
import { Table } from '@/components/ui/Table'
import { ConfidenceBar } from '@/components/ui/ConfidenceBar'
import { formatDateTime, riskTone } from '@/lib/utils'
import { useActionsStore } from '@/stores/actionsStore'

export function ActionsPreviewPage() {
  const {
    preview,
    selectedIds,
    approved,
    execution,
    undoBanner,
    error,
    loading,
    loadRecommendations,
    evaluateRisk,
    toggleSelected,
    selectLowRisk,
    approveSelection,
    executeApproved,
    dismissUndoBanner,
  } = useActionsStore()
  const [confirmOpen, setConfirmOpen] = useState(false)

  const highRiskCount = useMemo(
    () => preview?.items.filter((item) => item.risk_level === 'high').length ?? 0,
    [preview?.items],
  )

  useEffect(() => {
    const onPrimary = () => {
      if (!preview) {
        void loadRecommendations()
        return
      }
      if (!approved) {
        void approveSelection()
        return
      }
      setConfirmOpen(true)
    }
    window.addEventListener('aliv:primary-action', onPrimary)
    return () => window.removeEventListener('aliv:primary-action', onPrimary)
  }, [approveSelection, approved, loadRecommendations, preview])

  return (
    <div className="space-y-6">
      <PageHeader
        eyebrow="Guardrails"
        title="Safe File Actions"
        description="Review rename recommendations, inspect risk signals, approve a bounded batch, and only then execute against the workspace."
        actions={
          <>
            <Button variant="secondary" onClick={() => void loadRecommendations()}>
              <RefreshCw className="h-4 w-4" />
              Generate preview
            </Button>
            <Button variant="secondary" onClick={() => void evaluateRisk()} disabled={!preview?.items.length}>
              <ShieldAlert className="h-4 w-4" />
              Re-evaluate risk
            </Button>
            <Button variant="primary" onClick={() => setConfirmOpen(true)} disabled={!approved}>
              <Play className="h-4 w-4" />
              Execute approved
            </Button>
          </>
        }
      />

      <div className="grid gap-4 md:grid-cols-4">
        <div className="panel p-4">
          <div className="text-[11px] uppercase tracking-[0.08em] text-zinc-500">Recommendations</div>
          <div className="mt-3 text-2xl font-semibold text-zinc-50">{preview?.total ?? 0}</div>
        </div>
        <div className="panel p-4">
          <div className="text-[11px] uppercase tracking-[0.08em] text-zinc-500">Selected</div>
          <div className="mt-3 text-2xl font-semibold text-zinc-50">{selectedIds.length}</div>
        </div>
        <div className="panel p-4">
          <div className="text-[11px] uppercase tracking-[0.08em] text-zinc-500">High Risk</div>
          <div className="mt-3 text-2xl font-semibold text-zinc-50">{highRiskCount}</div>
        </div>
        <div className="panel p-4">
          <div className="text-[11px] uppercase tracking-[0.08em] text-zinc-500">Approval</div>
          <div className="mt-3 text-2xl font-semibold text-zinc-50">{approved ? 'Ready' : 'Pending'}</div>
        </div>
      </div>

      {undoBanner ? (
        <div className="flex items-center justify-between rounded-xl border border-emerald-500/20 bg-emerald-500/8 px-4 py-3 text-sm text-emerald-300">
          <span>
            Executed batch <span className="font-mono">{undoBanner.batchId}</span> with {undoBanner.count} successful actions.
          </span>
          <Button variant="ghost" onClick={dismissUndoBanner}>Dismiss</Button>
        </div>
      ) : null}

      {execution?.errors.length ? (
        <div className="rounded-xl border border-red-500/20 bg-red-500/8 px-4 py-3 text-sm text-red-300">{execution.errors.join('; ')}</div>
      ) : null}
      {error ? <div className="rounded-xl border border-red-500/20 bg-red-500/8 px-4 py-3 text-sm text-red-300">{error}</div> : null}

      <section className="panel p-5">
        <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
          <div>
            <div className="text-sm font-medium text-zinc-100">Preview table</div>
            <div className="mt-1 text-sm text-zinc-500">High-risk items remain unselected until you choose them explicitly.</div>
          </div>
          <div className="flex gap-3">
            <Button variant="secondary" onClick={selectLowRisk} disabled={!preview?.items.length}>
              <CheckCheck className="h-4 w-4" />
              Select low-risk
            </Button>
            <Button variant="secondary" onClick={() => void approveSelection()} disabled={selectedIds.length === 0}>
              Approve selection
            </Button>
          </div>
        </div>

        <Table
          columns={[
            {
              key: 'selected',
              header: '',
              className: 'w-12',
              render: (row) => (
                <input type="checkbox" title={`Select action ${row.id}`} aria-label={`Select action ${row.id}`} checked={selectedIds.includes(row.id)} onChange={() => toggleSelected(row.id)} />
              ),
            },
            {
              key: 'source',
              header: 'Source',
              render: (row) => (
                <div>
                  <div className="font-mono text-xs text-zinc-200">{row.source_path}</div>
                  <div className="mt-1 font-mono text-[11px] text-zinc-500">{row.proposed_path}</div>
                </div>
              ),
            },
            {
              key: 'risk',
              header: 'Risk',
              render: (row) => (
                <div className="space-y-2">
                  <Badge tone={riskTone(row.risk_level)}>{row.risk_level}</Badge>
                  <div className="font-mono text-[11px] text-zinc-500">{row.risk_flags.join(', ') || 'none'}</div>
                </div>
              ),
            },
            {
              key: 'confidence',
              header: 'Confidence',
              render: (row) => (
                <div className="min-w-[140px]">
                  <div className="mb-2 text-xs text-zinc-400">{Math.round(row.confidence * 100)}%</div>
                  <ConfidenceBar value={row.confidence} />
                </div>
              ),
            },
            {
              key: 'evidence',
              header: 'Evidence',
              render: (row) => (
                <div className="text-sm text-zinc-300">{row.evidence.join(' | ') || 'No evidence'}</div>
              ),
            },
            {
              key: 'created_at',
              header: 'Created',
              render: (row) => <span className="font-mono text-[11px] text-zinc-500">{formatDateTime(row.created_at)}</span>,
            },
          ]}
          rows={preview?.items ?? []}
          loading={loading}
          error={error}
          rowKey={(row) => row.id}
          rowClassName={(row) => (row.risk_level === 'high' ? 'bg-red-500/5' : undefined)}
          emptyTitle="No recommendations loaded"
          emptyDescription="Generate a preview to review suggested safe file actions."
        />
      </section>

      <Modal
        open={confirmOpen}
        title="Execute approved actions"
        description="This will run only the currently approved batch and create undo records for each successful operation."
        confirmLabel="Execute now"
        onClose={() => setConfirmOpen(false)}
        onConfirm={() => {
          setConfirmOpen(false)
          void executeApproved()
        }}
      >
        <div className="rounded-xl border border-amber-500/20 bg-amber-500/8 px-4 py-3 text-sm text-amber-200">
          <div className="flex items-start gap-2">
            <AlertTriangle className="mt-0.5 h-4 w-4" />
            <span>
              Selected {selectedIds.length} items. High-risk rows should only be included when the evidence and path change are defensible.
            </span>
          </div>
        </div>
      </Modal>
    </div>
  )
}