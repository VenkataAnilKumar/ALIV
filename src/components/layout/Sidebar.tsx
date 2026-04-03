import {
  Activity,
  ClipboardList,
  Database,
  FileSearch,
  FolderTree,
  Gauge,
  History,
  ScrollText,
  Shield,
} from 'lucide-react'
import { NavLink } from 'react-router-dom'
import { cn } from '@/lib/utils'

const items = [
  { to: '/workspace', label: 'Workspace', icon: FolderTree },
  { to: '/ingestion', label: 'Ingestion', icon: Database },
  { to: '/query', label: 'Query', icon: FileSearch },
  { to: '/extraction', label: 'Extraction', icon: ScrollText },
  { to: '/actions', label: 'Actions Preview', icon: Shield },
  { to: '/audit', label: 'Actions Audit', icon: History },
  { to: '/memory', label: 'Memory', icon: Activity },
  { to: '/benchmarks', label: 'Benchmarks', icon: Gauge },
  { to: '/diagnostics', label: 'Diagnostics', icon: ClipboardList },
]

export function Sidebar() {
  return (
    <aside className="relative flex h-full flex-col overflow-hidden border-r border-cyan-900/20 bg-slate-950/30 px-4 py-6 backdrop-blur-2xl">
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top_left,rgba(83,211,209,0.16),transparent_34%),radial-gradient(circle_at_bottom_right,rgba(255,157,92,0.14),transparent_26%)]" />
      <div className="relative rounded-[28px] border border-white/8 bg-slate-950/45 px-4 py-5 shadow-panel">
        <div className="font-mono text-xs uppercase tracking-[0.22em] text-cyan-300">ALIV</div>
        <div className="mt-3 text-2xl font-bold tracking-tight text-slate-50">Legal Intelligence Workbench</div>
        <div className="mt-2 text-sm leading-6 text-slate-300/78">A calmer 2026 operator console for local-first review, retrieval, memory, and safe file actions.</div>
        <div className="mt-4 flex flex-wrap gap-2">
          <span className="glass-pill text-xs">Local-first</span>
          <span className="glass-pill text-xs">Retrieval-native</span>
          <span className="glass-pill text-xs">Safer actions</span>
        </div>
      </div>

      <nav className="relative mt-8 flex flex-1 flex-col gap-2">
        {items.map((item) => {
          const Icon = item.icon
          return (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) =>
                cn(
                  'group flex items-center gap-3 rounded-2xl px-4 py-3.5 text-sm transition-all duration-200',
                  isActive
                    ? 'bg-gradient-to-r from-cyan-300/16 via-cyan-300/10 to-orange-300/12 text-white shadow-[inset_0_0_0_1px_rgba(83,211,209,0.28)]'
                    : 'text-slate-300/80 hover:-translate-y-0.5 hover:bg-slate-900/70 hover:text-slate-50',
                )
              }
            >
              {({ isActive }) => (
                <>
                  <Icon className={cn('h-4 w-4', isActive ? 'text-cyan-200' : 'text-slate-500 group-hover:text-orange-200')} />
                  <span className="font-medium tracking-tight">{item.label}</span>
                </>
              )}
            </NavLink>
          )
        })}
      </nav>

      <div className="panel relative px-4 py-4">
        <div className="font-mono text-[11px] uppercase tracking-[0.16em] text-slate-400">Shortcuts</div>
        <div className="mt-3 space-y-2 text-sm text-slate-300/82">
          <div><span className="text-cyan-100">Ctrl/Cmd+K</span> quick switch</div>
          <div><span className="text-cyan-100">Ctrl/Cmd+Enter</span> primary action</div>
          <div><span className="text-cyan-100">Esc</span> close modal</div>
        </div>
      </div>
    </aside>
  )
}