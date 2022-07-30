use super::*;

type AssemblerError = (usize, &'static str);

#[derive(Default)]
pub(super) struct AssemblerWindowState {
	source: String,
	
	result: Option<Result<Vec<u32>, Vec<AssemblerError>>>,
	insert_at: u32,
}

impl AssemblerWindowState {
	pub(super) fn show(&mut self, mem: &mut Memory, ctx: &egui::Context) {
		egui::Window::new("Assembler").show(ctx, |ui| {
			ui.add(egui::TextEdit::multiline(&mut self.source)
				.code_editor()
				.desired_rows(16)
				.desired_width(f32::INFINITY)
			);
			
			ui.horizontal_wrapped(|ui| {
				if ui.button("Assemble").clicked() {
					self.result = Some(Self::assemble(&self.source));
				}
				let has_errors = matches!(self.result, Some(Err(_)));
				if ui.add_enabled(has_errors, egui::Button::new("Clear Errors")).clicked() {
					self.result = None;
				}
			});
			
			ui.separator();
			
			let error_count = match self.result {
				Some(Err(ref err)) => err.len(),
				_ => 0,
			};
			
			ui.collapsing(format!("{error_count} errors"), |ui| {
				if let Some(Err(ref inner)) = self.result {
					for &(line, err) in inner {
						ui.monospace(format!("#{line}: {err}"));
					}
				} else {
					ui.label("No errors!");
				}
			});
			
			ui.separator();
			
			ui.add_enabled_ui(matches!(self.result, Some(Ok(_))), |ui| {
				ui.horizontal_wrapped(|ui| {
					ui.label("Insert code at: ");
					ui.add(
						egui::DragValue::new(&mut self.insert_at)
							.clamp_range(0..=0x0_FFFF) // arbitrary limit
					);
					
					if ui.button("Insert").clicked() {
						if let Some(Ok(ref slice)) = self.result {
							let bytes: Vec<u8> = slice.iter().cloned().flat_map(u32::to_le_bytes).collect();
							mem.set_slice(self.insert_at, &bytes);
						}
					}
				});
			});
		});
	}
	
	fn assemble(source: &str) -> Result<Vec<u32>, Vec<(usize, &'static str)>> {
		let mut code = Vec::new();
		let mut errors = Vec::new();
		
		for (i, l) in source.lines().enumerate()
			.map(|(i, s)| (i, s.trim_start()))
			.filter(|&(_, s)| !(s.is_empty() || s.starts_with(['#', '.'])))
			.map(|(i, s)| (i, s.split('#').next().unwrap().trim_end()))
		{
			match Cpu::from_assembly(l) {
				Ok(w) => code.push(w),
				Err(m) => errors.push((i, m)),
			};
		}
		
		if errors.is_empty() { Ok(code) } else { Err(errors) }
	}
}
