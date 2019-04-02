use crate::util::{Result};
use std::io::{Read};

static MIN_REPEAT_SIZE: u8 = 3;

pub struct IntegerReader<R: Read>{
    r: R,
    signed: bool,
}

impl<R: Read> IntegerReader<R> {
    pub fn new(r: R, signed: bool) -> IntegerReaderV2 {
        IntegerReader{
            r: r,
            signed: signed,
        }
    }

    pub fn next_uint(&mut self) -> Result<u64> {

    }

}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integer_reader() {
        let b = vec![0x61, 0x00, 0x07];
        let r = IntegerReader::new(&*b);
        let mut len = 0;
        for i in r {
            len+=1;
            assert_eq!(i, 7u64);
        }
        assert_eq!(len, 100);
    }
}