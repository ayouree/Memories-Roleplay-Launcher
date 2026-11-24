use crate::{util};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq)]
enum K { E, B, P, R } // enum angka, bool, persen (0..1), resolusi
// Daftar elemen yang dikelola (mapping label UI -> elemen XML ada di src/shared/gtaSpec.ts).
const IDS: &[(&str, K)] = &[
    ("TextureQuality", K::E), ("ShaderQuality", K::E), ("ShadowQuality", K::E), ("ReflectionQuality", K::E), ("ReflectionMSAA", K::E),
    ("WaterQuality", K::E), ("ParticleQuality", K::E), ("GrassQuality", K::E), ("PostFX", K::E), ("Tessellation", K::E),
    ("AnisotropicFiltering", K::E), ("SSAO", K::E), ("MSAA", K::E), ("FXAA_Enabled", K::B), ("TXAA_Enabled", K::B),
    ("Shadow_SoftShadows", K::E), ("Shadow_ParticleShadows", K::B), ("Shadow_LongShadows", K::B), ("UltraShadows_Enabled", K::B),
    ("Reflection_MipBlur", K::B), ("DoF", K::B), ("MotionBlurStrength", K::P), ("HdStreamingInFlight", K::B),
    ("CityDensity", K::P), ("PedVarietyMultiplier", K::P), ("VehicleVarietyMultiplier", K::P), ("LodScale", K::P), ("MaxLodScale", K::P),
    ("Resolution", K::R), ("Windowed", K::E), ("VSync", K::E), ("PauseOnFocusLoss", K::E),
];

// ---- Logika murni: ubah nilai atribut saja, byte lain tidak tersentuh ----
fn tag_range(x: &str, t: &str) -> Option<(usize, usize)> {
    let pat = format!("<{}", t);
    let mut from = 0;
    while let Some(p) = x[from..].find(&pat) {
        let s = from + p + pat.len();
        let rest = &x[s..];
        if rest.starts_with(char::is_whitespace) {
            let end = rest.find('>')?;
            if let Some(v) = rest[..end].find("value=\"") {
                let a = s + v + 7;
                let b = x[a..].find('"')?;
                return Some((a, a + b));
            }
        }
        from = s;
    }
    None
}
fn read_tag(x: &str, t: &str) -> Option<String> { tag_range(x, t).map(|(a, b)| x[a..b].to_string()) }
pub fn read_all(x: &str) -> HashMap<String, Option<String>> {
    IDS.iter().map(|(id, _)| {
        let v = if *id == "Resolution" {
            match (read_tag(x, "ScreenWidth"), read_tag(x, "ScreenHeight")) { (Some(w), Some(h)) => Some(format!("{}x{}", w, h)), _ => None }
        } else { read_tag(x, id) };
        (id.to_string(), v)
    }).collect()
}
fn set_tag(x: &mut String, t: &str, v: &str) -> Result<(), String> {
    let (a, b) = tag_range(x, t).ok_or(format!("Setting {} tidak ada di file; tidak dibuat sembarangan.", t))?;
    x.replace_range(a..b, v);
    Ok(())
}
pub fn apply_all(x: &str, ch: &HashMap<String, String>) -> Result<String, String> {
    let mut s = x.to_string();
    for (id, v) in ch {
        if id == "Resolution" {
            let (w, h) = v.split_once('x').ok_or("Resolusi tidak valid.")?;
            set_tag(&mut s, "ScreenWidth", w)?;
            set_tag(&mut s, "ScreenHeight", h)?;
        } else { set_tag(&mut s, id, v)?; }
    }
    Ok(s)
}
pub fn looks_valid(x: &str) -> bool {
    x.contains("<Settings>") && x.trim_end().ends_with("</Settings>") && x.contains("<graphics>") && x.contains("</graphics>")
}
fn gpu_name(x: &str) -> String {
    let t = "<VideoCardDescription>";
    match x.find(t) { Some(a) => { let r = &x[a + t.len()..]; r[..r.find('<').unwrap_or(0)].to_string() } None => String::new() }
}
fn valid_value(k: K, v: &str, cur: Option<&str>) -> bool {
    if Some(v) == cur { return true; }
    match k {
        K::B => v == "true" || v == "false",
        K::P => v.len() <= 8 && v.chars().all(|c| c.is_ascii_digit() || c == '.') && v.parse::<f64>().map(|n| (0.0..=1.0).contains(&n)).unwrap_or(false),
        K::R => match v.split_once('x') {
            Some((w, h)) => match (w.parse::<u32>(), h.parse::<u32>()) { (Ok(w), Ok(h)) => (640..=15360).contains(&w) && (480..=8640).contains(&h), _ => false },
            None => false,
        },
        K::E => !v.is_empty() && v.len() <= 2 && v.chars().all(|c| c.is_ascii_digit()),
    }
}

