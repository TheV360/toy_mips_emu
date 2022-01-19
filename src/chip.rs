#[allow(non_camel_case_types)]
type word = u32;
const WORD_BYTES: usize = word::BITS as usize / 8;

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

struct Cpu {
	pub mem: Box<[u8; 0xFF_FFFF]>,
	pub reg: [word; 32],
	pub pc: word,
}
impl Default for Cpu {
	fn default() -> Self {
		let mem = vec![0u8; 0xFF_FFFF].into_boxed_slice();
		let mem = mem.try_into().expect("This should never fail.");
		Cpu {
			mem,
			reg: Default::default(),
			pc: Default::default(),
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
		let opcode = (ins >> 26) & 0x3F;
		let rs = ((ins >> 21) & Self::REGISTER_SIZE) as usize;
		let rt = ((ins >> 16) & Self::REGISTER_SIZE) as usize;
		let rd = ((ins >> 11) & Self::REGISTER_SIZE) as usize;
		let imm = ins & 0xFFFF; // also "zero extension"
		let se_imm = ((ins << 16) as i32 >> 16) as u32; // sign extension
		let b_addr = ((imm << 18) as i32 >> 16) as u32; // sign-extended address
		println!("{se_imm:08x}");
		// FIXME: sign extension does Not work.
		
		// R format only
		let function = ins & 0x3F;
		let shamt = (ins >> 6) & 0x1F;
		
		// J format only
		let j_addr = (ins & 0x03FF_FFFF) << 2;
		
		println!("0x{ins:08x} {:?} = {:?} op {:?}", Register::from(rd), Register::from(rs), Register::from(rt));
		
		match opcode {
			0x00 => {
				if function == 0x0c { // booo
					self.do_syscall();
				} else {
					println!("\t\t\t↳ R format; function: {:02x}", function);
					self.reg[rd] = match function {
						/*sll  */ 0x00 => self.reg[rt] << shamt,
						/*srl  */ 0x01 => self.reg[rt] >> shamt,
						/*add  */ 0x20 => (self.reg[rs] as i32 + self.reg[rt] as i32) as u32,
						/*addu */ 0x21 => self.reg[rs] + self.reg[rt],
						/*and  */ 0x24 => self.reg[rs] & self.reg[rt],
						/*slt  */ 0x2a => ((self.reg[rs] as i32) < (self.reg[rt] as i32)) as u32,
						/*sltu */ 0x2b => (self.reg[rs] < self.reg[rt]) as u32,
						_ => panic!("no impl"),
					};
				}
			},
			/*beq  */ 0x04 => if self.reg[rs] == self.reg[rt] { self.pc = self.pc.wrapping_add(b_addr); },
			/*bne  */ 0x05 => if self.reg[rs] != self.reg[rt] { self.pc = self.pc.wrapping_add(b_addr); },
			/*addi */ 0x08 => self.reg[rt] = (self.reg[rs] as i32 + imm as i32) as u32,
			/*addiu*/ 0x09 => self.reg[rt] = self.reg[rs].wrapping_add(se_imm),
			/*slti */ 0x0a => self.reg[rt] = ((self.reg[rs] as i32) < (se_imm as i32)) as u32,
			/*sltiu*/ 0x0b => self.reg[rt] = (self.reg[rs] < se_imm) as u32,
			/*lui  */ 0x0f => self.reg[rt] = imm << 16,
			/*lw   */ 0x23 => self.reg[rt] = self.get_word(self.reg[rs] + se_imm),
			/*lbu  */ 0x24 => self.reg[rt] = self.get_byte(self.reg[rs] + se_imm) as word,
			/*lhu  */ 0x25 => self.reg[rt] = self.get_word(self.reg[rs] + se_imm) & 0xFFFF, // TODO: wasteful
			/*sb   */ 0x28 => self.set_byte(self.reg[rs] + se_imm, (self.reg[rt] & 0xFF) as u8),
			/*sh   */ 0x29 => self.set_word(self.reg[rs] + se_imm, self.reg[rt] & 0xFFFF), // TODO: also wasteful
			/*sw   */ 0x2b => self.set_word(self.reg[rs] + se_imm, self.reg[rt]),
			_ => panic!("no impl"),
		}
		
		self.pc += WORD_BYTES as word;
	}
	
	pub fn do_syscall(&mut self) {
		use Register::*;
		
		let service = self.reg[v0 as usize];
		
		match service {
			1 => print!("{}", self.reg[a0 as usize]),
			32 => println!("imagine i slept for {} milliseconds.", self.reg[a0 as usize]),
			_ => panic!("no impl"),
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
	
	#[test]
	fn silly_stuff() {
		// .data 0x002000
		// .text 0x000000
		
		let mut cpu = Cpu::default();
		let data = include_bytes!("../bitmap_example.data.bin");
		let text = include_bytes!("../bitmap_example.text.bin");
		
		cpu.mem[0x00_2000..][..data.len()].copy_from_slice(data);
		cpu.mem[0x00_0000..][..text.len()].copy_from_slice(text);
		cpu.pc = 0x00_0000;
		
		for i in 0..128 { print!("#{i:3} | pc 0x{:08x} | ", cpu.pc); cpu.tick(); }
	}
}
