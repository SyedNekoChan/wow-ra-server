// One place for every call into the Rust side. In a plain browser (design preview) a small mock answers instead.
import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { listen as tauriListen } from '@tauri-apps/api/event'
import { open as tauriOpen } from '@tauri-apps/plugin-dialog'
import { getCurrentWindow } from '@tauri-apps/api/window'

export const inTauri = typeof window !== 'undefined' && !!window.__TAURI_INTERNALS__

const mockStatus = {
  client_dir: 'D:\\Games\\WoW 3.3.5a',
  server_dir: 'D:\\RA Realm\\server',
  setup_done: true,
  installed_version: '0.1.0',
  mysql_running: true,
  server_running: false,
  launcher_version: '0.1.0',
}
async function mockInvoke(cmd) {
  await new Promise((r) => setTimeout(r, 250))
  switch (cmd) {
    case 'get_status':
    case 'set_client_folder':
    case 'set_server_folder':
      return mockStatus
    case 'check_update':
      return { available: false, installed: '0.1.0', latest: '0.1.0', size: 0, error: null }
    default:
      return null
  }
}

export const invoke = inTauri ? tauriInvoke : mockInvoke
export const listen = inTauri ? tauriListen : async () => () => {}
export async function pickFolder(title) {
  if (!inTauri) return 'D:\\Games\\WoW 3.3.5a'
  const r = await tauriOpen({ directory: true, multiple: false, title })
  return typeof r === 'string' ? r : null
}

// Window buttons of the borderless window (the window has no system title bar). In a plain browser they do nothing.
export const minimizeWindow = () => (inTauri ? getCurrentWindow().minimize() : Promise.resolve())
export const closeWindow = () => (inTauri ? getCurrentWindow().close() : Promise.resolve())
