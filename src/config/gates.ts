// Sumber tunggal: src/shared/app-config.json (dibaca juga oleh Rust). Port 30120 = default FiveM; konfirmasi ke admin.
import cfg from '../shared/app-config.json'
export interface Gate { id: string; name: string; host: string; port: number }
export const GATES: Gate[] = cfg.gates
export const LINKS: Record<string, string> = cfg.links
export const SOCIALS = Object.entries(LINKS).map(([key, url]) => ({ key, label: key[0].toUpperCase() + key.slice(1), url }))
