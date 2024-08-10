use once_cell::sync::OnceCell;
use wasm_bindgen::prelude::*;

pub static ITCH_PREFIX: OnceCell<String> = OnceCell::new();

#[wasm_bindgen]
pub fn set_itch_prefix(path: String) {
    let mut path = std::path::PathBuf::from(path);
    let _ = path.pop();
    let path = path.to_str().unwrap().to_string();
    let _ = ITCH_PREFIX.get_or_init(|| path);
}

#[wasm_bindgen]
pub fn run_app() {
    crate::run()
}
