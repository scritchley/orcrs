use crate::bytereader::ByteReader;
use crate::util::{Result};
use std::io::{Read};

pub struct BoolReader<R: Read>{
    b: ByteReader<R>,
    current_byte: Option<u8>,
    remaining: u8,
}

impl<R: Read> BoolReader<R> {
    pub fn new(r: R) -> BoolReader<R> {
        BoolReader{
            b: ByteReader::new(r),
            current_byte: None,
            remaining: 0u8,
        }
    }

    pub fn next_bool(&mut self) -> Result<bool> {
        match self.current_byte {
            Some(_) => self.read_next_bool(),
            None => self.load_next_byte(),
        }
    }

    fn read_next_bool(&mut self) -> Result<bool> {
        self.remaining -= 1;
        let current_byte = self.current_byte.unwrap();
        let next_bool = current_byte & 0x80u8 != 0;
        if self.remaining == 0 {
            self.current_byte = None;
        } else {
            self.current_byte = Some(current_byte << 1);
        }
        Ok(Some(next_bool))
    }

    fn load_next_byte(&mut self) -> Result<bool> {
        match self.b.next_byte() {
            Ok(Some(next_byte)) => {
                self.current_byte = Some(next_byte);
                self.remaining = 8u8;
                return self.read_next_bool();
            },
            Err(e) => Err(e),
            Ok(None) => Ok(None),
        }
    }
}

impl<R: Read> Iterator for BoolReader<R> {
    type Item = bool;
    fn next(&mut self) -> Option<bool> {
        return self.next_bool().unwrap();
    }
}


#[cfg(test)]
mod tests {
    use super::BoolReader;
    macro_rules! test_equal {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let input = $input.to_vec();
                let expected = $expected.to_vec();
                let r = BoolReader::new(&*input);
                let mut l = 0;
                for i in r {
                    assert_eq!(i, expected[l]);
                    l += 1;
                }
                assert_eq!(l, expected.len());
            }
        };
    }
    test_equal!(test_bool_reader, [0xffu8, 0x80u8], [true, false, false, false, false, false, false, false]);
}