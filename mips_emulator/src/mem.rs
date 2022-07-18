use super::{word, WORD_BYTES};

const MEMORY_SIZE: usize = 0x10_0000;
const PAGE_SIZE: usize = 0x0_8000;
const PAGE_NUM: usize = MEMORY_SIZE / PAGE_SIZE;

const OFFSET_MASK: usize = PAGE_SIZE - 1;

const PAGE_SHIFT: usize = 15; // TODO: i don't know math.

#[derive(Default)]
pub struct Memory(pub [Option<Box<[u8; PAGE_SIZE]>>; PAGE_NUM]);
impl Memory {
	pub const fn is_aligned(addr: word) -> bool { (addr % 4) == 0 }
	
	pub const fn addr_to_indices(addr: word) -> (usize, usize) {
		(addr as usize >> PAGE_SHIFT, addr as usize & OFFSET_MASK)
	}
	
	pub fn make_page() -> Box<[u8; PAGE_SIZE]> {
		vec![0u8; PAGE_SIZE].into_boxed_slice().try_into().unwrap()
	}
	
	pub fn clear(&mut self) {
		for p in self.0.iter_mut() { p.take(); }
	}
	
	pub fn get_byte(&self, addr: word) -> Option<u8> {
		let (page, offset) = Memory::addr_to_indices(addr);
		self.0.get(page)?.as_ref().map(|b| *b.get(offset).unwrap()).or(Some(0))
	}
	
	pub fn get_byte_mut(&mut self, addr: word) -> Option<&mut u8> {
		let (page, offset) = Memory::addr_to_indices(addr);
		self.0.get_mut(page)?.get_or_insert_with(Self::make_page).get_mut(offset)
	}
	
	pub fn set_byte(&mut self, addr: word, val: u8) -> Option<()> {
		self.get_byte_mut(addr).map(|b| { *b = val; })
	}
	
	pub fn get_word(&self, addr: word) -> Option<word> {
		// If not word-aligned, we could possibly have a word across pages
		// That'd require more logic, so it's gonna be `fn get_word_misalign`??
		if !Memory::is_aligned(addr) { return None; }
		
		let (page, offset) = Memory::addr_to_indices(addr);
		
		let page = self.0.get(page)?.as_ref();
		
		if let Some(page) = page {
			let w = offset..(offset + WORD_BYTES as usize);
			Some(word::from_le_bytes(page.get(w)?.try_into().unwrap()))
		} else {
			Some(0)
		}
	}
	
	// no get_byte_mut because i'm scared of endianness
	
	pub fn set_word(&mut self, addr: word, val: word) -> Option<()> {
		self.write_slice(addr, &val.to_le_bytes())
	}
	
	// TODO: allow writes across pages..
	pub fn write_slice(&mut self, addr: word, data: &[u8]) -> Option<()> {
		let (page, offset) = Memory::addr_to_indices(addr);
		
		let end_addr = addr + (data.len() - 1) as word;
		let (end_page, end_offset) = Memory::addr_to_indices(end_addr);
		
		if page != end_page { return None; }
		
		self.0.get_mut(page)?.get_or_insert_with(Self::make_page)[offset as usize..=end_offset as usize].copy_from_slice(data);
		Some(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn pages_empty_zeroes() {
		let m = Memory::default();
		
		// test if empty page returns 0s
		assert_eq!(m.get_byte(0).expect("byte not some"), 0, "byte not 0");
		assert_eq!(m.get_word(0).expect("word not some"), 0, "word not 0");
		
		// test if beyond limit returns None
		assert!(m.get_byte(MEMORY_SIZE as u32).is_none(), "byte not none");
		assert!(m.get_word(MEMORY_SIZE as u32).is_none(), "word not none");
	}
	
	#[test]
	fn pages_initialize() {
		let mut m = Memory::default();
		*m.get_byte_mut(0).expect("page should be constructed") = 5;
		
		m.write_slice(1, &[1, 2, 3, 4]).expect("page good");
		assert_eq!(m.get_byte(4).unwrap(), 4);
	}
}

/*
mod old_memory {
	use super::*;
	
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
			let source = self.0.get(addr as usize..(addr + WORD_BYTES) as usize);
			source.map(|x| word::from_le_bytes(x.try_into().unwrap()))
		}
		
		/// Returns `Some` only if the operation succeeded.
		pub fn set_byte(&mut self, addr: word, val: u8) -> Option<()> {
			self.0.get_mut(addr as usize).map(|b| { *b = val; })
		}
		
		/// Returns `Some` only if the operation succeeded.
		pub fn set_word(&mut self, addr: word, val: word) -> Option<()> {
			let dest = self.0.get_mut(addr as usize..(addr + WORD_BYTES) as usize);
			dest.map(|x| x.copy_from_slice(&val.to_le_bytes()))
		}
	}
	impl Default for Memory {
		fn default() -> Self {
			Memory(vec![0u8; 0x10_0000].into_boxed_slice().try_into().unwrap())
		}
	}
}
*/
