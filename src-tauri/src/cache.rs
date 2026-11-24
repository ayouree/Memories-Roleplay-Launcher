use crate::{config, fivem, util};
use serde::Serialize;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const DIRS: [(&str, &str); 2] = [("cache", "cache"), ("server", "server-cache-priv")];

#[derive(Serialize)]
pub struct CacheInfo { id: String, dir: String, status: String, size: u64 }
#[derive(Serialize)]
pub struct Failed { name: String, code: String }
#[derive(Serialize)]
pub struct ClearResult { id: String, removed: u32, failed: Vec<Failed> }

fn root() -> PathBuf { fivem::detect_dir().unwrap_or_else(|| util::localappdata().join("FiveM")) }
fn dir_of(id: &str) -> Option<PathBuf> {
    DIRS.iter().find(|d| d.0 == id).map(|d| root().join("FiveM.app").join("data").join(d.1))
}
fn size(dir: &Path) -> u64 {
    let mut t = 0u64;
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            if let Ok(ft) = e.file_type() {
                if ft.is_symlink() { continue; }
                if ft.is_dir() { t += size(&e.path()); } else if let Ok(m) = e.metadata() { t += m.len(); }
            }
        }
    }
    t
}
pub fn scan() -> Result<Vec<CacheInfo>, String> {
    Ok(DIRS.iter().map(|(id, _)| {
        let dir = dir_of(id).unwrap_or_default();
        let (status, sz) = match fs::metadata(&dir) {
            Ok(m) if m.is_dir() => ("OK", size(&dir)),
            Ok(_) => ("ERROR", 0),
            Err(e) => (match e.kind() { ErrorKind::NotFound => "NOT_FOUND", ErrorKind::PermissionDenied => "DENIED", _ => "ERROR" }, 0),
        };
        CacheInfo { id: id.to_string(), dir: dir.to_string_lossy().to_string(), status: status.to_string(), size: sz }
    }).collect())
}
fn norm(p: &Path) -> String {
    p.to_string_lossy().trim_start_matches("\\\\?\\").replace('/', "\\").trim_end_matches('\\').to_lowercase()
}
pub fn clear(ids: Vec<String>) -> Result<Vec<ClearResult>, String> {
    if ids.is_empty() || !ids.iter().all(|i| DIRS.iter().any(|d| d.0 == i)) { return Err("Pilihan cache tidak valid.".into()); }
    if fivem::is_running() { return Err("FiveM sedang berjalan. Tutup FiveM terlebih dahulu.".into()); }
    let mut out = vec![];
    let mut log = String::new();
    for id in &ids {
        let dir = dir_of(id).ok_or("Cache tidak dikenal.")?;
        let real = fs::canonicalize(&dir).map_err(|e| format!("Folder tidak dapat diakses: {e}"))?;
        if norm(&real) != norm(&dir) { return Err(format!("Folder {id} adalah symbolic link; dibatalkan demi keamanan.")); }
        let (mut removed, mut failed) = (0u32, vec![]);
        for e in fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
            let p = e.path();
            let ft = match e.file_type() { Ok(f) => f, Err(_) => continue };
            let r = if ft.is_symlink() { fs::remove_file(&p).or_else(|_| fs::remove_dir(&p)) }
                    else if ft.is_dir() { fs::remove_dir_all(&p) } else { fs::remove_file(&p) };
            match r {
                Ok(_) => removed += 1,
                Err(err) => failed.push(Failed { name: e.file_name().to_string_lossy().to_string(), code: err.raw_os_error().map(|c| c.to_string()).unwrap_or_else(|| format!("{:?}", err.kind())) }),
            }
        }
        log.push_str(&format!("{} {} removed={} failed={}\n", chrono::Utc::now().to_rfc3339(), id, removed, failed.len()));
        out.push(ClearResult { id: id.clone(), removed, failed });
    }
    let dd = config::data_dir();
    let _ = fs::create_dir_all(&dd);
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(dd.join("cache-audit.log")) {
        use std::io::Write;
        let _ = f.write_all(log.as_bytes());
    }
    Ok(out)
}
