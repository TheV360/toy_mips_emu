use super::{word, WORD_BYTES, mem::Memory, bits_span, smear_bit};

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cp0Register {
	/// Memory address where access/write exception occurred (if any)
	BadVAddr = 8,
	
	/// Contains
	/// - Interrupt mask
	/// - Enable bits
	/// - Status when exception occurred
	Status = 12,
	
	/// Contains
	/// - Cause of exception
	/// - Pending interrupt bits
	Cause = 13,
	
	/// Program counter at where exception occurred
	EPC = 14,
}
impl From<usize> for Cp0Register {
	fn from(r: usize) -> Self {
		match r {
			 8 => Cp0Register::BadVAddr,
			12 => Cp0Register::Status,
			13 => Cp0Register::Cause,
			14 => Cp0Register::EPC,
			
			_  => panic!("Invalid Register"),
		}
	}
}

#[repr(u8)]
pub enum ExceptionCause {
	/// Hardware Interrupt
	Int = 0,
	
	AdEL = 4,
	AdES = 5,
	
	Ibe = 6,
	Dbe = 7,
	
	/// Syscall
	Sys = 8,
	
	/// Breakpoint
	Bp  = 9,
	
	/// Reserved Instruction
	Ri  = 10,
	
	/// Unimplemented Coprocessor
	CpU = 11,
	
	/// Arithmetic Overflow
	Ov  = 12,
	
	/// Trap
	Tr  = 13,
	
	/// Floating Point Exception
	Fpe = 15,
}

#[derive(Default)]
pub struct Cpu {
	pub reg: [word; 32],
	pub pc: word,
	
	/// Co-processor 0, which provides exceptions and memory management.
	pub cp0: Cp0,
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

#[derive(Default)]
pub struct Cp0 {
	pub halt: bool,
	pub reg: [word; 16],
}
impl core::ops::Index<Cp0Register> for Cp0 {
	type Output = word;
	fn index(&self, index: Cp0Register) -> &Self::Output {
		&self.reg[index as usize]
	}
}
impl core::ops::IndexMut<Cp0Register> for Cp0 {
	fn index_mut(&mut self, index: Cp0Register) -> &mut Self::Output {
		&mut self.reg[index as usize]
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Opcode {
	/// Typical opcode
	General(word), // -> 0x??
	
	/// Opcode `0x00` contains several different functions.
	Function(word), // -> 0x00 0x??
	
	/// Not sure if this is gonna cover all coprocessor stuff. We'll see.
	/// It's under opcode `0x10`.
	Coprocessor(word), // -> 0x10 0x??
}

impl Cpu {
	/// Register size as in "number of bits a register takes up in the
	/// bytecode representation".
	const REGISTER_SIZE: usize = 5;
	
	/// Table of operations
	const INSTRUCTIONS: &'static [(Opcode, &'static str, InsFormat)] = {
		use Opcode::*; use InsFormat::*;
	&[
		(Function(0x00), "sll"      , R),
		(Function(0x02), "srl"      , R),
		(Function(0x08), "jr"       , J),
		(Function(0x09), "jalr"     , J),
		(Function(0x0c), "syscall", Sys),
		(Function(0x20), "add"      , R),
		(Function(0x21), "addu"     , R),
		(Function(0x22), "sub"      , R),
		(Function(0x23), "subu"     , R),
		(Function(0x24), "and"      , R),
		(Function(0x25), "or"       , R),
		(Function(0x26), "xor"      , R),
		(Function(0x27), "nor"      , R),
		(Function(0x2a), "slt"      , R),
		(Function(0x2b), "sltu"     , R),
		(General(0x02), "j"    , J),
		(General(0x03), "jal"  , J),
		(General(0x04), "beq"  , I),
		(General(0x05), "bne"  , I),
		(General(0x08), "addi" , I),
		(General(0x09), "addiu", I),
		(General(0x0a), "slti" , I),
		(General(0x0b), "sltiu", I),
		(General(0x0d), "ori"  , I),
		(General(0x0e), "xori" , I),
		(General(0x0f), "lui"  , I),
		(Coprocessor(0x00), "mfc0", R),
		(General(0x23), "lw"   , I),
		(General(0x24), "lbu"  , I),
		(General(0x25), "lhu"  , I),
		(General(0x28), "sb"   , I),
		(General(0x29), "sh"   , I),
		(General(0x2b), "sw"   , I),
	]};
	
	pub fn tick(&mut self, mem: &mut Memory) {
		self.do_instruction(mem.get_word(self.pc).unwrap(), mem);
		self.pc += WORD_BYTES as word;
	}
	
