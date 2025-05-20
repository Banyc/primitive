//! A buffer for reading data from the network.

use std::io::{Read, Result as IoResult};

type BoxedChunk<const STORAGE_SIZE: usize> = Box<[u8; STORAGE_SIZE]>;

/// A FIFO buffer for reading packets from the network.
#[derive(Debug)]
pub struct ReadBuffer<const STORAGE_SIZE: usize> {
    storage: BoxedChunk<STORAGE_SIZE>,
    hi_pos: usize,
    lo_pos: usize,
}
impl<const STORAGE_SIZE: usize> Default for ReadBuffer<STORAGE_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}
impl<const STORAGE_SIZE: usize> ReadBuffer<STORAGE_SIZE> {
    pub fn new() -> Self {
        Self {
            storage: Box::new([0; STORAGE_SIZE]),
            hi_pos: 0,
            lo_pos: 0,
        }
    }
    pub fn fill_with<S: Read>(&mut self, stream: &mut S) -> IoResult<usize> {
        self.clean_up();
        let size = stream.read(&mut self.storage[self.hi_pos..])?;
        self.hi_pos += size;
        Ok(size)
    }
    fn clean_up(&mut self) {
        self.storage.copy_within(self.lo_pos..self.hi_pos, 0);
        let new_hi_pos = self.hi_pos - self.lo_pos;
        self.lo_pos = 0;
        self.hi_pos = new_hi_pos;
    }
    pub fn remaining(&self) -> &[u8] {
        &self.storage[self.lo_pos..self.hi_pos]
    }
    pub fn advance(&mut self, cnt: usize) {
        self.lo_pos += cnt;
        assert!(self.lo_pos <= self.hi_pos);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn simple_reading() {
        let mut input = Cursor::new(b"Hello World!".to_vec());
        let mut buffer = ReadBuffer::<4096>::new();
        let size = buffer.fill_with(&mut input).unwrap();
        assert_eq!(size, 12);
        assert_eq!(buffer.remaining(), b"Hello World!");
    }

    #[test]
    fn partial_reading() {
        let mut inp = Cursor::new(b"Hello World!".to_vec());
        let mut buf: ReadBuffer<4> = ReadBuffer::new();

        let size = buf.fill_with(&mut inp).unwrap();
        assert_eq!(size, 4);
        assert_eq!(buf.remaining(), b"Hell");

        buf.advance(2);
        assert_eq!(buf.remaining(), b"ll");
        assert_eq!(&buf.storage[..buf.hi_pos], b"Hell");

        buf.advance(2);
        let size = buf.fill_with(&mut inp).unwrap();
        assert_eq!(size, 4);
        assert_eq!(buf.remaining(), b"o Wo");
        assert_eq!(&buf.storage[..buf.hi_pos], b"o Wo");

        buf.advance(4);
        let size = buf.fill_with(&mut inp).unwrap();
        assert_eq!(size, 4);
        assert_eq!(buf.remaining(), b"rld!");
    }
}
