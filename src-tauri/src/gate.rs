use crate::{appcfg, appcfg::Gate, util};
use serde::Serialize;
use serde_json::Value;
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const PROBES: u32 = 5;
const TIMEOUT_MS: u64 = 2000;
static LOCK: Mutex<()> = Mutex::new(());

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateResult {
    pub id: String, pub status: &'static str,
    pub ping: Option<u32>, pub jitter: Option<u32>, pub loss: Option<u32>, pub min: Option<u32>, pub max: Option<u32>,
    pub at: u64, pub note: Option<String>,
}
fn bad(g: &Gate, status: &'static str, loss: Option<u32>, note: Option<String>, at: u64) -> GateResult {
    GateResult { id: g.id.clone(), status, ping: None, jitter: None, loss, min: None, max: None, at, note }
}
/// Ping = rata-rata waktu TCP connect. Jitter = rata-rata |selisih| antar probe sukses berurutan.
/// Loss = probe gagal / total x 100. Bukan ICMP, jadi bisa berbeda dengan ping in-game.
pub fn measure(g: &Gate) -> GateResult {
    let at = util::now_ms();
    let addrs: Vec<SocketAddr> = (g.host.as_str(), g.port).to_socket_addrs().map(|a| a.collect()).unwrap_or_default();
    let Some(addr) = addrs.iter().find(|a| a.is_ipv4()).or(addrs.first()).copied() else {
        return bad(g, "UNAVAILABLE", None, Some("Hostname resolution failed".into()), at);
    };
    let mut ok: Vec<u32> = vec![];
    let mut refused = false;
    let mut note: Option<String> = None;
    for i in 0..PROBES {
        let t = Instant::now();
        match TcpStream::connect_timeout(&addr, Duration::from_millis(TIMEOUT_MS)) {
            Ok(_) => ok.push(t.elapsed().as_millis() as u32),
            Err(e) => {
                if e.kind() == ErrorKind::ConnectionRefused { refused = true; }
                if note.is_none() {
                    note = Some(match e.kind() {
                        ErrorKind::TimedOut => "TIMEOUT".to_string(),
                        ErrorKind::ConnectionRefused => "ECONNREFUSED".to_string(),
                        k => format!("{:?}", k),
                    });
                }
            }
        }
        if i < PROBES - 1 { std::thread::sleep(Duration::from_millis(150)); }
    }
    let loss = (PROBES - ok.len() as u32) * 100 / PROBES;
    if ok.is_empty() { return bad(g, if refused { "UNAVAILABLE" } else { "OFFLINE" }, Some(loss), note, at); }
    let n = ok.len() as u32;
    let sum: u32 = ok.iter().sum();
    let ping = ((sum as f64) / (n as f64)).round() as u32;
    let jitter = if n > 1 {
        let d: u32 = ok.windows(2).map(|w| if w[0] > w[1] { w[0] - w[1] } else { w[1] - w[0] }).sum();
        ((d as f64) / ((n - 1) as f64)).round() as u32
    } else { 0 };
    GateResult {
        id: g.id.clone(), status: "ONLINE", ping: Some(ping), jitter: Some(jitter), loss: Some(loss),
        min: ok.iter().min().copied(), max: ok.iter().max().copied(), at, note: None,
    }
}
/// Satu pengukuran pada satu waktu; ketiga gerbang diukur paralel.
pub fn measure_all() -> Result<Vec<GateResult>, String> {
    let _g = LOCK.lock().map_err(|_| "lock gagal".to_string())?;
    let hs: Vec<_> = appcfg::gates().iter().cloned().map(|g| std::thread::spawn(move || measure(&g))).collect();
    hs.into_iter().map(|h| h.join().map_err(|_| "pengukuran gagal".to_string())).collect()
}
fn get_json(url: &str, secs: u64) -> Option<Value> {
    ureq::get(url).timeout(Duration::from_secs(secs)).call().ok()?.into_json().ok()
}
fn num(v: &Value) -> Option<u64> { v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())) }

#[derive(Serialize)]
pub struct ServerInfo { pub clients: u64, pub max: u64 }
pub fn server_info_any() -> Option<ServerInfo> {
    for g in appcfg::gates() {
        if let Some(v) = get_json(&format!("http://{}:{}/dynamic.json", g.host, g.port), 3) {
            if let (Some(clients), Some(max)) = (num(&v["clients"]), num(&v["sv_maxclients"])) {
                return Some(ServerInfo { clients, max });
            }
        }
    }
    None
}
#[derive(Serialize)]
pub struct DiscordInfo { pub name: String, pub members: u64, pub online: u64 }
pub fn discord() -> Option<DiscordInfo> {
    let link = appcfg::link("discord")?;
    let code = link.rsplit('/').next()?;
    let v = get_json(&format!("https://discord.com/api/v9/invites/{}?with_counts=true", code), 4)?;
    Some(DiscordInfo {
        name: v["guild"]["name"].as_str().unwrap_or("Discord").to_string(),
        members: v["approximate_member_count"].as_u64()?,
        online: v["approximate_presence_count"].as_u64().unwrap_or(0),
    })
}
pub fn open_link(key: &str) -> Result<(), String> {
    match appcfg::link(key) {
        Some(u) if u.starts_with("https://") => open::that(u).map_err(|e| e.to_string()),
        _ => Err("Link belum dikonfigurasi.".into()),
    }
}
