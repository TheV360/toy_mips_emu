#[allow(non_camel_case_types)]
type word = u32;

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[repr(usize)]
#[derive(Debug)]
enum Register {
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

#[derive(Default)]
struct Cpu {
	reg: [word; 32],
	pc: usize,
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

// your gonna have to have the RAM with the register state
// because syscalls fuck everything up. sorry about it.

impl Cpu {
	const REGISTER_SIZE: u32 = 0x20 - 1;
	
	pub fn do_instruction(&mut self, ins: word) {
		let opcode = (ins >> 26) & 0x3F;
		let rs = ((ins >> 21) & Self::REGISTER_SIZE) as usize;
		let rt = ((ins >> 16) & Self::REGISTER_SIZE) as usize;
		let rd = ((ins >> 11) & Self::REGISTER_SIZE) as usize;
		let imm = ins & 0xFFFF;
		let se_imm = (ins << 16) >> 16; // sign extension
		let address = ins & 0x03FF_FFFF;
		
		println!("{:?} = {:?} op {:?}", Register::from(rd), Register::from(rs), Register::from(rt));
		
		match opcode {
			0x00 => {
				let function = ins & 0x3F;
				let shamt = (ins >> 6) & 0x1F;
				println!("R format; function: {:02x}", function);
				self.reg[rd] = match function {
					/*sll  */ 0x00 => self.reg[rt] << shamt,
					/*srl  */ 0x01 => self.reg[rt] >> shamt,
					/*add  */ 0x20 => (self.reg[rs] as i32 + self.reg[rt] as i32) as u32,
					/*addu */ 0x21 => self.reg[rs] + self.reg[rt],
					/*and  */ 0x24 => self.reg[rs] & self.reg[rt],
					/*slt  */ 0x2a => ((self.reg[rs] as i32) < (self.reg[rt] as i32)) as u32,
					/*sltu */ 0x2b => (self.reg[rs] < self.reg[rt]) as u32,
					_ => { println!("no impl"); self.reg[rd] },
				};
			},
			0x08 => self.reg[rt] = (self.reg[rs] as i32 + imm as i32) as u32,
			0x09 => self.reg[rt] = self.reg[rs] + imm,
			_ => println!("no impl"),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	fn op(o: word, x: word) -> word { (o << 26) | x }
	
	fn op_r(f: u8, rd: Register, rs: Register, rt: Register, shamt: u8) -> word {
		((rs as word) << 21) | ((rt as word) << 16) | ((rd as word) << 11) | ((shamt as word) << 6) | f as word
	}
	
	#[test]
	fn basic_computation() {
		use Register::*;
		
		let mut cpu = Cpu::default();
		
		cpu[t1] = 32; cpu[t2] = 3;
		cpu.do_instruction(op_r(0x20, t0, t1, t2, 0));
		assert_eq!(cpu[t0], 32 + 3);
		
		cpu[t4] = 10;
		cpu.do_instruction(op_r(0x00, t3, zero, t4, 2));
		assert_eq!(cpu[t3], 10 << 2);
	}
}
