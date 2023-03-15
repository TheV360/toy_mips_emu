use super::*;

pub(super) struct VirtScreen {
	look: MemoryPosition,
	cells: (usize, usize),
	size: egui::Vec2,
}
impl Default for VirtScreen {
	fn default() -> Self {
		VirtScreen {
			look: MemoryPosition::Position(0x01_0000),
			cells: (16, 16),
			size: egui::vec2(16.0, 16.0),
		}
	}
}

impl VirtScreen {
	pub(super) fn show(&mut self, mem: &Memory, ctx: &egui::Context) {
		egui::Window::new("Virtual Display").show(ctx, |ui| {
			ui.menu_button("View", |ui| {
				ui.horizontal(|ui| {
					ui.label("Cells:");
					ui.add(
						egui::DragValue::new(&mut self.cells.0)
							.clamp_range(2..=128)
							.speed(0.125)
					);
					ui.label("×");
					ui.add(
						egui::DragValue::new(&mut self.cells.1)
							.clamp_range(2..=128)
							.speed(0.125)
					);
					
					ui.separator();
					
					ui.label("Size:");
					ui.add(
						egui::DragValue::new(&mut self.size.x)
							.max_decimals(0)
							.clamp_range(4..=64)
							.speed(0.125)
							.suffix("px")
					);
					// self.size.y = self.size.x;
					ui.label("×");
					ui.add(
						egui::DragValue::new(&mut self.size.y)
							.max_decimals(0)
							.clamp_range(4..=64)
							.speed(0.125)
							.suffix("px")
					);
				});
				
				ui.separator();
				
				ui.horizontal(|ui| {
					let vl = &mut self.look;
					
					// ui.selectable_value(vl, MemoryPosition::ProgramCounter, "PC");
					ui.selectable_value(vl, MemoryPosition::Position(0x00_0000), ".text");
					ui.selectable_value(vl, MemoryPosition::Position(0x00_2000), ".data");
					ui.selectable_value(vl, MemoryPosition::Position(0x01_0000), "MMIO");
					
					if let MemoryPosition::Position(look) = &mut self.look {
						if ui.add_enabled(
							*look > 0x00_0000, egui::Button::new("←").small()
						).clicked() {
							*look = look.saturating_sub((self.cells.0 as u32) << 2);
						}
						if ui.add_enabled(
							*look < 0x01_0000, egui::Button::new("→").small()
						).clicked() {
							*look = look.saturating_add((self.cells.0 as u32) << 2).min(0x01_0000);
						}
					}
				});
			});
			
			ui.separator();
			
			let look = match self.look {
				MemoryPosition::Position(n) => n,
				_ => unimplemented!(),
			} as usize;
			let mem_take = self.cells.0 * self.cells.1 * 4;
			ui.vertical_centered_justified(|ui| {
				if let Some(mem_slice) = mem.get_slice(look as u32, mem_take) {
					mmio_display(ui, mem_slice, self.cells, self.size);
				} else {
					ui.label(format!(
						"oops! {look:#010X} isn't in a valid page,\n\
						or is across a page boundary.\n\
						you didn't do anything wrong, it's my bad;\n\
						i have a half-baked \"sparse memory\" system..."
					));
				}
			});
		});
	}
}
