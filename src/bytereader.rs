use crate::util::{ReaderError, Result};
use std::io::{Read, Bytes};

static MIN_REPEAT_SIZE: u8 = 3;

pub struct ByteReader<R: Read> {
    i: Bytes<R>,
    current_byte: u8,
    num_repeat: u8,
    num_literal: u8,
}

impl<R: Read> ByteReader<R> {
    pub fn new(r: R) -> ByteReader<R> {
        ByteReader{
            i: r.bytes(),
            current_byte: 0u8,
            num_repeat: 0u8,
            num_literal: 0u8,
        }
    }

    fn read_raw_byte(&mut self) -> Result<u8> {
        match self.i.next() {
            Some(Ok(next_byte)) => Ok(Some(next_byte)),
            Some(Err(e)) => Err(ReaderError::IO(e)),
            None => Ok(None),
        }
    }

    fn load_next_byte(&mut self) -> Result<u8> {
        match self.i.next() {
            Some(Ok(next_byte)) => {
                self.current_byte = next_byte;
                return self.next_byte();  
            },
            Some(Err(e)) => Err(ReaderError::IO(e)),
            None => Ok(None),
        }
    }

    pub fn next_byte(&mut self) -> Result<u8> {
        if self.num_repeat > 0 {
            self.num_repeat -= 1;
            return Ok(Some(self.current_byte))
        }
        if self.num_literal > 0 {
            self.num_literal -= 1;
            return self.read_raw_byte();
        }
        match self.i.next() {
            Some(Ok(control)) => {
                if control < 0x80u8 {
                    self.num_repeat = control + MIN_REPEAT_SIZE;
                    return self.load_next_byte()
                } 
                self.num_literal = control - 0x80u8;
                return self.next_byte();
            },
            Some(Err(e)) => Err(ReaderError::IO(e)),
            None => Ok(None),
        }
    }
}

impl<R: Read> Iterator for ByteReader<R> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        return self.next_byte().unwrap();
    }
}

macro_rules! test_equal {
    ($name:ident, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let input = $input.to_vec();
            let expected = $expected.to_vec();
            let r = ByteReader::new(&*input);
            let mut l = 0;
            for i in r {
                assert_eq!(i, expected[l]);
                l += 1;
            }
            assert_eq!(l, expected.len());
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    test_equal!(test_100_zeros, [0x61u8, 0x00u8], [0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8]);
    test_equal!(test_4_ones, [1u8, 1u8], [1u8,1u8,1u8,1u8]);
    test_equal!(test_literals, [0xFEu8, 0x44u8, 0x45u8], [0x44u8, 0x45u8]);
}
