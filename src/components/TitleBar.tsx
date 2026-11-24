import { useEffect, useState } from 'react'
import { Minus, Square, X, Copy } from 'lucide-react'
export default function TitleBar() {
  const [max, setMax] = useState(false)
  useEffect(() => window.api.win.onState(setMax), [])
  const b = 'h-9 w-11 text-txt2 hover:text-white'
  return (
    <header data-tauri-drag-region className="drag flex h-9 shrink-0 items-center justify-between border-b border-line bg-black pl-4">
      <span data-tauri-drag-region className="pointer-events-none text-[10px] tracking-[.35em] text-muted">MEMORIES ROLEPLAY LAUNCHER</span>
      <div className="nodrag flex">
        <button onClick={() => window.api.win.minimize()} className={`${b} hover:bg-elev`}><Minus size={14} className="mx-auto" /></button>
        <button onClick={() => window.api.win.toggleMax()} className={`${b} hover:bg-elev`}>{max ? <Copy size={12} className="mx-auto" /> : <Square size={12} className="mx-auto" />}</button>
        <button onClick={() => window.api.win.close()} className={`${b} hover:bg-danger`}><X size={14} className="mx-auto" /></button>
      </div>
    </header>)
}
