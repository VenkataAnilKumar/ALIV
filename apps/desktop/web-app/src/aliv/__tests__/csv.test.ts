import { describe, expect, it } from 'vitest'
import { registerCsv } from '../csv'
import type { GridRow } from '../types'

describe('registerCsv', () => {
  it('quotes commas and neutralizes formulas', () => {
    const row = {
      document: { fileName: 'a,b.txt' },
      fields: [
        { key: 'parties', value: '=HYPERLINK("x")', confidence: 1, chunkOrdinal: 0, snippet: '' },
        { key: 'amount', value: '$1,000', confidence: 1, chunkOrdinal: 0, snippet: '' },
      ],
    } as unknown as GridRow
    const [header, line] = registerCsv([row]).trim().split('\n')
    expect(header.startsWith('File,Type,Parties,Effective')).toBe(true)
    expect(line).toBe('"a,b.txt",,"\'=HYPERLINK(""x"")",,,,"$1,000",,,')
  })
})
