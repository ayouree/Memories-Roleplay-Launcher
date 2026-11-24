import { useCallback, useEffect, useRef, useState } from 'react'
import { GATES, SOCIALS } from '../config/gates'
import { Panel, Heading, Btn, Spark } from '../components/ui'
import logo from '../assets/logo.png'

const M = ({ k, v, u }: { k: string; v: number | null; u: string }) => (
  <div className="w-16 text-right"><div className="text-[9px] tracking-[.2em] text-muted">{k}</div>
    <div className="text-sm tabular-nums">{v === null ? 'N/A' : v}{v !== null && <span className="text-[10px] text-txt2"> {u}</span>}</div></div>)
const score = (r: GateResult) => r.status === 'ONLINE' && r.ping !== null && r.jitter !== null && r.loss !== null ? r.loss * 50 + r.ping + r.jitter * 2 : null
const STALE_MS = 20000

export default function Gate() {
  const [res, setRes] = useState<Record<string, GateResult>>({})
  const [hist, setHist] = useState<Record<string, number[]>>({})
  const [busy, setBusy] = useState(false)
  const [connecting, setConnecting] = useState('')
  const [msg, setMsg] = useState('')
  const [now, setNow] = useState(Date.now())
  const alive = useRef(true)
  const running = useRef(false)
  const [dc, setDc] = useState<{ name: string; members: number; online: number } | null>(null)
  useEffect(() => { window.api.community.discord().then(setDc).catch(() => {}) }, [])
  const refresh = useCallback(async (manual = false) => {
    if (manual) setBusy(true)
    try {
      const r = await window.api.gate.measure()
      if (!alive.current) return
      setRes(Object.fromEntries(r.map(x => [x.id, x])))
      setHist(h => { const n = { ...h }; r.forEach(x => { if (x.ping !== null) n[x.id] = [...(n[x.id] ?? []), x.ping].slice(-12) }); return n })
    } catch (e) { setMsg((e as Error).message) } finally { if (manual && alive.current) setBusy(false) }
  }, [])
  useEffect(() => {
    alive.current = true; let t: ReturnType<typeof setTimeout>
    const loop = async () => {
      if (running.current) return
      running.current = true
      try { if (!document.hidden) await refresh() } finally { running.current = false; if (alive.current) t = setTimeout(loop, 5000) }
    }
    const vis = () => { if (!document.hidden) { clearTimeout(t); loop() } }
    loop(); const c = setInterval(() => setNow(Date.now()), 5000); document.addEventListener('visibilitychange', vis)
    return () => { alive.current = false; clearTimeout(t); clearInterval(c); document.removeEventListener('visibilitychange', vis) }
  }, [refresh])

  const scores = GATES.map(g => ({ id: g.id, s: res[g.id] ? score(res[g.id]) : null })).filter(x => x.s !== null) as { id: string; s: number }[]
  const best = scores.length >= 2 ? scores.sort((a, b) => a.s - b.s)[0].id : scores.length === 1 ? scores[0].id : ''
  const last = Object.values(res)[0]?.at

  const connect = async (id: string) => {
    setConnecting(id); setMsg('')
    try { const r = await window.api.fivem.connect(id); setMsg(r.running ? 'Perintah koneksi dikirim ke FiveM yang sedang berjalan.' : 'FiveM diluncurkan. Memeriksa koneksi dilakukan oleh FiveM.') }
    catch (e) { setMsg((e as Error).message) } finally { setConnecting('') }
  }
  return (
    <div className="p-8">
      <Panel className="relative mb-8 overflow-hidden px-8 py-7">
        <img src={logo} alt="" className="pointer-events-none absolute -right-8 -top-10 h-56 w-56 object-contain opacity-[0.07]" />
        <div className="text-[10px] tracking-[.35em] text-muted">SELAMAT DATANG</div>
        <div className="mt-1 text-xl font-semibold">Pilih rute tercepat menuju kota.</div>
        <p className="mt-2 max-w-md text-xs leading-relaxed text-txt2">Diukur dengan TCP dari PC kamu ke tiap gerbang, jadi bisa berbeda dengan ping di dalam game.</p>
      </Panel>
      <Heading kicker="RUTE MASUK" right={<div className="flex items-center gap-3"><span className="text-[10px] text-muted">{last ? `Diukur ${new Date(last).toLocaleTimeString('id-ID')}` : ''}</span><Btn onClick={() => refresh(true)} disabled={busy}>{busy ? 'MENGUKUR…' : 'UKUR ULANG'}</Btn></div>}>Gerbang tersedia</Heading>
      {msg && <div className="mb-3 border border-line px-3 py-2 text-xs text-txt2">{msg}</div>}
      <div className="space-y-2">
        {GATES.map((g, i) => {
          const r = res[g.id]; const stale = r && now - r.at > STALE_MS
          const status = !r ? 'CHECKING' : r.status
          return (
            <Panel key={g.id} className="flex items-center gap-5 px-5 py-4 transition-colors hover:border-muted">
              <div className="w-6 text-lg font-light text-muted">{i + 1}</div>
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-3 text-sm font-semibold">{g.name}
                  {best === g.id && <span className="border border-accent px-1.5 text-[9px] tracking-widest text-accent">RECOMMENDED</span>}
                  <span className={`text-[9px] tracking-widest ${status === 'ONLINE' ? 'text-ok' : status === 'CHECKING' ? 'text-muted' : status === 'OFFLINE' ? 'text-danger' : 'text-warn'}`}>{status}{stale ? ' · USANG' : ''}</span></div>
                <div className="truncate text-[11px] text-muted" title={r?.note}>{g.host}{r?.note ? ` · ${r.note}` : ''}</div>
              </div>
              <Spark data={hist[g.id] ?? []} />
              <M k="PING" v={r?.ping ?? null} u="ms" /><M k="JITTER" v={r?.jitter ?? null} u="ms" /><M k="LOSS" v={r?.loss ?? null} u="%" />
              <Btn onClick={() => connect(g.id)} disabled={!!connecting}>{connecting === g.id ? 'MELUNCURKAN…' : 'MASUK'}</Btn>
            </Panel>)
        })}
      </div>
      <div className="mt-8 grid grid-cols-2 gap-4">
        <Panel className="p-5"><div className="mb-2 text-[10px] tracking-[.3em] text-muted">KOMUNITAS</div>
          <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs">{SOCIALS.map(x => x.url
            ? <button key={x.key} onClick={() => window.api.link.open(x.key).catch(e => setMsg((e as Error).message))} className="text-txt2 hover:text-white hover:underline">{x.label}</button>
            : <span key={x.key} className="text-muted">{x.label} · belum diatur</span>)}</div></Panel>
        <Panel className="flex items-center justify-between p-5"><div><div className="text-sm font-semibold">{dc?.name ?? 'Discord Memories'}</div>
          <div className="text-[11px] text-muted">{dc ? `${dc.members.toLocaleString('id-ID')} member · ${dc.online.toLocaleString('id-ID')} online` : 'Data member tidak tersedia'}</div></div>
          <Btn onClick={() => window.api.link.open('discord').catch(e => setMsg((e as Error).message))}>GABUNG</Btn></Panel>
      </div>
    </div>)
}
