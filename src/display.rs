use eframe::egui::{Ui, Vec2, Response, Sense, Rect, Pos2, Color32};

pub fn mmio_display(ui: &mut Ui, data: &[u8], cells: (usize, usize), cell_size: Vec2) -> Response {
	let cells_v = Vec2::new(cells.0 as f32, cells.1 as f32);
	
	let (rect, response) =
		ui.allocate_exact_size(cells_v * cell_size, Sense::hover());
	
	if ui.is_rect_visible(rect) {
		let mut addr = 0;
		
		for y in 0..cells.1 {
			let tly = cell_size.y * y as f32;
			for x in 0..cells.0 {
				let tlx = cell_size.x * x as f32;
				let tl = Pos2::new(tlx, tly);
				let c_rect = Rect::from_min_size(tl, cell_size).translate(rect.left_top().to_vec2());
				
				let c: [u8; 4] = data[addr..][..4].try_into().unwrap();
				let [b, g, r, _] = c;
				let fill_color = Color32::from_rgb(r, g, b);
				
				ui.painter().rect_filled(c_rect, 0.0, fill_color);
				
				addr += 4;
			}
		}
	}
	
	response
}
