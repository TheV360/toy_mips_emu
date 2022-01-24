// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod chip;
mod gui; use gui::EmuGui;

fn main() {
	let options = eframe::NativeOptions::default();
	eframe::run_native(Box::new(EmuGui::default()), options);
}
