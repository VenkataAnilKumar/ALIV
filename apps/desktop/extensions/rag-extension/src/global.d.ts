import type { BaseExtension, ExtensionTypeEnum } from '@aliv/core'

declare global {
  interface Window {
    core?: {
      extensionManager: {
        get<T = BaseExtension>(type: ExtensionTypeEnum): T | undefined
        getByName(name: string): BaseExtension | undefined
      }
    }
  }
}

export {}
