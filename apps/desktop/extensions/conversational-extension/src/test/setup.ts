import { vi } from 'vitest'

// The extension delegates every method to window.core.api.*; a base class is
// the only value-level import it needs from @aliv/core (the rest are types).
vi.mock('@aliv/core', () => ({
  ConversationalExtension: class {
    onLoad() {}
    onUnload() {}
  },
}))
