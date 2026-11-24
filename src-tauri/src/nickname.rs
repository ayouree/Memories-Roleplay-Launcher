use crate::{fivem, util};
use rusty_leveldb::{LdbIterator, Options, DB};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

// Nickname dibaca/ditulis di localStorage Chromium (LevelDB) milik FiveM:
//   key   = "_" + origin + 0x00 + [1 byte encoding] + nama key
//   value = [1 byte encoding: 0 = UTF-16LE, 1 = Latin-1] + isi
// Key TIDAK dikarang: dicari dari data (nama key = "nickname", atau yang mengandung "nick") dan hanya key yang ada yang ditulis.
#[derive(Serialize)]
pub struct NickInfo { found: bool, nickname: String, key: String, detail: String }
impl NickInfo {
    fn missing(d: &str) -> Self { NickInfo { found: false, nickname: String::new(), key: String::new(), detail: d.to_string() } }
}
struct Found { raw_key: Vec<u8>, name: String, value: String, json: bool }

fn ldb_dir() -> Result<PathBuf, String> {
    let d = fivem::detect_dir().ok_or("FiveM tidak ditemukan. Silakan pilih lokasi instalasi FiveM.")?;
    Ok(d.join("FiveM.app").join("data").join("nui-storage").join("Local Storage").join("leveldb"))
}
fn decode(b: &[u8]) -> Option<String> {
    let (&f, rest) = b.split_first()?;
    match f {
        1 => Some(rest.iter().map(|&c| c as char).collect()),
        0 => {
            if rest.len() % 2 != 0 { return None; }
            let u: Vec<u16> = rest.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
            Some(String::from_utf16_lossy(&u))
        }
        _ => None,
    }
}
fn parse_key(k: &[u8]) -> Option<(String, String)> {
    if k.first() != Some(&b'_') { return None; }
    let s = k[1..].iter().position(|&c| c == 0)? + 1;
    let origin = String::from_utf8_lossy(&k[1..s]).to_string();
    let name = decode(&k[s + 1..])?;
    Some((origin, name))
}
fn opts() -> Options { let mut o = Options::default(); o.create_if_missing = false; o }
fn scan(db: &mut DB) -> Result<(Option<Found>, Vec<String>), String> {
    let mut it = db.new_iter().map_err(|e| e.to_string())?;
    let (mut k, mut v) = (vec![], vec![]);
    let mut best: Option<(u8, Found)> = None;
    let mut names: Vec<String> = vec![];
    while it.advance() {
        if !it.current(&mut k, &mut v) { continue; }
        let Some((origin, name)) = parse_key(&k) else { continue };
        if names.len() < 20 { names.push(format!("{}:{}", origin, name)); }
        let low = name.to_lowercase();
        if !low.contains("nick") { continue; }
        let score: u8 = if low == "nickname" { 2 } else { 1 };
        if best.as_ref().map(|b| b.0 >= score).unwrap_or(false) { continue; }
        let Some(raw) = decode(&v) else { continue };
        let (value, json) = match serde_json::from_str::<String>(&raw) {
            Ok(s) if raw.starts_with('"') => (s, true),
            _ => (raw.clone(), false),
        };
        best = Some((score, Found { raw_key: k.clone(), name, value, json }));
    }
    Ok((best.map(|b| b.1), names))
}
fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for e in fs::read_dir(src)? {
        let e = e?;
        if e.file_type()?.is_file() && e.file_name() != "LOCK" { fs::copy(e.path(), dst.join(e.file_name()))?; }
    }
    Ok(())
}
/// Membaca SELALU dari salinan sementara, agar database asli (yang mungkin sedang dipakai FiveM) tidak tersentuh.
pub fn get() -> Result<NickInfo, String> {
    let dir = ldb_dir()?;
    if !dir.is_dir() { return Ok(NickInfo::missing("Database FiveM belum ada. Buka FiveM sekali lalu coba lagi.")); }
    let tmp = std::env::temp_dir().join(format!("memories-ldb-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    copy_dir(&dir, &tmp).map_err(|_| "Tidak dapat menyalin database FiveM (sedang dipakai?). Tutup FiveM lalu coba lagi.".to_string())?;
    let r = (|| -> Result<(Option<Found>, Vec<String>), String> {
        let mut db = DB::open(&tmp, opts()).map_err(|e| format!("Database FiveM tidak bisa dibuka: {e}"))?;
        let out = scan(&mut db)?;
        let _ = db.close();
        Ok(out)
    })();
    let _ = fs::remove_dir_all(&tmp);
    let (found, names) = r?;
    Ok(match found {
        Some(f) => NickInfo { found: true, nickname: f.value, key: f.name, detail: String::new() },
        None => NickInfo::missing(&format!("Key nickname tidak ditemukan di database FiveM. Contoh key: {}", if names.is_empty() { "(kosong)".to_string() } else { names.join(", ") })),
    })
}
fn restore(dir: &Path, backup: &Path) {
    if let Ok(rd) = fs::read_dir(dir) { for e in rd.flatten() { if e.path().is_file() { let _ = fs::remove_file(e.path()); } } }
    let _ = copy_dir(backup, dir);
}
pub fn set(name: &str) -> Result<NickInfo, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 32 || !name.chars().all(|c| c.is_alphanumeric() || " _-.[]|".contains(c)) {
        return Err("Nickname tidak valid (maks 32 karakter: huruf, angka, spasi, _ - . [ ] |).".into());
    }
    if fivem::is_running() { return Err("FiveM sedang berjalan. Tutup FiveM terlebih dahulu agar database tidak rusak.".into()); }
    let dir = ldb_dir()?;
    if !dir.is_dir() { return Err("Database FiveM tidak ditemukan.".into()); }
    let backup = dir.with_file_name(format!("leveldb.backup.{}", util::stamp()));
    copy_dir(&dir, &backup).map_err(|e| format!("Backup database gagal: {e}"))?;
    let res = (|| -> Result<(), String> {
        let mut db = DB::open(&dir, opts()).map_err(|e| format!("Database FiveM tidak bisa dibuka: {e}"))?;
        let (found, _) = scan(&mut db)?;
        let f = found.ok_or("Key nickname tidak ditemukan; tidak ada yang diubah.")?;
        let body = if f.json { serde_json::to_string(name).map_err(|e| e.to_string())? } else { name.to_string() };
        let latin = body.chars().all(|c| (c as u32) <= 255);
        let mut val: Vec<u8> = vec![if latin { 1 } else { 0 }];
        if latin { val.extend(body.chars().map(|c| c as u8)); } else { for u in body.encode_utf16() { val.extend_from_slice(&u.to_le_bytes()); } }
        db.put(&f.raw_key, &val).map_err(|e| e.to_string())?;
        db.flush().map_err(|e| e.to_string())?;
        db.close().map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(e) = res { restore(&dir, &backup); return Err(format!("{e} (database dipulihkan dari backup)")); }
    match get() {
        Ok(n) if n.found && n.nickname == name => Ok(n),
        _ => { restore(&dir, &backup); Err("Verifikasi gagal; database dipulihkan dari backup.".into()) }
    }
}
