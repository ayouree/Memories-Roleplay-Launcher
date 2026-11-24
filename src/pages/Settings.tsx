import { useState } from 'react'
import { Heading } from '../components/ui'
import FiveMSettings from './FiveMSettings'
import GtaSettings from './GtaSettings'
import Crashes from './Crashes'
const SUB = [{ id: 'fivem', label: 'FiveM' }, { id: 'gta', label: 'Grafis GTA V' }, { id: 'crash', label: 'Riwayat Crash' }]
export default function Settings() {
  const [s, setS] = useState('fivem')
  return (
    <div className="p-8">
      <Heading kicker="KONFIGURASI">Pengaturan</Heading>
      <div className="grid grid-cols-[180px_1fr] gap-6">
        <ul className="h-fit space-y-px border border-line">
          {SUB.map(x => (
            <li key={x.id}><button onClick={() => setS(x.id)}
              className={`w-full px-4 py-3 text-left text-sm transition-colors ${s === x.id ? 'bg-white text-black' : 'text-txt2 hover:bg-white/5 hover:text-white'}`}>{x.label}</button></li>))}
        </ul>
        <div className="min-w-0">{s === 'fivem' ? <FiveMSettings /> : s === 'gta' ? <GtaSettings /> : <Crashes />}</div>
      </div>
    </div>)
}
