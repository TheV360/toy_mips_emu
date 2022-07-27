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
	use eframe::egui::{Rounding, Style, Visuals, Stroke, Color32};
	use eframe::egui::style::{Widgets, WidgetVisuals, Selection};
	
	let style = if dark_theme {
		const GRUVBOX_BG: &[Color32] = &[
			Color32::from_rgb(0x28, 0x28, 0x28),
			Color32::from_rgb(0x32, 0x30, 0x2F),
			Color32::from_rgb(0x3C, 0x38, 0x36),
			Color32::from_rgb(0x50, 0x49, 0x45),
			// Color32::from_rgb(0x66, 0x5C, 0x54),
			// Color32::from_rgb(0x7C, 0x6F, 0x64),
			];
		const GRUVBOX_FG: &[Color32] = &[
			Color32::from_rgb(0xFB, 0xF1, 0xC7),
			Color32::from_rgb(0xEB, 0xDB, 0xB2),
			// Color32::from_rgb(0xD5, 0xC4, 0xA1),
			// Color32::from_rgb(0xBD, 0xAE, 0x93),
			// Color32::from_rgb(0xA8, 0x99, 0x84),
		];
		
		let fg_thickness = 1.0;
		let bg_thickness = 1.0;
		
		Style {
			animation_time: 0.0,
			visuals: Visuals {
				dark_mode: true,
				popup_shadow: Default::default(),
				window_shadow: Default::default(),
				collapsing_header_frame: true,
				window_rounding: Rounding::none(),
				widgets: Widgets {
					noninteractive: WidgetVisuals {
						fg_stroke: Stroke::new(fg_thickness, GRUVBOX_FG[1]),
						bg_fill: GRUVBOX_BG[0],
						bg_stroke: Stroke::new(bg_thickness, GRUVBOX_BG[3]),
						rounding: Rounding::none(),
						expansion: 0.0,
					},
					inactive: WidgetVisuals {
						fg_stroke: Stroke::new(fg_thickness, GRUVBOX_FG[0]),
						bg_fill: GRUVBOX_BG[0],
						bg_stroke: Stroke::new(bg_thickness, GRUVBOX_BG[3]),
						rounding: Rounding::none(),
						expansion: 0.0,
					},
					hovered: WidgetVisuals {
						fg_stroke: Stroke::new(fg_thickness, GRUVBOX_FG[0]),
						bg_fill: Color32::from_rgb(0x50, 0x49, 0x45),
						bg_stroke: Stroke::new(bg_thickness, GRUVBOX_BG[3]),
						rounding: Rounding::none(),
						expansion: 0.0,
					},
					active: WidgetVisuals {
						fg_stroke: Stroke::new(fg_thickness, GRUVBOX_FG[1]),
						bg_fill: Color32::from_rgb(0x50, 0x49, 0x45),
						bg_stroke: Stroke::new(bg_thickness, GRUVBOX_BG[3]),
						rounding: Rounding::none(),
						expansion: 0.0,
					},
					open: WidgetVisuals {
						fg_stroke: Stroke::new(fg_thickness, GRUVBOX_FG[0]),
						bg_fill: GRUVBOX_BG[3],
						bg_stroke: Stroke::new(0.0, GRUVBOX_BG[3]),
						rounding: Rounding::none(),
						expansion: 0.0,
					},
				},
				
				selection: Selection {
					bg_fill: Color32::from_rgb(0x50, 0x6F, 0x51),
					stroke: Stroke::new(fg_thickness, GRUVBOX_FG[0]),
				},
				
				faint_bg_color: GRUVBOX_BG[1],
				extreme_bg_color: GRUVBOX_BG[2],
				code_bg_color: Color32::TRANSPARENT,
				
				..Visuals::dark()
			},
			..Default::default()
		}
	} else {
		// it's based on plum theme but it just looks like beige light theme
		// it doesn't have the accent colors..
		// i just don't think egui can look good easily. it's either dark and
		// barely readable or light and extremely thin and grey everywhere..
		Style {
			animation_time: 0.0,
			visuals: Visuals {
				dark_mode: false,
				popup_shadow: Default::default(),
				window_shadow: Default::default(),
				collapsing_header_frame: true,
				window_rounding: Rounding::none(),
				widgets: Widgets {
					noninteractive: WidgetVisuals {
						fg_stroke: Stroke::new(1.0, Color32::BLACK),
						bg_stroke: Stroke::new(1.0, Color32::from_rgb(0x78, 0x60, 0x58)),
						bg_fill: Color32::from_rgb(0xD8, 0xD0, 0xC8),
						rounding: Rounding::none(),
						expansion: 0.0,
					},
					inactive: WidgetVisuals {
						fg_stroke: Stroke::new(1.0, Color32::BLACK),
						bg_stroke: Stroke::new(1.0, Color32::from_rgb(0xA8, 0x98, 0x90)),
						bg_fill: Color32::from_rgb(0xD8, 0xD0, 0xC8),
						rounding: Rounding::none(),
						expansion: 0.0,
					},
					hovered: WidgetVisuals {
						fg_stroke: Stroke::new(1.0, Color32::BLACK),
						bg_stroke: Stroke::new(1.0, Color32::from_rgb(0xA8, 0x98, 0x90)),
						bg_fill: Color32::from_rgb(0xB3, 0xA5, 0x9E),
						rounding: Rounding::none(),
						expansion: 0.0,
					},
					active: WidgetVisuals {
						fg_stroke: Stroke::new(1.0, Color32::BLACK),
						bg_stroke: Stroke::new(1.0, Color32::from_rgb(0x20, 0x1A, 0x18)),
						bg_fill: Color32::from_rgb(0x78, 0x60, 0x58),
						rounding: Rounding::none(),
						expansion: 0.0,
					},
					..Widgets::light()
				},
				
				faint_bg_color: Color32::from_rgb(0xC2, 0xAF, 0xA6),
				extreme_bg_color: Color32::from_rgb(0xFF, 0xFF, 0xE1),
				code_bg_color: Color32::TRANSPARENT,
				
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
