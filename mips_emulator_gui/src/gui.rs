use eframe::egui;

use mips_emulator::mem::Memory;
use mips_emulator::chip::{Cpu, Register, Cp0Register, ExceptionCause};

use crate::util;

use crate::display::mmio_display;
use crate::timer::CpuTimer;

mod registers; use registers::RegisterMonitorState;
mod memory; use memory::MemoryWindowState;
mod display; use display::VirtScreen;
mod assembler; use assembler::AssemblerWindowState;

pub struct EmuGui {
	dark_theme: bool,
	
	cpus: Vec<Core>,
	focused_core: usize,
	
	mem: Memory,
	// places: MemoryPlaces,
	
	screen: VirtScreen,
	mem_win: MemoryWindowState,
	assember: AssemblerWindowState,
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
enum MemoryPosition {
	ProgramCounter,
	LastException,
	Position(u32),
}

const PRG_TEXT: &[u8] = include_bytes!("../../program/out.text.bin");
const PRG_DATA: &[u8] = include_bytes!("../../program/out.data.bin");

impl Default for EmuGui {
	fn default() -> Self {
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
			focused_core: 0,
			
			mem: {
				let mut mem = Memory::default();
				reset_mem(&mut mem);
				mem
			},
			// places: MemoryPlaces::default(),
			
			screen: VirtScreen::default(),
			mem_win: MemoryWindowState::default(),
			assember: AssemblerWindowState::default(),
		}
	}
}

fn reset_cpu(cpu: &mut Cpu) {
	cpu.cp0.halt = false;
	
	cpu[Register::gp] = 0x1800;
	cpu[Register::sp] = 0x3FFC;
	
	cpu.pc = 0x00_0000;
	cpu.after_delay = None;
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
				
				if ui.button("Add core").clicked() {
					cores.push(Core::default());
				}
			});
			
			ui.separator();
			
			let mut remove_which_core = None;
			
			fn v_separator(ui: &mut egui::Ui) -> egui::Response {
				ui.add(egui::Separator::default().spacing(0.0).vertical())
			}
			
			egui::Grid::new("Cores")
				.striped(true)
				.min_col_width(0.0)
				.show(ui, |ui| {
				for (i, core) in cores.iter_mut().enumerate() {
					ui.radio_value(&mut self.focused_core, i, format!("Core {}", i + 1));
					
					v_separator(ui);
					
					ui.monospace(format!("PC: {:#010X}", core.inner.pc));
					
					v_separator(ui);
					
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
					
					v_separator(ui);
					
					if ui.small_button("×")
					.on_hover_text("Remove this Core")
					.clicked() {
						remove_which_core = Some(i);
					}
					
					ui.end_row();
				}
			});
			
			if let Some(core) = remove_which_core {
				if cores.len() > 1 {
					cores.remove(core);
					self.focused_core = self.focused_core.min(cores.len().saturating_sub(1));
				}
			}
			
			ui.separator();
		});
		
		egui::CentralPanel::default().show(ctx, |_|());
		
		self.assember.show(&mut self.mem, ctx);
		
		self.mem_win.show(&mut self.cpus[self.focused_core], &mut self.mem, ctx);
		
		for (i, core) in self.cpus.iter_mut().enumerate() {
			core.reg_state.show((i, &mut core.inner), ctx);
		}
		
		self.screen.show(&self.mem, ctx);
	}
}
