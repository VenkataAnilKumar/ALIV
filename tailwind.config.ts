import type { Config } from 'tailwindcss'

export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Space Grotesk', 'Segoe UI', 'sans-serif'],
        mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
      },
      colors: {
        surface: {
          app: '#08111a',
          base: '#101c29',
          raised: '#122233',
          overlay: '#193248',
        },
        border: {
          default: '#24425d',
          subtle: '#173148',
        },
        accent: {
          primary: '#53d3d1',
          hover: '#7be5e0',
          warm: '#ff9d5c',
          warmHover: '#ffb987',
        },
      },
      boxShadow: {
        glow: '0 0 0 1px rgba(83,211,209,0.26), 0 24px 60px rgba(10,26,38,0.35)',
        panel: '0 18px 45px rgba(2,10,18,0.28)',
      },
      keyframes: {
        shimmer: {
          '0%': { backgroundPosition: '-200% 0' },
          '100%': { backgroundPosition: '200% 0' },
        },
        'fade-in': {
          '0%': { opacity: '0', transform: 'translateY(8px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        'slide-down': {
          '0%': { opacity: '0', transform: 'translateY(-8px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
      },
      animation: {
        shimmer: 'shimmer 1.5s linear infinite',
        'fade-in': 'fade-in 150ms ease-out',
        'slide-down': 'slide-down 200ms ease-out',
      },
    },
  },
  plugins: [],
} satisfies Config
