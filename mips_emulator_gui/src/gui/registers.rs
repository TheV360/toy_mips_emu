use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RegisterMonitorState { Cpu, Cp0, }

impl RegisterMonitorState {
	pub(super) fn show(&mut self, (i, cpu): (usize, &mut Cpu), ctx: &egui::Context) {
		egui::Window::new(format!("Register Monitor (Core {})", i + 1))
			.resizable(false)
			.min_width(16.0)
			.show(ctx,
		|ui| {
			use RegisterMonitorState::*;
			
			ui.horizontal_wrapped(|ui| {
				ui.selectable_value(self, Cpu, "CPU");
				ui.selectable_value(self, Cp0, "Coproc. 0");
			});
			
			match self {
				Cpu =>
				egui::Grid::new("RegistersCpu")
					.striped(true)
					.show(ui, |ui| {
						for reg in 0..32 {
							let reg_e = Register::from(reg);
							let reg_val = cpu[reg_e];
							
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
						
						const fn bits_range(w: u32, b_start: usize, b_end: usize) -> u32 {
							let mask = (1 << (b_end - b_start + 1)) - 1;
							(w >> b_start) & mask
						}
						
						
						struct UiRegister(Cp0Register, &'static str, Display);
						struct UiBitRange(&'static str, usize, usize, &'static str, Display);
						
						
						#[derive(Clone, Copy)]
						enum Display {
							Address,
							Binary,
							Func(&'static dyn Fn(u32) -> &'static str),
							BitRanges(&'static [UiBitRange]),
						}
						
						fn n_to_cause(x: u32) -> &'static str {
							ExceptionCause::try_from(x as usize).map(ExceptionCause::friendly_name).unwrap_or("Unknown")
						}
						
						let regs: &[UiRegister] = &[
							UiRegister(BadVAddr, "Short for \"Bad Virtual Address\".\nHolds the address that failed to be fetched, if any.", Display::Address),
							UiRegister(Status,   "Hell", Display::BitRanges(&[
								UiBitRange("Interrupt Mask", 8, 15, "often abbreviated IMx where x is blah", Display::Binary),
							])),
							UiRegister(Cause,    "tooltip", Display::BitRanges(&[
								UiBitRange("Exception Code", 2, 6, "Bits 2 through 6 of the Cause register.\nIndicates what caused the exception.", Display::Func(&n_to_cause)),
								UiBitRange("Interrupt Pending", 8, 15, "Bits 8 through 15 of the Cause register.\nExceptions at levels 0 and 1 are software-generated.", Display::Binary)
							])),
							UiRegister(EPC,      "Short for \"Error Program Counter\".\nHolds the address of the most recent instruction\nthat caused an exception.", Display::Address),
						];
						
						fn display_it(ui: &mut egui::Ui, reg_val: u32, display: Display) {
							match display {
								Display::Address => { ui.monospace(format!("{reg_val:#010X}")); },
								Display::Binary => { ui.monospace(format!("{reg_val:#034b}")); },
								Display::Func(f) => {
									let reg_str = f(reg_val);
									ui.label(format!("{reg_str} ({reg_val:#X})"));
								},
								Display::BitRanges(ranges) => {
									for &UiBitRange(name, b_start, b_end, tooltip, display) in ranges {
										ui.label(format!("{name} ({b_end}..{b_start})")).on_hover_text(tooltip);
										let bit_val = bits_range(reg_val, b_start, b_end);
										let bit_len = b_end - b_start + 1;
										ui.set_min_width(0.0);
										match display {
											Display::Address => unimplemented!(),
											Display::Binary => {
												ui.monospace(format!("{bit_val:#0$X}", bit_len));
											},
											Display::Func(f) => {
												let bit_str = f(bit_val);
												ui.label(format!("{bit_str} ({bit_val:#X})"));
											},
											Display::BitRanges(_) => unimplemented!(),
										}
									}
								},
							}
						}
						
						for &UiRegister(reg_e, tooltip, display) in regs {
							let reg = reg_e as usize;
							let reg_val = cpu.cp0[reg_e];
							
							ui.label(format!("{reg_e:?} ({reg:02})")).on_hover_text(tooltip);
							
							ui.horizontal_top(|ui| { display_it(ui, reg_val, display); });
							
							ui.end_row();
						}
					})
			}
		});
	}
}
