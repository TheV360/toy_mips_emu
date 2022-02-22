use std::time::Instant;

use eframe::{egui, epi};

use crate::chip::{Cpu, Register};
use crate::display::mmio_display;

pub struct EmuGui {
	cpu: Cpu,
	play: bool,
	timer: CpuTimer,
	mem_interp: MemoryInterpretation,
	mem_look: MemoryPosition,
	virt_screen: VirtScreen,
}

struct VirtScreen {
	look: MemoryPosition,
	cells: (usize, usize),
	size: egui::Vec2,
}

#[allow(dead_code)]
enum CpuTimer {
	/// Timer that steps at a set interval in microseconds, using `std::time`.
	/// Because of this, it will not work in WASM builds.
	Micro { interval: u64, last: Option<Instant>, },
	
	/// Simpler timer that steps after a set amount of ticks. Not accurate at
	/// all, but works well enough for WASM builds.
	Frames { interval: f32, left: usize, },
}
impl CpuTimer {
	/// Makes a `Micro`second timer
	#[allow(dead_code)]
	fn micro(interval: u64) -> Self {
		CpuTimer::Micro { interval, last: None, }
	}
	
	/// Makes a `Frame` timer
	#[allow(dead_code)]
	fn frames(interval: f32) -> Self {
		CpuTimer::Frames { interval, left: 0, }
	}
	
	/// Returns how many times the CPU should step.
	fn tick(&mut self) -> usize {
		use CpuTimer::*;
		match self {
			Micro { interval, last } => {
				let now = Instant::now();
				match last {
					None => { *last = Some(now); 0 },
					Some(last_tick) => {
						let since = now.duration_since(*last_tick);
						let times = (since.as_micros() as u64 / *interval) as usize;
						if times > 0 { *last = Some(now); }
						times
					}
				}
			},
			Frames { interval, left } => {
				if *left > 0 {
					*left -= 1; 0
				} else if *interval > 1.0 {
					*left = interval.round() as usize; 1
				} else {
					interval.recip().round() as usize
				}
			},
		}
	}
	
