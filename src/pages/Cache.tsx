import { useCallback, useEffect, useState } from 'react'
import { Panel, Heading, Btn } from '../components/ui'
const META = {
  cache: { name: 'Cache FiveM', desc: 'Cache launcher dan browser. Dibuat ulang otomatis saat FiveM dibuka.' },
  server: { name: 'Cache aset server', desc: 'Aset server yang pernah dimasuki. Jika dihapus, perlu diunduh ulang.' }
}
const STATUS: Record<string, string> = { OK: 'Ditemukan', NOT_FOUND: 'Not Found', DENIED: 'Akses ditolak', ERROR: 'Error' }
export const fmt = (b: number) => b >= 1073741824 ? `${(b / 1073741824).toFixed(2)} GB` : `${(b / 1048576).toFixed(2)} MB`
export default function CachePage() {
  const [items, setItems] = useState<CacheInfo[]>([])
  const [sel, setSel] = useState<Record<string, boolean>>({ cache: true, server: false })
  const [busy, setBusy] = useState(''); const [confirm, setConfirm] = useState(false); const [msg, setMsg] = useState('')
  const scan = useCallback(async () => { setBusy('scan'); try { setItems(await window.api.cache.scan()); setMsg('') } catch (e) { setMsg((e as Error).message) } finally { setBusy('') } }, [])
  useEffect(() => { scan() }, [scan])
  const chosen = items.filter(i => sel[i.id] && i.status === 'OK'); const total = chosen.reduce((a, b) => a + b.size, 0)
  const run = async () => {
    setConfirm(false); setBusy('clear'); setMsg('Menghapus…')
    try {
      const r = await window.api.cache.clear(chosen.map(c => c.id)); const fail = r.reduce((a, b) => a + b.failed.length, 0)
      await scan(); setMsg(fail ? `Selesai SEBAGIAN: ${fail} item gagal dihapus (kemungkinan sedang digunakan). Detail di cache-audit.log.` : 'Cache berhasil dihapus.')
    } catch (e) { setMsg((e as Error).message) } finally { setBusy('') }
  }
  return (
    <div className="p-8">
      <Heading kicker="PEMBERSIHAN" right={<div className="text-right"><div className="text-[9px] tracking-[.2em] text-muted">DIPILIH</div><div className="text-lg tabular-nums">{fmt(total)}</div></div>}>Kelola cache</Heading>
      <div className="grid grid-cols-2 gap-4">
        {items.map(i => (
          <Panel key={i.id} className={`p-5 transition-colors ${sel[i.id] ? 'border-accent' : 'hover:border-muted'}`}>
            <label className="block cursor-pointer">
              <div className="flex items-center justify-between"><span className="text-sm font-semibold">{META[i.id].name}</span>
                <input type="checkbox" className="h-4 w-4 accent-white" checked={!!sel[i.id]} disabled={i.status !== 'OK'} onChange={e => setSel({ ...sel, [i.id]: e.target.checked })} /></div>
              <p className="mt-2 min-h-[32px] text-xs text-txt2">{META[i.id].desc}</p>
              <div className="mt-4 text-2xl font-light tabular-nums">{i.status === 'OK' ? fmt(i.size) : '–'}</div>
              <div className="mt-1 truncate text-[10px] text-muted" title={i.dir}>{STATUS[i.status]} · {i.dir}</div>
            </label></Panel>))}
      </div>
      <div className="mt-6 flex items-center gap-3">
        <Btn danger disabled={!chosen.length || !!busy} onClick={() => setConfirm(true)}>HAPUS TERPILIH</Btn>
        <Btn disabled={!!busy} onClick={scan}>{busy === 'scan' ? 'MENGHITUNG…' : 'HITUNG ULANG'}</Btn>
        <span className="text-xs text-txt2">{msg}</span>
      </div>
      {confirm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70">
          <Panel className="w-[420px] bg-bg2 p-6">
            <div className="text-sm font-semibold tracking-wide">KONFIRMASI HAPUS CACHE</div>
            <ul className="mt-3 space-y-1 text-xs text-txt2">{chosen.map(c => <li key={c.id} className="flex justify-between"><span>{META[c.id].name}</span><span>{fmt(c.size)}</span></li>)}
              <li className="flex justify-between border-t border-line pt-1 text-white"><span>Total</span><span>{fmt(total)}</span></li></ul>
            {sel.server && <p className="mt-3 text-xs text-warn">Aset server mungkin perlu diunduh ulang.</p>}
            <div className="mt-5 flex justify-end gap-2"><Btn onClick={() => setConfirm(false)}>BATAL</Btn><Btn danger onClick={run}>HAPUS SEKARANG</Btn></div>
          </Panel></div>)}
    </div>)
}
