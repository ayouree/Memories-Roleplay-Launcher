use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Deserialize, Clone)]
pub struct Gate { pub id: String, pub name: String, pub host: String, pub port: u16 }
#[derive(Deserialize)]
struct File { gates: Vec<Gate>, links: HashMap<String, String> }
static F: OnceLock<File> = OnceLock::new();
fn f() -> &'static File {
    F.get_or_init(|| serde_json::from_str(include_str!("../../src/shared/app-config.json")).expect("app-config.json tidak valid"))
}
pub fn gates() -> &'static [Gate] { &f().gates }
pub fn link(k: &str) -> Option<&'static String> { f().links.get(k) }
