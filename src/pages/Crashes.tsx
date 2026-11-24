import { useCallback, useEffect, useState } from 'react'
import { Panel, Btn } from '../components/ui'
export default function Crashes() {
  const [data, setData] = useState<{ exists: boolean; total: number; items: CrashItem[] } | null>(null)
  const [msg, setMsg] = useState(''); const [busy, setBusy] = useState('')
  const load = useCallback(async () => { try { setData(await window.api.crash.list()); setMsg('') } catch (e) { setMsg((e as Error).message) } }, [])
  useEffect(() => { load() }, [load])
  const act = async (id: string, kind: 'open' | 'copy' | 'zip') => {
    setBusy(id + kind); setMsg(kind === 'zip' ? 'Menyiapkan dan mengompresi file…' : '')
    try {
      if (kind === 'open') await window.api.crash.open(id)
      else if (kind === 'copy') { await navigator.clipboard.writeText(await window.api.crash.info(id)); setMsg('Info crash disalin.') }
      else { const r = await window.api.crash.zip(id); setMsg(`ZIP selesai: ${r.path}${r.skipped.length ? ` (tidak tersedia: ${r.skipped.join(', ')})` : ''}`) }
    } catch (e) { setMsg(`Gagal: ${(e as Error).message}`) } finally { setBusy('') }
  }
  return (
    <div className="space-y-3">
      <div className="flex items-end justify-between"><div className="text-lg font-semibold">Riwayat Crash</div>
        <div className="flex items-center gap-3"><span className="border border-line px-2 py-0.5 text-[10px] text-txt2">{data?.total ?? 0} CRASH</span><Btn onClick={load}>MUAT ULANG</Btn></div></div>
      {msg && <div className="break-all border border-line px-3 py-2 text-xs text-txt2">{msg}</div>}
      {data && !data.items.length && <Panel className="p-6 text-sm text-txt2">{data.exists ? 'No Crash Found.' : 'Folder crashes tidak ditemukan.'}</Panel>}
      <div className="space-y-px">
        {data?.items.map(c => (
          <Panel key={c.id} className="flex items-center gap-3 px-4 py-3">
            <div className="min-w-0 flex-1"><div className="truncate text-sm">{c.title}</div><div className="text-[11px] text-muted">{new Date(c.time).toLocaleString('id-ID')}{c.hasLog ? '' : ' · gamelog tidak ada'}</div></div>
            <Btn disabled={!!busy} onClick={() => act(c.id, 'open')}>EXPLORER</Btn>
            <Btn disabled={!!busy} onClick={() => act(c.id, 'copy')}>COPY</Btn>
            <Btn disabled={!!busy} onClick={() => act(c.id, 'zip')}>{busy === c.id + 'zip' ? 'MENGOMPRES…' : 'BUAT ZIP'}</Btn>
          </Panel>))}
      </div>
      <p className="text-[11px] text-muted">ZIP berisi file dump, gamelog, dan ringkasan crash, disimpan di folder Downloads. File dump bisa memuat data memori game; kirim hanya ke tim support. Hanya 200 crash terbaru ditampilkan.</p>
    </div>)
}
