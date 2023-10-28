// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod util;

mod gui; use gui::EmuGui;
mod display;
mod timer;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
	let logger = 
		simple_logger::SimpleLogger::new()
			.with_level(log::LevelFilter::Debug);
	
	let name = "Toy MIPS I Emulator";
	let options = eframe::NativeOptions {
		initial_window_size: Some(eframe::emath::vec2(1280.0, 720.0)),
		drag_and_drop_support: true,
		..Default::default()
	};
	eframe::run_native(name, options, Box::new(|cc| {
		util::set_default_fonts(&cc.egui_ctx);
		util::set_ui_theme(&cc.egui_ctx, true);
		
		Box::<EmuGui>::default()
	}))
}

#[cfg(target_arch = "wasm32")]
fn main() {
	eframe::WebLogger::init(log::LevelFilter::Debug).ok();
	
	let canvas = "main_canvas";
	let options = eframe::WebOptions::default();
	wasm_bindgen_futures::spawn_local(async {
		eframe::WebRunner::new()
			.start(
				canvas, options,
				Box::new(|cc| {
					util::set_default_fonts(&cc.egui_ctx);
					util::set_ui_theme(&cc.egui_ctx, true);
					
					Box::<EmuGui>::default()
				})
			)
			.await
			.expect("failed to start eframe");
	});
}
