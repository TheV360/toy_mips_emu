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
	mem_win: MemoryWindowState,
}

struct Core {
	// naming is kinda screwed up but...
	inner: Cpu,
	play: bool,
	timer: CpuTimer,
	
	reg_state: RegisterMonitorState,
	
	breakpoints: Vec<u32>,
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
			
			reg_state: RegisterMonitorState::Cpu,
			
			breakpoints: Vec::new(),
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RegisterMonitorState { Cpu, Cp0, }

struct MemoryWindowState {
	look: MemoryPosition,
	interp: MemoryInterpretation,
	core: usize,
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
				/*Core {
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
				},*/
			],
			mem,
			screen: VirtScreen {
				look: MemoryPosition::Position(0x01_0000),
				cells: (16, 16),
				size: egui::vec2(16.0, 16.0),
			},
			mem_win: MemoryWindowState {
				look: MemoryPosition::ProgramCounter,
				interp: MemoryInterpretation::Instruction,
				core: 0,
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
	mem.set_slice(0x00_0000, PRG_TEXT);
	mem.set_slice(0x00_2000, PRG_DATA);
	
	// HACK: init MMIO page because I'm lazy.
	// I later access it directly, which is a sin.
	mem.set_byte(0x01_0000, 0);
}

impl eframe::App for EmuGui {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		let Self { cpus: cores, mem, .. } = self;
		
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
			ui.horizontal(|ui| {
				if frame.is_web() {
					ui.heading("Toy MIPS I Emulator");
					ui.separator();
					ui.hyperlink_to("GitHub", "https://github.com/TheV360/toy_mips_emu");
					ui.separator();
				}
				
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
							
							// TODO: wtf is going on with the width of this window
							
							let regs = [
								(BadVAddr, "Short for \"Bad Virtual Address\".\nHolds the address that failed to be fetched, if any."),
								(Status,   "asddslgjlkajsflkasgj"),
								(Cause,    "asddslgjlkajsflkasgj 2"),
								(EPC,      "Short for \"Error Program Counter\".\nHolds the address of the most recent instruction\nthat caused an exception."),
							];
							
							for (reg_e, tooltip) in regs {
								let reg = reg_e as usize;
								let reg_val = core.inner.cp0[reg_e];
								
								ui.vertical_centered(|ui| {
									ui.label(format!("{reg_e:?} ({reg:02})")).on_hover_text(tooltip);
									ui.horizontal(|ui| {
										ui.monospace(format!("{reg_val:#010X}"));
										ui.monospace(format!("{reg_val:#034b}"));
									});
								});
								
								ui.end_row();
							}
						})
				}
			});
		}
	}
	
	fn show_memory_monitor(&mut self, ctx: &egui::Context) {
		let Self { cpus: cores, mem, mem_win, .. } = self;
		
		use MemoryPosition::*;
		use MemoryInterpretation::*;
		// https://github.com/emilk/egui/blob/master/egui_demo_lib/src/demo/scrolling.rs
		// https://github.com/emilk/egui/blob/master/egui_demo_lib/src/demo/mod.rs
		
		egui::Window::new("Memory Monitor").show(ctx, |ui| {
			let MemoryWindowState {
				look,
				interp,
				core: core_i,
			} = mem_win;
			
			// TODO: aoaoauauagh. actually scroll to whatever's in here.
			let looked = {
				let core = &cores[*core_i];
				
				match look {
					ProgramCounter => (core.inner.pc >> 2).saturating_sub(3) << 2,
					LastException => (core.inner.cp0[Cp0Register::EPC] >> 2).saturating_sub(3) << 2,
					Position(p) => *p,
				}
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
					
					ui.separator();
					
					ui.label("In terms of...");
					for i in 0..cores.len() {
						ui.selectable_value(core_i, i, format!("Core {}", i + 1));
					}
				});
			});
			
			ui.separator();
			
			let row_height = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
			
			const TEXT_WIDTH: usize = 8;
			let row_eat = match interp {
				Instruction => Cpu::INSTRUCTION_BYTES,
				Text => TEXT_WIDTH,
			};
			
			let core = &mut cores[*core_i];
			
			let row_num = mips_emulator::mem::MEMORY_SIZE / row_eat;
			
			egui::ScrollArea::vertical().auto_shrink([false; 2]).show_rows(
				ui, row_height, row_num,
				|ui, row_range| {
					egui::Grid::new(match interp {
						Instruction => "MemoryIns",
						Text => "MemoryText",
					}).min_col_width(12.0).show(ui, |ui| {
						for row in row_range {
							if row % 2 == 1 {
								let rect = egui::Rect::from_min_size(ui.cursor().min, egui::Vec2::new(f32::INFINITY, row_height));
								let rect = rect.expand2(0.5 * ui.spacing().item_spacing);
								ui.painter().rect_filled(rect, 0.0, ui.style().visuals.faint_bg_color);
							}
							
							let addr = (row as u32) << row_eat.trailing_zeros();
						
						let pc = core.inner.pc;
						let epc = core.inner.cp0[Cp0Register::EPC];
						
						let line_highlight = match addr {
							_ if addr == pc => Some(("→", egui::Color32::BLACK, egui::Color32::from_rgb(255, 255, 0))),
							_ if addr == epc => Some(("⚠", egui::Color32::WHITE, egui::Color32::RED)),
							_ => None,
						};
						
						if let Some((label, color, bg_color)) = line_highlight {
								ui.label(egui::RichText::new(label).color(color).background_color(bg_color));
							} else { ui.label(""); }
							
							let brk = core.breakpoints.iter().enumerate().find(|(_, brk)| **brk == addr);
							
							if ui.radio(brk.is_some(), "").clicked() {
								match brk {
									Some((i, _)) => {
										core.breakpoints.swap_remove(i);
									},
									None => core.breakpoints.push(addr),
								}
							}
							
							ui.monospace(format!("{addr:#010X}"));
							
							ui.separator();
							
							if let Some(bytes) = mem.get_slice(addr, row_eat) {
								ui.horizontal(|ui| {
									for b in bytes {
										ui.monospace(format!("{b:02X}"));
									}
									// ui.monospace(bytes.iter().fold(
									// 	String::with_capacity(bytes.len() * 3),
									// 	|d, b| d + &format!("{b:02X} ")
									// ).trim_end());
								});
							} else {
								ui.label("Page Fault :)");
							}
							
							ui.separator();
							
							match interp {
								Instruction => {
									let word = mem.get_word(addr).unwrap_or(0);
									
									if let Some(disasm) = Cpu::get_disassembly(word) {
										ui.monospace(disasm);
									} else {
										ui.label("Invalid");
									}
								},
								Text => {
									let bytes = mem.get_slice(addr, TEXT_WIDTH).unwrap_or(&[0u8; TEXT_WIDTH]);
									
									let text = String::from_utf8_lossy(bytes)
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
				}
			);
		});
	}
	
	fn show_mmio_display(&mut self, ctx: &egui::Context) {
		let Self {
			cpus,
			mem,
			screen,
			..
		} = self;
		
		egui::Window::new("Virtual Display").show(ctx, |ui| {
			ui.menu_button("View", |ui| {
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
			ui.vertical_centered_justified(|ui| {
				mmio_display(ui, mem.get_slice(look as u32, mem_take).unwrap(), screen.cells, screen.size);
			});
		});
	}
}
