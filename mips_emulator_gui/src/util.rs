use eframe::egui;

#[allow(dead_code)]
pub fn set_default_fonts(ctx: &egui::Context) {
	use eframe::egui::{FontDefinitions, FontData, FontFamily::*};
	
	let mut font_defs = FontDefinitions::default();
	
	let fonts = [
		// keep file size down in WASM builds by not including an emoji font
		#[cfg(not(target_arch = "wasm"))]
		("emoji",   include_bytes!("../fonts/emoji.ttf").to_vec()),
		// but these other two can stay.
		("sf_pro",  include_bytes!("../fonts/sf-pro.otf").to_vec()),
		("iosevka", include_bytes!("../fonts/iosevka-term.ttf").to_vec()),
	];
	let fonts = fonts.map(
		|(name, path)| {
			let name = name.to_owned();
			// let file = std::fs::read(path).unwrap();
			(name, path)
		}
	);
	
	let data = &mut font_defs.font_data;
	
	for (name, file) in fonts {
		data.insert(name, FontData::from_owned(file));
	}
	
	let family = &mut font_defs.families;
	
	family.insert(
		Monospace,
		vec![
			"iosevka".to_owned(),
			#[cfg(not(target_arch = "wasm"))]
			"emoji".to_owned(),
		],
	);
	family.insert(
		Proportional,
		vec![
			"sf_pro".to_owned(),
			"iosevka".to_owned(),
			#[cfg(not(target_arch = "wasm"))]
			"emoji".to_owned(),
		],
	);
	
	ctx.set_fonts(font_defs);
}

pub fn set_ui_theme(ctx: &egui::Context, dark_theme: bool) {
	use eframe::egui::{Rounding, Style, Visuals};
	use eframe::egui::style::Widgets;
	
	let style = if dark_theme {
		Style {
			animation_time: 0.0,
			visuals: Visuals {
				dark_mode: true,
				popup_shadow: Default::default(),
				window_shadow: Default::default(),
				collapsing_header_frame: true,
				window_rounding: Rounding::none(),
				widgets: Widgets::dark(),
				..Visuals::dark()
			},
			..Default::default()
		}
	} else {
		Style {
			animation_time: 0.0,
			visuals: Visuals {
				dark_mode: false,
				popup_shadow: Default::default(),
				window_shadow: Default::default(),
				collapsing_header_frame: true,
				window_rounding: Rounding::none(),
				widgets: Widgets::light(),
				..Visuals::light()
			},
			..Default::default()
		}
	};
	
	ctx.set_style(style);
}

pub fn replace_control_char(c: char) -> char {
	match c as u32 {
		0x00..=0x1F => {
			char::from_u32(c as u32 + 0x2400)
			.unwrap_or(char::REPLACEMENT_CHARACTER)
		},
		0x7F => '\u{2421}',
		_ => c,
	}
}
