export const cn = (...values: Array<string | false | null | undefined>) =>
  values.filter(Boolean).join(' ')

export const downloadTextFile = (filename: string, content: string, mimeType = 'text/plain;charset=utf-8') => {
  const blob = new Blob([content], { type: mimeType })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = filename
  anchor.click()
  URL.revokeObjectURL(url)
}

export const toCsv = (rows: Array<Record<string, unknown>>) => {
  if (rows.length === 0) return ''
  const headers = [...new Set(rows.flatMap((row) => Object.keys(row)))]
  const escape = (value: unknown) => {
    const text = value == null ? '' : String(value)
    if (/[",\n]/.test(text)) {
      return `"${text.replace(/"/g, '""')}"`
    }
    return text
  }
  return [
    headers.join(','),
    ...rows.map((row) => headers.map((header) => escape(row[header])).join(',')),
  ].join('\n')
}

export const formatNumber = (value: number) => new Intl.NumberFormat().format(value)

export const formatDateTime = (value?: string) => {
  if (!value) return 'n/a'
  const numeric = Number(value)
  if (!Number.isNaN(numeric) && numeric > 0) {
    return new Date(numeric * 1000).toLocaleString()
  }
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString()
}

export const riskTone = (risk: string) => {
  if (risk === 'high') return 'danger'
  if (risk === 'medium') return 'warning'
  return 'success'
}

export const confidenceTone = (value?: number) => {
  if (value === undefined) return 'pending'
  if (value >= 0.85) return 'success'
  if (value >= 0.6) return 'warning'
  return 'danger'
}
