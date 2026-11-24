use crate::{config, util};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Entry { pub orig_line: Option<usize>, pub key: String, pub command: String, pub enabled: bool }

// ---- Logika murni (line-preserving): hanya "bind KEYBOARD" yang dikelola ----
pub struct Doc { pub lines: Vec<String>, pub eol: String, pub bom: bool }
pub fn split_doc(t: &str) -> Doc {
    let (bom, t) = match t.strip_prefix('\u{feff}') { Some(r) => (true, r), None => (false, t) };
    let eol = (if t.contains("\r\n") { "\r\n" } else { "\n" }).to_string();
    let lines = if t.is_empty() { vec![] } else { t.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l).to_string()).collect() };
    Doc { lines, eol, bom }
}
pub fn join_doc(d: &Doc) -> String { format!("{}{}", if d.bom { "\u{feff}" } else { "" }, d.lines.join(&d.eol)) }
fn take_word(s: &str) -> Option<(&str, &str)> {
    let s = s.trim_start();
    if s.is_empty() { return None; }
    let end = s.find(char::is_whitespace)?;
    Some((&s[..end], s[end..].trim_start()))
}
pub fn parse_command(rest: &str) -> Option<String> {
    let rest = rest.trim();
    if rest.is_empty() { return None; }
    if !rest.starts_with('"') { return Some(rest.to_string()); }
    let mut out = String::new();
    let mut it = rest.chars().skip(1);
    while let Some(c) = it.next() {
        if c == '\\' { if let Some(n) = it.next() { out.push(n); continue; } }
        if c == '"' { return Some(out); }
        out.push(c);
    }
    None // kutip tidak tertutup: dianggap baris tak dikenal (dipertahankan)
}
pub fn parse_binds(lines: &[String]) -> Vec<(usize, String, String)> {
    let mut r = vec![];
    for (i, l) in lines.iter().enumerate() {
        let Some((w1, rest)) = take_word(l) else { continue };
        if !w1.eq_ignore_ascii_case("bind") { continue; } // "rbind" tidak cocok
        let Some((w2, rest)) = take_word(rest) else { continue };
        if !w2.eq_ignore_ascii_case("KEYBOARD") { continue; } // "bind MOUSE1" tidak cocok
        let Some((key, rest)) = take_word(rest) else { continue };
        if let Some(c) = parse_command(rest) { r.push((i, key.to_string(), c)); }
    }
    r
}
pub fn format_bind(key: &str, cmd: &str) -> String {
    format!("bind KEYBOARD {} \"{}\"", key.to_uppercase(), cmd.replace('\\', "\\\\").replace('"', "\\\""))
}
fn validate(e: &Entry) -> Result<(), String> {
    if e.key.is_empty() || e.key.len() > 32 || !e.key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') { return Err(format!("Tombol tidak valid: \"{}\"", e.key)); }
    if e.command.is_empty() || e.command.chars().count() > 500 || e.command.contains(['\r', '\n', '\0']) { return Err(format!("Perintah tidak valid untuk tombol {}", e.key)); }
    Ok(())
}
pub fn apply_changes(lines: &[String], entries: &[Entry]) -> Result<Vec<String>, String> {
    for e in entries.iter().filter(|e| e.enabled) { validate(e)?; }
    let managed: HashMap<usize, (String, String)> = parse_binds(lines).into_iter().map(|(i, k, c)| (i, (k, c))).collect();
    let by_line: HashMap<usize, &Entry> = entries.iter().filter(|e| e.enabled && e.orig_line.is_some()).map(|e| (e.orig_line.unwrap_or(0), e)).collect();
    let mut out: Vec<String> = vec![];
    for (i, l) in lines.iter().enumerate() {
        match managed.get(&i) {
            None => out.push(l.clone()),
            Some((k, c)) => {
                if let Some(e) = by_line.get(&i) { // tidak ada = dihapus / dinonaktifkan
                    if e.key.to_uppercase() == k.to_uppercase() && e.command == *c { out.push(l.clone()); } else { out.push(format_bind(&e.key, &e.command)); }
                }
            }
        }
    }
    let adds: Vec<String> = entries.iter().filter(|e| e.enabled && e.orig_line.is_none()).map(|e| format_bind(&e.key, &e.command)).collect();
    let mut end = out.len();
    if end > 0 && out[end - 1].is_empty() { end -= 1; }
    for (n, a) in adds.into_iter().enumerate() { out.insert(end + n, a); }
    Ok(out)
}