	/// Resets the timer. Can be called while still reset, that's fine too.
	fn reset(&mut self) {
		use CpuTimer::*;
		match self {
			Micro { last, .. } => *last = None,
			Frames { left, ..} => *left = 0,
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MemoryPosition {
	ProgramCounter,
	Position(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MemoryInterpretation {
	Instruction, Text,
}

const PRG_TEXT: &[u8] = include_bytes!("../program/out.text.bin");
const PRG_DATA: &[u8] = include_bytes!("../program/out.data.bin");

impl Default for EmuGui {
	fn default() -> Self {
		let mut cpu = Cpu::default();
		reset_cpu(&mut cpu, false);
		
		EmuGui {
			cpu,
			play: false,
			#[cfg(target_arch = "wasm32")]
			timer: CpuTimer::frames(16.0),
			#[cfg(not(target_arch = "wasm32"))]
			timer: CpuTimer::micro(100_000),
			mem_look: MemoryPosition::Position(0x00_0000),
			mem_interp: MemoryInterpretation::Instruction,
			virt_screen: VirtScreen {
				look: MemoryPosition::Position(0x01_0000),
				cells: (16, 16),
				size: egui::vec2(16.0, 16.0),
			},
		}
	}
}

fn reset_cpu(cpu: &mut Cpu, reset_mem: bool) {
	cpu.halt = false;
	cpu[Register::gp] = 0x1800;
	cpu[Register::sp] = 0x3FFC;
	
	if reset_mem { cpu.mem.fill(0); }
	
	cpu.mem[0x00_0000..][..PRG_TEXT.len()].copy_from_slice(PRG_TEXT);
	cpu.mem[0x00_2000..][..PRG_DATA.len()].copy_from_slice(PRG_DATA);
	cpu.pc = 0x00_0000;
}

impl epi::App for EmuGui {
	fn name(&self) -> &str {
		"Cool Swag MIPS Emulator"
	}
	
	fn max_size_points(&self) -> egui::Vec2 {
		egui::Vec2::splat(f32::INFINITY)
	}
	
	fn setup(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame, _storage: Option<&dyn epi::Storage>) {
		use eframe::egui::{Color32, Stroke, /*Rounding,*/ Style, Visuals};
		use eframe::egui::style::{Widgets, WidgetVisuals};
		
		const DARK_MODE: bool = true;
		
		let style = if DARK_MODE {
			Style {
				animation_time: 0.0,
				visuals: Visuals {
					dark_mode: false,
					popup_shadow: Default::default(),
					window_shadow: Default::default(),
					collapsing_header_frame: true,
					window_corner_radius: 0.0,
					// window_rounding: Rounding::none(),
					widgets: Widgets {
						noninteractive: WidgetVisuals {
							bg_fill: Color32::from_gray(16), // window background
							bg_stroke: Stroke::new(1.0, Color32::from_gray(64)), // separators, indentation lines, window outlines
							fg_stroke: Stroke::new(1.0, Color32::WHITE), // normal text color
							corner_radius: 0.0,
							// rounding: Rounding::none(),
							expansion: 0.0,
						},
						..Widgets::dark()
					},
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
					window_corner_radius: 0.0,
					// window_rounding: Rounding::none(),
					widgets: Widgets {
						noninteractive: WidgetVisuals {
							bg_fill: Color32::from_gray(235), // window background
							bg_stroke: Stroke::new(1.0, Color32::from_gray(190)), // separators, indentation lines, window outlines
							fg_stroke: Stroke::new(1.0, Color32::BLACK), // normal text color
							corner_radius: 0.0,
							// rounding: Rounding::none(),
							expansion: 0.0,
						},
						..Widgets::light()
					},
					..Visuals::light()
				},
				..Default::default()
			}
		};
		ctx.set_style(style);
		
		use eframe::egui::{FontDefinitions, FontData, FontFamily::*};
		
		let mut font_defs = FontDefinitions::default();
		
		let fonts = [
			// ("emoji",   include_bytes!("../fonts/emoji.ttf").to_vec()),
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
		
		let family = &mut font_defs.fonts_for_family;
		
		family.insert(
			Monospace,
			vec![
				"iosevka".to_owned(),
				// "emoji".to_owned(),
			],
		);
		family.insert(
			Proportional,
			vec![
				"sf_pro".to_owned(),
				"iosevka".to_owned(),
				// "emoji".to_owned(),
			],
		);
		
		ctx.set_fonts(font_defs);
	}
	
	fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
		let Self { cpu, .. } = self;
		
		// TODO: figure out what the hell will happen
		// if someone wants to inspect a byte at a time instead of a word at a time...
		// TODO: fix font sizes / maybe include some of my own fonts
		
		if cpu.halt { self.play = false; }
		if self.play {
			for _ in 0..self.timer.tick() { cpu.tick(); }
			ctx.request_repaint();
		} else {
			self.timer.reset();
		}
		
		egui::TopBottomPanel::top("Title").show(ctx, |ui| {
			if frame.is_web() {
				ui.heading("MIPS I Emulator");
				ui.separator();
			}
			
			ui.horizontal(|ui| {
				ui.monospace(format!("PC: 0x{:08X}", cpu.pc));
				
				ui.separator();
				
				if ui.button("Reset")
				.on_hover_text("Resets the CPU's state -- the memory,\nthe registers, the PC, everything.")
				.clicked() {
					println!("~~ Reset CPU ~~");
					reset_cpu(cpu, true);
				}
				
				ui.add_enabled_ui(!cpu.halt, |ui| {
					if ui.add_enabled(!self.play, egui::Button::new("Step"))
					.on_hover_text("Steps the CPU forward a single instruction.")
					.on_disabled_hover_text("The CPU has halted, and needs to reset\nbefore it can do more.")
					.clicked() {
						cpu.tick();
					}
					
					let play_text = if self.play { "⏸" } else { "▶" };
					if ui.button(play_text)
					.on_hover_text("Play or pause execution.")
					.on_disabled_hover_text("The CPU has halted, and needs to reset\nbefore it can do more.")
					.clicked() {
						self.play = !self.play;
					}
				});
				
				use CpuTimer::*;
				match &mut self.timer {
					Micro { interval, .. } => {
						ui.add(
							egui::Slider::new(interval, 10..=10_000_000)
								.suffix(" μs")
								.logarithmic(true)
						).on_hover_text("Frequency of CPU steps, in microseconds.\n10 μs = 1 step every 10 microseconds, and so on.");
					},
					Frames { interval, .. } => {
						ui.add(
							egui::Slider::new(interval, 0.001f32..=128.0f32)
								.suffix(" fr")
								.logarithmic(true)
						).on_hover_text("Frequency of CPU steps, in frames.\nFractional frames means multiple steps per frame.");
					},
				}
				
				/*
				ui.add(
					egui::Slider::new(&mut self.tick_ms, 10..=10_000_000)
						.suffix(" Hz")
						.logarithmic(true)
				).on_hover_text("Frequency of CPU steps.\n10 Hz = 10 steps every second, and so on -- to a ludicrous degree.");
				*/
			});
		});
		
		egui::CentralPanel::default().show(ctx, |_|());
		
		// egui::Window::new("Settings blah blah")
		// 	.show(ctx, |ui| { ctx.settings_ui(ui); });
		
		self.show_memory_monitor(ctx);
		self.show_register_monitor(ctx);
		self.show_mmio_display(ctx);
	}
}

impl EmuGui {
	fn show_register_monitor(&mut self, ctx: &egui::CtxRef) {
		let Self { cpu, .. } = self;
		
		egui::Window::new("Register Monitor")
			.resizable(false)
			.show(ctx,
		|ui| {
			egui::Grid::new("Registers")
				.striped(true)
				.show(ui,
			|ui| {
				for reg in 0..32 {
					let reg_e = Register::from(reg);
					let reg_val = cpu[reg_e];
					
					ui.vertical_centered(|ui| {
						ui.set_min_width(80.0);
						ui.label(format!("{reg_e:?} ({reg:02})"));
						ui.monospace(format!("0x{reg_val:08X}"));
					});
					
					if reg % 4 == 3 { ui.end_row(); }
				}
			});
		});
	}
	
	fn show_memory_monitor(&mut self, ctx: &egui::CtxRef) {
		let Self { cpu, .. } = self;
		
		egui::Window::new("Memory Monitor").show(ctx, |ui| {
			use MemoryPosition::*;
			use MemoryInterpretation::*;
			// https://github.com/emilk/egui/blob/master/egui_demo_lib/src/apps/demo/scrolling.rs
			// https://github.com/emilk/egui/blob/master/egui_demo_lib/src/apps/demo/mod.rs
			
			let mem_looked = match self.mem_look {
				ProgramCounter => (cpu.pc >> 2).saturating_sub(3) << 2,
				Position(p) => p,
			};
			
			ui.horizontal(|ui| {
				let ml = &mut self.mem_look;
				let mi = &mut self.mem_interp;
				
				egui::ComboBox::from_id_source("Memory Interpretation")
					.selected_text(format!("{:?}", mi))
					.show_ui(ui,
				|ui| {
					ui.selectable_value(mi, Instruction, "Instruction");
					ui.selectable_value(mi, Text, "Text (UTF-8)");
				});
				
				ui.selectable_value(ml, ProgramCounter, "PC");
				ui.selectable_value(ml, Position(0x00_0000), ".text");
				ui.selectable_value(ml, Position(0x00_2000), ".data");
				ui.selectable_value(ml, Position(0x01_0000), "MMIO");
				
				if let Position(pos) = ml {
					if ui.small_button("←").clicked() {
						*pos = pos.saturating_sub(0x10);
					}
					if ui.small_button("→").clicked() {
						*pos = pos.saturating_add(0x10);
					}
				}
			});
			
			ui.separator();
			
			egui::Grid::new("Memory")
				.striped(true)
				.min_col_width(32.0)
				.show(ui,
			|ui| {
				for i in 0..16u32 {
					let addr = mem_looked.saturating_add(i << 2);
					
					if cpu.pc == addr {
						ui.add(
							egui::Label::new(
								egui::RichText::new("→")
								.color(egui::Color32::BLACK)
								.background_color(egui::Color32::from_rgb(255, 255, 0))
							)
						);
					} else {
						ui.label("");
					}
					ui.monospace(format!("0x{addr:08X}"));
					
					let word = cpu.get_word(addr);
					ui.monospace(format!("0x{word:08X}"));
					
					match self.mem_interp {
						Instruction => {
							let ins = cpu.get_word(addr);
							
							if let Some(disasm) = cpu.get_disassembly(ins) {
								ui.monospace(disasm);
							} else {
								ui.label("No idea");
							}
						},
						Text => {
							let text = &cpu.mem[addr as usize..][..4];
							let text = String::from_utf8_lossy(text)
								.into_owned();
							
							let text = text.chars().map(|c| {
								match c as u32 {
									0x00..=0x1F => {
										char::from_u32(c as u32 + 0x2400)
										.unwrap_or(char::REPLACEMENT_CHARACTER)
									},
									0x7F => '\u{2421}',
									_ => c,
								}
							}).collect::<String>();
							
							ui.monospace(text);
						}
					}
					
					ui.end_row();
				}
			});
		});
	}
	
	fn show_mmio_display(&mut self, ctx: &egui::CtxRef) {
		let Self {
			cpu,
			virt_screen,
			..
		} = self;
		
		egui::Window::new("Virtual Display").show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.label("Cells: ");
				ui.add(
					egui::DragValue::new(&mut virt_screen.cells.0)
						.clamp_range(2..=128)
						.speed(0.125)
				);
				ui.label("×");
				ui.add(
					egui::DragValue::new(&mut virt_screen.cells.1)
						.clamp_range(2..=128)
						.speed(0.125)
				);
				
				ui.separator();
				
				ui.label("Size: ");
				ui.add(
					egui::DragValue::new(&mut virt_screen.size.x)
						.max_decimals(0)
						.clamp_range(4..=64)
						.speed(0.125)
						.suffix("px")
				);
				ui.label("×");
				ui.add(
					egui::DragValue::new(&mut virt_screen.size.y)
						.max_decimals(0)
						.clamp_range(4..=64)
						.speed(0.125)
						.suffix("px")
				);
			});
			
			ui.separator();
			
			ui.horizontal(|ui| {
				let vl = &mut virt_screen.look;
				
				ui.selectable_value(vl, MemoryPosition::ProgramCounter, "PC");
				ui.selectable_value(vl, MemoryPosition::Position(0x00_0000), ".text");
				ui.selectable_value(vl, MemoryPosition::Position(0x00_2000), ".data");
				ui.selectable_value(vl, MemoryPosition::Position(0x01_0000), "MMIO");
				
				if let MemoryPosition::Position(look) = &mut virt_screen.look {
					if ui.add_enabled(
						*look > 0x00_0000, egui::Button::new("←").small()
					).clicked() {
						*look = look.saturating_sub((virt_screen.cells.0 as u32) << 2);
					}
					if ui.add_enabled(
						*look < 0x01_0000, egui::Button::new("→").small()
					).clicked() {
						*look = look.saturating_add((virt_screen.cells.0 as u32) << 2).min(0x01_0000);
					}
				}
			});
			
			ui.separator();
			
			let look = match virt_screen.look {
				MemoryPosition::Position(n) => n,
				MemoryPosition::ProgramCounter => cpu.pc,
			} as usize;
			let mem_take = virt_screen.cells.0 * virt_screen.cells.1 * 4;
			ui.vertical_centered_justified(|ui| {
				mmio_display(ui, &cpu.mem[look..][..mem_take], virt_screen.cells, virt_screen.size);
			});
		});
	}
}
