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
	const PRG_DATA: &[u8] = include_bytes!("../bitmap_example.data.bin");
	const PRG_TEXT: &[u8] = include_bytes!("../bitmap_example.text.bin");
	
	fn reset_cpu(cpu: &mut Cpu) {
		cpu.mem.fill(0);
		cpu.mem[0x00_2000..][..PRG_DATA.len()].copy_from_slice(PRG_DATA);
		cpu.mem[0x00_0000..][..PRG_TEXT.len()].copy_from_slice(PRG_TEXT);
		cpu.pc = 0x00_0000;
		cpu.halt = false;
		cpu[Register::gp] = 0x1800;
		cpu[Register::sp] = 0x3FFC;
	}
	reset_cpu(&mut cpu);
	
	let mut ticked = true;
	let mut mouse = MyMouse::default();
	let mut full_steam_ahead = false;
	let mut steam_speed = 1i32;
	let mut frame_clock = 0;
	let mut look_addr = 0x0000usize;
	
	while win.is_open() {
		if let Some((x, y)) = win.get_mouse_pos(MouseMode::Discard) {
			mouse.pos = (x as Coord, y as Coord);
		}
		
		let btn = win.get_mouse_down(MouseButton::Left);
		mouse.btn_press = btn && !mouse.btn_down;
		mouse.btn_down = btn;
		
		let funnny = &format!("0x{:08X} + 4", cpu.pc);
		if imm_button(&mut buffer, &mouse, if cpu.halt { "halt" } else { funnny }, (32, 32), (128, 16)) {
			if cpu.halt {
				reset_cpu(&mut cpu); frame_clock = 0;
				full_steam_ahead = false; ticked = true;
			} else if !full_steam_ahead {
				cpu.tick();
				ticked = true;
			}
		}
		
		if imm_button(&mut buffer, &mouse, "-", (204, 32), (14, 16)) {
			steam_speed = steam_speed.saturating_sub(1);
		}
		if imm_button(&mut buffer, &mouse, "+", (222, 32), (14, 16)) {
			steam_speed += 1;
		}
		
		if imm_button(&mut buffer, &mouse, &format!("{steam_speed}x"), (164, 32), (32, 16)) {
			if cpu.halt {
				reset_cpu(&mut cpu); frame_clock = 0;
				full_steam_ahead = false; ticked = true;
			} else {
				full_steam_ahead = !full_steam_ahead;
			}
		}
		if full_steam_ahead {
			if steam_speed > 0 {
				for _ in 0..steam_speed { cpu.tick(); }
				ticked = true;
			} else {
				let denom = 2 - steam_speed;
				if frame_clock % denom == 0 {
					cpu.tick();
					ticked = true;
				}
				frame_clock += 1;
			}
		}
		
		if imm_button(&mut buffer, &mouse, "^3", (348, 8), (24, 16)) {
			look_addr = look_addr.saturating_sub(0x1000); ticked = true;
		}
		if imm_button(&mut buffer, &mouse, "^^", (348, 26), (24, 16)) {
			look_addr = look_addr.saturating_sub(0x400); ticked = true;
		}
		if imm_button(&mut buffer, &mouse, "^", (348, 44), (24, 16)) {
			look_addr = look_addr.saturating_sub(0x40); ticked = true;
		}
		if imm_button(&mut buffer, &mouse, "v", (348, 62), (24, 16)) {
			look_addr = look_addr.saturating_add(0x40); ticked = true;
		}
		if imm_button(&mut buffer, &mouse, "vv", (348, 80), (24, 16)) {
			look_addr = look_addr.saturating_add(0x400); ticked = true;
		}
		if imm_button(&mut buffer, &mouse, "v3", (348, 98), (24, 16)) { //60
			look_addr = look_addr.saturating_add(0x1000); ticked = true;
		}
		if imm_button(&mut buffer, &mouse, "ds", (348, 116), (24, 16)) { //60
			if look_addr == 0x01_0000 {
				look_addr = 0x00_0000;
			} else if look_addr == 0x00_0000 {
				look_addr = 0x00_2000;
			} else {
				look_addr = 0x01_0000;
			}
			ticked = true;
		}
		
		draw_fill_rect(&mut buffer, (360, 0), (64, 7), 0x201D1A);
		draw_text(&mut buffer, &format!("{look_addr:06X}"), (360, 0), 0xFFCCAA);
		
		if ticked {
			let ins = cpu.get_word(cpu.pc);
			if let Some((ins_n, i_fmt)) = cpu.get_instruction_info(ins) {
				use chip::InsFormat::*;
				use chip::Register::*;
				
				let rs = Register::from(((ins >> 21) & Cpu::REGISTER_SIZE) as usize);
				let rt = Register::from(((ins >> 16) & Cpu::REGISTER_SIZE) as usize);
				let rd = Register::from(((ins >> 11) & Cpu::REGISTER_SIZE) as usize);
				
				let s = &format!("{ins_n} {}", match i_fmt {
					R => {
						let shamt = (ins >> 6) & 0x1F;
						format!("{rd:?}, {rs:?}, {rt:?}; {shamt}")
					},
					I => {
						let imm = ins & 0xFFFF;
						format!("{rt:?}, {rs:?}, 0x{imm:X}")
					},
					J => {
						let j_addr = (ins & 0x03FF_FFFF) << 2;
						format!("0x{j_addr:08X}")
					},
					Sys => format!("(service {})", cpu[v0]),
				});
				
				draw_fill_rect(&mut buffer, (8, 8), (256, 8), 0x201D1A);
				draw_text(&mut buffer, s, (8, 8), 0xFFCCAA);
			}
			
			draw_registers(&mut buffer, &cpu, (8, 80));
			draw_memory(&mut buffer, &cpu, (376, 8), look_addr);
			
			ticked = false;
		}
		
		let hovering_over_display = vec2_within_rect(mouse.pos, (376, 8), (256, 256));
		if hovering_over_display {
			let box_size = (88, 32);
			let box_pos = (mouse.pos.0.min(CORNER.0 - box_size.0 / 2) - box_size.0 / 2, mouse.pos.1 + box_size.1 / 2);
			
			let local_mouse = (mouse.pos.0 - 376, mouse.pos.1 - 8);
			let cell_over = (local_mouse.0 / 16, local_mouse.1 / 16);
			let hover_addr = look_addr + (cell_over.1 as usize * 16 + cell_over.0 as usize) * 4;
			
			let data = &cpu.mem[hover_addr..][..4];
			let data: [u8; 4] = data.try_into().unwrap();
			let [b0, b1, b2, b3] = data;
			
			draw_fill_rect(&mut buffer, box_pos, box_size, 0x201D1A);
			draw_rect(&mut buffer, box_pos, box_size, 0xFFCCAA);
			draw_text(
				&mut buffer,
				&format!("0x{hover_addr:08X}\n0x{b0:02X} 0x{b1:02X}\n0x{b2:02X} 0x{b3:02X}"),
				(box_pos.0 + 4, box_pos.1 + 2),
				0xFFCCAA
			);
			
			ticked = true;
		}
		
		win.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
		
		if hovering_over_display {
			let box_size = (88, 32);
			let box_pos = (mouse.pos.0.min(CORNER.0 - box_size.0 / 2) - box_size.0 / 2, mouse.pos.1 + box_size.1 / 2);
			draw_fill_rect(&mut buffer, box_pos, box_size, 0x201D1A);
		}
	}
}

