import { useMemo, useState } from 'react'
import { CheckCircle2, Circle, FolderPlus, Sparkles } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { useWorkspaceStore } from '@/stores/workspaceStore'
import { cn } from '@/lib/utils'

interface OnboardingModalProps {
  ollamaStatus: 'online' | 'offline' | 'checking'
  ollamaUrl: string
  onComplete: () => void
}

const steps = [
  { title: 'Welcome', icon: Sparkles },
  { title: 'Workspace', icon: FolderPlus },
  { title: 'Ollama', icon: CheckCircle2 },
]

export function OnboardingModal({ ollamaStatus, ollamaUrl, onComplete }: OnboardingModalProps) {
  const { addFolder, setActiveWorkspace, saveWorkspace, refreshRuntime, isSaving } = useWorkspaceStore()
  const [step, setStep] = useState(0)
  const [workspacePath, setWorkspacePath] = useState('')
  const [error, setError] = useState<string | null>(null)

  const statusTone = useMemo(() => {
    if (ollamaStatus === 'online') return 'border-emerald-300/30 bg-emerald-300/10 text-emerald-200'
    if (ollamaStatus === 'offline') return 'border-orange-300/30 bg-orange-300/10 text-orange-200'
    return 'border-cyan-300/30 bg-cyan-300/10 text-cyan-200'
  }, [ollamaStatus])

  const registerWorkspacePath = async () => {
    const nextPath = workspacePath.trim()
    if (!nextPath) {
      setError('Enter a workspace folder before continuing.')
      return
    }

    setError(null)
    try {
      addFolder(nextPath)
      setActiveWorkspace(nextPath)
      await saveWorkspace()
      await refreshRuntime()
      setStep(2)
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : String(nextError))
    }
  }

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center bg-slate-950/82 p-4 backdrop-blur-xl">
      <div className="relative w-full max-w-3xl overflow-hidden rounded-[32px] border border-white/10 bg-slate-950/92 shadow-[0_24px_80px_rgba(2,6,23,0.5)]">
        <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top_left,rgba(83,211,209,0.18),transparent_34%),radial-gradient(circle_at_bottom_right,rgba(255,157,92,0.16),transparent_28%)]" />
        <div className="relative grid gap-8 p-8 lg:grid-cols-[220px_1fr] lg:p-10">
          <aside className="rounded-[28px] border border-white/8 bg-slate-950/40 p-5">
            <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-cyan-300">First Run</div>
            <div className="mt-3 text-2xl font-bold tracking-[-0.03em] text-slate-50">Set ALIV up once</div>
            <div className="mt-2 text-sm leading-6 text-slate-300/78">
              This setup stays local to the device and keeps your legal or finance matter folders out of cloud workflows.
            </div>
            <div className="mt-6 space-y-3">
              {steps.map((item, index) => {
                const Icon = item.icon
                const done = index < step
                const active = index === step
                return (
                  <div
                    key={item.title}
                    className={cn(
                      'flex items-center gap-3 rounded-2xl border px-3 py-3 text-sm',
                      active
                        ? 'border-cyan-300/26 bg-cyan-300/10 text-white'
                        : done
                          ? 'border-emerald-300/20 bg-emerald-300/8 text-emerald-100'
                          : 'border-white/8 bg-slate-950/40 text-slate-400',
                    )}
                  >
                    {done ? <CheckCircle2 className="h-4 w-4" /> : active ? <Icon className="h-4 w-4" /> : <Circle className="h-4 w-4" />}
                    <span>{item.title}</span>
                  </div>
                )
              })}
            </div>
          </aside>

          <section>
            {step === 0 ? (
              <>
                <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-orange-200">Welcome to ALIV</div>
                <h2 className="mt-3 text-4xl font-bold tracking-[-0.04em] text-slate-50">A local-first workbench for high-trust document review</h2>
                <p className="mt-4 max-w-2xl text-[15px] leading-7 text-slate-300/78">
                  ALIV indexes contracts, PDFs, and matter folders on your machine, then layers retrieval, extraction, memory, and controlled file actions on top. Nothing in this flow requires cloud upload.
                </p>
                <div className="mt-6 grid gap-3 md:grid-cols-3">
                  <div className="panel p-4 text-sm text-slate-300/78">Use a dedicated matter folder so ingestion and actions stay predictable.</div>
                  <div className="panel p-4 text-sm text-slate-300/78">Keep Ollama running locally for model availability and retrieval workflows.</div>
                  <div className="panel p-4 text-sm text-slate-300/78">Diagnostics and logs stay on-device for release and incident triage.</div>
                </div>
                <div className="mt-8 flex flex-wrap gap-3">
                  <Button variant="primary" onClick={() => setStep(1)}>
                    Get started
                  </Button>
                  <Button variant="ghost" onClick={onComplete}>
                    Skip for now
                  </Button>
                </div>
              </>
            ) : null}

            {step === 1 ? (
              <>
                <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-orange-200">Set your workspace</div>
                <h2 className="mt-3 text-3xl font-bold tracking-[-0.04em] text-slate-50">Choose the folder ALIV will index and supervise</h2>
                <p className="mt-4 max-w-2xl text-[15px] leading-7 text-slate-300/78">
                  Point ALIV at the folder it should watch, index, and act on. A dedicated legal matter or finance review folder keeps ingestion clean and action previews safe.
                </p>
                <div className="mt-6 panel p-5">
                  <label className="mb-2 block text-sm text-slate-300/78">Workspace folder path</label>
                  <input
                    className="input-base w-full"
                    placeholder="V:/Matters/Deal-Review"
                    value={workspacePath}
                    onChange={(event) => setWorkspacePath(event.target.value)}
                  />
                  <div className="mt-3 text-sm text-slate-400">
                    Hint: ALIV expects a stable local folder, not a temp export or downloads pile.
                  </div>
                  {error ? <div className="mt-4 rounded-2xl border border-red-400/20 bg-red-500/8 px-4 py-3 text-sm text-red-200">{error}</div> : null}
                </div>
                <div className="mt-8 flex flex-wrap gap-3">
                  <Button variant="secondary" onClick={() => setStep(0)}>
                    Back
                  </Button>
                  <Button variant="primary" onClick={() => void registerWorkspacePath()} disabled={isSaving}>
                    {isSaving ? 'Registering...' : 'Register and continue'}
                  </Button>
                  <Button variant="ghost" onClick={onComplete}>
                    Skip for now
                  </Button>
                </div>
              </>
            ) : null}

            {step === 2 ? (
              <>
                <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-orange-200">Confirm Ollama</div>
                <h2 className="mt-3 text-3xl font-bold tracking-[-0.04em] text-slate-50">Make sure your local model runtime is available</h2>
                <p className="mt-4 max-w-2xl text-[15px] leading-7 text-slate-300/78">
                  ALIV checks the local Ollama endpoint before you start retrieval and extraction workflows. If you do not see an online status, start Ollama and pull the default model.
                </p>
                <div className="mt-6 flex flex-wrap items-center gap-3">
                  <span className={cn('rounded-full border px-3 py-1 text-sm font-medium capitalize', statusTone)}>
                    Ollama {ollamaStatus}
                  </span>
                  <span className="text-sm text-slate-400">Endpoint: {ollamaUrl}</span>
                </div>
                <div className="mt-6 rounded-[24px] border border-white/8 bg-slate-950/50 p-5">
                  <div className="font-mono text-[11px] uppercase tracking-[0.16em] text-slate-400">Suggested command</div>
                  <pre className="mt-3 overflow-x-auto rounded-2xl border border-cyan-300/14 bg-slate-950/80 p-4 font-mono text-sm text-cyan-100">ollama pull llama3.2:3b</pre>
                </div>
                <div className="mt-8 flex flex-wrap gap-3">
                  <Button variant="secondary" onClick={() => setStep(1)}>
                    Back
                  </Button>
                  <Button variant="primary" onClick={onComplete}>
                    Open ALIV
                  </Button>
                </div>
              </>
            ) : null}
          </section>
        </div>
      </div>
    </div>
  )
}