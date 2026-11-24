use crate::{appcfg, config, util};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

#[derive(Serialize)]
pub struct Detect { pub found: bool, pub dir: String }
#[derive(Serialize)]
pub struct Running { pub running: bool }

fn has(d: &Path) -> bool { !d.as_os_str().is_empty() && d.join("FiveM.exe").is_file() }

/// FiveM mendaftarkan protokol fivem:// ke lokasi FiveM.exe yang sebenarnya (termasuk instalasi di folder lain).
fn from_registry() -> Option<PathBuf> {
    static R: OnceLock<Option<PathBuf>> = OnceLock::new();
    R.get_or_init(|| {
        let out = util::cmd("reg").args(["query", "HKCR\\fivem\\shell\\open\\command", "/ve"]).output().ok()?;
        let s = String::from_utf8_lossy(&out.stdout).to_string();
        let a = s.find('"')?;
        let b = s[a + 1..].find('"')?;
        let exe = &s[a + 1..a + 1 + b];
        if !exe.to_lowercase().ends_with("fivem.exe") { return None; }
        Path::new(exe).parent().map(|p| p.to_path_buf())
    })
    .clone()
}
/// Prioritas: lokasi pilihan pemain > default > registry.
pub fn detect_dir() -> Option<PathBuf> {
    let saved = PathBuf::from(config::get().fivem_path);
    if has(&saved) { return Some(saved); }
    let def = util::localappdata().join("FiveM");
    if has(&def) { return Some(def); }
    from_registry().filter(|d| has(d))
}
pub fn detect() -> Detect {
    match detect_dir() {
        Some(d) => Detect { found: true, dir: d.to_string_lossy().to_string() },
        None => Detect { found: false, dir: String::new() },
    }
}
pub fn browse() -> Result<Detect, String> {
    let Some(sel) = rfd::FileDialog::new().set_title("Pilih folder instalasi FiveM (yang berisi FiveM.exe)").pick_folder() else {
        return Ok(detect());
    };
    let parent = sel.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let dir = [sel.clone(), parent].into_iter().find(|d| has(d))
        .ok_or("FiveM.exe tidak ditemukan di folder tersebut. Pilih folder yang berisi FiveM.exe.")?;
    let s = dir.to_string_lossy().to_string();
    config::update(|c| c.fivem_path = s)?;
    Ok(detect())
}
pub fn is_running() -> bool {
    util::cmd("tasklist").args(["/FO", "CSV", "/NH"]).output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_lowercase().contains("fivem"))
        .unwrap_or(false)
}
/// Hanya menerima id gerbang; hostname diambil dari konfigurasi, bukan dari UI.
pub fn connect(id: &str) -> Result<Running, String> {
    let g = appcfg::gates().iter().find(|g| g.id == id).ok_or("Gerbang tidak dikenal.")?.clone();
    let d = detect_dir().ok_or("FiveM tidak ditemukan. Silakan pilih lokasi instalasi FiveM.")?;
    let running = is_running();
    // FiveM menolak fivem:// yang dipicu langsung oleh aplikasi lain; lewat explorer.exe pemanggilnya adalah shell Windows.
    let r = if config::get().connect_method == "direct" {
        Command::new(d.join("FiveM.exe")).args(["+connect", g.host.as_str()]).current_dir(&d).spawn()
    } else {
        Command::new("explorer.exe").arg(format!("fivem://connect/{}", g.host)).spawn()
    };
    r.map_err(|e| format!("Gagal meluncurkan FiveM: {e}"))?;
    let gid = g.id.clone();
    let _ = config::update(|c| c.last_selected_gate = gid);
    Ok(Running { running })
}
