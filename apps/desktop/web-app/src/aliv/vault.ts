import { isPlatformTauri } from '@/lib/platform/utils'
import { DemoVaultService } from './demo-service'
import type { VaultService } from './service'
import { TauriVaultService } from './tauri-service'

let instance: VaultService | null = null

/** The desktop vault in the Tauri app; the seeded demo vault in a browser. */
export function getVault(): VaultService {
  if (!instance) {
    instance = isPlatformTauri() ? new TauriVaultService() : new DemoVaultService()
  }
  return instance
}

/** Test hook. */
export function setVault(vault: VaultService | null) {
  instance = vault
}
