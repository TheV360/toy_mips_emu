use super::{word, WORD_BYTES};

pub struct Memory(pub Box<[u8; 0x10_0000]>);
impl Memory {
	/// Returns true if the address is word aligned.
	pub const fn is_aligned(addr: word) -> bool {
		(addr % 4) == 0
	}
	
	pub fn get_byte(&self, addr: word) -> Option<u8> {
		self.0.get(addr as usize).cloned()
	}
	
	/// Note: this fn doesn't check if this is an aligned read.
	pub fn get_word(&self, addr: word) -> Option<word> {
		let addr = addr as usize;
		let source = self.0.get(addr..(addr + WORD_BYTES));
		source.map(|x| word::from_le_bytes(x.try_into().unwrap()))
	}
	
	/// Returns `Some` only if the operation succeeded.
	pub fn set_byte(&mut self, addr: word, val: u8) -> Option<()> {
		self.0.get_mut(addr as usize).map(|b| { *b = val; })
	}
	
	/// Returns `Some` only if the operation succeeded.
	pub fn set_word(&mut self, addr: word, val: word) -> Option<()> {
		let addr = addr as usize;
		let dest = self.0.get_mut(addr..(addr + WORD_BYTES));
		dest.map(|x| x.copy_from_slice(&val.to_le_bytes()))
	}
}
impl Default for Memory {
	fn default() -> Self {
		Memory(vec![0u8; 0x10_0000].into_boxed_slice().try_into().unwrap())
	}
}
