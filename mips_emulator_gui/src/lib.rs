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
	eframe::start_web(canvas_id, Box::new(|cc| {
		util::set_default_fonts(&cc.egui_ctx);
		util::set_ui_theme(&cc.egui_ctx, true);
		
		Box::new(EmuGui::default())
	}))
}
