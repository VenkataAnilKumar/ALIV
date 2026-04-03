import { Command, FolderOpen, ServerCog } from 'lucide-react'
import { Badge } from '@/components/ui/Badge'
import { Button } from '@/components/ui/Button'

interface TopBarProps {
  title: string
  activeWorkspace?: string
  previewQueueCount: number
  commandCount: number
  ollamaStatus: 'online' | 'offline' | 'checking'
  onOpenPalette: () => void
}

export function TopBar({
  title,
  activeWorkspace,
  previewQueueCount,
  commandCount,
  ollamaStatus,
  onOpenPalette,
}: TopBarProps) {
  return (
    <header className="relative flex flex-col gap-4 border-b border-white/8 bg-slate-950/18 px-6 py-5 backdrop-blur-xl lg:flex-row lg:items-center lg:justify-between">
      <div>
        <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-cyan-300/78">{title}</div>
        <div className="mt-3 flex flex-wrap items-center gap-3 text-sm text-slate-300/82">
          <span className="glass-pill max-w-full">
            <FolderOpen className="h-4 w-4 text-orange-300" />
            {activeWorkspace ?? 'No workspace selected'}
          </span>
          <Badge tone={ollamaStatus === 'online' ? 'success' : ollamaStatus === 'offline' ? 'danger' : 'pending'}>
            <ServerCog className="h-3.5 w-3.5" />
            Ollama {ollamaStatus}
          </Badge>
          <Badge tone="info">Preview queue {previewQueueCount}</Badge>
          <Badge tone="pending">Commands {commandCount}</Badge>
        </div>
      </div>

      <Button variant="secondary" className="min-w-[240px] justify-between rounded-2xl" onClick={onOpenPalette}>
        <span className="text-slate-200">Jump to screen</span>
        <span className="inline-flex items-center gap-2 font-mono text-[11px] text-slate-400">
          <Command className="h-3.5 w-3.5" />
          Ctrl/Cmd+K
        </span>
      </Button>
    </header>
  )
}