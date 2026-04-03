import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'

interface StatCardProps {
  label: string
  value: ReactNode
  detail?: ReactNode
  className?: string
}

export function StatCard({ label, value, detail, className }: StatCardProps) {
  return (
    <div className={cn('panel relative overflow-hidden p-5', className)}>
      <div className="pointer-events-none absolute inset-x-0 top-0 h-20 bg-gradient-to-r from-cyan-300/10 via-transparent to-orange-300/12" />
      <div className="relative text-[11px] uppercase tracking-[0.16em] text-slate-400">{label}</div>
      <div className="relative mt-3 text-3xl font-bold tracking-[-0.03em] text-slate-50">{value}</div>
      {detail ? <div className="relative mt-2 text-sm leading-6 text-slate-300/72">{detail}</div> : null}
    </div>
  )
}