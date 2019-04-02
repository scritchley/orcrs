use std::io::{Read, Bytes};

pub trait ColumnReader<T> {
    fn next(&self) -> Result<T, ReaderError>;
}

pub struct RunLengthByteReader<R: Read> {
    i: Bytes<R>,
}

impl<R: Read> RunLengthByteReader<R> {
    pub fn new(r: R) -> RunLengthByteReader<R> {
        RunLengthByteReader{
            i: r.bytes(),
        }
    }
    pub fn next(&mut self) -> Result<bool, ReaderError> {
       match self.i.next() {
           Some(_) => Ok(true),
           None => Ok(false),
       }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn run_length_byte_reader() {
        let b = vec![0x61u8, 0x00u8];
        let mut r = RunLengthByteReader::new(&*b);
        let n = r.next();
        assert_eq!(n.unwrap(), false);
    }
}