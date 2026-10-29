import { ExternalLink, FileText, FolderPlus } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { useServiceHub } from '@/hooks/useServiceHub'
import type { VaultDocument } from '../types'
import { errorMessage } from '../useMatter'
import { getVault } from '../vault'
import { EmptyState, Pill, Section } from './primitives'

function size(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

export function DocumentsPanel({ documents, onAddDocuments }: { documents: VaultDocument[]; onAddDocuments: () => void }) {
  const serviceHub = useServiceHub()
  const desktop = getVault().mode === 'desktop'

  const open = async (path: string) => {
    try {
      await serviceHub.opener().openPath(path)
    } catch (e) {
      toast.error(errorMessage(e))
    }
  }

  if (!documents.length) {
    return (
      <Section>
        <EmptyState
          icon={<FileText className="size-6" />}
          title="No documents yet"
          body="Supported: PDF (text), Word, Excel, PowerPoint, HTML, plain text and .eml emails. Scanned images are listed but need OCR, which is not available yet."
          action={
            <Button onClick={onAddDocuments}>
              <FolderPlus className="size-4" /> Add folder
            </Button>
          }
        />
      </Section>
    )
  }

  const indexed = documents.filter((d) => d.status === 'indexed').length
  return (
    <Section
      title="Documents"
      description={`${indexed} indexed · ${documents.length - indexed} not readable. Re-adding a folder only re-reads files that changed.`}
      actions={
        <Button size="sm" variant="outline" onClick={onAddDocuments}>
          <FolderPlus className="size-4" /> Add folder
        </Button>
      }
    >
      <div className="overflow-x-auto border-t">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-xs text-muted-foreground bg-muted/30">
              <th className="px-4 py-2 font-medium">Name</th>
              <th className="px-3 py-2 font-medium">Type</th>
              <th className="px-3 py-2 font-medium">Status</th>
              <th className="px-3 py-2 font-medium text-right">Passages</th>
              <th className="px-3 py-2 font-medium text-right">Size</th>
              <th className="px-3 py-2" />
            </tr>
          </thead>
          <tbody className="divide-y">
            {documents.map((d) => (
              <tr key={d.id} className="hover:bg-muted/20">
                <td className="px-4 py-2">
                  <div className="font-medium truncate max-w-80">{d.fileName}</div>
                  <div className="text-xs text-muted-foreground truncate max-w-80" title={d.path}>
                    {d.path}
                  </div>
                </td>
                <td className="px-3 py-2 whitespace-nowrap">{d.docType ?? <span className="text-muted-foreground">—</span>}</td>
                <td className="px-3 py-2">
                  {d.status === 'indexed' ? (
                    <Pill tone="success">indexed</Pill>
                  ) : (
                    <Pill tone="warning" title={d.error ?? undefined}>
                      {d.error ?? d.status}
                    </Pill>
                  )}
                </td>
                <td className="px-3 py-2 text-right tabular-nums">{d.chunkCount}</td>
                <td className="px-3 py-2 text-right tabular-nums whitespace-nowrap">{size(d.size)}</td>
                <td className="px-3 py-2 text-right">
                  {desktop && (
                    <Button size="icon-xs" variant="ghost" onClick={() => open(d.path)} aria-label={`Open ${d.fileName}`}>
                      <ExternalLink className="size-3.5" />
                    </Button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </Section>
  )
}
