#[allow(non_camel_case_types)]
type word = u32;
const WORD_BYTES: usize = word::BITS as usize / 8;

pub mod mem;
pub mod chip;
