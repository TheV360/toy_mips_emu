use eframe::egui;

use mips_emulator::mem::Memory;
use mips_emulator::chip::{Cpu, Register, Cp0Register};

use crate::util;

use crate::display::mmio_display;
use crate::timer::CpuTimer;

pub struct EmuGui {
	dark_theme: bool,
	cpus: Vec<Core>,
	mem: Memory,
	screen: VirtScreen,
}

struct Core {
	// naming is kinda screwed up but...
	inner: Cpu,
	play: bool,
	timer: CpuTimer,
	
	mem_win: MemoryWindowState,
	reg_state: RegisterMonitorState,
}
impl Default for Core {
	fn default() -> Self {
		Core {
			inner: Cpu::default(),
			play: false,
			
			#[cfg(target_arch = "wasm32")]
			timer: CpuTimer::frames(16.0),
			#[cfg(not(target_arch = "wasm32"))]
			timer: CpuTimer::micro(100_000),
					
			mem_win: MemoryWindowState {
				look: MemoryPosition::ProgramCounter,
				interp: MemoryInterpretation::Instruction,
			},
			reg_state: RegisterMonitorState::Cpu,
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RegisterMonitorState {
	Cpu, Cp0,
}

struct MemoryWindowState {
	look: MemoryPosition,
	interp: MemoryInterpretation,
}

struct VirtScreen {
	look: MemoryPosition,
	cells: (usize, usize),
	size: egui::Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MemoryPosition {
	ProgramCounter,
	LastException,
	Position(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MemoryInterpretation {
	Instruction, Text,
}

const PRG_TEXT: &[u8] = include_bytes!("../../program/out.text.bin");
const PRG_DATA: &[u8] = include_bytes!("../../program/out.data.bin");

impl Default for EmuGui {
	fn default() -> Self {
		let mut mem = Memory::default();
		reset_mem(&mut mem);
		
		EmuGui {
			dark_theme: true,
			cpus: vec![
				Core {
					inner: {
						let mut cpu = Cpu::default();
						reset_cpu(&mut cpu);
						cpu
					},
					..Default::default()
				},
				Core {
					inner: {
						let mut cpu = Cpu::default();
						reset_cpu(&mut cpu);
						cpu
					},
					
					#[cfg(target_arch = "wasm32")]
					timer: CpuTimer::frames(18.0),
					#[cfg(not(target_arch = "wasm32"))]
					timer: CpuTimer::micro(110_000),
					
					..Default::default()
				},
			],
			mem,
			screen: VirtScreen {
				look: MemoryPosition::Position(0x01_0000),
				cells: (16, 16),
				size: egui::vec2(16.0, 16.0),
			},
		}
	}
}

fn reset_cpu(cpu: &mut Cpu) {
	cpu.cp0.halt = false;
	
	cpu[Register::gp] = 0x1800;
	cpu[Register::sp] = 0x3FFC;
	cpu.pc = 0x00_0000;
}

fn reset_mem(mem: &mut Memory) {
	mem.clear();
	mem.write_slice(0x00_0000, PRG_TEXT);
	mem.write_slice(0x00_2000, PRG_DATA);
	
	// HACK: init MMIO page because I'm lazy.
	// I later access it directly, which is a sin.
	mem.set_byte(0x01_0000, 0);
}

impl eframe::App for EmuGui {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		let Self { cpus: cores, mem, .. } = self;
		
		// TODO: figure out what the hell will happen
		// if someone wants to inspect a byte at a time instead of a word at a time...
		
		for core in cores.iter_mut() {
			if core.inner.cp0.halt { core.play = false; }
			if core.play {
				let ticked = core.timer.tick();
				for _ in 0..ticked { core.inner.tick(mem); }
				// if ticked > 0 { ctx.request_repaint(); }
				ctx.request_repaint();
			} else {
				core.timer.reset();
			}
		}
		
		egui::TopBottomPanel::top("Title").show(ctx, |ui| {
			if frame.is_web() {
				ui.horizontal(|ui| {
					ui.heading("Toy MIPS I Emulator");
					ui.separator();
					ui.hyperlink_to("GitHub Repository", "https://github.com/TheV360/toy_mips_emu");
				});
				ui.separator();
			}
			
			ui.horizontal(|ui| {
				let theme_str = if self.dark_theme { "Lite" } else { "Dark" };
				if ui.small_button(theme_str).clicked() {
					self.dark_theme = !self.dark_theme;
					util::set_ui_theme(ctx, self.dark_theme);
				}
				ui.separator();
				if ui.button("Open...").clicked() {
					
				}
			});
			
			for (i, core) in cores.iter_mut().enumerate() {
				ui.separator();
				
				ui.horizontal(|ui| {
					ui.monospace(format!("Core {}", i + 1));
					
					ui.separator();
					
					ui.monospace(format!("PC: {:#010X}", core.inner.pc));
					
					ui.separator();
					
					if ui.button("Reset")
					.on_hover_text("Resets the CPU's state -- the memory,\nthe registers, the PC, everything.")
					.clicked() {
						println!("~~ Reset CPU ~~");
						reset_cpu(&mut core.inner);
						reset_mem(mem);
					}
					
					ui.add_enabled_ui(!core.inner.cp0.halt, |ui| {
						if ui.add_enabled(!core.play, egui::Button::new("Step"))
						.on_hover_text("Steps the CPU forward a single instruction.")
						.on_disabled_hover_text("The CPU has halted, and needs to reset\nbefore it can do more.")
						.clicked() {
							core.inner.tick(mem);
						}
						
						let play_text = if core.play { "⏸" } else { "▶" };
						if ui.button(play_text)
						.on_hover_text("Play or pause execution.")
						.on_disabled_hover_text("The CPU has halted, and needs to reset\nbefore it can do more.")
						.clicked() {
							core.play = !core.play;
						}
					});
					
					match &mut core.timer {
						CpuTimer::Micro { interval, .. } => {
							ui.add(
								egui::Slider::new(interval, 10..=10_000_000u64)
									.suffix(" μs")
									.logarithmic(true)
							).on_hover_text("Frequency of CPU steps, in microseconds.\n10 μs = 1 step every 10 microseconds, and so on.");
						},
						CpuTimer::Frames { interval, .. } => {
							ui.add(
								egui::Slider::new(interval, 0.001f32..=128.0f32)
									.suffix(" fr")
									.logarithmic(true)
							).on_hover_text("Frequency of CPU steps, in frames.\nFractional frames means multiple steps per frame.");
						},
					}
				});
			}
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
	fn show_register_monitor(&mut self, ctx: &egui::Context) {
		let Self { cpus: cores, .. } = self;
		
		for (i, core) in cores.iter_mut().enumerate() {
			egui::Window::new(format!("Register Monitor (Core {})", i + 1))
				.resizable(false)
				.show(ctx,
			|ui| {
				use RegisterMonitorState::*;
				
				ui.horizontal_wrapped(|ui| {
					ui.selectable_value(&mut core.reg_state, Cpu, "CPU");
					ui.selectable_value(&mut core.reg_state, Cp0, "Coproc. 0");
				});
				
				ui.separator();
				
				match core.reg_state {
					Cpu =>
					egui::Grid::new("RegistersCpu")
						.striped(true)
						.show(ui, |ui| {
							for reg in 0..32 {
								let reg_e = Register::from(reg);
								let reg_val = core.inner[reg_e];
								
								ui.vertical_centered(|ui| {
									ui.set_min_width(80.0);
									ui.label(format!("{reg_e:?} ({reg:02})"));
									ui.monospace(format!("{reg_val:#010X}"));
								});
								
								if reg % 4 == 3 { ui.end_row(); }
							}
						}),
					Cp0 =>
					egui::Grid::new("RegistersCp0")
						.striped(true)
						.show(ui, |ui| {
							use Cp0Register::*;
							for reg_e in [ BadVAddr, Status, Cause, EPC ] {
								let reg = reg_e as usize;
								let reg_val = core.inner.cp0[reg_e];
								
								ui.label(format!("{reg_e:?} ({reg:02})"));
								ui.monospace(format!("{reg_val:#010X}"));
								ui.monospace(format!("{reg_val:#034b}"));
								ui.end_row();
							}
						})
				}
				// TODO: let user switch between these lol
			});
		}
	}
	
	fn show_memory_monitor(&mut self, ctx: &egui::Context) {
		let Self { cpus: cores, mem, .. } = self;
		
		for (i, core) in cores.iter_mut().enumerate() {
			use MemoryPosition::*;
			use MemoryInterpretation::*;
			// https://github.com/emilk/egui/blob/master/egui_demo_lib/src/apps/demo/scrolling.rs
			// https://github.com/emilk/egui/blob/master/egui_demo_lib/src/apps/demo/mod.rs
			
			let title = format!("Memory Monitor (Core {})", i + 1);
			egui::Window::new(title).show(ctx, |ui| {
				let MemoryWindowState {
					look,
					interp
				} = &mut core.mem_win;
				
				let looked = match look {
					ProgramCounter => (core.inner.pc >> 2).saturating_sub(3) << 2,
					LastException => (core.inner.cp0[Cp0Register::EPC] >> 2).saturating_sub(3) << 2,
					Position(p) => *p,
				};
				
				ui.horizontal(|ui| {
					ui.menu_button("View", |ui| {
						ui.label("See data as...");
						ui.selectable_value(interp, Instruction, "Instructions");
						ui.selectable_value(interp, Text, "Text (UTF-8)");
						
						ui.separator();
						
						ui.label("Jump to...");
						ui.horizontal_wrapped(|ui| {
							ui.selectable_value(look, ProgramCounter, "PC");
							ui.selectable_value(look, Position(0x00_0000), ".text");
							ui.selectable_value(look, Position(0x00_2000), ".data");
							ui.selectable_value(look, Position(0x01_0000), "MMIO");
							ui.selectable_value(look, LastException, "Exception");
						});
					});
					
					if ui.small_button("←").clicked() {
						*look = Position(looked.saturating_sub(0x10));
					}
					if ui.small_button("→").clicked() {
						*look = Position(looked.saturating_add(0x10));
					}
				});
				
				ui.separator();
				
				egui::Grid::new("Memory")
					.striped(true)
					.min_col_width(32.0)
					.show(ui,
				|ui| {
					for i in 0..16u32 {
						let addr = looked.saturating_add(i << 2);
						
						let pc = core.inner.pc;
						let epc = core.inner.cp0[Cp0Register::EPC];
						
						let line_highlight = match addr {
							_ if addr == pc => Some(("→", egui::Color32::BLACK, egui::Color32::from_rgb(255, 255, 0))),
							_ if addr == epc => Some(("⚠", egui::Color32::WHITE, egui::Color32::RED)),
							_ => None,
						};
						
						if let Some((label, color, bg_color)) = line_highlight {
							ui.add(
							egui::Label::new(
								egui::RichText::new(label)
								.color(color)
								.background_color(bg_color)
								)
							);
						} else {
							ui.label("");
						}
						ui.monospace(format!("{addr:#010X}"));
						
						let word = mem.get_word(addr).unwrap();
						ui.monospace(format!("{word:#010X}"));
						
						match core.mem_win.interp {
							Instruction => {
								let ins = mem.get_word(addr).unwrap();
								
								if let Some(disasm) = Cpu::get_disassembly(ins) {
									ui.monospace(disasm);
								} else {
									ui.label("Invalid");
								}
							},
							Text => {
								let text = &mem.get_word(addr).unwrap().to_le_bytes();
								let text = String::from_utf8_lossy(text)
									.into_owned();
								
								let text = text.chars()
									.map(util::replace_control_char)
									.collect::<String>();
								
								ui.monospace(text);
							}
						}
						
						ui.end_row();
					}
				});
			});
		}
	}
	
	fn show_mmio_display(&mut self, ctx: &egui::Context) {
		let Self {
			cpus,
			mem,
			screen,
			..
		} = self;
		
		egui::Window::new("Virtual Display").show(ctx, |ui| {
			// TODO: move into menu bar? (restore functionality of Size?)
			ui.collapsing("Settings", |ui| {
				ui.horizontal(|ui| {
					ui.label("Cells:");
					ui.add(
						egui::DragValue::new(&mut screen.cells.0)
							.clamp_range(2..=128)
							.speed(0.125)
					);
					ui.label("×");
					ui.add(
						egui::DragValue::new(&mut screen.cells.1)
							.clamp_range(2..=128)
							.speed(0.125)
					);
					
					ui.separator();
					
					ui.label("Size:");
					ui.add(
						egui::DragValue::new(&mut screen.size.x)
							.max_decimals(0)
							.clamp_range(4..=64)
							.speed(0.125)
							.suffix("px")
					);
					// screen.size.y = screen.size.x;
					ui.label("×");
					ui.add(
						egui::DragValue::new(&mut screen.size.y)
							.max_decimals(0)
							.clamp_range(4..=64)
							.speed(0.125)
							.suffix("px")
					);
				});
				
				ui.separator();
				
				ui.horizontal(|ui| {
					let vl = &mut screen.look;
					
					ui.selectable_value(vl, MemoryPosition::ProgramCounter, "PC");
					ui.selectable_value(vl, MemoryPosition::Position(0x00_0000), ".text");
					ui.selectable_value(vl, MemoryPosition::Position(0x00_2000), ".data");
					ui.selectable_value(vl, MemoryPosition::Position(0x01_0000), "MMIO");
					
					if let MemoryPosition::Position(look) = &mut screen.look {
						if ui.add_enabled(
							*look > 0x00_0000, egui::Button::new("←").small()
						).clicked() {
							*look = look.saturating_sub((screen.cells.0 as u32) << 2);
						}
						if ui.add_enabled(
							*look < 0x01_0000, egui::Button::new("→").small()
						).clicked() {
							*look = look.saturating_add((screen.cells.0 as u32) << 2).min(0x01_0000);
						}
					}
				});
			});
			
			ui.separator();
			
			let look = match screen.look {
				MemoryPosition::Position(n) => n,
				MemoryPosition::ProgramCounter => {
					cpus.iter()
						.find(|core| core.play)
						.map(|core| core.inner.pc)
						.unwrap_or_default()
				},
				_ => panic!("wtf it's not very useful to attach the screen to the err"),
			} as usize;
			let mem_take = screen.cells.0 * screen.cells.1 * 4;
			let (page, offset) = Memory::addr_to_indices(look as u32);
			ui.vertical_centered_justified(|ui| {
				mmio_display(ui, &mem.0[page].as_ref().unwrap()[offset..][..mem_take], screen.cells, screen.size);
			});
		});
	}
}
