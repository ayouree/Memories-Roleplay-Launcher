import { ReactNode } from 'react'
export const Panel = ({ children, className = '' }: { children: ReactNode; className?: string }) =>
  <div className={`border border-line bg-white/[0.02] ${className}`}>{children}</div>
export const Heading = ({ kicker, children, right }: { kicker?: string; children: ReactNode; right?: ReactNode }) => (
  <div className="mb-6 flex items-end justify-between">
    <div>
      {kicker && <div className="mb-1 flex items-center gap-2 text-[10px] tracking-[.3em] text-muted"><span className="h-px w-6 bg-muted" />{kicker}</div>}
      <h1 className="text-2xl font-semibold tracking-tight">{children}</h1>
    </div>{right}
  </div>)
export const Btn = ({ children, onClick, danger, disabled, full }: { children: ReactNode; onClick?: () => void; danger?: boolean; disabled?: boolean; full?: boolean }) => (
  <button disabled={disabled} onClick={onClick}
    className={`${full ? 'w-full' : ''} border px-4 py-2 text-[11px] font-semibold tracking-[.2em] transition-colors disabled:cursor-not-allowed disabled:opacity-35 ${danger ? 'border-danger text-danger hover:bg-danger hover:text-white' : 'border-accent text-accent hover:bg-white hover:text-black'}`}>{children}</button>)
export const Spark = ({ data }: { data: number[] }) => {
  if (data.length < 2) return <span className="h-8 w-24 text-[10px] text-muted">…</span>
  const max = Math.max(...data), min = Math.min(...data), r = max - min || 1
  const pts = data.map((v, i) => `${(i / (data.length - 1)) * 100},${28 - ((v - min) / r) * 24}`).join(' ')
  return <svg viewBox="0 0 100 32" className="h-8 w-24" preserveAspectRatio="none"><polyline points={pts} fill="none" stroke="#D9D9D9" strokeWidth="1.5" vectorEffect="non-scaling-stroke" /></svg>
}
