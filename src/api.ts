// Jembatan UI -> Rust (Tauri). Antarmuka window.api dipertahankan agar halaman UI tidak perlu diubah.
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { getVersion } from '@tauri-apps/api/app'
async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try { return await invoke<T>(cmd, args) } catch (e) { throw new Error(typeof e === 'string' ? e : (e as Error)?.message ?? String(e)) }
}
window.api = {
  appInfo: async () => ({ version: await getVersion() }),
  win: {
    minimize: () => { void getCurrentWindow().minimize() },
    toggleMax: () => { void getCurrentWindow().toggleMaximize() },
    close: () => { void getCurrentWindow().close() },
    onState: cb => {
      const w = getCurrentWindow(); let un: (() => void) | null = null
      w.onResized(() => { w.isMaximized().then(cb) }).then(u => { un = u })
      return () => { un?.() }
    }
  },
  gate: { measure: () => call('gate_measure'), info: () => call('gate_info') },
  fivem: { detect: () => call('fivem_detect'), browse: () => call('fivem_browse'), connect: id => call('fivem_connect', { id }) },
  cache: { scan: () => call('cache_scan'), clear: ids => call('cache_clear', { ids }) },
  keybind: { load: () => call('keybind_load'), save: payload => call('keybind_save', { payload }) },
  gta: { load: () => call('gta_load'), save: payload => call('gta_save', { payload }) },
  crash: { list: () => call('crash_list'), info: id => call('crash_info', { id }), open: id => call('crash_open', { id }), zip: id => call('crash_zip', { id }) },
  link: { open: key => call('link_open', { key }) },
  community: { discord: () => call('community_discord') },
  nickname: { get: () => call('nickname_get'), set: name => call('nickname_set', { name }) },
  config: { get: () => call('config_get'), set: patch => call('config_set', { patch }) }
}
