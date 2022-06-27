// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod util;

mod gui; use gui::EmuGui;
mod display;
mod timer;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
	let name = "Toy MIPS I Emulator";
	let options = eframe::NativeOptions {
		drag_and_drop_support: true,
		..Default::default()
	};
	eframe::run_native(name, options, Box::new(|cc| {
		util::set_default_fonts(&cc.egui_ctx);
		util::set_ui_theme(&cc.egui_ctx, true);
		
		Box::new(EmuGui::default())
	}));
}
