use std::time::{Instant, Duration};

use eframe::{egui, epi};

use crate::chip::{Cpu, Register};
use crate::display::mmio_display;

pub struct EmuGui {
	cpu: Cpu,
	play: bool,
	tick_ms: u64,
	last_tick: Option<Instant>,
	mem_interp: MemoryInterpretation,
	mem_look: MemoryPosition,
	scr_look: usize,
	scr_cells: (usize, usize),
	scr_size: egui::Vec2,
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

const PRG_TEXT: &[u8] = include_bytes!("../bitmap_example.text.bin");
const PRG_DATA: &[u8] = include_bytes!("../bitmap_example.data.bin");

impl Default for EmuGui {
	fn default() -> Self {
		let mut cpu = Cpu::default();
		reset_cpu(&mut cpu, false);
		
		EmuGui {
			cpu,
			play: false,
			tick_ms: 100_000,
			last_tick: None,
			mem_look: MemoryPosition::Position(0x00_0000),
			mem_interp: MemoryInterpretation::Instruction,
			scr_look: 0x01_0000,
			scr_cells: (16, 16),
			scr_size: egui::vec2(16.0, 16.0),
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
	
	fn setup(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame, _storage: Option<&dyn epi::Storage>) {
		use eframe::egui::{Color32, Stroke, Style, Visuals, style::{Widgets, WidgetVisuals}};
		
		let s = Style {
			animation_time: 0.0,
			visuals: Visuals {
				dark_mode: false,
				popup_shadow: Default::default(),
				window_shadow: Default::default(),
				collapsing_header_frame: true,
				window_corner_radius: 0.0,
				widgets: Widgets {
					noninteractive: WidgetVisuals {
						bg_fill: Color32::from_gray(235), // window background
						bg_stroke: Stroke::new(1.0, Color32::from_gray(190)), // separators, indentation lines, window outlines
						fg_stroke: Stroke::new(1.0, Color32::BLACK), // normal text color
						corner_radius: 0.0,
						expansion: 0.0,
					},
					..Widgets::light()
				},
				..Visuals::light()
			},
			..Default::default()
		};
		ctx.set_style(s);
		
		// ctx.fonts().
	}
	
	fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
		let Self { cpu, .. } = self;
		
		// TODO: switch to chrono maybe for the interval stuff
		//       (i found a way around things, but chrono is needed for wasm)
		// TODO: merge memory viewer & "decomp" viewer.
		// TODO: figure out what the hell will happen
		// if someone wants to inspect a byte at a time instead of a word at a time...
		// TODO: fix font sizes / maybe include some of my own fonts
		
		if cpu.halt { self.play = false; }
		if self.play {
			// TODO: is this good to use ?
			let now = Instant::now();
			match self.last_tick {
				None => self.last_tick = Some(now),
				Some(last_tick) => {
					let since = now.duration_since(last_tick);
					let times = since.as_micros() as u64 / self.tick_ms;
					if times > 0 {
						for _ in 0..times { cpu.tick(); }
						self.last_tick = Some(now);
					}
				},
			}
			ctx.request_repaint();
		} else {
			self.last_tick = None;
		}
		
		egui::TopBottomPanel::top("Title").show(ctx, |ui| {
			if frame.is_web() {
				ui.heading("💻 MIPS Emulator");
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
				
				ui.add(
					egui::Slider::new(&mut self.tick_ms, 10..=10_000_000)
						.suffix(" μs")
						.logarithmic(true)
				).on_hover_text("Frequency of CPU steps.\n10 μs = 1 step every 10 microseconds, and so on.");
				
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
				
				ui.radio_value(ml, ProgramCounter, "PC");
				ui.radio_value(ml, Position(0x00_0000), ".text");
				ui.radio_value(ml, Position(0x00_2000), ".data");
				ui.radio_value(ml, Position(0x01_0000), "MMIO");
				
				if let Position(pos) = ml {
					if ui.small_button("⬅").clicked() {
						*pos = pos.saturating_sub(0x10);
					}
					if ui.small_button("➡").clicked() {
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
							egui::Label::new(egui::RichText::new("➡").background_color(egui::Color32::from_rgb(255, 255, 0)))
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
							
							if let Some((ins_name, ins_fmt)) = cpu.get_instruction_info(ins) {
								use crate::chip::InsFormat::*;
								
								let rs = Register::from(((ins >> 21) & Cpu::REGISTER_SIZE) as usize);
								let rt = Register::from(((ins >> 16) & Cpu::REGISTER_SIZE) as usize);
								let rd = Register::from(((ins >> 11) & Cpu::REGISTER_SIZE) as usize);
								
								match ins_fmt {
									R => {
										let shamt = (ins >> 6) & 0x1F;
										ui.monospace(format!("{ins_name} {rd:?}, {rs:?}, {rt:?}; {shamt}"));
									},
									I => {
										let imm = ins & 0xFFFF;
										ui.monospace(format!("{ins_name} {rt:?}, {rs:?}, 0x{imm:X}"));
									},
									J => {
										let j_addr = (ins & 0x03FF_FFFF) << 2;
										ui.monospace(format!("{ins_name} 0x{j_addr:08X}"));
									},
									Sys => { ui.monospace("syscall"); },
								}
							} else {
								ui.label("No idea");
							}
						},
						Text => {
							let text = &cpu.mem[addr as usize..][..4];
							let text = String::from_utf8_lossy(text)
								.into_owned()
								.replace('\n', "␊");
							
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
			scr_look,
			scr_cells,
			scr_size,
			..
		} = self;
		
		egui::Window::new("Virtual Display").show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.label("Cells: ");
				ui.add(
					egui::DragValue::new(&mut scr_cells.0)
						.clamp_range(2..=128)
						.speed(0.125)
				);
				ui.label("×");
				ui.add(
					egui::DragValue::new(&mut scr_cells.1)
						.clamp_range(2..=128)
						.speed(0.125)
				);
				
				ui.separator();
				
				ui.label("Size: ");
				ui.add(
					egui::DragValue::new(&mut scr_size.x)
						.max_decimals(0)
						.clamp_range(4..=64)
						.speed(0.125)
						.suffix("px")
				);
				ui.label("×");
				ui.add(
					egui::DragValue::new(&mut scr_size.y)
						.max_decimals(0)
						.clamp_range(4..=64)
						.speed(0.125)
						.suffix("px")
				);
			});
			
			ui.separator();
			
			ui.horizontal(|ui| {
				ui.radio_value(scr_look, 0x00_0000, ".text");
				ui.radio_value(scr_look, 0x00_2000, ".data");
				ui.radio_value(scr_look, 0x01_0000, "MMIO");
				
				if ui.add_enabled(
					*scr_look > 0x00_0000, egui::Button::new("⬅").small()
				).clicked() {
					*scr_look = scr_look.saturating_sub(scr_cells.0 << 2);
				}
				if ui.add_enabled(
					*scr_look < 0x01_0000, egui::Button::new("➡").small()
				).clicked() {
					*scr_look = scr_look.saturating_add(scr_cells.0 << 2)
						.min(0x01_0000);
				}
			});
			
			ui.separator();
			
			let mem_take = scr_cells.0 * scr_cells.1 * 4;
			mmio_display(ui, &cpu.mem[*scr_look..][..mem_take], *scr_cells, *scr_size);
		});
	}
}
