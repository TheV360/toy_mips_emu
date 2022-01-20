#[allow(non_camel_case_types)]
type word = u32;
const WORD_BYTES: usize = word::BITS as usize / 8;

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

pub struct Cpu {
	pub mem: Box<[u8; 0xFF_FFFF]>,
	pub reg: [word; 32],
	pub pc: word,
	pub halt: bool,
}
impl Default for Cpu {
	fn default() -> Self {
		let mem = vec![0u8; 0xFF_FFFF].into_boxed_slice();
		let mem = mem.try_into().expect("This should never fail.");
		Cpu {
			mem,
			reg: Default::default(),
			pc: Default::default(),
			halt: false,
		}
	}
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
// also it's a von neumann machine.. instructions Will be
// with everything else.
// and you can't increment the pc after exiting do_instruction,
// since jump instructions'll thing.

impl Cpu {
	const REGISTER_SIZE: word = 0x20 - 1;
	
	pub fn get_byte(&self, addr: word) -> u8 {
		self.mem[addr as usize]
	}
	
	pub fn get_word(&self, addr: word) -> word {
		assert_eq!(addr % 4, 0, "Tried to get word not on boundary! Better error message soon? lol");
		let addr = addr as usize;
		let w = &self.mem[addr..][..WORD_BYTES];
		word::from_le_bytes(w.try_into().unwrap())
	}
	
	pub fn set_byte(&mut self, addr: word, val: u8) {
		self.mem[addr as usize] = val;
	}
	
	pub fn set_word(&mut self, addr: word, val: word) {
		assert_eq!(addr % 4, 0, "Tried to get word not on boundary! Better error message soon? lol");
		let addr = addr as usize;
		self.mem[addr..][..WORD_BYTES].copy_from_slice(&val.to_le_bytes());
	}
	
	pub fn tick(&mut self) {
		self.do_instruction(self.get_word(self.pc));
	}
	
	pub fn do_instruction(&mut self, ins: word) {
		use Register::*;
		
		if self.halt { return; }
		
		let opcode = (ins >> 26) & 0x3F;
		let rs = ((ins >> 21) & Self::REGISTER_SIZE) as usize;
		let rt = ((ins >> 16) & Self::REGISTER_SIZE) as usize;
		let rd = ((ins >> 11) & Self::REGISTER_SIZE) as usize;
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
				/*sll  */ 0x00 => self.reg[rd] = self.reg[rt] << shamt,
				/*srl  */ 0x01 => self.reg[rd] = self.reg[rt] >> shamt,
				/*jr   */ 0x08 => self.pc = self.reg[rs] - WORD_BYTES as word,
				/*well.*/ 0x0c => self.do_syscall(),
				/*add  */ 0x20 => self.reg[rd] = (self.reg[rs] as i32 + self.reg[rt] as i32) as u32,
				/*addu */ 0x21 => self.reg[rd] = self.reg[rs] + self.reg[rt],
				/*and  */ 0x24 => self.reg[rd] = self.reg[rs] & self.reg[rt],
				/*or   */ 0x25 => self.reg[rd] = self.reg[rs] | self.reg[rt],
				/*slt  */ 0x2a => self.reg[rd] = ((self.reg[rs] as i32) < (self.reg[rt] as i32)) as u32,
				/*sltu */ 0x2b => self.reg[rd] = (self.reg[rs] < self.reg[rt]) as u32,
				_ => panic!("no impl for {opcode:02x} fn {function:02x}"),
			},
			/*j    */ 0x02 => self.pc = j_addr - WORD_BYTES as word, // TODO: well it's sucks
			/*jal  */ 0x03 => { self[ra] = self.pc; self.pc = j_addr - WORD_BYTES as word; },
			/*beq  */ 0x04 => if self.reg[rs] == self.reg[rt] { self.pc = self.pc.wrapping_add(b_addr); },
			/*bne  */ 0x05 => if self.reg[rs] != self.reg[rt] { self.pc = self.pc.wrapping_add(b_addr); },
			/*addi */ 0x08 => self.reg[rt] = (self.reg[rs] as i32 + imm as i32) as u32,
			/*addiu*/ 0x09 => self.reg[rt] = self.reg[rs].wrapping_add(se_imm),
			/*slti */ 0x0a => self.reg[rt] = ((self.reg[rs] as i32) < (se_imm as i32)) as u32,
			/*sltiu*/ 0x0b => self.reg[rt] = (self.reg[rs] < se_imm) as u32,
			/*ori  */ 0x0d => self.reg[rt] = self.reg[rs] | imm,
			/*lui  */ 0x0f => self.reg[rt] = imm << 16,
			/*lw   */ 0x23 => self.reg[rt] = self.get_word(self.reg[rs] + se_imm),
			/*lbu  */ 0x24 => self.reg[rt] = self.get_byte(self.reg[rs] + se_imm) as word,
			/*lhu  */ 0x25 => self.reg[rt] = self.get_word(self.reg[rs] + se_imm) & 0xFFFF, // TODO: wasteful
			/*sb   */ 0x28 => self.set_byte(self.reg[rs] + se_imm, (self.reg[rt] & 0xFF) as u8),
			/*sh   */ 0x29 => self.set_word(self.reg[rs] + se_imm, self.reg[rt] & 0xFFFF), // TODO: also wasteful
			/*sw   */ 0x2b => self.set_word(self.reg[rs] + se_imm, self.reg[rt]),
			_ => panic!("no impl for {opcode:02x}"),
		}
		
