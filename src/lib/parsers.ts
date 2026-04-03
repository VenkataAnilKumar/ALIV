import type * as T from '../types/contracts'

export const parseRuntimeSnapshot = (detail: string): T.RuntimeSnapshot => {
  const parts = Object.fromEntries(
    detail.split(', ').map((entry) => {
      const [key, ...rest] = entry.split('=')
      return [key, rest.join('=')]
    }),
  )

  return {
    workspace_count: Number(parts.workspaces ?? 0),
    command_count: Number(parts.commands ?? 0),
    preview_queue_count: Number(parts.preview_queue ?? 0),
    benchmark_run_count: Number(parts.benchmark_runs ?? 0),
    last_ingestion_target: parts.last_ingestion === 'none' ? undefined : parts.last_ingestion,
  }
}

export const parseIngestionTelemetry = (detail: string): T.IngestionTelemetry => {
  const tokens = detail.split(' ')
  const parts = Object.fromEntries(
    tokens.map((token) => {
      const [key, ...rest] = token.split('=')
      return [key, rest.join('=')]
    }),
  )

  return {
    runs: Number(parts.runs ?? 0),
    discovered_total: Number(parts.discovered_total ?? 0),
    indexed_total: Number(parts.indexed_total ?? 0),
    skipped_total: Number(parts.skipped_total ?? 0),
    parsed_total: Number(parts.parsed_total ?? 0),
    failed_total: Number(parts.failed_total ?? 0),
    files: Number(parts.files ?? 0),
    parsed: Number(parts.parsed ?? 0),
    chunks: Number(parts.chunks ?? 0),
    embeddings: Number(parts.embeddings ?? 0),
    avg_parser_confidence: Number(parts.avg_parser_confidence ?? 0),
    last_run: parts.last_run === 'none' ? undefined : parts.last_run,
    last_workspace: parts.last_workspace === 'none' ? undefined : parts.last_workspace,
    last_mode: parts.last_mode === 'none' ? undefined : parts.last_mode,
    last_created_at: parts.last_created_at === 'none' ? undefined : parts.last_created_at,
  }
}

export const parseExtractionRows = (detail: string): T.ExtractionFieldRow[] => {
  try {
    const payload = JSON.parse(detail) as {
      template?: string
      query?: string
      retrieval_hits?: number
      value?: unknown
      confidence?: unknown
      citations?: unknown[]
    }
    const citations = Array.isArray(payload.citations) ? payload.citations : []

    if (payload.value && typeof payload.value === 'object' && !Array.isArray(payload.value)) {
      const confidenceMap = payload.confidence && typeof payload.confidence === 'object' && !Array.isArray(payload.confidence)
        ? payload.confidence as Record<string, unknown>
        : {}
      return Object.entries(payload.value as Record<string, unknown>).map(([field, value]) => ({
        field,
        value: typeof value === 'string' ? value : JSON.stringify(value ?? ''),
        confidence: typeof confidenceMap[field] === 'number' ? confidenceMap[field] as number : null,
        citations,
      }))
    }

    return [{
      field: payload.template ?? 'Result',
      value: typeof payload.value === 'string' ? payload.value : JSON.stringify(payload.value ?? ''),
      confidence: typeof payload.confidence === 'number' ? payload.confidence : null,
      citations,
    }]
  } catch {
    return []
  }
}

export const parseExtractionPayload = (detail: string) => {
  try {
    return JSON.parse(detail) as {
      template?: string
      query?: string
      value?: unknown
      confidence?: unknown
      citations?: unknown[]
      retrieval_hits?: number
    }
  } catch {
    return null
  }
}

export const parseBenchmarkReport = (detail: string): T.BenchmarkSuiteReport | null => {
  try {
    const parsed = JSON.parse(detail) as T.BenchmarkSuiteReport
    if (!parsed || !Array.isArray(parsed.runs) || !parsed.phase4_exit) {
      return null
    }
    return parsed
  } catch {
    return null
  }
}
