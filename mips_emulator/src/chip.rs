use super::{word, WORD_BYTES, mem::Memory};

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[repr(usize)]
#[derive(Clone, Copy, Debug)]
pub enum Register {
	/// Zero (constant)
	zero = 0,
	
	/// Assembler Temporary
	at = 1,
	
	/// Results / Eval
	v0 = 2, v1 = 3,
	
	/// Arguments
	a0 = 4, a1 = 5, a2 = 6, a3 = 7,
	
	/// Temporaries
	t0 =  8, t1 =  9, t2 = 10, t3 = 11,
	t4 = 12, t5 = 13, t6 = 14, t7 = 15,
	
	/// Saved Temporaries
	s0 = 16, s1 = 17, s2 = 18, s3 = 19,
	s4 = 20, s5 = 21, s6 = 22, s7 = 23,
	
	/// Temporaries (More)
	t8 = 24, t9 = 25,
	
	/// Kernel
	k0 = 26, k1 = 27,
	
	/// Global Pointer
	gp = 28,
	
	/// Stack Pointer
	sp = 29,
	
	/// Frame Pointer
	fp = 30,
	
	/// Return Address
	ra = 31,
}
impl From<usize> for Register {
	fn from(r: usize) -> Self {
		match r {
			// SAFE: it's within the correct boudns lol
			0..=31 => unsafe { std::mem::transmute(r) },
			
			_ => panic!("Invalid Register"),
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub enum InsFormat {
	/// "Result" format
	/// -- Saves the result of an operation on two registers
	/// into the `rd` register.
	R,
	
	/// "Immediate" format
	/// -- Saves the result of an operation between a register
	/// and an immediate value into the `rt`/`rs` register.
	/// (`rs` only if using store operations.)
	I,
	
	/// "Jump" format
	/// -- Jumps to the specified address.
	J,
	
	/// "`syscall`" format
	/// -- `syscall`.
	Sys,
}

#[derive(Default)]
pub struct Cpu {
	pub reg: [word; 32],
	pub pc: word,
	pub halt: bool,
}

impl core::ops::Index<Register> for Cpu {
	type Output = word;
	fn index(&self, index: Register) -> &Self::Output {
		&self.reg[index as usize]
	}
}
impl core::ops::IndexMut<Register> for Cpu {
	fn index_mut(&mut self, index: Register) -> &mut Self::Output {
		&mut self.reg[index as usize]
	}
}

impl Cpu {
	/// Register size as in size of a register in the condensed bytecode repr.
	/// And honestly it's more for masking out unrelated bytes, which is why
	/// it's subtracted by 1.
	const REGISTER_SIZE: word = 0x20 - 1;
	
	pub fn tick(&mut self, mem: &mut Memory) {
		self.do_instruction(mem.get_word(self.pc).unwrap(), mem);
		self.pc += WORD_BYTES as word;
	}
	
	pub fn do_instruction(&mut self, ins: word, mem: &mut Memory) {
		use Register::*;
		
		if self.halt { return; }
		
		let opcode = (ins >> 26) & 0x3F;
		let rs = Register::from(((ins >> 21) & Self::REGISTER_SIZE) as usize);
		let rt = Register::from(((ins >> 16) & Self::REGISTER_SIZE) as usize);
		let rd = Register::from(((ins >> 11) & Self::REGISTER_SIZE) as usize);
		
		// I format only
		let imm = ins & 0xFFFF; // also "zero extension"
		let se_imm = ((ins << 16) as i32 >> 16) as u32; // sign extension
		let b_addr = ((imm << 18) as i32 >> 16) as u32; // sign-extended address
		
		// R format only
		let function = ins & 0x3F;
		let shamt = (ins >> 6) & 0x1F;
		
		// J format only
		let j_addr = (ins & 0x03FF_FFFF) << 2;
		
		match opcode {
			0x00 => match function {
				/*sll  */ 0x00 => self[rd] = self[rt] << shamt,
				/*srl  */ 0x02 => self[rd] = self[rt] >> shamt,
				/*jr   */ 0x08 => self.pc = self[rs] - WORD_BYTES as word,
				/*jalr */ 0x09 => { self[ra] = self.pc; self.pc = self[rs] - WORD_BYTES as word; },
				/*well.*/ 0x0c => self.do_syscall(mem),
				/*add  */ 0x20 => self[rd] = (self[rs] as i32 + self[rt] as i32) as u32,
				/*addu */ 0x21 => self[rd] = self[rs].wrapping_add(self[rt]),
				/*sub  */ 0x22 => self[rd] = (self[rs] as i32 - self[rt] as i32) as u32,
				/*subu */ 0x23 => self[rd] = self[rs].wrapping_sub(self[rt]),
				/*and  */ 0x24 => self[rd] = self[rs] & self[rt],
				/*or   */ 0x25 => self[rd] = self[rs] | self[rt],
				/*xor  */ 0x26 => self[rd] = self[rs] ^ self[rt],
				/*nor  */ 0x27 => self[rd] = !(self[rs] | self[rt]),
				/*slt  */ 0x2a => self[rd] = ((self[rs] as i32) < (self[rt] as i32)) as u32,
				/*sltu */ 0x2b => self[rd] = (self[rs] < self[rt]) as u32,
				_ => panic!("no impl for {opcode:02x} fn {function:02x}"),
			},
			/*j    */ 0x02 => self.pc = j_addr - WORD_BYTES as word,
			/*jal  */ 0x03 => { self[ra] = self.pc; self.pc = j_addr - WORD_BYTES as word; },
			/*beq  */ 0x04 => if self[rs] == self[rt] { self.pc = self.pc.wrapping_add(b_addr); },
			/*bne  */ 0x05 => if self[rs] != self[rt] { self.pc = self.pc.wrapping_add(b_addr); },
			/*addi */ 0x08 => self[rt] = (self[rs] as i32 + imm as i32) as u32,
			/*addiu*/ 0x09 => self[rt] = self[rs].wrapping_add(se_imm),
			/*slti */ 0x0a => self[rt] = ((self[rs] as i32) < (se_imm as i32)) as u32,
			/*sltiu*/ 0x0b => self[rt] = (self[rs] < se_imm) as u32,
			/*ori  */ 0x0d => self[rt] = self[rs] | imm,
			/*xori */ 0x0e => self[rt] = self[rs] ^ imm,
			/*lui  */ 0x0f => self[rt] = imm << 16,
			/*lw   */ 0x23 => self[rt] = mem.get_word(self[rs] + se_imm).unwrap(),
			/*lbu  */ 0x24 => self[rt] = mem.get_byte(self[rs] + se_imm).unwrap() as word,
			/*lhu  */ 0x25 => self[rt] = mem.get_word(self[rs] + se_imm).unwrap() & 0xFFFF,
			/*sb   */ 0x28 => { mem.set_byte(self[rs] + se_imm, (self[rt] & 0xFF) as u8); },
			/*sh   */ 0x29 => { mem.set_word(self[rs] + se_imm, self[rt] & 0xFFFF); },
			/*sw   */ 0x2b => { mem.set_word(self[rs] + se_imm, self[rt]); },
			_ => panic!("no impl for {opcode:02x}"),
		}
	}
	
	pub fn get_instruction_info(&self, ins: word) -> Option<(&'static str, InsFormat)> {
		use InsFormat::*;
		
		let opcode = (ins >> 26) & 0x3F;
		let function = ins & 0x3F;
		
		match opcode {
			0x00 => match function {
				0x00 => Some(("sll"      , R)),
				0x02 => Some(("srl"      , R)),
				0x08 => Some(("jr"       , J)),
				0x09 => Some(("jalr"     , J)),
				0x0c => Some(("syscall", Sys)),
				0x20 => Some(("add"      , R)),
				0x21 => Some(("addu"     , R)),
				0x22 => Some(("sub"      , R)),
				0x23 => Some(("subu"     , R)),
				0x24 => Some(("and"      , R)),
				0x25 => Some(("or"       , R)),
				0x26 => Some(("xor"      , R)),
				0x27 => Some(("nor"      , R)),
				0x2a => Some(("slt"      , R)),
				0x2b => Some(("sltu"     , R)),
				_ => None,
			},
			0x02 => Some(("j"    , J)),
			0x03 => Some(("jal"  , J)),
			0x04 => Some(("beq"  , I)),
			0x05 => Some(("bne"  , I)),
			0x08 => Some(("addi" , I)),
			0x09 => Some(("addiu", I)),
			0x0a => Some(("slti" , I)),
			0x0b => Some(("sltiu", I)),
			0x0d => Some(("ori"  , I)),
			0x0e => Some(("xori" , I)),
			0x0f => Some(("lui"  , I)),
			0x23 => Some(("lw"   , I)),
			0x24 => Some(("lbu"  , I)),
			0x25 => Some(("lhu"  , I)),
			0x28 => Some(("sb"   , I)),
			0x29 => Some(("sh"   , I)),
			0x2b => Some(("sw"   , I)),
			_ => None,
		}
	}
	
	pub fn get_disassembly(&self, ins: word) -> Option<String> {
		if let Some((ins_name, ins_fmt)) = self.get_instruction_info(ins) {
			use InsFormat::*;
			
			let rs = Register::from(((ins >> 21) & Cpu::REGISTER_SIZE) as usize);
			let rt = Register::from(((ins >> 16) & Cpu::REGISTER_SIZE) as usize);
			let rd = Register::from(((ins >> 11) & Cpu::REGISTER_SIZE) as usize);
			
			match ins_fmt {
				R => {
					let shamt = (ins >> 6) & 0x1F;
					Some(format!("{ins_name} {rd:?}, {rs:?}, {rt:?}; {shamt}"))
				},
				I => {
					let imm = ins & 0xFFFF;
					Some(format!("{ins_name} {rt:?}, {rs:?}, 0x{imm:X}"))
				},
				J => {
					let j_addr = (ins & 0x03FF_FFFF) << 2;
					Some(format!("{ins_name} 0x{j_addr:08X}"))
				},
				Sys => Some("syscall".to_owned()),
			}
		} else {
			None
		}
	}
	
	// TODO: this should maybe be implemented outside the chip,
	//       as in real MIPS processors this raises an exception
	//       that the environment must then acknowledge. seems
	//       much better than what i have going on here.
	pub fn do_syscall(&mut self, mem: &mut Memory) {
		use Register::*;
		
		let service = self[v0];
		let arg0 = self[a0];
		
		match service {
			1 => print!("{arg0}"),
			4 => {
				let str_start = arg0 as usize;
				let str_len =
					mem.0[str_start..]
					.iter().position(|&x| x == 0)
					.unwrap_or(0);
				let s = &mem.0[str_start..][..str_len];
				let s = std::str::from_utf8(s).expect("Dang it");
				print!("{s}");
			},
			17 => {
				println!("quit with exit code {arg0:X}");
				self.halt = true;
			},
			32 => println!("imagine i slept for {arg0} milliseconds."),
			_ => panic!("no impl for {service}"),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	fn op(o: u8, x: word) -> word { ((o as word) << 26) | x }
	
	fn op_r(f: u8, rd: Register, rs: Register, rt: Register, shamt: u8) -> word {
		((rs as word) << 21) | ((rt as word) << 16) | ((rd as word) << 11) | ((shamt as word) << 6) | f as word
	}
	
	fn op_i(rs: Register, rt: Register, imm: i16) -> word {
		((rs as word) << 21) | ((rt as word) << 16) | ((imm as word) & 0xFFFF)
	}
	
	#[test]
	fn basic_computation() {
		use Register::*;
		
		let mut cpu = Cpu::default();
		let mut mem = Memory::default();
		
		cpu[t1] = 32; cpu[t2] = 3;
		cpu.do_instruction(op_r(0x20, t0, t1, t2, 0), &mut mem);
		assert_eq!(cpu[t0], 32 + 3);
		
		cpu[t4] = 10;
		cpu.do_instruction(op_r(0x00, t3, zero, t4, 2), &mut mem);
		assert_eq!(cpu[t3], 10 << 2);
	}
	
	#[test]
	fn sign_ext() {
		use Register::*;
		
		let mut cpu = Cpu::default();
		let mut mem = Memory::default();
		
		//addiu $sp, $sp, -4 pls
		
		// 0x09 $t1, $t1, -16 maybe.
		
		cpu[t1] = 32;
		cpu.do_instruction(op(0x09, op_i(t1, t1, -16)), &mut mem);
		
		println!("{:08x}", cpu[t1]);
		assert_eq!(cpu[t1], 16);
	}
}
