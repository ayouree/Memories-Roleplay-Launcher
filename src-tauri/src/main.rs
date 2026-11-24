#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod appcfg; mod cache; mod config; mod crash; mod fivem; mod gate; mod gta; mod keybind; mod nickname; mod util;
use tauri::Manager;

/// Semua pekerjaan berat dijalankan di thread terpisah agar UI tidak membeku.
async fn blk<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string())?
}
#[tauri::command] async fn gate_measure() -> Result<Vec<gate::GateResult>, String> { blk(gate::measure_all).await }
#[tauri::command] async fn gate_info() -> Result<Option<gate::ServerInfo>, String> { blk(|| Ok(gate::server_info_any())).await }
#[tauri::command] async fn fivem_detect() -> Result<fivem::Detect, String> { blk(|| Ok(fivem::detect())).await }
#[tauri::command] async fn fivem_browse() -> Result<fivem::Detect, String> { blk(fivem::browse).await }
#[tauri::command] async fn fivem_connect(id: String) -> Result<fivem::Running, String> { blk(move || fivem::connect(&id)).await }
#[tauri::command] async fn cache_scan() -> Result<Vec<cache::CacheInfo>, String> { blk(cache::scan).await }
#[tauri::command] async fn cache_clear(ids: Vec<String>) -> Result<Vec<cache::ClearResult>, String> { blk(move || cache::clear(ids)).await }
#[tauri::command] async fn keybind_load() -> Result<keybind::KeybindData, String> { blk(keybind::load).await }
#[tauri::command] async fn keybind_save(payload: keybind::SavePayload) -> Result<keybind::KeybindData, String> { blk(move || keybind::save(payload)).await }
#[tauri::command] async fn gta_load() -> Result<gta::GtaData, String> { blk(gta::load).await }
#[tauri::command] async fn gta_save(payload: gta::SavePayload) -> Result<gta::GtaData, String> { blk(move || gta::save(payload)).await }
#[tauri::command] async fn crash_list() -> Result<crash::CrashList, String> { blk(crash::list).await }
#[tauri::command] async fn crash_info(id: String) -> Result<String, String> { blk(move || crash::info_text(&id)).await }
#[tauri::command] async fn crash_open(id: String) -> Result<(), String> { blk(move || crash::open_folder(&id)).await }
#[tauri::command]
async fn crash_zip(app: tauri::AppHandle, id: String) -> Result<crash::ZipRes, String> {
    let dl = app.path().download_dir().map_err(|e| e.to_string())?;
    blk(move || crash::zip(&id, dl)).await
}
#[tauri::command] async fn link_open(key: String) -> Result<(), String> { blk(move || gate::open_link(&key)).await }
#[tauri::command] async fn community_discord() -> Result<Option<gate::DiscordInfo>, String> { blk(|| Ok(gate::discord())).await }
#[tauri::command] async fn nickname_get() -> Result<nickname::NickInfo, String> { blk(nickname::get).await }
#[tauri::command] async fn nickname_set(name: String) -> Result<nickname::NickInfo, String> { blk(move || nickname::set(&name)).await }
#[tauri::command] async fn config_get() -> Result<config::AppCfg, String> { blk(|| Ok(config::app_cfg())).await }
#[tauri::command] async fn config_set(patch: config::Patch) -> Result<config::AppCfg, String> { blk(move || config::apply(patch)).await }

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            gate_measure, gate_info, fivem_detect, fivem_browse, fivem_connect, cache_scan, cache_clear,
            keybind_load, keybind_save, gta_load, gta_save, crash_list, crash_info, crash_open, crash_zip,
            link_open, community_discord, nickname_get, nickname_set, config_get, config_set
        ])
        .run(tauri::generate_context!())
        .expect("gagal menjalankan aplikasi");
}
