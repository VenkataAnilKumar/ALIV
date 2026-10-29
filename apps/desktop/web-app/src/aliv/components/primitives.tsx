/* eslint-disable react-refresh/only-export-components */
import { cn } from '@/lib/utils'
import { Cloud, Lock } from 'lucide-react'
import type { ReactNode } from 'react'
import { describeDays, urgency } from '../dates'
import type { MatterPolicy, VaultAction } from '../types'

type Tone = 'neutral' | 'info' | 'success' | 'warning' | 'danger' | 'accent'

const TONES: Record<Tone, string> = {
  neutral: 'bg-muted text-muted-foreground border-border',
  info: 'bg-sky-500/10 text-sky-700 dark:text-sky-300 border-sky-500/20',
  success: 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border-emerald-500/20',
  warning: 'bg-amber-500/10 text-amber-700 dark:text-amber-300 border-amber-500/25',
  danger: 'bg-red-500/10 text-red-700 dark:text-red-300 border-red-500/20',
  accent: 'bg-violet-500/10 text-violet-700 dark:text-violet-300 border-violet-500/20',
}

export function Pill({
  tone = 'neutral',
  className,
  children,
  title,
}: {
  tone?: Tone
  className?: string
  children: ReactNode
  title?: string
}) {
  return (
    <span
      title={title}
      className={cn(
        'inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs font-medium whitespace-nowrap',
        TONES[tone],
        className
      )}
    >
      {children}
    </span>
  )
}

export function PolicyPill({ policy }: { policy: MatterPolicy }) {
  return policy === 'local_only' ? (
    <Pill tone="success" title="Passages from this matter never leave this computer">
      <Lock className="size-3" /> Local only
    </Pill>
  ) : (
    <Pill tone="info" title="Cloud models are allowed; every call is logged">
      <Cloud className="size-3" /> Hybrid
    </Pill>
  )
}

export function RiskPill({ risk }: { risk: VaultAction['risk'] }) {
  const tone: Tone = risk === 'low' ? 'success' : risk === 'medium' ? 'warning' : 'danger'
  return <Pill tone={tone}>{risk} risk</Pill>
}

const STATUS_TONE: Record<VaultAction['status'], Tone> = {
  proposed: 'neutral',
  approved: 'info',
  rejected: 'neutral',
  executed: 'success',
  undone: 'accent',
  failed: 'danger',
}

export function ActionStatusPill({ status }: { status: VaultAction['status'] }) {
  return <Pill tone={STATUS_TONE[status]}>{status}</Pill>
}

export function DaysPill({ days, className }: { days: number | null; className?: string }) {
  const u = urgency(days)
  if (u === null) return null
  const tone: Tone = u === 'overdue' ? 'danger' : u === 'soon' ? 'warning' : u === 'upcoming' ? 'info' : 'neutral'
  return (
    <Pill tone={tone} className={className}>
      {u === 'overdue' ? 'passed ' : ''}
      {describeDays(days)}
    </Pill>
  )
}

export function Confidence({ value }: { value: number }) {
  const pct = Math.round(value * 100)
  const tone = value >= 0.85 ? 'bg-emerald-500' : value >= 0.7 ? 'bg-amber-500' : 'bg-red-500'
  return (
    <span className="inline-flex items-center gap-1.5 text-xs text-muted-foreground" title="Rule confidence">
      <span className="h-1.5 w-10 rounded-full bg-muted overflow-hidden">
        <span className={cn('block h-full rounded-full', tone)} style={{ width: `${pct}%` }} />
      </span>
      {pct}%
    </span>
  )
}