		self.pc += WORD_BYTES as word;
	}
	
	pub fn get_instruction_str(&self, ins: word) -> &'static str {
		let opcode = (ins >> 26) & 0x3F;
		let function = ins & 0x3F;
		
		match opcode {
			0x00 => match function {
				0x00 => "sll",
				0x01 => "srl",
				0x0c => "syscall",
				0x20 => "add",
				0x21 => "addu",
				0x24 => "and",
				0x25 => "or",
				0x2a => "slt",
				0x2b => "sltu",
				_ => "IDK",
			},
			0x02 => "j",
			0x03 => "jal",
			0x04 => "beq",
			0x05 => "bne",
			0x08 => "addi",
			0x09 => "addiu",
			0x0a => "slti",
			0x0b => "sltiu",
			0x0d => "ori",
			0x0f => "lui",
			0x23 => "lw",
			0x24 => "lbu",
			0x25 => "lhu",
			0x28 => "sb",
			0x29 => "sh",
			0x2b => "sw",
			_ => "IDK", 
		}
	}
	
	pub fn do_syscall(&mut self) {
		use Register::*;
		
		let service = self.reg[v0 as usize];
		let arg0 = self[a0];
		
		match service {
			1 => print!("{arg0}"),
			4 => {
				let mut str_ptr = arg0 as usize;
				while self.mem[str_ptr] != 0 {
					str_ptr += 1;
				}
				let s = &self.mem[arg0 as usize..str_ptr];
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
		
		cpu[t1] = 32; cpu[t2] = 3;
		cpu.do_instruction(op_r(0x20, t0, t1, t2, 0));
		assert_eq!(cpu[t0], 32 + 3);
		
		cpu[t4] = 10;
		cpu.do_instruction(op_r(0x00, t3, zero, t4, 2));
		assert_eq!(cpu[t3], 10 << 2);
	}
	
	#[test]
	fn sign_ext() {
		use Register::*;
		
		let mut cpu = Cpu::default();
		
		//addiu $sp, $sp, -4 pls
		
		// 0x09 $t1, $t1, -16 maybe.
		
		cpu[t1] = 32;
		cpu.do_instruction(op(0x09, op_i(t1, t1, -16)));
		
		println!("{:08x}", cpu[t1]);
		assert_eq!(cpu[t1], 16);
	}
}
