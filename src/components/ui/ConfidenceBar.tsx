import { cn } from '@/lib/utils'

export function ConfidenceBar({ value, className }: { value?: number; className?: string }) {
  const progressValue = Math.max(0, Math.min(100, Math.round((value ?? 0) * 100)))
  const fill =
    value === undefined
      ? 'confidence-bar--neutral'
      : value >= 0.85
        ? 'confidence-bar--high'
        : value >= 0.6
          ? 'confidence-bar--medium'
          : 'confidence-bar--low'

  return (
    <progress
      className={cn('confidence-bar h-1.5 w-full overflow-hidden rounded-full bg-zinc-800', fill, className)}
      max={100}
      value={progressValue}
    />
  )
}
