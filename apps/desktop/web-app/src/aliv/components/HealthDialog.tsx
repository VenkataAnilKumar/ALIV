import { useEffect, useState } from 'react'
import { Copy, Loader2 } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import type { VaultDiagnostics } from '../types'
import { errorMessage } from '../useMatter'
import { getVault } from '../vault'
import { Pill } from './primitives'

/**
 * The vault's health report: what it holds, what it could not read and why,
 * the OCR tools it found and whether the audit chain verifies. Counts and
 * reasons only, so the copied report can go to support without client data.
 */
export function HealthDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }) {
  const [report, setReport] = useState<VaultDiagnostics | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!open) return
    setReport(null)
    setError(null)
    getVault()
      .diagnostics()
      .then(setReport)
      .catch((e) => setError(errorMessage(e)))
  }, [open])

  const copy = async () => {
    if (!report) return
    try {
      await navigator.clipboard.writeText(JSON.stringify(report, null, 2))
      toast.success('Health report copied. It holds counts and reasons only, no document text.')
    } catch (e) {
      toast.error(errorMessage(e))
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Vault health</DialogTitle>
          <DialogDescription>What this computer's vault holds, and anything that needs attention.</DialogDescription>
        </DialogHeader>
        {error ? (
          <p className="text-sm text-destructive">{error}</p>
        ) : !report ? (
          <Loader2 className="size-5 animate-spin text-muted-foreground" />
        ) : (
          <div className="flex flex-col gap-3 text-sm">
            <dl className="grid grid-cols-2 gap-x-4 gap-y-1.5">
              <Row label="Matters" value={report.matters} />
              <Row label="Documents indexed" value={`${report.indexed} of ${report.documents}`} />
              <Row label="Read by OCR" value={report.ocrDocuments} />
              <Row label="Meaning index" value={`${report.embedded} of ${report.passages} passages`} />
              <Row label="Database" value={`${(report.databaseBytes / 1048576).toFixed(1)} MB`} />
              <Row label="Vault version" value={report.version} />
            </dl>
            <div className="flex flex-wrap gap-2">
              {report.audit.ok ? (
                <Pill tone="success">Audit chain verified · {report.audit.entries} entries</Pill>
              ) : (
                <Pill tone="danger">Audit chain broken at entry {report.audit.firstBrokenId}</Pill>
              )}
              <Pill tone={report.tesseract ? 'success' : 'warning'} title={report.tesseract ?? undefined}>
                {report.tesseract ? 'OCR available' : 'OCR not installed'}
              </Pill>
              {report.tesseract && !report.pdftoppm && <Pill tone="warning">Scanned PDFs need Poppler</Pill>}
            </div>
            {report.unreadable.length > 0 && (
              <div>
                <div className="font-medium mb-1">Not readable</div>
                <ul className="text-xs text-muted-foreground flex flex-col gap-0.5">
                  {report.unreadable.map((r) => (
                    <li key={r.reason}>
                      {r.count} × {r.reason}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        )}
        <DialogFooter>
          <Button variant="outline" onClick={copy} disabled={!report}>
            <Copy className="size-4" /> Copy report
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function Row({ label, value }: { label: string; value: string | number }) {
  return (
    <>
      <dt className="text-muted-foreground">{label}</dt>
      <dd className="tabular-nums text-right">{value}</dd>
    </>
  )
}
