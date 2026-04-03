import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'
import { Skeleton } from './Skeleton'

export interface TableColumn<T> {
  key: string
  header: string
  className?: string
  render: (row: T) => ReactNode
}

interface TableProps<T> {
  columns: TableColumn<T>[]
  rows: T[]
  loading?: boolean
  emptyTitle?: string
  emptyDescription?: string
  error?: string | null
  rowKey: (row: T) => string
  rowClassName?: (row: T) => string | undefined
}

export function Table<T>({
  columns,
  rows,
  loading,
  emptyTitle,
  emptyDescription,
  error,
  rowKey,
  rowClassName,
}: TableProps<T>) {
  if (error) {
    return <div className="panel border-red-500/20 bg-red-500/5 p-4 text-sm text-red-300">{error}</div>
  }

  if (loading) {
    return (
      <div className="panel overflow-hidden">
        <div className="grid gap-2 p-4">
          {Array.from({ length: 6 }).map((_, index) => (
            <Skeleton key={index} className="h-10 w-full" />
          ))}
        </div>
      </div>
    )
  }

  if (rows.length === 0) {
    return (
      <div className="panel p-8 text-center">
        <p className="text-sm font-medium text-zinc-100">{emptyTitle ?? 'No data available'}</p>
        {emptyDescription ? <p className="mt-2 text-sm text-zinc-500">{emptyDescription}</p> : null}
      </div>
    )
  }

  return (
    <div className="panel overflow-hidden">
      <table className="min-w-full border-collapse text-sm">
        <thead className="border-b border-zinc-800 bg-zinc-950/40">
          <tr>
            {columns.map((column) => (
              <th
                key={column.key}
                className={cn(
                  'px-4 py-3 text-left text-[11px] uppercase tracking-[0.08em] text-zinc-500',
                  column.className,
                )}
              >
                {column.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr
              key={rowKey(row)}
              className={cn('border-b border-zinc-900 transition-colors hover:bg-surface-overlay/80', rowClassName?.(row))}
            >
              {columns.map((column) => (
                <td key={column.key} className={cn('px-4 py-3 align-top text-zinc-200', column.className)}>
                  {column.render(row)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
