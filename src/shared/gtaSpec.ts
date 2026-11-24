// Mapping eksplisit label UI -> elemen XML, dari gta5_settings.xml ASLI (2 sampel) + daftar opsi resmi dari admin.
// Nilai file = urutan opsi (0,1,2,...), dikonfirmasi oleh sampel: Windowed=2 Borderless, SoftShadows=4 AMD CHS, PostFX=3 Ultra, Shadow 3 Very High, dst.
// Opt = [nilai di file, label, 1 jika BELUM diverifikasi dari sampel]. Nilai di luar daftar tampil "Kustom (n)" dan dipertahankan.
export type Opt = [string, string, number?]
export interface Item { id: string; label: string; group: string; kind: 'enum' | 'bool' | 'pct' | 'res'; opts?: Opt[] }
const Q3: Opt[] = [['0', 'Normal'], ['1', 'High'], ['2', 'Very High']]
const Q4: Opt[] = [['0', 'Normal'], ['1', 'High'], ['2', 'Very High'], ['3', 'Ultra']]
const AA: Opt[] = [['0', 'Off'], ['2', 'X2'], ['4', 'X4'], ['8', 'X8']]
const OO: Opt[] = [['0', 'Off'], ['1', 'On']]
const RESX = (w: number, h: number, n = ''): Opt => [`${w}x${h}`, `${w} × ${h}${n ? ' (' + n + ')' : ''}`]
export const BOOL: Opt[] = [['false', 'Off'], ['true', 'On']]
export const PCT: Opt[] = Array.from({ length: 11 }, (_, i) => [(i / 10).toFixed(6), `${i * 10}%`] as Opt)
const RES: Opt[] = [RESX(1280, 720), RESX(1366, 768), RESX(1600, 900), RESX(1920, 1080, 'Full HD'), RESX(1920, 1200), RESX(2560, 1080, 'UltraWide'), RESX(2560, 1440, '2K'), RESX(3440, 1440), RESX(3840, 2160)]
export const GROUPS = ['KUALITAS GRAFIS', 'ANTI-ALIASING & FILTERING', 'BAYANGAN & EFEK', 'POPULASI & JARAK', 'LAYAR']
const [G1, G2, G3, G4, G5] = GROUPS
const en = (id: string, label: string, group: string, opts: Opt[]): Item => ({ id, label, group, kind: 'enum', opts })
const bo = (id: string, label: string, group: string): Item => ({ id, label, group, kind: 'bool' })
const pc = (id: string, label: string, group: string): Item => ({ id, label, group, kind: 'pct' })
export const SPEC: Item[] = [
  en('TextureQuality', 'Texture Quality', G1, Q3), en('ShaderQuality', 'Shader Quality', G1, Q3),
  en('ShadowQuality', 'Shadow Quality', G1, [['0', 'Off'], ['1', 'Normal'], ['2', 'High'], ['3', 'Very High']]),
  en('ReflectionQuality', 'Reflection Quality', G1, Q4), en('ReflectionMSAA', 'Reflection MSAA', G1, AA),
  en('WaterQuality', 'Water Quality', G1, Q3), en('ParticleQuality', 'Particles Quality', G1, Q3), en('GrassQuality', 'Grass Quality', G1, Q4),
  en('PostFX', 'Post FX', G1, [['0', 'Normal'], ['1', 'High'], ['2', 'Very High'], ['3', 'Ultra']]),
  en('Tessellation', 'Tessellation', G1, [['0', 'Off'], ['1', 'Normal'], ['2', 'High'], ['3', 'Very High']]),
  en('AnisotropicFiltering', 'Anisotropic Filtering', G2, [['2', 'X2'], ['4', 'X4'], ['8', 'X8'], ['16', 'X16']]),
  en('SSAO', 'Ambient Occlusion', G2, [['0', 'Off'], ['1', 'Normal'], ['2', 'High']]),
  en('MSAA', 'MSAA', G2, AA), bo('FXAA_Enabled', 'FXAA', G2), bo('TXAA_Enabled', 'TXAA', G2),
  en('Shadow_SoftShadows', 'Soft Shadows', G3, [['0', 'Sharp'], ['1', 'Soft'], ['2', 'Softer'], ['3', 'Softest'], ['4', 'AMD CHS'], ['5', 'NVIDIA PCSS']]),
  bo('Shadow_ParticleShadows', 'Particle Shadows', G3), bo('Shadow_LongShadows', 'Long Shadows', G3),
  bo('UltraShadows_Enabled', 'High Resolution Shadows', G3), bo('Reflection_MipBlur', 'Reflection MipMap Blur', G3),
  bo('DoF', 'Depth of Field', G3), pc('MotionBlurStrength', 'Motion Blur Strength', G3), bo('HdStreamingInFlight', 'High Detail Streaming While Flying', G3),
  pc('CityDensity', 'Population Density', G4), pc('PedVarietyMultiplier', 'Pedestrian Variety', G4), pc('VehicleVarietyMultiplier', 'Vehicle Variety', G4),
  pc('LodScale', 'Distance Scaling', G4), pc('MaxLodScale', 'Extended Distance Scaling', G4),
  { id: 'Resolution', label: 'Resolution', group: G5, kind: 'res', opts: RES }, // ScreenWidth + ScreenHeight
  en('Windowed', 'Screen Type', G5, [['0', 'Fullscreen'], ['1', 'Windowed'], ['2', 'Windowed Borderless']]),
  en('VSync', 'VSync', G5, [['0', 'Off'], ['1', 'On'], ['2', 'Half']]), en('PauseOnFocusLoss', 'Pause Game on Focus Loss', G5, OO)
]
export const optionsOf = (it: Item): Opt[] => it.kind === 'bool' ? BOOL : it.kind === 'pct' ? PCT : it.opts ?? []
export function optLabel(it: Item, v: string): string {
  const o = optionsOf(it).find(x => x[0] === v); if (o) return o[1]
  if (it.kind === 'pct' && !isNaN(Number(v))) return `${Math.round(Number(v) * 100)}%`
  if (it.kind === 'res') return v.replace('x', ' × ')
  return `Kustom (${v})`
}
export function validValue(it: Item, v: string, cur: string | null): boolean {
  if (v === cur) return true
  if (it.kind === 'bool') return v === 'true' || v === 'false'
  if (it.kind === 'pct') return /^\d(\.\d{1,6})?$/.test(v) && Number(v) >= 0 && Number(v) <= 1
  if (it.kind === 'res') { const m = /^(\d{3,5})x(\d{3,5})$/.exec(v); return !!m && +m[1] >= 640 && +m[2] >= 480 && +m[1] <= 15360 && +m[2] <= 8640 }
  return (it.opts ?? []).some(o => o[0] === v)
}