fn draw_registers(buf: &mut Box<[u32]>, cpu: &Cpu, p: Vec2) {
	let cell_size = (80, 32);
	let mut reg = 0;
	for cy in 0..8 {
		for cx in 0..4 {
			let reg_e = Register::from(reg);
			let reg_val = cpu[reg_e];
			let cell_pos = (p.0 + cx * cell_size.0, p.1 + cy * cell_size.1);
			
			draw_fill_rect(buf, cell_pos, cell_size, reg_val);
			draw_rect(buf, cell_pos, cell_size, 0xFFCCAA);
			
			draw_text(
				buf,
				&format!("${reg_e:?} ({reg:02})\n{reg_val:08X}"),
				(cell_pos.0 + 4, cell_pos.1 + 2), 0xFFCCAA
			);
			reg += 1;
		}
	}
}

fn draw_memory(buf: &mut Box<[u32]>, cpu: &Cpu, p: Vec2, mem_addr: usize) {
	// fits 192 words          0xFF
	let s = (256, 256);
	let cell_size = (16, 16);
	draw_rect(buf, (p.0 - 1, p.1 - 1), (s.0 + 2, s.1 + 2), 0x8F4020);
	
	assert!(cpu.mem.len() > 0x01_FFFF);
	
	let mut addr = mem_addr;
	for y in 0..16 {
		let py = p.1 + y * cell_size.1;
		for x in 0..16 {
			let c = &cpu.mem[addr..][..4].try_into().unwrap();
			let c = u32::from_le_bytes(*c);
			draw_fill_rect(buf, (p.0 + x * cell_size.0, py), cell_size, c);
			if cpu.pc as usize == addr {
				draw_rect(buf, (p.0 + x * cell_size.0, py), cell_size, 0x8F4020);
			}
			addr += 4;
		}
	}
}

fn imm_button(buf: &mut Box<[u32]>, mouse: &MyMouse, text: &str, p: Vec2, s: Vec2) -> bool {
	let tc = 0x201D1A;
	let bc = 0xFFCCAA;
	
	let hover = vec2_within_rect(mouse.pos, p, s);
	
	if hover && !mouse.btn_down {
		draw_fill_rect(buf, (p.0, p.1 + s.1 - 1), (s.0, 1), 0xCC9966);
		draw_fill_rect(buf, p, (s.0, s.1 - 1), bc);
		draw_text(buf, text, (p.0 + 4, p.1 + 2), 0x802010);
	} else {
		draw_fill_rect(buf, p, (s.0, 1), 0x201D1A);
		draw_fill_rect(buf, (p.0, p.1 + 1), (s.0, s.1 - 1), bc);
		draw_text(buf, text, (p.0 + 4, p.1 + 3), tc);
	}
	
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
		if ch == '\n' {
			ofs = (p.0, ofs.1 + 8); continue;
		}
		
		if !vec2_within(ofs) || !vec2_within((ofs.0 + 7, ofs.1 + 7)) {
			continue;
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