// ---- Layanan file ----
fn file() -> PathBuf { util::appdata().join("CitizenFX").join("gta5_settings.xml") }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GtaData { exists: bool, valid: bool, path: String, gpu: String, hash: String, values: HashMap<String, Option<String>> }
pub fn load() -> Result<GtaData, String> {
    let path = file().to_string_lossy().to_string();
    match fs::read_to_string(file()) {
        Ok(raw) => {
            if !looks_valid(&raw) { return Ok(GtaData { exists: true, valid: false, path, gpu: String::new(), hash: util::hash(&raw), values: HashMap::new() }); }
            Ok(GtaData { exists: true, valid: true, path, gpu: gpu_name(&raw), hash: util::hash(&raw), values: read_all(&raw) })
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(GtaData { exists: false, valid: false, path, gpu: String::new(), hash: String::new(), values: HashMap::new() }),
        Err(_) => Err("Tidak dapat membaca gta5_settings.xml (izin akses atau bukan UTF-8).".into()),
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePayload { pub base_hash: String, pub values: HashMap<String, Option<String>> }
pub fn save(p: SavePayload) -> Result<GtaData, String> {
    let raw = fs::read_to_string(file()).map_err(|_| "gta5_settings.xml tidak ditemukan.".to_string())?;
    if !looks_valid(&raw) { return Err("gta5_settings.xml rusak; tidak ditimpa.".into()); }
    if util::hash(&raw) != p.base_hash { return Err("External modification detected: file berubah sejak dibaca. Klik MUAT ULANG.".into()); }
    let cur = read_all(&raw);
    let mut ch: HashMap<String, String> = HashMap::new();
    for (id, k) in IDS {
        let Some(Some(v)) = p.values.get(*id) else { continue };
        let c = cur.get(*id).cloned().flatten();
        if Some(v.as_str()) == c.as_deref() { continue; }
        if c.is_none() { return Err(format!("{} tidak didukung di file ini.", id)); }
        if !valid_value(*k, v, c.as_deref()) { return Err(format!("Nilai tidak valid untuk {}.", id)); }
        ch.insert(id.to_string(), v.clone());
    }
    if ch.is_empty() { return load(); }
    let out = apply_all(&raw, &ch)?;
    if !looks_valid(&out) { return Err("Hasil edit tidak valid; dibatalkan.".into()); }
    let backup = file().with_file_name(format!("gta5_settings.backup.{}.xml", util::stamp()));
    fs::copy(file(), &backup).map_err(|e| format!("Backup gagal: {e}"))?;
    let tmp = file().with_extension("xml.tmp");
    fs::write(&tmp, &out).map_err(|e| e.to_string())?;
    fs::rename(&tmp, file()).map_err(|e| e.to_string())?;
    if fs::read_to_string(file()).ok().as_deref() != Some(out.as_str()) {
        let _ = fs::copy(&backup, file());
        return Err("Verifikasi gagal; file dipulihkan dari backup.".into());
    }
    load()
}

#[cfg(test)]
mod tests {
    use super::*;
    const N: &str = include_str!("../../test/fixtures/normal.xml");
    const E: &str = include_str!("../../test/fixtures/editor.xml");
    #[test]
    fn sampel_asli() {
        assert!(looks_valid(N) && looks_valid(E));
        assert_eq!(gpu_name(N), "Radeon RX 570 Series");
        let (vn, ve) = (read_all(N), read_all(E));
        assert!(vn.values().all(|v| v.is_some()), "semua setting ter-mapping di file asli");
        assert_eq!(vn["Resolution"].as_deref(), Some("1920x1080"));
        assert_eq!(vn["Windowed"].as_deref(), Some("2"));
        assert_eq!(ve["LodScale"].as_deref(), Some("0.600000"));
        let mut ch = HashMap::new();
        for (id, _) in IDS { if vn[*id] != ve[*id] { ch.insert(id.to_string(), ve[*id].clone().unwrap()); } }
        let out = apply_all(N, &ch).unwrap();
        assert!(out.contains("<Shader_SSA value=\"false\" />")); // tidak dikelola -> utuh
        let mut ssa = HashMap::new();
        ssa.insert("Shader_SSA".to_string(), "true".to_string());
        assert_eq!(apply_all(&out, &ssa).unwrap(), E); // identik byte-per-byte dengan file Editor
        let mut bad = HashMap::new();
        bad.insert("TidakAda".to_string(), "1".to_string());
        assert!(apply_all(N, &bad).is_err());
        assert!(valid_value(K::R, "2560x1440", None) && !valid_value(K::B, "yes", None) && !valid_value(K::P, "1.5", None));
    }
}
