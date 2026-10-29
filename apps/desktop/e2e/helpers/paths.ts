import { join } from 'node:path'

/**
 * The directory the Rust side resolves for app data, inside a throwaway profile.
 *
 * Mirrors `app_data_dir_with_fallback()` -> `data_dir()/ALIV`
 * (src-tauri/src/core/app/commands.rs, core/app/paths.rs). Kept as a plain
 * function of the test HOME so specs can assert against real on-disk paths
 * rather than against the environment the harness itself injected.
 */
export function alivDataDir(testHome: string) {
  return process.platform === 'darwin'
    ? join(testHome, 'Library/Application Support/ALIV')
    : process.platform === 'win32'
      ? join(testHome, 'AppData/Roaming/ALIV')
      : join(testHome, '.local/share/ALIV')
}

/**
 * The folder ALIV stores user data in -- threads, and everything else the Rust
 * side writes per-thread.
 *
 * Not the same directory as alivDataDir(): that one holds `settings.json`, and
 * `settings.json` in turn points `data_folder` at this subdirectory. The Rust
 * thread code takes that folder as an argument
 * (core/threads/utils.rs get_thread_dir), so a spec checking for a thread on
 * disk has to go through the extra `data` segment or it asserts against a path
 * that never existed -- which passes for the wrong reason when the assertion is
 * that something is absent.
 */
export function alivUserDataDir(testHome: string) {
  return join(alivDataDir(testHome), 'data')
}
