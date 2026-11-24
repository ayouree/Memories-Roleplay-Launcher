use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub fn appdata() -> PathBuf { PathBuf::from(std::env::var("APPDATA").unwrap_or_default()) }
pub fn localappdata() -> PathBuf { PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_default()) }
pub fn now_ms() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0) }
pub fn stamp() -> String { chrono::Local::now().format("%Y%m%d_%H%M%S").to_string() }
/// Proses anak tanpa jendela konsol.
pub fn cmd(program: &str) -> Command {
    #[allow(unused_mut)]
    let mut c = Command::new(program);
    #[cfg(windows)]
    c.creation_flags(0x0800_0000);
    c
}
/// Hash non-kriptografis; hanya dipakai membandingkan isi file dalam satu sesi (deteksi perubahan eksternal).
pub fn hash(s: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    format!("{:016x}-{}", h.finish(), s.len())
}
