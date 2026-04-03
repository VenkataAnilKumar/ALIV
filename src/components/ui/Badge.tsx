import type { PropsWithChildren } from 'react'
import { cn } from '@/lib/utils'

type Tone = 'success' | 'warning' | 'danger' | 'pending' | 'info'

const tones: Record<Tone, string> = {
  success: 'border-emerald-300/18 bg-emerald-400/10 text-emerald-200',
  warning: 'border-orange-300/18 bg-orange-400/10 text-orange-200',
  danger: 'border-red-300/18 bg-red-400/10 text-red-200',
  pending: 'border-white/10 bg-slate-900/60 text-slate-300',
  info: 'border-cyan-300/18 bg-cyan-300/10 text-cyan-100',
}

export function Badge({
  children,
  tone = 'pending',
  className,
}: PropsWithChildren<{ tone?: Tone; className?: string }>) {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 rounded-full border px-2.5 py-1 font-mono text-[11px] font-medium',
        tones[tone],
        className,
      )}
    >
      {children}
    </span>
  )
}
