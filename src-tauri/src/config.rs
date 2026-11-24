use crate::util;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Disabled { pub key: String, pub command: String }

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct Cfg {
    pub version: u32,
    pub connect_method: String,
    pub last_selected_gate: String,
    pub fivem_path: String,
    pub disabled_keybinds: Vec<Disabled>,
}
impl Default for Cfg {
    fn default() -> Self {
        Cfg { version: 1, connect_method: "explorer".into(), last_selected_gate: "main".into(), fivem_path: String::new(), disabled_keybinds: vec![] }
    }
}
pub fn data_dir() -> PathBuf { util::appdata().join("MemoriesLauncher") }
fn file() -> PathBuf { data_dir().join("config.json") }
pub fn get() -> Cfg { fs::read_to_string(file()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default() }
pub fn save(c: &Cfg) -> Result<(), String> {
    fs::create_dir_all(data_dir()).map_err(|e| e.to_string())?;
    let tmp = file().with_extension("json.tmp");
    let txt = serde_json::to_string_pretty(c).map_err(|e| e.to_string())?;
    fs::write(&tmp, txt).map_err(|e| e.to_string())?;
    fs::rename(&tmp, file()).map_err(|e| e.to_string())
}
pub fn update(f: impl FnOnce(&mut Cfg)) -> Result<Cfg, String> {
    let mut c = get();
    f(&mut c);
    save(&c)?;
    Ok(c)
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppCfg { pub connect_method: String }
pub fn app_cfg() -> AppCfg { AppCfg { connect_method: get().connect_method } }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch { pub connect_method: Option<String> }
pub fn apply(p: Patch) -> Result<AppCfg, String> {
    if let Some(m) = &p.connect_method {
        if m != "explorer" && m != "direct" { return Err("Metode connect tidak valid.".into()); }
        update(|c| c.connect_method = m.clone())?;
    }
    Ok(app_cfg())
}
