use crate::{fivem, util};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::UNIX_EPOCH;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

// ---- Parser minidump (MDMP) minimal: ExceptionStream(6) + ModuleListStream(4) ----
pub struct DumpInfo { pub title: String, pub module: Option<String>, pub code: Option<u32> }
fn u32le(b: &[u8], o: usize) -> Option<u32> { b.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]])) }
fn u64le(b: &[u8], o: usize) -> Option<u64> { b.get(o..o + 8).map(|s| { let mut a = [0u8; 8]; a.copy_from_slice(s); u64::from_le_bytes(a) }) }
pub fn parse_dump<F: FnMut(usize, u64) -> Option<Vec<u8>>>(rd: &mut F, size: u64) -> DumpInfo {
    inner(rd, size).unwrap_or(DumpInfo { title: "Unknown".into(), module: None, code: None })
}
fn inner<F: FnMut(usize, u64) -> Option<Vec<u8>>>(rd: &mut F, size: u64) -> Option<DumpInfo> {
    let h = rd(32, 0)?;
    if u32le(&h, 0)? != 0x504d444d { return None; }
    let n = u32le(&h, 8)?.min(256) as usize;
    let dir = rd(n * 12, u32le(&h, 12)? as u64)?;
    let (mut ex, mut mods) = (None, None);
    for i in 0..n {
        let o = i * 12;
        let (t, sz, rva) = (u32le(&dir, o)?, u32le(&dir, o + 4)? as u64, u32le(&dir, o + 8)? as u64);
        if t == 6 && rva + 32 <= size { ex = Some(rva); }
        if t == 4 && sz <= 8 * 1024 * 1024 && rva + sz <= size { mods = Some((rva, sz)); }
    }
    let eb = rd(32, ex?)?;
    let code = u32le(&eb, 8)?;
    let addr = u64le(&eb, 24)?;
    let name = match code {
        0x80000003 => "Breakpoint".to_string(), 0xc0000005 => "Access Violation".to_string(), 0x80000001 => "Guard Page Violation".to_string(),
        0xc00000fd => "Stack Overflow".to_string(), 0xc000001d => "Illegal Instruction".to_string(), 0xc0000094 => "Integer Divide by Zero".to_string(),
        0xe06d7363 => "C++ Exception".to_string(), c => format!("Exception 0x{:X}", c),
    };
    if let Some((mr, ms)) = mods {
        let mb = rd(ms as usize, mr)?;
        let cnt = u32le(&mb, 0)? as usize;
        for i in 0..cnt {
            let o = 4 + i * 108;
            if o + 108 > mb.len() { break; }
            let base = u64le(&mb, o)?;
            let msz = u32le(&mb, o + 8)? as u64;
            if addr >= base && addr < base + msz {
                let nrva = u32le(&mb, o + 20)? as u64;
                let len = (u32le(&rd(4, nrva)?, 0)? as usize).min(1024);
                let nb = rd(len, nrva + 4)?;
                let units: Vec<u16> = nb.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
                let full = String::from_utf16_lossy(&units);
                let md = full.rsplit(|c: char| c == '\\' || c == '/').next().unwrap_or(&full).to_string();
                return Some(DumpInfo { title: format!("{} in {}+0x{:X}", name, md, addr - base), module: Some(md), code: Some(code) });
            }
        }
    }
    Some(DumpInfo { title: format!("{} at 0x{:X}", name, addr), module: None, code: Some(code) })
}

// ---- Layanan ----
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Item { id: String, title: String, module: Option<String>, code: Option<u32>, time: u64, has_log: bool, size: u64 }
#[derive(Serialize)]
pub struct CrashList { exists: bool, total: usize, items: Vec<Item> }
#[derive(Serialize)]
pub struct ZipRes { path: String, skipped: Vec<String> }

fn dir() -> PathBuf {
    fivem::detect_dir().unwrap_or_else(|| util::localappdata().join("FiveM")).join("FiveM.app").join("crashes")
}
fn valid_id(id: &str) -> bool { (8..=64).contains(&id.len()) && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') }
fn cache() -> &'static Mutex<HashMap<String, Item>> {
    static C: OnceLock<Mutex<HashMap<String, Item>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}
