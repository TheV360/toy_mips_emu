use super::*;

pub(super) struct MemoryWindowState {
	look: MemoryPosition,
	interp: MemoryInterpretation,
	core: usize,
}
impl Default for MemoryWindowState {
	fn default() -> Self {
		MemoryWindowState {
			look: MemoryPosition::ProgramCounter,
			interp: MemoryInterpretation::Instruction,
			core: 0,
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MemoryInterpretation {
	Instruction, Text,
}

impl MemoryWindowState {
	pub(super) fn show(&mut self, cores: &mut Vec<Core>, mem: &mut Memory, ctx: &egui::Context) {
		use MemoryPosition::*;
		use MemoryInterpretation::*;
		// https://github.com/emilk/egui/blob/master/egui_demo_lib/src/demo/scrolling.rs
		// https://github.com/emilk/egui/blob/master/egui_demo_lib/src/demo/mod.rs
		
		egui::Window::new("Memory Monitor").show(ctx, |ui| {
			let MemoryWindowState {
				look,
				interp,
				core: core_i,
			} = self;
			
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
			
			fn v_divider(ui: &mut egui::Ui) -> egui::Response {
				ui.add_sized([4.0, ui.available_height()], egui::Separator::default().spacing(0.0).vertical())
			}
			
			egui::ScrollArea::vertical().auto_shrink([false; 2]).show_rows(
				ui, row_height, row_num,
				|ui, row_range| {
					egui::Grid::new(match interp {
						Instruction => "MemoryIns",
						Text => "MemoryText",
					}).min_col_width(1.0).show(ui, |ui| {
						for row in row_range {
							if row % 2 == 1 {
								let rect = egui::Rect::from_min_size(ui.cursor().min, egui::Vec2::new(f32::INFINITY, row_height));
								let rect = rect.expand2(0.5 * ui.spacing().item_spacing);
								ui.painter().rect_filled(rect, 0.0, ui.style().visuals.faint_bg_color);
							}
							
							let addr = (row as u32) << row_eat.trailing_zeros();
							let delay_slot = core.inner.after_delay;
							
							let pc = core.inner.pc;
							let epc = core.inner.cp0[Cp0Register::EPC];
							
							let line_highlight = match addr {
								_ if Some(addr) == delay_slot => Some(("→", egui::Color32::WHITE, egui::Color32::BLUE)),
								_ if delay_slot.is_some() && addr == pc => Some(("→", egui::Color32::BLACK, egui::Color32::from_rgb(0xFE, 0x80, 0x19))),
								_ if delay_slot.is_none() && addr == pc => Some(("→", egui::Color32::BLACK, egui::Color32::YELLOW)),
								_ if addr == epc => Some(("⚠", egui::Color32::WHITE, egui::Color32::RED)),
								_ => None,
							};
							
							if let Some((label, color, bg_color)) = line_highlight {
								ui.label(egui::RichText::new(label).color(color).background_color(bg_color));
							} else { ui.add_sized([12.0, ui.available_height()], egui::Label::new("")); }
							
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
							
							v_divider(ui);
							
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
							
							v_divider(ui);
							
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
}
