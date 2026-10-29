import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    globals: true,
    css: true,
    testTimeout: 30000,
    hookTimeout: 30000,
    coverage: {
      reporter: ['text', 'json', 'html', 'lcov'],
      include: ['src/**/*.{ts,tsx}'],
      exclude: [
        'node_modules/',
        'dist/',
        'coverage/',
        'src/**/*.test.ts',
        'src/**/*.test.tsx',
        'src/test/**/*',
      ],
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@aliv/assistant-extension': path.resolve(__dirname, '../extensions/assistant-extension/dist/index.js'),
      '@aliv/conversational-extension': path.resolve(__dirname, '../extensions/conversational-extension/dist/index.js'),
      '@aliv/download-extension': path.resolve(__dirname, '../extensions/download-extension/dist/index.js'),
      '@aliv/llamacpp-extension': path.resolve(__dirname, '../extensions/llamacpp-extension/dist/index.js'),
      '@aliv/mlx-extension': path.resolve(__dirname, '../extensions/mlx-extension/dist/index.js'),
      '@aliv/rag-extension': path.resolve(__dirname, '../extensions/rag-extension/dist/index.js'),
      '@aliv/vector-db-extension': path.resolve(__dirname, '../extensions/vector-db-extension/dist/index.js'),
      // Resolve the plugin to guest-js source, not its built dist-js, so
      // tests exercise the wrappers the app actually calls.
      '@aliv/tauri-plugin-llamacpp-api': path.resolve(__dirname, '../src-tauri/plugins/tauri-plugin-llamacpp/guest-js/index.ts'),
      // The plugin ships its own @tauri-apps/api install; without this the
      // aliased guest-js would get a second module instance that
      // vi.mock('@tauri-apps/api/core') cannot intercept.
      '@tauri-apps/api/core': path.resolve(__dirname, './node_modules/@tauri-apps/api/core.js'),
    },
  },
  define: {
    IS_TAURI: JSON.stringify(false),
    IS_WEB_APP: JSON.stringify(false),
    IS_MACOS: JSON.stringify(false),
    IS_WINDOWS: JSON.stringify(false),
    IS_LINUX: JSON.stringify(false),
    IS_IOS: JSON.stringify(false),
    IS_ANDROID: JSON.stringify(false),
    PLATFORM: JSON.stringify('web'),
    VERSION: JSON.stringify('test'),
    POSTHOG_KEY: JSON.stringify(''),
    POSTHOG_HOST: JSON.stringify(''),
    AUTO_UPDATER_DISABLED: JSON.stringify('false'),
  },
})
