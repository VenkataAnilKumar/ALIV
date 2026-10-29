import { describe, expect, it } from 'vitest'
import { daysUntil, deadlines, describeDays, urgency } from '../dates'
import type { GridRow } from '../types'

const today = new Date(2026, 9, 29) // 29 Oct 2026, local time

describe('dates', () => {
  it('counts whole days in either direction', () => {
    expect(daysUntil('2026-10-29', today)).toBe(0)
    expect(daysUntil('2026-11-15', today)).toBe(17)
    expect(daysUntil('2026-03-01', today)).toBe(-242)
    expect(daysUntil('not a date', today)).toBeNull()
  })

  it('buckets urgency', () => {
    expect(urgency(-1)).toBe('overdue')
    expect(urgency(10)).toBe('soon')
    expect(urgency(60)).toBe('upcoming')
    expect(urgency(120)).toBe('later')
    expect(urgency(null)).toBeNull()
  })

  it('describes days in words', () => {
    expect(describeDays(0)).toBe('today')
    expect(describeDays(17)).toBe('in 17 days')
    expect(describeDays(-3)).toBe('3 days ago')
  })

  it('lists expiry and renewal deadlines soonest first', () => {
    const row = (name: string, fields: [string, string][]): GridRow => ({
      document: { id: name, fileName: name } as GridRow['document'],
      fields: fields.map(([key, value]) => ({ key, value, confidence: 0.9, chunkOrdinal: 0, snippet: '' })) as GridRow['fields'],
    })
    const rows = [
      row('a', [['expiry_date', '2026-12-15'], ['renewal_deadline', '2026-11-15']]),
      row('b', [['expiry_date', '2026-03-01'], ['amount', '$1']]),
    ]
    expect(deadlines(rows, today).map((d) => `${d.row.document.id}:${d.kind}`)).toEqual([
      'b:expiry_date',
      'a:renewal_deadline',
      'a:expiry_date',
    ])
  })
})
