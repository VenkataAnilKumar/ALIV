import { useEffect, useState } from 'react'
import { RefreshCw, Save, Settings2 } from 'lucide-react'
import { PageHeader } from '@/components/ui/PageHeader'
import { StatCard } from '@/components/ui/StatCard'
import { Button } from '@/components/ui/Button'
import { useWorkspaceStore } from '@/stores/workspaceStore'

export function WorkspacePage() {
  const {
    folders,
    activeWorkspace,
    runtime,
    policy,
    ollamaUrl,
    modelName,
    embeddingModel,
    isSaving,
    error,
    addFolder,
    removeFolder,
    setActiveWorkspace,
    setPolicy,
    setOllamaUrl,
    setModelName,
    refreshRuntime,
    saveWorkspace,
  } = useWorkspaceStore()
  const [draftPath, setDraftPath] = useState(activeWorkspace ?? '')

  useEffect(() => {
    setDraftPath(activeWorkspace ?? '')
  }, [activeWorkspace])

  useEffect(() => {
    const onPrimary = () => {
      void saveWorkspace()
    }
    window.addEventListener('aliv:primary-action', onPrimary)
    return () => window.removeEventListener('aliv:primary-action', onPrimary)
  }, [saveWorkspace])

  return (
    <div className="space-y-6">
      <PageHeader
        eyebrow="Foundation"
        title="Workspace Control"
        description="Register the active matter set, tune local model endpoints, and confirm runtime status before starting ingestion or retrieval."
        actions={
          <>
            <Button variant="secondary" onClick={() => void refreshRuntime()}>
              <RefreshCw className="h-4 w-4" />
              Refresh runtime
            </Button>
            <Button variant="primary" onClick={() => void saveWorkspace()} disabled={isSaving}>
              <Save className="h-4 w-4" />
              {isSaving ? 'Saving...' : 'Save workspace'}
            </Button>
          </>
        }
      />

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <StatCard label="Workspaces" value={runtime?.workspace_count ?? folders.length} detail="Registered folders in runtime state" />
        <StatCard label="Commands" value={runtime?.command_count ?? 0} detail="Backend command entries recorded" />
        <StatCard label="Preview Queue" value={runtime?.preview_queue_count ?? 0} detail="Pending action recommendations" />
        <StatCard label="Benchmarks" value={runtime?.benchmark_run_count ?? 0} detail="Stored benchmark runs" />
      </div>

      <div className="grid gap-6 xl:grid-cols-[1.1fr_0.9fr]">
        <section className="panel p-5">
          <div className="flex items-center gap-2 text-sm font-medium text-zinc-100">
            <Settings2 className="h-4 w-4 text-indigo-300" />
            Registration
          </div>

          <div className="mt-4 space-y-4">
            <div>
              <label className="mb-2 block text-sm text-zinc-400">Workspace path</label>
              <div className="flex gap-3">
                <input
                  className="input-base flex-1"
                  value={draftPath}
                  onChange={(event) => setDraftPath(event.target.value)}
                  placeholder="V:/AI Engineer/GitHub/ALIV"
                />
                <Button
                  variant="secondary"
                  onClick={() => {
                    if (!draftPath.trim()) return
                    addFolder(draftPath.trim())
                    setActiveWorkspace(draftPath.trim())
                  }}
                >
                  Add
                </Button>
              </div>
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <div>
                <label className="mb-2 block text-sm text-zinc-400">Policy</label>
                <select className="input-base w-full" title="Safety policy" aria-label="Safety policy" value={policy} onChange={(event) => setPolicy(event.target.value as typeof policy)}>
                  <option>Safe</option>
                  <option>Standard</option>
                  <option>Aggressive</option>
                </select>
              </div>
              <div>
                <label className="mb-2 block text-sm text-zinc-400">Embedding model</label>
                <input className="input-base w-full" title="Embedding model" aria-label="Embedding model" value={embeddingModel} readOnly />
              </div>
            </div>

            <div>
              <label className="mb-2 block text-sm text-zinc-400">Ollama URL</label>
              <input className="input-base w-full" title="Ollama URL" aria-label="Ollama URL" placeholder="http://localhost:11434" value={ollamaUrl} onChange={(event) => setOllamaUrl(event.target.value)} />
            </div>

            <div>
              <label className="mb-2 block text-sm text-zinc-400">Chat model</label>
              <input className="input-base w-full" title="Chat model" aria-label="Chat model" placeholder="llama3.1:8b" value={modelName} onChange={(event) => setModelName(event.target.value)} />
            </div>
          </div>

          {error ? <div className="mt-4 rounded-lg border border-red-500/20 bg-red-500/5 px-4 py-3 text-sm text-red-300">{error}</div> : null}
        </section>

        <section className="panel p-5">
          <div className="text-sm font-medium text-zinc-100">Registered folders</div>
          <div className="mt-4 space-y-3">
            {folders.length === 0 ? (
              <div className="rounded-xl border border-dashed border-zinc-800 px-4 py-8 text-sm text-zinc-500">
                No folders registered yet. Add a path above and save it into runtime.
              </div>
            ) : (
              folders.map((folder) => (
                <div key={folder} className="rounded-xl border border-zinc-800 bg-zinc-950/50 px-4 py-3">
                  <div className="flex items-start justify-between gap-3">
                    <button
                      type="button"
                      className="text-left"
                      onClick={() => {
                        setActiveWorkspace(folder)
                        setDraftPath(folder)
                      }}
                    >
                      <div className="font-mono text-sm text-zinc-100">{folder}</div>
                      <div className="mt-1 text-xs text-zinc-500">{folder === activeWorkspace ? 'Active workspace' : 'Click to activate'}</div>
                    </button>
                    <Button variant="ghost" className="px-2 py-1 text-xs" onClick={() => removeFolder(folder)}>
                      Remove
                    </Button>
                  </div>
                </div>
              ))
            )}
          </div>
        </section>
      </div>
    </div>
  )
}