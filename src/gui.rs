use std::time::{Instant, Duration};

use eframe::{egui, epi};

use crate::chip::{Cpu, Register};

pub struct EmuGui {
	cpu: Cpu,
	play: bool,
	last_tick: Option<Instant>,
	mem_interp: MemoryInterpretation,
	mem_look: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MemoryInterpretation {
	Instruction,
	Hexadecimal,
	Text,
	Color,
}

impl Default for EmuGui {
	fn default() -> Self {
		let mut cpu = Cpu::default();
		
		const PRG_TEXT: &[u8] = include_bytes!("../bitmap_example.text.bin");
		const PRG_DATA: &[u8] = include_bytes!("../bitmap_example.data.bin");
		
		cpu.mem[0x00_0000..][..PRG_TEXT.len()].copy_from_slice(PRG_TEXT);
		cpu.mem[0x00_2000..][..PRG_DATA.len()].copy_from_slice(PRG_DATA);
		cpu.pc = 0x00_0000;
		
		cpu[Register::gp] = 0x1800;
		cpu[Register::sp] = 0x3FFC;
		
		EmuGui {
			cpu,
			play: false,
			last_tick: None,
			mem_look: 0x00_0000,
			mem_interp: MemoryInterpretation::Instruction,
		}
	}
}

impl epi::App for EmuGui {
	fn name(&self) -> &str {
		"Cool Swag MIPS Emulator"
	}
	
	fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
		let Self { cpu, .. } = self;
		
		// TODO: switch to chrono maybe for the interval stuff
		// TODO: merge memory viewer & "decomp" viewer.
		// TODO: figure out what the hell will happen
		// if someone wants to inspect a byte at a time instead of a word at a time...
		
		if self.play {
			// TODO: is this good to use ?
			let now = Instant::now();
			match self.last_tick {
				None => self.last_tick = Some(now),
				Some(last_tick) => {
					if now.duration_since(last_tick) > Duration::from_millis(250) {
						cpu.tick();
						self.last_tick = Some(now);
					}
				},
			}
		} else {
			self.last_tick = None;
		}
		
		{
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
		
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.heading("💻 MIPS Emulator");
			
			ui.separator();
			
			ui.horizontal(|ui| {
				if ui.button(if self.play { "⏸" } else { "▶" }).clicked() {
					self.play = !self.play;
				}
				if ui.button("Step").clicked() {
					cpu.tick();
				}
				ui.monospace(format!("PC: 0x{:08X}", cpu.pc));
			});
			
			egui::Window::new("Register Monitor").show(ctx, |ui| {
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
			
			egui::Window::new("Memory Monitor").show(ctx, |ui| {
				use MemoryInterpretation::*;
				// https://github.com/emilk/egui/blob/master/egui_demo_lib/src/apps/demo/scrolling.rs
				// https://github.com/emilk/egui/blob/master/egui_demo_lib/src/apps/demo/mod.rs
				
				ui.horizontal(|ui| {
					let mi = &mut self.mem_interp;
					egui::ComboBox::from_id_source("Memory Interpretation")
						.selected_text(format!("{:?}", mi))
						.show_ui(ui,
					|ui| {
						ui.selectable_value(mi, Instruction, "Instruction");
						ui.selectable_value(mi, Hexadecimal, "Hexadecimal");
					});
					
					// TODO: ".text (Follow Program Counter)" could be a thing
					ui.radio_value(&mut self.mem_look, 0x00_0000, ".text");
					ui.radio_value(&mut self.mem_look, 0x00_2000, ".data");
					ui.radio_value(&mut self.mem_look, 0x01_0000, "MMI/O");
					
					if ui.small_button("⬅").clicked() {
						self.mem_look = self.mem_look.saturating_sub(0x10);
					}
					if ui.small_button("➡").clicked() {
						self.mem_look = self.mem_look.saturating_add(0x10);
					}
					
					if self.mem_look < 0x00_2000 {
						self.mem_look = (cpu.pc >> 2).saturating_sub(3) << 2;
					}
				});
				
				match self.mem_interp {
					Instruction => {
						egui::Grid::new("Memory")
							.striped(true)
							.show(ui,
						|ui| {
							for i in 0..16u32 {
								let ins_addr = self.mem_look.saturating_add(i << 2);
								let ins = cpu.get_word(ins_addr);
								
								ui.label(if cpu.pc == ins_addr { "➡" } else { "" });
								ui.monospace(format!("0x{ins_addr:08X}"));
								
								if let Some((ins_name, ins_fmt)) = cpu.get_instruction_info(ins) {
									use crate::chip::InsFormat::*;
									use crate::chip::Register::*;
									
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
								ui.end_row();
							}
						});
					},
					Hexadecimal => {
						egui::Grid::new("Memory")
							.striped(true)
							.show(ui,
						|ui| {
							for i in 0..16 {
								let d_addr = self.mem_look.saturating_add(i << 2);
								let d = cpu.get_word(d_addr);
								
								ui.label(if cpu.pc == d_addr { "➡" } else { "" });
								ui.monospace(format!("0x{d_addr:08X}"));
								ui.monospace(format!("0x{d:08X}"));
								
								ui.end_row();
							}
						});
					},
					_ => { ui.label("not impl"); }
				}
			});
		});
	}
}
