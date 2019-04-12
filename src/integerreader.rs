use crate::util::{ReaderError, Result};
use std::io::{Bytes, Read};

static MIN_REPEAT_SIZE: u8 = 3;

pub struct IntegerReader<R: Read> {
    i: Bytes<R>,
    num_literals: u8,
    num_repeat: u8,
    delta: i8,
    current: Option<i64>,
}

impl<R: Read> IntegerReader<R> {
    pub fn new(r: R) -> IntegerReader<R> {
        IntegerReader {
            i: r.bytes(),
            num_literals: 0u8,
            num_repeat: 0u8,
            delta: 0i8,
            current: None,
        }
    }
    pub fn new_unsigned(r: R) -> UnsignedIntegerReader<R> {
        return UnsignedIntegerReader(IntegerReader::new(r));
    }
    pub fn next_int(&mut self) -> Result<i64> {
        if self.num_repeat > 0 {
            self.num_repeat -= 1;
            let current = self.current.unwrap();
            self.current = Some(current + self.delta as i64);
            return Ok(Some(current));
        }
        if self.num_literals > 0 {
            self.num_literals -= 1;
            return self.read_var_int(0u64, 0u64);
        }
        match self.i.next() {
            Some(Ok(control)) => {
                if control < 0x80u8 {
                    self.num_repeat = control + MIN_REPEAT_SIZE;
                    return self.read_delta();
                }
                self.num_literals = control - 0x80u8;
                return self.next_int();
            }
            Some(Err(e)) => Err(ReaderError::IO(e)),
            None => Ok(None),
        }
    }
    fn read_delta(&mut self) -> Result<i64> {
        match self.i.next() {
            Some(Ok(delta)) => {
                self.delta = delta as i8;
                self.current = self.read_var_int(0u64, 0u64)?;
                return self.next_int();
            }
            Some(Err(e)) => Err(ReaderError::IO(e)),
            None => Ok(None),
        }
    }
    fn read_var_int(&mut self, mut r: u64, mut s: u64) -> Result<i64> {
        match self.i.next() {
            Some(Ok(b)) => {
                let msb_dropped = b & 0b01111111;
                r |= (msb_dropped as u64) << s;
                s += 7;
                if b & 0b10000000 == 0 || s > (10 * 7) {
                    return Ok(Some(r as i64));
                }
                return self.read_var_int(r, s);
            }
            Some(Err(e)) => Err(ReaderError::IO(e)),
            None => Ok(None),
        }
    }
}

pub struct UnsignedIntegerReader<R: Read>(IntegerReader<R>);

impl<R: Read> UnsignedIntegerReader<R> {
    pub fn next_uint(&mut self) -> Result<u64> {
        match self.0.next_int() {
            Ok(Some(signed)) => Ok(Some(signed as u64)),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl<R: Read> Iterator for IntegerReader<R> {
    type Item = i64;
    fn next(&mut self) -> Option<i64> {
        return self.next_int().unwrap();
    }
}

impl<R: Read> Iterator for UnsignedIntegerReader<R> {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        return self.next_uint().unwrap();
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
                let r = IntegerReader::new_unsigned(&*input);
                let mut l = 0;
                for i in r {
                    assert_eq!(i, expected[l]);
                    l += 1;
                }
                assert_eq!(l, expected.len());
            }
        };
    }
    test_equal_unsigned!(
        test_100_sevens,
        [0x61, 0x00, 0x07],
        [
            7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64,
            7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64,
            7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64,
            7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64,
            7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64,
            7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64,
            7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64, 7u64,
            7u64, 7u64
        ]
    );
    test_equal_unsigned!(
        test_100_to_0,
        [0x61, 0xff, 0x64],
        [
            100u64, 99u64, 98u64, 97u64, 96u64, 95u64, 94u64, 93u64, 92u64, 91u64, 90u64, 89u64,
            88u64, 87u64, 86u64, 85u64, 84u64, 83u64, 82u64, 81u64, 80u64, 79u64, 78u64, 77u64,
            76u64, 75u64, 74u64, 73u64, 72u64, 71u64, 70u64, 69u64, 68u64, 67u64, 66u64, 65u64,
            64u64, 63u64, 62u64, 61u64, 60u64, 59u64, 58u64, 57u64, 56u64, 55u64, 54u64, 53u64,
            52u64, 51u64, 50u64, 49u64, 48u64, 47u64, 46u64, 45u64, 44u64, 43u64, 42u64, 41u64,
            40u64, 39u64, 38u64, 37u64, 36u64, 35u64, 34u64, 33u64, 32u64, 31u64, 30u64, 29u64,
            28u64, 27u64, 26u64, 25u64, 24u64, 23u64, 22u64, 21u64, 20u64, 19u64, 18u64, 17u64,
            16u64, 15u64, 14u64, 13u64, 12u64, 11u64, 10u64, 9u64, 8u64, 7u64, 6u64, 5u64, 4u64,
            3u64, 2u64, 1u64,
        ]
    );
    test_equal_unsigned!(
        test_4_ones,
        [0xfb, 0x02, 0x03, 0x04, 0x07, 0xb],
        [2u64, 3u64, 4u64, 7u64, 11u64]
    );
}
