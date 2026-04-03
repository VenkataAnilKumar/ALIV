import { Component, type ErrorInfo, type ReactNode } from 'react'
import { Button } from '@/components/ui/Button'
import { downloadTextFile } from '@/lib/utils'

interface ErrorBoundaryState {
  error: Error | null
}

export class ErrorBoundary extends Component<{ children: ReactNode }, ErrorBoundaryState> {
  state: ErrorBoundaryState = { error: null }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('[ALIV] Uncaught frontend error:', error, info)
  }

  render() {
    if (!this.state.error) return this.props.children

    return (
      <div className="flex min-h-screen items-center justify-center px-6 py-10">
        <div className="panel w-full max-w-2xl p-8">
          <div className="font-mono text-[11px] uppercase tracking-[0.18em] text-red-400">Unrecoverable Error</div>
          <h1 className="mt-4 text-3xl font-bold tracking-[-0.04em] text-slate-50">ALIV encountered a frontend crash</h1>
          <p className="mt-3 max-w-xl text-sm leading-7 text-slate-300/78">
            Rendering stopped because a React error escaped the current screen. Your indexed data and SQLite state remain intact.
          </p>
          <pre className="mt-6 max-h-[360px] overflow-auto rounded-2xl border border-red-400/20 bg-red-500/8 p-4 font-mono text-xs leading-6 text-red-200">
            {this.state.error.message}
            {'\n\n'}
            {this.state.error.stack}
          </pre>
          <div className="mt-6 flex flex-wrap gap-3">
            <Button variant="primary" onClick={() => this.setState({ error: null })}>
              Try to recover
            </Button>
            <Button
              variant="secondary"
              onClick={() =>
                downloadTextFile(
                  'aliv-crash.txt',
                  `${this.state.error?.message ?? 'Unknown error'}\n\n${this.state.error?.stack ?? ''}`,
                  'text/plain;charset=utf-8',
                )
              }
            >
              Export crash log
            </Button>
          </div>
        </div>
      </div>
    )
  }
}