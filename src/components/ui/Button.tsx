import type { ButtonHTMLAttributes, PropsWithChildren } from 'react'
import { forwardRef } from 'react'
import { cn } from '@/lib/utils'

type Variant = 'primary' | 'secondary' | 'destructive' | 'ghost'

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant
}

const variants: Record<Variant, string> = {
  primary:
    'bg-gradient-to-r from-cyan-300 to-teal-300 text-slate-950 shadow-glow hover:from-cyan-200 hover:to-orange-200 disabled:from-cyan-300/40 disabled:to-cyan-300/30 disabled:text-slate-800 disabled:shadow-none',
  secondary:
    'border border-white/10 bg-slate-950/35 text-slate-100 hover:border-cyan-300/28 hover:bg-slate-900/70 hover:text-white disabled:text-slate-500',
  destructive:
    'border border-red-400/30 bg-red-500/6 text-red-300 hover:bg-red-500/14 disabled:text-red-900',
  ghost: 'bg-transparent text-slate-300/80 hover:bg-white/5 hover:text-white disabled:text-slate-600',
}

export const Button = forwardRef<HTMLButtonElement, PropsWithChildren<ButtonProps>>(function Button(
  { className, variant = 'secondary', type = 'button', children, ...props },
  ref,
) {
  return (
    <button
      ref={ref}
      type={type}
      className={cn(
        'inline-flex items-center justify-center gap-2 rounded-xl px-4 py-2.5 text-sm font-medium transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-300/40 disabled:cursor-not-allowed disabled:opacity-70',
        variants[variant],
        className,
      )}
      {...props}
    >
      {children}
    </button>
  )
})
