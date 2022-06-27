#[allow(non_camel_case_types)]
type word = u32;
const WORD_BYTES: word = word::BITS / 8;

/// extracts a span of bits from a word. hopefully more readable
/// than "do a random bit shift and then mask it by this constant"
/// even though it'll compile to the same thing.
const fn bits_span(w: word, start_bit: usize, length: usize) -> word {
	let mask = (1 << length) - 1;
	(w >> start_bit) & mask
}

/// "smears" the specified bit. useful for sign extension.
/// i.e. smearing the 2nd bit in 0101 results in 111..1101
const fn smear_bit(w: word, bit: usize) -> word {
	let shift = (word::BITS as usize) - (bit + 1);
	((w << shift) as i32 >> shift) as u32
}

pub mod mem;
pub mod chip;
