use crate::util::{ReaderError, Result};
use std::io::{Bytes, Read};

static MIN_REPEAT_SIZE: u8 = 3;

pub struct IntegerReaderV2<R: Read> {
    i: Bytes<R>,
    current_byte: Option<u8>,
    buffer: Vec<u64>,
}

impl<R: Read> IntegerReaderV2<R> {
    pub fn new(r: R) -> IntegerReaderV2<R> {
        IntegerReaderV2 {
            i: r.bytes(),
            current_byte: None,
            buffer: Vec::new(),
        }
    }
    pub fn new_signed(r: R) -> SignedIntegerReaderV2<R> {
        return SignedIntegerReaderV2(IntegerReaderV2::new(r));
    }
    pub fn next_uint(&mut self) -> Result<u64> {
        if self.buffer.len() > 0 {
            return Ok(Some(self.buffer.remove(0)));
        }
        return self.read_mode();
    }
    fn read_mode(&mut self) -> Result<u64> {
        match self.read_byte() {
            Some(first_byte) => {
                match (first_byte as u64) >> 6 & 0x03 {
                    // 0 => EncodingType::ShortRepeat.next_int(),
                    1 => self.initialize_direct(),
                    // 2 => EncodingType::PatchedBase,
                    // 3 => EncodingType::Delta,
                    _ => {
                        return Err(ReaderError::EncodingType);
                    },
                }
            },
            None => Ok(None),
        }
    }
    fn read_byte(&mut self) -> Option<u8> {
        match self.i.next() {
            Some(Ok(next_byte)) => {
                self.current_byte = Some(next_byte);
                return self.current_byte
            },
            Some(Err(_e)) => None,
            None => None,
        }
    }
    fn read_short_repeat(&mut self) -> Result<u64> {
        Ok(None)
    }
    fn read_patched_base(&mut self) -> Result<u64> {
        Ok(None)
    }
    fn initialize_direct(&mut self) -> Result<u64> {
        let b = self.current_byte.unwrap();
        let fb = b >> 1 & 0x1fu8;
        let w = decode_bit_width(fb, false);
        match self.i.next() {
            Some(Ok(next_byte)) => {
                let mut l = 0x00u16 | ((b << 7) as u16) << 1;
                l |= next_byte as u16;
                l += 1;
                return self.read_direct_to_buffer(w, l);
            },
            Some(Err(err)) => Err(ReaderError::IO(err)),
            None => Ok(None),
        }
    }
    fn read_direct_to_buffer(&mut self, width:u8, length:u16) -> Result<u64> {
        // Read all the bytes that represent the bit packed ints.
        let mut bytes: Vec<u8> = Vec::new();
        let mut num_bytes = if width as u16 * length / 8 > 0 { width as u16 * length / 8 } else { 1 };
        loop {
            num_bytes -= 1;
            match self.i.next() {
                Some(Ok(next_byte)) => bytes.push(next_byte),
                Some(Err(_)) => return Err(ReaderError::Reader),
                None => return Err(ReaderError::Reader),
            };
            if num_bytes == 0 {
                break
            }
        }
        let min_bytes_per_int = bits_to_min_bytes(width) as usize;
        let trailing_bits = (min_bytes_per_int as u64 * 8) - width as u64; 
        let mut consumed_bits = 0usize;
        // Transform the bit packed ints and add them to the vector.
        for i in 0..length {
            // Determine the starting and ending offsets of the bytes.
            let starting_byte_offset = consumed_bits / 8;
            let ending_byte_offset = starting_byte_offset + min_bytes_per_int;
            // Create a slice of the bytes containing our int.
            let containing_bytes = &bytes[starting_byte_offset..ending_byte_offset];
            // Determine the starting offset of the bit packed int within the first byte.
            let bit_offset_in_starting_byte = consumed_bits % 8;
            // The resulting int that bit wise ops will be applied to.
            let mut int = 0u64;
            // Iterate over the bytes
            for j in 0..containing_bytes.len() {
                let op = 0u64 | containing_bytes[j] as u64;
                int |= op << (8 - (j * 8)) as u64;
            }

            // Increment the consumed bits by the width of the varint. 
            consumed_bits += width as usize;
            self.buffer.push(int>>trailing_bits);
        }
        return self.next_uint();
    }
    fn read_delta(&mut self) -> Result<u64> {
        Ok(None)
    }
}

fn decode_bit_width(b: u8, is_delta: bool) -> u8 {
    match b {
        0 => {
            if is_delta { 0 } else { 1 }
        },
        1 => 2,
        3 => 4,
        7 => 15,
        15 => 16,
        23 => 24,
        27 => 32,
        28 => 40,
        29 => 48,
        30 => 56,
        31 => 64,
        2 => 3,
        24 => 26,
        25 => 28,
        26 => 30,
        _ => b + 1,
    }
}

fn bits_to_min_bytes(num_bits: u8) -> u8 {
    let m = num_bits % 8;
    if m == 0 {
        return num_bits / 8;
    } else {
        return (num_bits / 8) + 1;
    }
}

fn bits_to_starting_offset(num_bits: u8) -> u8 {
    8 - (num_bits % 8)
}

pub struct SignedIntegerReaderV2<R: Read>(IntegerReaderV2<R>);

impl<R: Read> SignedIntegerReaderV2<R> {
    pub fn next_int(&mut self) -> Result<i64> {
        match self.0.next_uint() {
            Ok(Some(signed)) => Ok(Some(signed as i64)),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl<R: Read> Iterator for IntegerReaderV2<R> {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        return self.next_uint().unwrap();
    }
}

impl<R: Read> Iterator for SignedIntegerReaderV2<R> {
    type Item = i64;
    fn next(&mut self) -> Option<i64> {
        return self.next_int().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! test_equal_unsigned {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let input = $input.to_vec();
                let expected = $expected.to_vec();
                let r = IntegerReaderV2::new(&*input);
                let mut l = 0;
                for i in r {
                    assert_eq!(i, expected[l]);
                    l += 1;
                }
                assert_eq!(l, expected.len());
            }
        };
    }
    // test_equal_unsigned!(
    //     test_100_sevens,
    //     [0x8eu8, 0x09u8, 0x2bu8, 0x21u8, 0x07u8, 0xd0u8, 0x1eu8, 0x00u8, 0x14u8, 0x70u8, 0x28u8, 0x32u8, 0x3cu8, 0x46u8, 0x50u8, 0x5au8, 0xfcu8, 0xe8u8],
    //     [
    //         2030u64, 2000u64, 2020u64, 1000000u64, 2040u64, 2050u64, 2060u64, 2070u64, 2080u64, 2090u64,
    //     ]
    // );
    test_equal_unsigned!(
        test_direct,
        [0x5eu8, 0x03u8, 0x5cu8, 0xa1u8, 0xabu8, 0x1eu8, 0xdeu8, 0xadu8, 0xbeu8, 0xefu8],
        [23713u64, 43806u64, 57005u64, 48879u64]
    );
}
