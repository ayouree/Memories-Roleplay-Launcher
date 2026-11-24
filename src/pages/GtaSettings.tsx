import { useCallback, useEffect, useState } from 'react'
import { Panel, Btn } from '../components/ui'
import { SPEC, GROUPS, optionsOf, optLabel, validValue } from '../shared/gtaSpec'
const sel = 'w-60 border border-line bg-input px-3 py-1.5 text-sm outline-none focus:border-accent disabled:opacity-40'
export default function GtaSettings() {
  const [d, setD] = useState<GtaData | null>(null); const [vals, setVals] = useState<Record<string, string | null>>({})
  const [msg, setMsg] = useState(''); const [modal, setModal] = useState<string | null>(null)
  const [prev, setPrev] = useState<{ id: string; label: string; from: string; to: string }[] | null>(null); const [perr, setPerr] = useState('')
  const load = useCallback(async () => { try { const x = await window.api.gta.load(); setD(x); setVals(x.values); setMsg('') } catch (e) { setMsg((e as Error).message) } }, [])
  useEffect(() => { load() }, [load])
  const dirty = !!d && SPEC.some(i => vals[i.id] !== null && vals[i.id] !== d.values[i.id])
  const save = async () => { if (!d) return; try { const x = await window.api.gta.save({ baseHash: d.hash, values: vals }); setD(x); setVals(x.values); setMsg('Tersimpan. Backup dibuat dan hasil diverifikasi.') } catch (e) { setMsg((e as Error).message) } }
  const copy = async () => {
    const settings = Object.fromEntries(SPEC.filter(i => vals[i.id] !== null && vals[i.id] !== undefined).map(i => [i.id, vals[i.id]]))
    try { await navigator.clipboard.writeText(JSON.stringify({ app: 'memories-launcher', type: 'gta5-graphics', version: 1, settings })); setMsg('Kode disalin ke clipboard.') } catch { setMsg('Gagal menyalin ke clipboard.') }
  }
  const preview = () => {
    setPerr(''); setPrev(null)
    try {
      const j = JSON.parse(modal ?? ''); if (j?.app !== 'memories-launcher' || j?.type !== 'gta5-graphics' || typeof j.settings !== 'object' || !j.settings) throw new Error('Format kode tidak dikenali.')
      const out: { id: string; label: string; from: string; to: string }[] = []
      for (const [k, v] of Object.entries(j.settings)) {
        const it = SPEC.find(s => s.id === k); if (!it || vals[k] === null || vals[k] === undefined) continue
        if (typeof v !== 'string' || !validValue(it, v, vals[k] as string)) throw new Error(`Nilai tidak valid untuk ${k}.`)
        if (v !== vals[k]) out.push({ id: k, label: it.label, from: optLabel(it, vals[k] as string), to: optLabel(it, v) })
      }
      setPrev(out); if (!out.length) setPerr('Tidak ada perubahan.')
      ;(window as unknown as { __paste: unknown }).__paste = j.settings
    } catch (e) { setPerr(e instanceof SyntaxError ? 'Kode bukan JSON yang valid.' : (e as Error).message) }
  }
  const apply = () => { const s = (window as unknown as { __paste: Record<string, string> }).__paste; const n = { ...vals }; prev?.forEach(p => { n[p.id] = s[p.id] }); setVals(n); setModal(null); setPrev(null); setMsg('Kode diterapkan ke daftar. Klik SIMPAN untuk menulis ke file.') }
  if (d && !d.exists) return <Panel className="p-6 text-sm text-warn">gta5_settings.xml tidak ditemukan di {d.path}. Jalankan FiveM/GTA V sekali agar file dibuat.</Panel>
  if (d && !d.valid) return <Panel className="p-6 text-sm text-danger">gta5_settings.xml tampak rusak. Launcher tidak akan menimpanya.</Panel>
  return (
    <div className="space-y-4">
      <div className="flex items-end justify-between">
        <div><div className="text-[10px] tracking-[.3em] text-muted">{d?.gpu || 'GPU TIDAK TERDETEKSI'}</div><div className="text-lg font-semibold">Grafis GTA V</div></div>
        <div className="flex gap-2"><Btn onClick={copy}>SALIN KODE</Btn><Btn onClick={() => { setModal(''); setPrev(null); setPerr('') }}>TEMPEL KODE</Btn><Btn onClick={load}>MUAT ULANG</Btn><Btn disabled={!dirty} onClick={save}>SIMPAN</Btn></div>
      </div>
      {msg && <div className="border border-line px-3 py-2 text-xs text-txt2">{msg}</div>}
      {GROUPS.map(g => (
        <Panel key={g} className="p-5">
          <div className="mb-3 text-[10px] tracking-[.3em] text-muted">{g}</div>
          <div className="space-y-2">
            {SPEC.filter(i => i.group === g).map(it => {
              const cur = vals[it.id]; const unsupported = cur === null || cur === undefined
              const base = optionsOf(it); const extra = !unsupported && !base.some(o => o[0] === cur) ? [[cur as string, optLabel(it, cur as string)]] : []
              return (
                <div key={it.id} className="flex items-center justify-between text-sm">
                  <span className={cur !== d?.values[it.id] && !unsupported ? 'text-white' : 'text-txt2'}>{it.label}{!unsupported && cur !== d?.values[it.id] ? ' •' : ''}</span>
                  <select className={sel} disabled={unsupported} value={unsupported ? '' : (cur as string)} onChange={e => setVals({ ...vals, [it.id]: e.target.value })}>
                    {unsupported && <option value="">Tidak didukung</option>}
                    {[...base, ...extra].map(o => <option key={o[0]} value={o[0]}>{o[1]}{o[2] ? ' (belum diverifikasi)' : ''}</option>)}
                  </select>
                </div>)
            })}
          </div>
        </Panel>))}
      {modal !== null && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70">
          <Panel className="w-[520px] bg-bg2 p-6">
            <div className="text-sm font-semibold tracking-wide">TEMPEL KODE GRAFIS</div>
            <textarea className="mt-3 h-28 w-full border border-line bg-input p-2 text-xs outline-none" placeholder="Tempel kode (Ctrl+V)…" value={modal} onChange={e => { setModal(e.target.value); setPrev(null); setPerr('') }} />
            {perr && <p className="mt-2 text-xs text-warn">{perr}</p>}
            {prev && prev.length > 0 && <ul className="mt-3 max-h-40 space-y-1 overflow-y-auto text-xs text-txt2">{prev.map(p => <li key={p.id} className="flex justify-between"><span>{p.label}</span><span>{p.from} → {p.to}</span></li>)}</ul>}
            <div className="mt-4 flex justify-end gap-2"><Btn onClick={() => setModal(null)}>BATAL</Btn><Btn onClick={preview}>PRATINJAU</Btn><Btn disabled={!prev?.length} onClick={apply}>TERAPKAN</Btn></div>
          </Panel></div>)}
    </div>)
}
