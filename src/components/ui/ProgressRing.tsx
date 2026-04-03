export function ProgressRing({ value, label }: { value: number; label?: string }) {
  const radius = 34
  const circumference = 2 * Math.PI * radius
  const clamped = Math.max(0, Math.min(100, value))
  const offset = circumference - (clamped / 100) * circumference

  return (
    <div className="relative inline-flex items-center justify-center">
      <svg width="80" height="80" viewBox="0 0 80 80" className="-rotate-90">
        <circle cx="40" cy="40" r={radius} stroke="#27272a" strokeWidth="6" fill="none" />
        <circle
          cx="40"
          cy="40"
          r={radius}
          stroke="#6366f1"
          strokeWidth="6"
          fill="none"
          strokeLinecap="round"
          strokeDasharray={circumference}
          strokeDashoffset={offset}
          className="progress-ring__value"
        />
      </svg>
      <div className="absolute text-center">
        <div className="text-sm font-semibold text-zinc-50">{Math.round(clamped)}%</div>
        {label ? <div className="text-[11px] text-zinc-500">{label}</div> : null}
      </div>
    </div>
  )
}
