import { useCallback, useEffect, useRef, useState } from 'react'
import { Trash2, Plus } from 'lucide-react'
import { Panel, Btn } from '../components/ui'
type Row = KeyEntry & { id: number }
const inp = 'w-full border border-line bg-input px-3 py-2 text-sm outline-none focus:border-accent'
export default function FiveMSettings() {
  const [nick, setNick] = useState(''); const [nloaded, setNloaded] = useState(''); const [nfound, setNfound] = useState(false); const [nmsg, setNmsg] = useState('')
  const [data, setData] = useState<KeybindData | null>(null); const [rows, setRows] = useState<Row[]>([])
  const [dirty, setDirty] = useState(false); const [msg, setMsg] = useState('')
  const [method, setMethod] = useState<'explorer' | 'direct'>('explorer')
  const nid = useRef(1)
  const load = useCallback(async () => {
    try { const d = await window.api.keybind.load(); setData(d); setRows([...d.entries, ...d.disabled].map(e => ({ ...e, id: nid.current++ }))); setDirty(false); setMsg('') }
    catch (e) { setMsg((e as Error).message) }
  }, [])
  const applyNick = (n: NickInfo) => { setNfound(n.found); setNick(n.nickname); setNloaded(n.nickname) }
  const loadNick = useCallback(async () => { try { const n = await window.api.nickname.get(); applyNick(n); setNmsg(n.found ? '' : n.detail) } catch (e) { setNmsg((e as Error).message) } }, [])
  useEffect(() => { load(); loadNick(); window.api.config.get().then(c => setMethod(c.connectMethod ?? 'explorer')).catch(() => {}) }, [load, loadNick])
  const upd = (id: number, p: Partial<KeyEntry>) => { setRows(r => r.map(x => x.id === id ? { ...x, ...p } : x)); setDirty(true) }
  const byKey = new Map<string, Set<string>>()
  rows.filter(r => r.enabled).forEach(r => { const k = r.key.toUpperCase(); byKey.set(k, (byKey.get(k) ?? new Set()).add(r.command)) })
  const dups = [...byKey].filter(([, s]) => s.size > 1).map(([k]) => k)
  const saveNick = async () => { try { const n = await window.api.nickname.set(nick.trim()); applyNick(n); setNmsg('Nickname tersimpan di data FiveM (backup database dibuat). Berlaku saat FiveM dibuka lagi.') } catch (e) { setNmsg((e as Error).message) } }
  const save = async () => {
    if (!data) return
    try { const d = await window.api.keybind.save({ baseHash: data.hash, entries: rows.map(({ id, ...e }) => ({ ...e, key: e.key.trim() })) }); setData(d); setRows([...d.entries, ...d.disabled].map(e => ({ ...e, id: nid.current++ }))); setDirty(false); setMsg('Tersimpan. Backup fivem.cfg dibuat dan hasil sudah diverifikasi.') }
    catch (e) { setMsg((e as Error).message) }
  }
  return (
    <div className="space-y-6">
      <Panel className="p-6">
        <div className="mb-3 text-[10px] tracking-[.3em] text-muted">METODE CONNECT</div>
        <select className={inp} value={method} onChange={async e => { const v = e.target.value as 'explorer' | 'direct'; setMethod(v); try { await window.api.config.set({ connectMethod: v }) } catch { /* abaikan */ } }}>
          <option value="explorer">Standar (lewat Explorer, disarankan)</option>
          <option value="direct">Langsung (FiveM.exe +connect)</option>
        </select>
        <p className="mt-2 text-[11px] text-muted">Jika FiveM menolak dengan pesan "should be launched directly from the shell", ganti metode.</p>
      </Panel>
      <Panel className="p-6">
        <div className="mb-3 text-[10px] tracking-[.3em] text-muted">NICKNAME</div>
        <div className="flex gap-2"><input className={inp} value={nick} maxLength={32} disabled={!nfound} onChange={e => setNick(e.target.value)} placeholder={nfound ? 'Nama pemain FiveM' : 'Nickname belum terbaca'} />
          <Btn disabled={!nfound || !nick.trim() || nick.trim() === nloaded} onClick={saveNick}>SIMPAN</Btn></div>
        <p className="mt-2 text-[11px] text-muted">Dibaca langsung dari data FiveM. Tutup FiveM sebelum menyimpan.</p>
        {nmsg && <p className="mt-2 text-[11px] text-txt2">{nmsg}</p>}
      </Panel>
      <Panel className="p-6">
        <div className="mb-3 flex items-center justify-between"><div className="text-[10px] tracking-[.3em] text-muted">GLOBAL KEYBINDS</div><span className="border border-line px-2 py-0.5 text-[10px] text-txt2">{rows.filter(r => r.enabled).length} BIND</span></div>
        {data && !data.exists && <p className="text-xs text-warn">fivem.cfg tidak ditemukan di {data.path}. Jalankan FiveM sekali agar file dibuat.</p>}
        {dups.length > 0 && <p className="mb-3 border border-warn px-3 py-2 text-xs text-warn">Peringatan: tombol {dups.join(', ')} dipakai oleh lebih dari satu perintah. Tidak ada yang dihapus otomatis; periksa kembali.</p>}
        <div className="space-y-2">
          {rows.map(r => (
            <div key={r.id} className={`grid grid-cols-[110px_130px_1fr_auto_auto] items-center gap-2 ${r.enabled ? '' : 'opacity-50'}`}>
              <div className="text-xs text-muted">Keyboard</div>
              <input className={inp} value={r.key} onChange={e => upd(r.id, { key: e.target.value.toUpperCase() })} />
              <input className={inp} value={r.command} onChange={e => upd(r.id, { command: e.target.value })} />
              <button onClick={() => upd(r.id, { enabled: !r.enabled })} className={`w-14 border px-2 py-2 text-[10px] tracking-widest ${r.enabled ? 'border-accent text-accent' : 'border-line text-muted'}`}>{r.enabled ? 'ON' : 'OFF'}</button>
              <button onClick={() => { setRows(x => x.filter(y => y.id !== r.id)); setDirty(true) }} className="p-2 text-muted hover:text-danger"><Trash2 size={15} /></button>
            </div>))}
        </div>
        <p className="mt-3 text-[11px] text-muted">Keybind OFF disimpan di launcher dan dicabut dari fivem.cfg saat SIMPAN. Baris rbind dan bind non-KEYBOARD tidak ditampilkan dan tidak diubah.</p>
        <div className="mt-4 flex items-center gap-2">
          <Btn disabled={!data?.exists} onClick={() => { setRows(r => [...r, { id: nid.current++, origLine: null, key: '', command: '', enabled: true }]); setDirty(true) }}><span className="inline-flex items-center gap-1"><Plus size={12} />TAMBAH</span></Btn>
          <Btn disabled={!dirty} onClick={save}>SIMPAN KEYBIND</Btn><Btn onClick={load}>MUAT ULANG</Btn>
          <span className="text-xs text-txt2">{msg}</span>
        </div>
      </Panel>
    </div>)
}
