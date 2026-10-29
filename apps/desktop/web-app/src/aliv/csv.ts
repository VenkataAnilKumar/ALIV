import { FIELD_LABELS, type FieldKey, type GridRow } from './types'

const CSV_COLUMNS: FieldKey[] = [
  'doc_type',
  'parties',
  'effective_date',
  'expiry_date',
  'renewal_deadline',
  'amount',
  'notice_period',
  'governing_law',
  'penalty',
]

/** Same layout and escaping as the desktop export (vault.rs export_register_csv). */
export function registerCsv(rows: GridRow[]): string {
  const cell = (s: string) => {
    // A leading =,+,-,@ would be evaluated as a formula by spreadsheet apps.
    const v = /^[=+\-@]/.test(s) ? `'${s}` : s
    return /[",\n]/.test(v) ? `"${v.replace(/"/g, '""')}"` : v
  }
  const header = ['File', ...CSV_COLUMNS.map((k) => FIELD_LABELS[k])]
  const lines = rows.map((r) =>
    [r.document.fileName, ...CSV_COLUMNS.map((k) => r.fields.find((f) => f.key === k)?.value ?? '')].map(cell).join(',')
  )
  return [header.join(','), ...lines].join('\n') + '\n'
}
