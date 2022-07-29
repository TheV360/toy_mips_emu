use super::*;

pub(super) struct MemoryWindowState {
	look: MemoryPosition,
	interp: MemoryInterpretation,
	core: usize,
	
	edit: Option<(u32, String)>,
}
impl Default for MemoryWindowState {
	fn default() -> Self {
		MemoryWindowState {
			look: MemoryPosition::ProgramCounter,
			interp: MemoryInterpretation::Instruction,
			core: 0,
			
			edit: None,
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MemoryInterpretation {
	Instruction, Text,
	// IntU8, IntS8,
	// IntU16, IntS16,
	// IntU32, IntS32,
}
impl MemoryInterpretation {
	// would like a "str_to_bytes(&self, &str) -> Vec<u8>" kinda thing
	fn str_to_bytes(&self, a: &str) -> Result<Vec<u8>, &'static str> {
		use MemoryInterpretation::*;
		match self {
			Instruction => Ok(Cpu::from_assembly(a)?.to_le_bytes().to_vec()),
			Text => Ok(a.as_bytes().to_vec()),
			// _ => Err("unimplemented"),
		}
	}
}

impl MemoryWindowState {
	pub(super) fn show(&mut self, cores: &mut Vec<Core>, mem: &mut Memory, ctx: &egui::Context) {
		use MemoryPosition::*;
		use MemoryInterpretation::*;
		// https://github.com/emilk/egui/blob/master/egui_demo_lib/src/demo/scrolling.rs
		// https://github.com/emilk/egui/blob/master/egui_demo_lib/src/demo/mod.rs
		
		egui::Window::new("Memory Monitor").show(ctx, |ui| {
			
			// TODO: aoaoauauagh. actually scroll to whatever's in here.
			let looked = {
				let core = &cores[self.core];
				
				match self.look {
					ProgramCounter => (core.inner.pc >> 2).saturating_sub(3) << 2,
					LastException => (core.inner.cp0[Cp0Register::EPC] >> 2).saturating_sub(3) << 2,
					Position(p) => p,
				}
			};
			
			ui.horizontal(|ui| {
				ui.menu_button("View", |ui| {
					ui.label("See data as...");
					ui.selectable_value(&mut self.interp, Instruction, "Instructions");
					ui.selectable_value(&mut self.interp, Text, "Text (UTF-8)");
					
					ui.separator();
					
					ui.label("Jump to...");
					ui.horizontal_wrapped(|ui| {
						ui.selectable_value(&mut self.look, ProgramCounter, "PC");
						ui.selectable_value(&mut self.look, Position(0x00_0000), ".text");
						ui.selectable_value(&mut self.look, Position(0x00_2000), ".data");
						ui.selectable_value(&mut self.look, Position(0x01_0000), "MMIO");
						ui.selectable_value(&mut self.look, LastException, "Exception");
					});
					
					ui.separator();
					
					ui.label("In terms of...");
					for i in 0..cores.len() {
						ui.selectable_value(&mut self.core, i, format!("Core {}", i + 1));
					}
				});
			});
			
			ui.separator();
			
			let row_height = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
			
			const TEXT_WIDTH: usize = 8;
			let row_eat = match self.interp {
				Instruction => Cpu::INSTRUCTION_BYTES,
				Text => TEXT_WIDTH,
				// _ => Cpu::INSTRUCTION_BYTES,
			};
			
			let core = &mut cores[self.core];
			
			let row_num = mips_emulator::mem::MEMORY_SIZE / row_eat;
			
			fn v_divider(ui: &mut egui::Ui) -> egui::Response {
				ui.add_sized([4.0, ui.available_height()], egui::Separator::default().spacing(0.0).vertical())
			}
			
			egui::ScrollArea::vertical().auto_shrink([false; 2]).show_rows(
				ui, row_height, row_num,
				|ui, row_range| {
					egui::Grid::new(match self.interp {
						Instruction => "MemoryIns",
						Text => "MemoryText",
						// _ => "MemoryBytes",
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
							
							if let Some((e_addr, e_text)) = &mut self.edit {
								if addr == *e_addr {
									if ui.text_edit_singleline(e_text).lost_focus() {
										if let Ok(b) = self.interp.str_to_bytes(e_text) {
											mem.set_slice(*e_addr, &b);
										}
										self.edit = None;
									}
									
									ui.end_row();
									continue;
								}
							}
							
							let button_rich = match self.interp {
								Instruction => {
									let word = mem.get_word(addr).unwrap_or(0);
									
									if let Some(disasm) = Cpu::get_disassembly(word) {
										egui::RichText::new(&disasm).monospace()
									} else {
										egui::RichText::new("Invalid")
									}
								},
								Text => {
									let bytes = mem.get_slice(addr, TEXT_WIDTH)
										.unwrap_or(&[0u8; TEXT_WIDTH]);
									
									let text = String::from_utf8_lossy(bytes)
										.into_owned();
									
									let text = text.chars()
										.map(util::replace_control_char)
										.collect::<String>();
									
									egui::RichText::new(&text).monospace()
								},
								// _ => unimplemented!(),
							};
							
							let button_text = button_rich.text().to_owned();
							
							if ui.button(button_rich).double_clicked() {
								self.edit = Some((addr, button_text));
							}
							
							ui.end_row();
						}
					});
				}
			);
		});
	}
}
