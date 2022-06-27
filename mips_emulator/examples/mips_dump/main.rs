use std::{env, fs};

use mips_emulator::chip::Cpu;

fn main() -> std::io::Result<()> {
	let args: Vec<String> = env::args().skip(1).collect();
	
	if args.is_empty() {
		println!("please have first argument be a file name, . thanks.");
		return Ok(());
	}
	
	let filename = args.first().unwrap();
	let fb = fs::read(filename)?;
	
	for (i, w) in fb.chunks_exact(4).enumerate() {
		let addr = i << 2;
		let word = u32::from_le_bytes(w.try_into().unwrap());
		let dis = Cpu::get_disassembly(word).unwrap_or_else(||"???".to_owned());
		println!("0x{addr:04x}: {word:08x} {dis:32}");
	}
	
	Ok(())
}