fn info(id: &str) -> Option<Item> {
    let p = dir().join(format!("{id}.dmp"));
    let m = fs::metadata(&p).ok()?;
    let time = m.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_millis() as u64).unwrap_or(0);
    let key = format!("{id}:{time}:{}", m.len());
    if let Some(c) = cache().lock().ok().and_then(|c| c.get(&key).cloned()) { return Some(c); }
    let d = match fs::File::open(&p) {
        Ok(mut f) => parse_dump(&mut |len: usize, pos: u64| {
            if len > 16 * 1024 * 1024 { return None; }
            f.seek(SeekFrom::Start(pos)).ok()?;
            let mut b = vec![0u8; len];
            let n = f.read(&mut b).ok()?;
            b.truncate(n);
            Some(b)
        }, m.len()),
        Err(_) => DumpInfo { title: "Unknown".into(), module: None, code: None },
    };
    let it = Item { id: id.to_string(), title: d.title, module: d.module, code: d.code, time, has_log: dir().join(format!("{id}.gamelog")).exists(), size: m.len() };
    if let Ok(mut c) = cache().lock() { c.insert(key, it.clone()); }
    Some(it)
}
pub fn list() -> Result<CrashList, String> {
    let rd = match fs::read_dir(dir()) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(CrashList { exists: false, total: 0, items: vec![] }),
        Err(_) => return Err("Tidak dapat membaca folder crashes (izin akses?).".into()),
    };
    let mut items: Vec<Item> = rd.flatten().filter_map(|e| {
        let n = e.file_name().to_string_lossy().to_string();
        let id = n.strip_suffix(".dmp").or_else(|| n.strip_suffix(".DMP"))?.to_string();
        if valid_id(&id) { info(&id) } else { None }
    }).collect();
    items.sort_by(|a, b| b.time.cmp(&a.time));
    let total = items.len();
    items.truncate(200);
    Ok(CrashList { exists: true, total, items })
}
fn need(id: &str) -> Result<Item, String> {
    if !valid_id(id) { return Err("ID crash tidak valid.".into()); }
    info(id).ok_or_else(|| "File crash tidak ditemukan.".to_string())
}
fn text(c: &Item) -> String {
    use chrono::TimeZone;
    let when = chrono::Local.timestamp_millis_opt(c.time as i64).single().map(|d| d.format("%d/%m/%Y %H.%M.%S").to_string()).unwrap_or_default();
    format!("Crash: {}\nWaktu: {}\nFolder: {}\nID: {}\nModul: {}\nException code: {}\n",
        c.title, when, dir().display(), c.id, c.module.clone().unwrap_or_else(|| "Unknown".into()),
        c.code.map(|x| format!("0x{:X}", x)).unwrap_or_else(|| "Unknown".into()))
}
pub fn info_text(id: &str) -> Result<String, String> { Ok(text(&need(id)?)) }
pub fn open_folder(id: &str) -> Result<(), String> {
    need(id)?;
    let p = dir().join(format!("{id}.dmp"));
    let mut c = std::process::Command::new("explorer.exe");
    #[cfg(windows)]
    c.raw_arg(format!("/select,\"{}\"", p.display()));
    #[cfg(not(windows))]
    c.arg(dir());
    c.spawn().map(|_| ()).map_err(|e| e.to_string())
}
pub fn zip(id: &str, downloads: PathBuf) -> Result<ZipRes, String> {
    let it = need(id)?;
    let out = downloads.join(format!("Memories_Crash_{}.zip", util::stamp()));
    let f = fs::File::create(&out).map_err(|e| format!("Tidak dapat membuat ZIP: {e}"))?;
    let mut zw = zip::ZipWriter::new(f);
    let opt = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut skipped = vec![];
    for ext in ["dmp", "gamelog"] {
        let name = format!("{id}.{ext}");
        match fs::File::open(dir().join(&name)) {
            Ok(mut fh) => {
                zw.start_file(name, opt.clone()).map_err(|e| e.to_string())?;
                std::io::copy(&mut fh, &mut zw).map_err(|e| e.to_string())?;
            }
            Err(_) => skipped.push(name),
        }
    }
    zw.start_file("crash-info.txt", opt.clone()).map_err(|e| e.to_string())?;
    zw.write_all(text(&it).as_bytes()).map_err(|e| e.to_string())?;
    zw.finish().map_err(|e| e.to_string())?;
    Ok(ZipRes { path: out.to_string_lossy().to_string(), skipped })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dump_sintetis() { // dump SINTETIS sesuai spesifikasi MDMP, bukan dump FiveM asli
        let name: Vec<u8> = "C:\\x\\libcef.dll".encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
        let mut b = vec![0u8; 1024];
        let w32 = |b: &mut Vec<u8>, o: usize, v: u32| b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        let w64 = |b: &mut Vec<u8>, o: usize, v: u64| b[o..o + 8].copy_from_slice(&v.to_le_bytes());
        w32(&mut b, 0, 0x504d444d); w32(&mut b, 8, 2); w32(&mut b, 12, 32);
        w32(&mut b, 32, 6); w32(&mut b, 36, 168); w32(&mut b, 40, 200);
        w32(&mut b, 44, 4); w32(&mut b, 48, 112); w32(&mut b, 52, 400);
        w32(&mut b, 208, 0x80000003); w64(&mut b, 224, 0x7ff800000000 + 0x5d5a466);
        w32(&mut b, 400, 1); w64(&mut b, 404, 0x7ff800000000); w32(&mut b, 412, 0x10000000); w32(&mut b, 424, 600);
        w32(&mut b, 600, name.len() as u32); b[604..604 + name.len()].copy_from_slice(&name);
        let size = b.len() as u64;
        let mut rd = |len: usize, pos: u64| { let p = pos as usize; b.get(p..(p + len).min(b.len())).map(|s| s.to_vec()) };
        assert_eq!(parse_dump(&mut rd, size).title, "Breakpoint in libcef.dll+0x5D5A466");
        let mut bad = |_: usize, _: u64| Some(vec![0u8; 64]);
        assert_eq!(parse_dump(&mut bad, 64).title, "Unknown");
    }
}
