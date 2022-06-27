mod util;

mod gui;
mod display;
mod timer;
pub use gui::EmuGui;

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
	eframe::start_web(canvas_id, Box::new(EmuGui::default()))
}
