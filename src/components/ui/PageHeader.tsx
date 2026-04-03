import type { ReactNode } from 'react'

interface PageHeaderProps {
  eyebrow: string
  title: string
  description: string
  actions?: ReactNode
}

export function PageHeader({ eyebrow, title, description, actions }: PageHeaderProps) {
  return (
    <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
      <div>
        <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-cyan-300">{eyebrow}</div>
        <h1 className="mt-3 text-4xl font-bold tracking-[-0.04em] text-slate-50 md:text-5xl">{title}</h1>
        <p className="mt-3 max-w-3xl text-[15px] leading-7 text-slate-300/78">{description}</p>
      </div>
      {actions ? <div className="flex flex-wrap items-center gap-3">{actions}</div> : null}
    </div>
  )
}