	pub fn do_instruction(&mut self, ins: word, mem: &mut Memory) {
		use Register::*;
		
		let opcode = bits_span(ins, 26, 6);
		let rs = Register::from(bits_span(ins, 21, Self::REGISTER_SIZE) as usize);
		let rt = Register::from(bits_span(ins, 16, Self::REGISTER_SIZE) as usize);
		let rd = Register::from(bits_span(ins, 11, Self::REGISTER_SIZE) as usize);
		
		// I format only
		let imm = bits_span(ins, 0, 16);      // immediate value
		let se_imm = smear_bit(imm, 15);      // sign-extended immediate val
		let b_addr = smear_bit(imm, 15) << 2; // sign-extended address
		
		// R format only
		let function = bits_span(ins, 0, 6);
		let shamt = bits_span(ins, 6, 5);
		
		// J format only
		let j_addr = bits_span(ins, 0, 26) << 2;
		
		match opcode {
			0x00 => match function {
				/* sll */ 0x00 => self[rd] = self[rt] << shamt,
				/* srl */ 0x02 => self[rd] = self[rt] >> shamt,
				
				// TODO: handle branch delay slot
				/*jr   */ 0x08 => self.pc = self[rs] - WORD_BYTES as word,
				
				/*jalr */ 0x09 => { self[ra] = self.pc; self.pc = self[rs] - WORD_BYTES as word; },
				/*sys☎*/ 0x0c => self.exception(ExceptionCause::Sys),
				
				// add :: integer overflow exception
				0x20 => if let Some(a) = (self[rs] as i32).checked_add(self[rt] as i32) { self[rd] = a as u32 } else { self.exception(ExceptionCause::Ov) }
				
				// addu :: no overflow exceptions ever
				0x21 => self[rd] = self[rs].wrapping_add(self[rt]),
				
				// sub :: integer overflow exception
				0x22 => if let Some(a) = (self[rs] as i32).checked_sub(self[rt] as i32) { self[rd] = a as u32 } else { self.exception(ExceptionCause::Ov) },
				
				// subu :: no overflow exceptions ever
				0x23 => self[rd] = self[rs].wrapping_sub(self[rt]),
				
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
			/*addi */ 0x08 => if let Some(a) = (self[rs] as i32).checked_add(imm as i32) { self[rt] = a as u32; } else { self.exception(ExceptionCause::Ov) },
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
	
	pub fn get_instruction_info(ins: word) -> Option<(&'static str, InsFormat)> {
		let opcode = bits_span(ins, 26, 6);
		let opcode_ty = match opcode {
			0x00 => Opcode::Function(bits_span(ins, 0, 6)),
			0x10 => Opcode::Coprocessor(bits_span(ins, 21, 5)),
			_    => Opcode::General(opcode),
		};
		
		Self::INSTRUCTIONS
			.iter().cloned()
			.find(|(other, _, _)| opcode_ty == *other)
			.map(|(_, ident, fmt)| (ident, fmt))
	}
	
	pub fn get_disassembly(ins: word) -> Option<String> {
		if let Some((ins_name, ins_fmt)) = Self::get_instruction_info(ins) {
			use InsFormat::*;
			
			let rs = Register::from(bits_span(ins, 21, Self::REGISTER_SIZE) as usize);
			let rt = Register::from(bits_span(ins, 16, Self::REGISTER_SIZE) as usize);
			let rd = Register::from(bits_span(ins, 11, Self::REGISTER_SIZE) as usize);
			
			match ins_fmt {
				R => {
					let shamt = bits_span(ins, 6, 5);
					Some(format!("{ins_name} ${rd:?}, ${rs:?}, ${rt:?}; {shamt}"))
				},
				I => {
					let imm = bits_span(ins, 0, 16);
					Some(format!("{ins_name} ${rt:?}, ${rs:?}, 0x{imm:X}"))
				},
				J => {
					let j_addr = bits_span(ins, 0, 26) << 2;
					Some(format!("{ins_name} 0x{j_addr:08X}"))
				},
				Sys => Some("syscall".to_owned()),
			}
		} else {
			None
		}
	}
	
	fn exception(&mut self, cause: ExceptionCause, ) {
		use Register::*; use Cp0Register::*;
		self.cp0[EPC] = self.pc;
		self.cp0[Cause] |= (cause as u32) << 2;
		// TODO: easy way to determine if cause is from this instruction or if
		//       it's an interrupt that just so happened to stop this instr.
		// (so that UI can easily display '!' or ';' on the EPC)
		// ⚠ ? ⋯ ?
		
		// self.pc = 0x8000_0080; // TODO: ahaha.. error handling. ...
		//                       // bc i do want have cfg'able memory layout
		//                      // maybe just.. suck it up and impl mem paging.
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
