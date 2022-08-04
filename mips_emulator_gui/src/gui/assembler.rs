use super::*;

type AssemblerError = (usize, &'static str);

#[derive(Default)]
pub(super) struct AssemblerWindowState {
	source: String,
	show_line_nums: bool,
	
	result: Option<Result<Vec<u32>, Vec<AssemblerError>>>,
	insert_at: u32,
}

impl AssemblerWindowState {
	fn layout_line_numbers(source: &str, ui: &egui::Ui, show: bool) -> egui::text::LayoutJob {
		use egui::{TextStyle, TextFormat, text::LayoutJob};
		
		let monospace = TextStyle::Monospace.resolve(ui.style());
		let text_color = ui.visuals().override_text_color
			.unwrap_or_else(|| ui.visuals().widgets.inactive.text_color());
			
		if !show {
			return LayoutJob::simple(source.to_owned(), monospace, text_color, 0.0);
		}
		
		let line_text_style = TextFormat::simple(monospace.clone(), text_color);
		let line_number_style = TextFormat::simple(monospace, text_color.linear_multiply(0.25));
		
		let mut layout = LayoutJob::default();
		let mut i = 1;
		for l in source.split_inclusive('\n') {
			layout.append(&format!("{i:3}. "), 0.0, line_number_style.clone());
			layout.append(l, 0.0, line_text_style.clone());
			i += 1;
		}
		if source.is_empty() || source.ends_with('\n') {
			layout.append(&format!("{i:3}. "), 0.0, line_number_style);
		}
		layout
	}
	
	pub(super) fn show(&mut self, mem: &mut Memory, ctx: &egui::Context) {
		egui::Window::new("Assembler").show(ctx, |ui| {
			self.show_line_nums = !ui.add(
				egui::TextEdit::multiline(&mut self.source)
				.font(egui::TextStyle::Monospace)
				.code_editor()
				.desired_rows(8)
				.desired_width(f32::INFINITY)
				.layouter(&mut |ui, source, wrap_width| {
					let mut layout_job = Self::layout_line_numbers(source, ui, self.show_line_nums);
					layout_job.wrap.max_width = wrap_width;
					ui.fonts().layout_job(layout_job)
				})
			).has_focus();
			
			if ui.button("Assemble").clicked() {
				self.result = Some(Self::assemble(&self.source));
			}
			
			ui.separator();
			
			ui.horizontal_wrapped(|ui| {
				match self.result {
					None => ui.label("Waiting for assembly..."),
					Some(Ok(_)) => ui.label("Assembled without errors."),
					Some(Err(ref err)) => {
						ui.label(format!("Assembled with {} errors.", err.len()))
					},
				};
				
				let has_errors = matches!(self.result, Some(Err(_)));
				if ui.add_enabled(has_errors, egui::Button::new("Clear Errors").small()).clicked() {
					self.result = None;
				}
			});
			
			if let Some(Err(ref inner)) = self.result {
				egui::ScrollArea::vertical()
					.auto_shrink([false, true])
					.max_height(70.0)
					.show(ui, |ui| {
						for &(line, err) in inner {
							ui.monospace(format!("Line {line}: {err}"));
						}
					});
			}
			
			ui.separator();
			
			ui.horizontal_wrapped(|ui| {
				ui.label("Insert code at: ");
				ui.add(
					egui::DragValue::new(&mut self.insert_at)
						.clamp_range(0..=0x0_FFFF) // arbitrary limit
				);
				
				if ui.add_enabled(matches!(self.result, Some(Ok(_))), egui::Button::new("Insert")).clicked() {
					if let Some(Ok(ref slice)) = self.result {
						let bytes: Vec<u8> = slice.iter().cloned().flat_map(u32::to_le_bytes).collect();
						mem.set_slice(self.insert_at, &bytes);
					}
				}
			});
		});
	}
	
	fn assemble(source: &str) -> Result<Vec<u32>, Vec<(usize, &'static str)>> {
		let mut code = Vec::new();
		let mut errors = Vec::new();
		
		for (i, l) in source.lines().enumerate()
			.map(|(i, s)| (i, s.trim_start()))
			.filter(|&(_, s)| !(s.is_empty() || s.starts_with(['#', '.'])))
			.map(|(i, s)| (i, s.split_once('#').unwrap().0.trim_end()))
		{
			match Cpu::from_assembly(l) {
				Ok(w) => code.push(w),
				Err(m) => errors.push((i + 1, m)),
			};
		}
		
		if errors.is_empty() { Ok(code) } else { Err(errors) }
	}
}
