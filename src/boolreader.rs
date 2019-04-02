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
    #[test]
    fn bool_reader() {
        let b = vec![0xffu8, 0x80u8];
        let expected = vec![true, false, false, false, false, false, false, false];
        let r = BoolReader::new(&*b);
        let mut len = 0;
        for i in r {
            // println!("{}, {}", i, expected[len]);
            assert_eq!(i, expected[len]);
            len+=1;
        }
        assert_eq!(len, 8);
    }
}