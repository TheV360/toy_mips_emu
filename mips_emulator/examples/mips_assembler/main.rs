use std::{env, fs, io::Write};

use mips_emulator::chip::Cpu;

fn main() -> std::io::Result<()> {
	let args: Vec<String> = env::args().skip(1).collect();
	
	if args.is_empty() {
		println!("this accepts an input file and an optional output file.");
		return Ok(());
	}
	
	let filename = args.first().unwrap();
	let s = fs::read_to_string(filename)?;
	
	let mut out_file = if args.len() > 1 {
		Some(fs::File::create(&args[1])?)
	} else { None };
	
	for (i, l) in s.lines()
		.enumerate()
		.map(|(i, s)| (i, s.trim_start()))
		.filter(|&(_, s)| !(s.is_empty() || s.starts_with(['#', '.'])))
		.map(|(i, s)| (i, s.split('#').next().unwrap().trim_end())) {
		print!("{:4}. ", i + 1);
		
		match Cpu::from_assembly(l) {
			Ok(w) => {
				print!("{w:#010X}");
				if let Some(f) = out_file.as_mut() {
					assert_eq!(f.write(&w.to_le_bytes())?, u32::BITS as usize / 8);
				}
			},
			Err(e) => print!("Error: {e}"),
		}
		println!(" ({l})");
	}
	
	Ok(())
}