// ---- Layanan file ----
fn file() -> PathBuf { util::appdata().join("CitizenFX").join("fivem.cfg") }
fn read_raw() -> std::io::Result<String> {
    String::from_utf8(fs::read(file())?).map_err(|_| std::io::Error::new(ErrorKind::InvalidData, "bukan UTF-8"))
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeybindData { exists: bool, hash: String, entries: Vec<Entry>, disabled: Vec<Entry>, path: String }
pub fn load() -> Result<KeybindData, String> {
    let disabled: Vec<Entry> = config::get().disabled_keybinds.iter()
        .map(|d| Entry { orig_line: None, key: d.key.clone(), command: d.command.clone(), enabled: false }).collect();
    let path = file().to_string_lossy().to_string();
    match read_raw() {
        Ok(raw) => {
            let doc = split_doc(&raw);
            let entries = parse_binds(&doc.lines).into_iter().map(|(i, k, c)| Entry { orig_line: Some(i), key: k, command: c, enabled: true }).collect();
            Ok(KeybindData { exists: true, hash: util::hash(&raw), entries, disabled, path })
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(KeybindData { exists: false, hash: String::new(), entries: vec![], disabled, path }),
        Err(e) if e.kind() == ErrorKind::InvalidData => Err("fivem.cfg bukan UTF-8; launcher tidak mengubahnya.".into()),
        Err(_) => Err("Tidak dapat membaca fivem.cfg (izin akses?).".into()),
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePayload { pub base_hash: String, pub entries: Vec<Entry> }
pub fn save(p: SavePayload) -> Result<KeybindData, String> {
    let raw = read_raw().map_err(|_| "fivem.cfg tidak ditemukan atau tidak bisa dibaca. Jalankan FiveM sekali agar file dibuat.".to_string())?;
    if util::hash(&raw) != p.base_hash { return Err("External modification detected: fivem.cfg berubah sejak dibaca. Klik MUAT ULANG.".into()); }
    let mut doc = split_doc(&raw);
    doc.lines = apply_changes(&doc.lines, &p.entries)?;
    let out = join_doc(&doc);
    let backup = file().with_file_name(format!("fivem.backup.{}.cfg", util::stamp()));
    fs::copy(file(), &backup).map_err(|e| format!("Backup gagal: {e}"))?;
    let tmp = file().with_extension("cfg.tmp");
    fs::write(&tmp, &out).map_err(|e| e.to_string())?;
    fs::rename(&tmp, file()).map_err(|e| e.to_string())?;
    if read_raw().ok().as_deref() != Some(out.as_str()) {
        let _ = fs::copy(&backup, file());
        return Err("Verifikasi gagal; fivem.cfg dipulihkan dari backup.".into());
    }
    let dis: Vec<config::Disabled> = p.entries.iter().filter(|e| !e.enabled).map(|e| config::Disabled { key: e.key.clone(), command: e.command.clone() }).collect();
    config::update(|c| c.disabled_keybinds = dis)?;
    load()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn en(o: Option<usize>, k: &str, c: &str, on: bool) -> Entry { Entry { orig_line: o, key: k.into(), command: c.into(), enabled: on } }
    #[test]
    fn rbind_dan_lainnya_aman() {
        let src = ["# komentar", "bind KEYBOARD C \"e prone\"", "rbind KEYBOARD X \"server\"", "bind KEYBOARD E \"painkiller\"", "bind MOUSE1 \"example\"", "set foo \"bar\"", ""].join("\r\n");
        let d = split_doc(&src);
        let b = parse_binds(&d.lines);
        assert_eq!(b.iter().map(|x| x.1.as_str()).collect::<Vec<_>>(), vec!["C", "E"]);
        assert_eq!(join_doc(&d), src);
        assert_eq!(parse_command("\"say \\\"hi\\\" there\"").unwrap(), "say \"hi\" there");
        let out = apply_changes(&d.lines, &[en(Some(1), "C", "e prone2", true), en(Some(3), "E", "painkiller", false), en(None, "u", "e parkour5", true)]).unwrap();
        assert!(out.contains(&"rbind KEYBOARD X \"server\"".to_string()) && out.contains(&"bind MOUSE1 \"example\"".to_string()) && out.contains(&"# komentar".to_string()));
        assert!(out.contains(&"bind KEYBOARD C \"e prone2\"".to_string()) && !out.iter().any(|l| l.contains("painkiller")) && out.contains(&"bind KEYBOARD U \"e parkour5\"".to_string()));
        assert_eq!(out.last().unwrap(), "");
        assert!(apply_changes(&d.lines, &[en(None, "A B", "x", true)]).is_err());
    }
}
