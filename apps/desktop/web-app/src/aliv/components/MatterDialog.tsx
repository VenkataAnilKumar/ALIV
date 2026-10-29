import { useEffect, useState } from 'react'
import { Briefcase, Cloud, Landmark, Lock, Scale } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'
import type { Matter, MatterInput, MatterKind, MatterPolicy } from '../types'

function Choice({
  selected,
  onClick,
  icon,
  title,
  body,
}: {
  selected: boolean
  onClick: () => void
  icon: React.ReactNode
  title: string
  body: string
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-pressed={selected}
      className={cn(
        'flex-1 text-left rounded-xl border p-3 transition-colors cursor-pointer',
        selected ? 'border-foreground/60 bg-accent' : 'hover:bg-accent/50'
      )}
    >
      <div className="flex items-center gap-2 text-sm font-medium">
        {icon}
        {title}
      </div>
      <p className="text-xs text-muted-foreground mt-1">{body}</p>
    </button>
  )
}

export function MatterDialog({
  open,
  onOpenChange,
  initial,
  onSubmit,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  initial?: Matter | null
  onSubmit: (input: MatterInput) => Promise<void>
}) {
  const [name, setName] = useState('')
  const [client, setClient] = useState('')
  const [kind, setKind] = useState<MatterKind>('legal')
  const [policy, setPolicy] = useState<MatterPolicy>('local_only')
  const [busy, setBusy] = useState(false)

  useEffect(() => {
    if (open) {
      setName(initial?.name ?? '')
      setClient(initial?.client ?? '')
      setKind(initial?.kind ?? 'legal')
      setPolicy(initial?.policy ?? 'local_only')
    }
  }, [open, initial])

  const submit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!name.trim()) return
    setBusy(true)
    try {
      await onSubmit({ name, client, kind, policy })
    } finally {
      setBusy(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <form onSubmit={submit} className="flex flex-col gap-4">
          <DialogHeader>
            <DialogTitle>{initial ? 'Matter settings' : 'New matter'}</DialogTitle>
            <DialogDescription>
              A matter holds the documents for one client engagement. Its model policy decides where passages may be sent.
            </DialogDescription>
          </DialogHeader>
          <label className="flex flex-col gap-1.5 text-sm">
            Matter name
            <Input autoFocus value={name} onChange={(e) => setName(e.target.value)} placeholder="e.g. Acme lease renewal" />
          </label>
          <label className="flex flex-col gap-1.5 text-sm">
            Client
            <Input value={client} onChange={(e) => setClient(e.target.value)} placeholder="e.g. Redstone Enterprises Inc." />
          </label>
          <div className="flex flex-col gap-1.5 text-sm">
            Practice
            <div className="flex gap-2">
              <Choice selected={kind === 'legal'} onClick={() => setKind('legal')} icon={<Scale className="size-4" />} title="Legal" body="Contracts, leases, NDAs, disputes" />
              <Choice selected={kind === 'finance'} onClick={() => setKind('finance')} icon={<Landmark className="size-4" />} title="Finance" body="Invoices, statements, tax, audit" />
            </div>
          </div>
          <div className="flex flex-col gap-1.5 text-sm">
            Model policy
            <div className="flex gap-2">
              <Choice
                selected={policy === 'local_only'}
                onClick={() => setPolicy('local_only')}
                icon={<Lock className="size-4" />}
                title="Local only"
                body="Answers use models on this computer. Nothing leaves the device."
              />
              <Choice
                selected={policy === 'hybrid'}
                onClick={() => setPolicy('hybrid')}
                icon={<Cloud className="size-4" />}
                title="Hybrid"
                body="Cloud models allowed. Only the matching passages are sent, and every call is logged."
              />
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={!name.trim() || busy}>
              <Briefcase className="size-4" />
              {initial ? 'Save' : 'Create matter'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
