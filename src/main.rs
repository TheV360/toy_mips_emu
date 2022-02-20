// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod chip;
mod gui; use gui::EmuGui;
mod display;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
	let options = eframe::NativeOptions::default();
	eframe::run_native(Box::new(EmuGui::default()), options);
}
