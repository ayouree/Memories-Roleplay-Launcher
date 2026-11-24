import { useState } from 'react'
import TitleBar from './components/TitleBar'
import Sidebar, { Tab } from './components/Sidebar'
import Gate from './pages/Gate'
import CachePage from './pages/Cache'
import Settings from './pages/Settings'

export default function App() {
  const [tab, setTab] = useState<Tab>('gate')
  return (
    <div className="flex h-full flex-col bg-bg">
      <TitleBar />
      <div className="flex min-h-0 flex-1">
        <Sidebar tab={tab} setTab={setTab} />
        <main className="min-w-0 flex-1 overflow-y-auto">
          <div key={tab} className="page">
            {tab === 'gate' && <Gate />}{tab === 'cache' && <CachePage />}{tab === 'settings' && <Settings />}
          </div>
        </main>
      </div>
    </div>)
}
