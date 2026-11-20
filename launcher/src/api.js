// One place for every call into the Rust side. In a plain browser (design preview) a small mock answers instead.
import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { listen as tauriListen } from '@tauri-apps/api/event'
import { open as tauriOpen } from '@tauri-apps/plugin-dialog'

export const inTauri = typeof window !== 'undefined' && !!window.__TAURI_INTERNALS__

const mockStatus = {
  client_dir: 'D:\\Games\\WoW 3.3.5a',
  server_dir: 'D:\\RA Realm\\server',
  setup_done: true,
  installed_version: '0.1.0',
  autologin: true,
  mysql_running: true,
  server_running: false,
  launcher_version: '0.1.0',
}
const mockChars = [
  { name: 'Aldrin', race: 'Human', class: 'Mage', class_id: 8, level: 10, playtime_hours: 1.4, online: false, achievements: 2 },
  { name: 'Brakka', race: 'Orc', class: 'Warrior', class_id: 1, level: 7, playtime_hours: 0.6, online: false, achievements: 1 },
  { name: 'Sylwen', race: 'Night Elf', class: 'Druid', class_id: 11, level: 3, playtime_hours: 0.2, online: false, achievements: 0 },
]
async function mockInvoke(cmd, args) {
  await new Promise((r) => setTimeout(r, 250))
  switch (cmd) {
    case 'get_status':
    case 'set_client_folder':
    case 'set_server_folder':
    case 'set_autologin':
      return mockStatus
    case 'check_update':
      return { available: false, installed: '0.1.0', latest: '0.1.0', size: 0, error: null }
    case 'login':
      return { username: String(args.user).toUpperCase(), gm_level: 3, characters: mockChars }
    case 'characters':
      return mockChars
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
