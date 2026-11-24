export {}
declare global {
  interface GateResult { id: string; status: 'ONLINE' | 'OFFLINE' | 'UNAVAILABLE'; ping: number | null; jitter: number | null; loss: number | null; min: number | null; max: number | null; at: number; note?: string }
  interface KeyEntry { origLine: number | null; key: string; command: string; enabled: boolean }
  interface KeybindData { exists: boolean; hash: string; entries: KeyEntry[]; disabled: KeyEntry[]; path: string }
  interface CacheInfo { id: 'cache' | 'server'; dir: string; status: 'OK' | 'NOT_FOUND' | 'DENIED' | 'ERROR'; size: number }
  interface CacheResult { id: string; removed: number; failed: { name: string; code: string }[] }
  interface AppCfg { connectMethod: 'explorer' | 'direct' }
  interface NickInfo { found: boolean; nickname: string; key: string; detail: string }
  interface GtaData { exists: boolean; valid: boolean; path: string; gpu: string; hash: string; values: Record<string, string | null> }
  interface CrashItem { id: string; title: string; module: string | null; code: number | null; time: number; hasLog: boolean; size: number }
  interface Window { api: {
    appInfo(): Promise<{ version: string }>
    win: { minimize(): void; toggleMax(): void; close(): void; onState(cb: (m: boolean) => void): () => void }
    gate: { measure(): Promise<GateResult[]>; info(): Promise<{ clients: number; max: number } | null> }
    fivem: { detect(): Promise<{ found: boolean; dir: string }>; browse(): Promise<{ found: boolean; dir: string }>; connect(id: string): Promise<{ running: boolean }> }
    cache: { scan(): Promise<CacheInfo[]>; clear(ids: string[]): Promise<CacheResult[]> }
    keybind: { load(): Promise<KeybindData>; save(p: { baseHash: string; entries: KeyEntry[] }): Promise<KeybindData> }
    gta: { load(): Promise<GtaData>; save(p: { baseHash: string; values: Record<string, string | null> }): Promise<GtaData> }
    crash: { list(): Promise<{ exists: boolean; total: number; items: CrashItem[] }>; info(id: string): Promise<string>; open(id: string): Promise<void>; zip(id: string): Promise<{ path: string; skipped: string[] }> }
    link: { open(k: string): Promise<void> }
    community: { discord(): Promise<{ name: string; members: number; online: number } | null> }
    nickname: { get(): Promise<NickInfo>; set(name: string): Promise<NickInfo> }
    config: { get(): Promise<AppCfg>; set(p: Partial<AppCfg>): Promise<AppCfg> }
  } }
}
