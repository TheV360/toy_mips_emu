#![allow(dead_code)]

use minifb::{Scale, Window, WindowOptions, MouseMode, MouseButton};
use font8x8::{BASIC_FONTS, UnicodeFonts};

// pub mod mem;
pub mod chip;
use chip::{Cpu, Register};

mod bresenham;
use bresenham::{Coord, Vec2, Line};

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

const CORNER: Vec2 = (WIDTH as Coord, HEIGHT as Coord);
const CENTER: Vec2 = (CORNER.0 / 2, CORNER.1 / 2);

#[derive(Default)]
struct MyMouse {
	pos: Vec2,
	btn_press: bool,
	btn_down: bool,
}

fn main() {
	let mut win = Window::new(
		"Cool Swag MIPS Emu", WIDTH, HEIGHT,
		WindowOptions {
			scale: Scale::X2,
			..Default::default()
		}
	).expect("Failed to open window!");
	
	let mut buffer: Box<[u32]> = vec![0x201d1a; WIDTH * HEIGHT].into_boxed_slice();
	
	let mut cpu = Cpu::default();
	let data = include_bytes!("../bitmap_example.data.bin");
	let text = include_bytes!("../bitmap_example.text.bin");
	
	cpu.mem[0x00_2000..][..data.len()].copy_from_slice(data);
	cpu.mem[0x00_0000..][..text.len()].copy_from_slice(text);
	cpu.pc = 0x00_0000;
	cpu[Register::gp] = 0x1800;
	cpu[Register::sp] = 0x3FFC;
	
	let mut ticked = true;
	let mut mouse = MyMouse::default();
	let mut full_steam_ahead = false;
	
	while win.is_open() {
		if let Some((x, y)) = win.get_mouse_pos(MouseMode::Discard) {
			mouse.pos = (x as Coord, y as Coord);
		}
		
		let btn = win.get_mouse_down(MouseButton::Left);
		mouse.btn_press = btn && !mouse.btn_down;
		mouse.btn_down = btn;
		
		if imm_button(&mut buffer, &mouse, &format!("0x{:08X} + 4", cpu.pc), (32, 32), (128, 16), 0x201D1A, 0xFFCCAA)
		&& !full_steam_ahead {
			cpu.tick();
			ticked = true;
		}
		if imm_button(&mut buffer, &mouse, "Go!", (164, 32), (32, 16), 0x201D1A, 0xFFCCAA) {
			full_steam_ahead = !full_steam_ahead;
		}
		if full_steam_ahead {
			for _ in 0..4 { cpu.tick(); }
			ticked = true;
		}
		
		if ticked {
			draw_registers(&mut buffer, &cpu, (8, 80));
			draw_memory(&mut buffer, &cpu, (376, 8));
			ticked = false;
		}
		
		win.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
	}
	
	// for i in 0..1024 { print!("#{i:3} | pc 0x{:08x} | ", cpu.pc); cpu.tick(); }
	
	// TODO: basic debugging display.
	// registers on right, step button, raw memory view on left
}

fn draw_registers(buf: &mut Box<[u32]>, cpu: &Cpu, p: Vec2) {
	let cell_size = (80, 32);
	let mut reg = 0;
	for cy in 0..8 {
		for cx in 0..4 {
			let reg_e = Register::from(reg);
			let cell_pos = (p.0 + cx * cell_size.0, p.1 + cy * cell_size.1);
			
			draw_fill_rect(buf, cell_pos, cell_size, 0x20408F);
			draw_rect(buf, cell_pos, cell_size, 0x4080FF);
			
			draw_text(
				buf,
				&format!("${:?} ({reg:02})\n{:08X}", reg_e, cpu[reg_e]),
				(cell_pos.0 + 4, cell_pos.1 + 2), 0xFFCCAA
			);
			reg += 1;
		}
	}
}

fn draw_memory(buf: &mut Box<[u32]>, cpu: &Cpu, p: Vec2) {
	// fits 192 words          0xFF
	let s = (256, 256);
	let cell_size = (16, 16);
	draw_rect(buf, (p.0 - 1, p.1 - 1), (s.0 + 2, s.1 + 2), 0x8F4020);
	
	assert!(cpu.mem.len() > 0x01_FFFF);
	
	let mut addr = 0x01_0000;
	for y in 0..16 {
		let py = p.1 + y * cell_size.1;
		for x in 0..16 {
			let c = &cpu.mem[addr..][..4].try_into().unwrap();
			let c = u32::from_le_bytes(*c);
			draw_fill_rect(buf, (p.0 + x * cell_size.0, py), cell_size, c);
			addr += 4;
		}
	}
}