export function StatCard({
  label,
  value,
  hint,
  icon,
  tone,
}: {
  label: string
  value: ReactNode
  hint?: ReactNode
  icon?: ReactNode
  tone?: 'warning' | 'danger'
}) {
  return (
    <div className="rounded-xl border bg-card p-4 flex flex-col gap-1 min-w-0">
      <div className="flex items-center justify-between text-xs text-muted-foreground">
        <span>{label}</span>
        {icon}
      </div>
      <div
        className={cn(
          'text-2xl font-semibold tabular-nums',
          tone === 'warning' && 'text-amber-600 dark:text-amber-400',
          tone === 'danger' && 'text-red-600 dark:text-red-400'
        )}
      >
        {value}
      </div>
      {hint && <div className="text-xs text-muted-foreground truncate">{hint}</div>}
    </div>
  )
}

export function Section({
  title,
  description,
  actions,
  children,
  className,
}: {
  title?: ReactNode
  description?: ReactNode
  actions?: ReactNode
  children: ReactNode
  className?: string
}) {
  return (
    <section className={cn('rounded-xl border bg-card min-w-0', className)}>
      {(title || actions) && (
        <header className="flex flex-wrap items-start justify-between gap-3 px-4 pt-4 pb-3">
          <div className="min-w-0">
            {title && <h2 className="text-sm font-semibold">{title}</h2>}
            {description && <p className="text-xs text-muted-foreground mt-0.5">{description}</p>}
          </div>
          {actions && <div className="flex items-center gap-2 flex-wrap">{actions}</div>}
        </header>
      )}
      {children}
    </section>
  )
}

export function EmptyState({
  icon,
  title,
  body,
  action,
}: {
  icon: ReactNode
  title: string
  body: ReactNode
  action?: ReactNode
}) {
  return (
    <div className="flex flex-col items-center justify-center text-center px-6 py-14 gap-3">
      <div className="size-12 rounded-2xl bg-muted flex items-center justify-center text-muted-foreground">
        {icon}
      </div>
      <div className="font-medium">{title}</div>
      <p className="text-sm text-muted-foreground max-w-md">{body}</p>
      {action}
    </div>
  )
}

export interface TabDef<T extends string> {
  id: T
  label: string
  badge?: number
}

export function Tabs<T extends string>({
  tabs,
  value,
  onChange,
}: {
  tabs: TabDef<T>[]
  value: T
  onChange: (t: T) => void
}) {
  return (
    <div role="tablist" className="flex gap-1 border-b overflow-x-auto -mx-1 px-1">
      {tabs.map((t) => (
        <button
          key={t.id}
          role="tab"
          aria-selected={value === t.id}
          onClick={() => onChange(t.id)}
          className={cn(
            'relative px-3 py-2 text-sm whitespace-nowrap transition-colors cursor-pointer',
            value === t.id ? 'text-foreground font-medium' : 'text-muted-foreground hover:text-foreground'
          )}
        >
          <span className="inline-flex items-center gap-1.5">
            {t.label}
            {!!t.badge && (
              <span className="rounded-full bg-primary text-primary-foreground text-[10px] leading-none px-1.5 py-0.5 tabular-nums">
                {t.badge}
              </span>
            )}
          </span>
          {value === t.id && <span className="absolute inset-x-2 -bottom-px h-0.5 rounded-full bg-foreground" />}
        </button>
      ))}
    </div>
  )
}

export function Segmented<T extends string>({
  options,
  value,
  onChange,
}: {
  options: { id: T; label: string; count?: number }[]
  value: T
  onChange: (v: T) => void
}) {
  return (
    <div className="inline-flex rounded-full border bg-muted/40 p-0.5">
      {options.map((o) => (
        <button
          key={o.id}
          onClick={() => onChange(o.id)}
          className={cn(
            'rounded-full px-3 py-1 text-xs transition-colors cursor-pointer whitespace-nowrap',
            value === o.id ? 'bg-background shadow-xs font-medium' : 'text-muted-foreground hover:text-foreground'
          )}
        >
          {o.label}
          {o.count !== undefined && <span className="ml-1 tabular-nums text-muted-foreground">{o.count}</span>}
        </button>
      ))}
    </div>
  )
}

export function fileBase(path: string) {
  return path.slice(Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\')) + 1)
}
