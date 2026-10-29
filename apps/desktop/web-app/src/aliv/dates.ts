import type { GridRow } from './types'

const DAY_MS = 24 * 60 * 60 * 1000

/** Whole days from `today` to an ISO date (negative when it has passed). */
export function daysUntil(isoDate: string, today: Date = new Date()): number | null {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(isoDate)
  if (!m) return null
  const target = Date.UTC(Number(m[1]), Number(m[2]) - 1, Number(m[3]))
  const base = Date.UTC(today.getFullYear(), today.getMonth(), today.getDate())
  return Math.round((target - base) / DAY_MS)
}

export type Urgency = 'overdue' | 'soon' | 'upcoming' | 'later'

export function urgency(days: number | null, windowDays = 90): Urgency | null {
  if (days === null) return null
  if (days < 0) return 'overdue'
  if (days <= 30) return 'soon'
  if (days <= windowDays) return 'upcoming'
  return 'later'
}

export function describeDays(days: number | null): string {
  if (days === null) return ''
  if (days === 0) return 'today'
  if (days === 1) return 'tomorrow'
  if (days === -1) return 'yesterday'
  return days > 0 ? `in ${days} days` : `${-days} days ago`
}

export interface Deadline {
  row: GridRow
  kind: 'expiry_date' | 'renewal_deadline'
  date: string
  days: number
}

/** Every dated obligation in the matter, soonest first. */
export function deadlines(rows: GridRow[], today: Date = new Date()): Deadline[] {
  const out: Deadline[] = []
  for (const row of rows) {
    for (const f of row.fields) {
      if (f.key !== 'expiry_date' && f.key !== 'renewal_deadline') continue
      const days = daysUntil(f.value, today)
      if (days !== null) out.push({ row, kind: f.key, date: f.value, days })
    }
  }
  return out.sort((a, b) => a.days - b.days)
}
