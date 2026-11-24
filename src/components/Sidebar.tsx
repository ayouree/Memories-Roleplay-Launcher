import { useCallback, useEffect, useState } from 'react'
import { Radio, HardDrive, SlidersHorizontal, Folder } from 'lucide-react'
import logo from '../assets/logo.png'
export type Tab = 'gate' | 'cache' | 'settings'
const NAV = [
  { id: 'gate' as Tab, label: 'Gerbang', icon: Radio },
  { id: 'cache' as Tab, label: 'Cache', icon: HardDrive },
  { id: 'settings' as Tab, label: 'Pengaturan', icon: SlidersHorizontal }
]
const shortPath = (p: string) => { const a = p.split('\\'); return a.length > 4 ? `${a[0]}\\…\\${a.slice(-2).join('\\')}` : p }
export default function Sidebar({ tab, setTab }: { tab: Tab; setTab: (t: Tab) => void }) {
  const [v, setV] = useState('1.0.0')
  const [fv, setFv] = useState<{ found: boolean; dir: string } | null>(null)
  const [online, setOnline] = useState<string>('–')
  const [err, setErr] = useState('')
  useEffect(() => {
    window.api.appInfo().then(i => setV(i.version)); window.api.fivem.detect().then(setFv).catch(() => setFv({ found: false, dir: '' }))
    const poll = () => { if (!document.hidden) window.api.gate.info().then(i => setOnline(i ? `${i.clients} / ${i.max}` : '–')).catch(() => setOnline('–')) }
    poll(); const t = setInterval(poll, 60000); return () => clearInterval(t)
  }, [])
  const browse = useCallback(async () => { try { setErr(''); setFv(await window.api.fivem.browse()) } catch (e) { setErr((e as Error).message) } }, [])
  return (
    <aside className="flex w-[210px] shrink-0 flex-col border-r border-line bg-black">
      <div className="flex justify-center px-4 py-7"><img src={logo} alt="Memories Roleplay" className="h-24 w-24 object-contain" /></div>
      <nav className="flex-1 space-y-1 px-3">
        {NAV.map(n => (
          <button key={n.id} onClick={() => setTab(n.id)}
            className={`flex w-full items-center gap-3 border-l-2 px-3 py-2.5 text-left text-sm transition-colors ${tab === n.id ? 'border-white bg-white/5 text-white' : 'border-transparent text-txt2 hover:bg-white/[0.03] hover:text-white'}`}>
            <n.icon size={16} strokeWidth={1.6} />{n.label}</button>))}
      </nav>
      <div className="space-y-2 border-t border-line p-4 text-[11px] text-muted">
        <div className="flex items-center gap-2" title={fv?.dir}><Folder size={12} className="shrink-0" />
          <span className="truncate">{fv?.found ? shortPath(fv.dir) : 'FiveM tidak ditemukan'}</span></div>
        <button onClick={browse} className="border border-line px-2 py-1 text-[10px] tracking-widest text-txt2 hover:border-accent hover:text-white">{fv?.found ? 'UBAH LOKASI' : 'PILIH LOKASI'}</button>
        {err && <div className="text-danger">{err}</div>}
        <div>Pemain online: {online}</div>
        <div className="text-[10px] tracking-widest">VERSI {v}</div>
      </div>
    </aside>)
}
