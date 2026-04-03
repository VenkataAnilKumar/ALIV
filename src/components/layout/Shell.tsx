import { useEffect, useState } from 'react'
import { Outlet, useLocation, useNavigate } from 'react-router-dom'
import { OnboardingModal } from '@/components/OnboardingModal'
import { Sidebar } from './Sidebar'
import { TopBar } from './TopBar'
import { Modal } from '@/components/ui/Modal'
import { useWorkspaceStore } from '@/stores/workspaceStore'

const routes = [
  { path: '/workspace', label: 'Workspace' },
  { path: '/ingestion', label: 'Ingestion' },
  { path: '/query', label: 'Query' },
  { path: '/extraction', label: 'Extraction' },
  { path: '/actions', label: 'Actions Preview' },
  { path: '/audit', label: 'Actions Audit' },
  { path: '/memory', label: 'Memory' },
  { path: '/benchmarks', label: 'Benchmarks' },
  { path: '/diagnostics', label: 'Diagnostics' },
]

export function Shell() {
  const location = useLocation()
  const navigate = useNavigate()
  const { activeWorkspace, runtime, refreshRuntime, ollamaUrl } = useWorkspaceStore()
  const [paletteOpen, setPaletteOpen] = useState(false)
  const [ollamaStatus, setOllamaStatus] = useState<'online' | 'offline' | 'checking'>('checking')
  const [showOnboarding, setShowOnboarding] = useState(false)

  useEffect(() => {
    void refreshRuntime()
  }, [refreshRuntime])

  useEffect(() => {
    let alive = true
    const checkOllama = async () => {
      setOllamaStatus('checking')
      try {
        const controller = new AbortController()
        const timeout = window.setTimeout(() => controller.abort(), 2500)
        const response = await fetch(`${ollamaUrl.replace(/\/$/, '')}/api/tags`, {
          method: 'GET',
          signal: controller.signal,
        })
        window.clearTimeout(timeout)
        if (!alive) return
        setOllamaStatus(response.ok ? 'online' : 'offline')
      } catch {
        if (!alive) return
        setOllamaStatus('offline')
      }
    }

    void checkOllama()
    const timer = window.setInterval(() => {
      void refreshRuntime()
      void checkOllama()
    }, 10000)

    return () => {
      alive = false
      window.clearInterval(timer)
    }
  }, [ollamaUrl, refreshRuntime])

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault()
        setPaletteOpen(true)
      }
      if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
        event.preventDefault()
        window.dispatchEvent(new Event('aliv:primary-action'))
      }
    }

    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [])

  useEffect(() => {
    setPaletteOpen(false)
  }, [location.pathname])

  useEffect(() => {
    const onboarded = window.localStorage.getItem('aliv_onboarded')
    if (onboarded) {
      setShowOnboarding(false)
      return
    }
    setShowOnboarding(!activeWorkspace && (runtime?.workspace_count ?? -1) === 0)
  }, [activeWorkspace, runtime?.workspace_count])

  const routeTitle = routes.find((route) => route.path === location.pathname)?.label ?? 'Workspace'

  return (
    <div className="grid min-h-screen bg-transparent lg:grid-cols-[300px_1fr]">
      <Sidebar />

      <div className="relative flex min-h-screen flex-col">
        <div className="pointer-events-none absolute inset-x-0 top-0 h-48 bg-gradient-to-b from-cyan-300/8 via-orange-300/6 to-transparent" />
        <TopBar
          title={routeTitle}
          activeWorkspace={activeWorkspace}
          previewQueueCount={runtime?.preview_queue_count ?? 0}
          commandCount={runtime?.command_count ?? 0}
          ollamaStatus={ollamaStatus}
          onOpenPalette={() => setPaletteOpen(true)}
        />
        <main className="flex-1 overflow-y-auto px-5 py-5 lg:px-8 lg:py-7">
          <div className="page-enter mx-auto w-full max-w-7xl">
            <Outlet />
          </div>
        </main>
      </div>

      <Modal open={paletteOpen} title="Quick Switch" description="Jump between ALIV work surfaces." onClose={() => setPaletteOpen(false)}>
        <div className="grid gap-2">
          {routes.map((route) => (
            <button
              key={route.path}
              type="button"
              className="flex items-center justify-between rounded-2xl border border-white/10 bg-slate-950/70 px-4 py-3 text-left text-sm text-slate-100 transition hover:-translate-y-0.5 hover:border-cyan-300/30 hover:bg-cyan-300/8"
              onClick={() => navigate(route.path)}
            >
              <span>{route.label}</span>
              <span className="font-mono text-[11px] uppercase tracking-[0.08em] text-zinc-500">
                {route.path.replace('/', '') || 'workspace'}
              </span>
            </button>
          ))}
        </div>
      </Modal>

      {showOnboarding ? (
        <OnboardingModal
          ollamaStatus={ollamaStatus}
          ollamaUrl={ollamaUrl}
          onComplete={() => {
            window.localStorage.setItem('aliv_onboarded', '1')
            setShowOnboarding(false)
          }}
        />
      ) : null}
    </div>
  )
}