fn imm_button(buf: &mut Box<[u32]>, mouse: &MyMouse, text: &str, p: Vec2, s: Vec2, tc: u32, bc: u32) -> bool {
	let hover = vec2_within_rect(mouse.pos, p, s);
	
	draw_fill_rect(buf, p, s, bc);
	draw_text(buf, text, (p.0 + 4, p.1 + 2), if hover { 0x802010 } else { tc });
	
	hover && mouse.btn_press
}

fn vec2_within(a: Vec2) -> bool {
	a.0 >= 0 && a.0 < CORNER.0 &&
	a.1 >= 0 && a.1 < CORNER.1
}
fn vec2_within_bounds(a: Vec2, tl: Vec2, br: Vec2) -> bool {
	a.0 >= tl.0 && a.0 < br.0 &&
	a.1 >= tl.1 && a.1 < br.1
}
fn vec2_within_rect(a: Vec2, tl: Vec2, s: Vec2) -> bool {
	a.0 >= tl.0 && a.0 < (tl.0 + s.0) &&
	a.1 >= tl.1 && a.1 < (tl.1 + s.1)
}

#[inline]
fn vec2_to_index(a: Vec2) -> usize {
	(a.0 + a.1 * CORNER.0) as usize
}

fn draw_rect(buf: &mut Box<[u32]>, tl: Vec2, s: Vec2, c: u32) {
	let br = (tl.0 + s.0 - 1, tl.1 + s.1 - 1);
	
	if !vec2_within(tl) || !vec2_within(br) { return; }
	
	unsafe {
		let mut th = vec2_to_index((tl.0, tl.1));
		let mut bh = vec2_to_index((tl.0, br.1));
		for _x in 0..s.0 {
			*buf.get_unchecked_mut(th) = c; th += 1;
			*buf.get_unchecked_mut(bh) = c; bh += 1;
		}
		
		let mut lh = vec2_to_index((tl.0, tl.1));
		let mut rh = vec2_to_index((br.0, tl.1));
		for _y in 0..s.1 {
			*buf.get_unchecked_mut(lh) = c; lh += WIDTH;
			*buf.get_unchecked_mut(rh) = c; rh += WIDTH;
		}
	}
}

fn draw_fill_rect(buf: &mut Box<[u32]>, tl: Vec2, s: Vec2, c: u32) {
	let br = (tl.0 + s.0 - 1, tl.1 + s.1 - 1);
	if !vec2_within(tl) || !vec2_within(br) { return; }
	
	unsafe {
		for y in 0..s.1 {
			let mut i = vec2_to_index((tl.0, tl.1 + y));
			for _x in 0..s.0 {
				*buf.get_unchecked_mut(i) = c; i += 1;
			}
		}
	}
}

fn draw_line(buf: &mut Box<[u32]>, a: Vec2, b: Vec2, c: u32) {
	if !vec2_within(a) || !vec2_within(b) { return; }
	
	unsafe {
		for p in Line::new(a, b) {
			*buf.get_unchecked_mut(vec2_to_index(p)) = c;
		}
	}
}

fn draw_text(buf: &mut Box<[u32]>, text: &str, p: Vec2, c: u32) {
	let mut ofs = p;
	for ch in text.chars() {
		if !vec2_within(ofs) || !vec2_within((ofs.0 + 7, ofs.1 + 7)) {
			continue;
		}
		
		if ch == '\n' {
			ofs = (p.0, ofs.1 + 8); continue;
		}
		
		if !ch.is_whitespace() {
			if let Some(glyph) = BASIC_FONTS.get(ch) {
				unsafe {
					for (y, &row) in glyph.iter().enumerate() {
						let index = vec2_to_index((ofs.0, ofs.1 + y as i32));
						for bit in 0..8 {
							if (row & (1 << bit)) > 0 {
								*buf.get_unchecked_mut(index + bit) = c;
							}
						}
					}
				}
			}
		}
		
		ofs = (ofs.0 + 8, ofs.1);
	}
}
