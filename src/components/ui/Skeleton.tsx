import { cn } from '@/lib/utils'

export function Skeleton({ className }: { className?: string }) {
  return (
    <div
      className={cn(
        'animate-shimmer rounded-md bg-[length:200%_100%] bg-[linear-gradient(90deg,#27272a_25%,#3f3f46_50%,#27272a_75%)]',
        className,
      )}
    />
  )
}
