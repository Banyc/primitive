use crate::ops::{
    clear::Clear,
    len::{Capacity, Len},
};

const BITS_PER_BYTE: usize = 8;
const USIZE_BITS: usize = core::mem::size_of::<usize>() * BITS_PER_BYTE;

#[derive(Debug, Clone)]
pub struct Iter<'a> {
    set: &'a BitSet,
    next: usize,
}
impl Iterator for Iter<'_> {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.set.capacity() == self.next {
            return None;
        }
        let bit = self.set.get(self.next);
        self.next += 1;
        Some(bit)
    }
}
impl BitSet {
    pub fn iter(&self) -> Iter<'_> {
        Iter { set: self, next: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct BitSet {
    words: Vec<usize>,
    ones: usize,
    capacity_bits: usize,
}
impl BitSet {
    #[must_use]
    pub fn new(bits: usize) -> Self {
        let bytes = bits.div_ceil(BITS_PER_BYTE);
        let words = bytes.div_ceil(core::mem::size_of::<usize>());
        Self {
            words: vec![0; words],
            ones: 0,
            capacity_bits: bits,
        }
    }
    #[must_use]
    pub fn get(&self, index: usize) -> bool {
        assert!(index < self.capacity_bits);
        let word = self.words[word_index(index)];
        let pos = 1 << bit_offset(index);
        let is_empty = word & pos == 0;
        !is_empty
    }
    fn bit_op(&mut self, bit_index: usize, op: impl Fn(BitOpArgs) -> usize) {
        assert!(bit_index < self.capacity_bits);
        let word = &mut self.words[word_index(bit_index)];
        let prev = word.count_ones();
        let pos = 1 << bit_offset(bit_index);
        let args = BitOpArgs { word: *word, pos };
        *word = op(args);
        let curr = word.count_ones();
        match prev.cmp(&curr) {
            core::cmp::Ordering::Less => self.ones += usize::try_from(curr - prev).unwrap(),
            core::cmp::Ordering::Equal => (),
            core::cmp::Ordering::Greater => self.ones -= usize::try_from(prev - curr).unwrap(),
        }
    }
    pub fn set(&mut self, index: usize) {
        self.bit_op(index, |args| args.word | args.pos);
    }
    pub fn clear_bit(&mut self, index: usize) {
        self.bit_op(index, |args| args.word & !args.pos);
    }
    pub fn toggle(&mut self, index: usize) {
        self.bit_op(index, |args| args.word ^ args.pos);
    }
}
struct BitOpArgs {
    pub word: usize,
    pub pos: usize,
}
impl Len for BitSet {
    fn len(&self) -> usize {
        self.ones
    }
}
impl Capacity for BitSet {
    fn capacity(&self) -> usize {
        self.capacity_bits
    }
}
impl Clear for BitSet {
    fn clear(&mut self) {
        self.words.iter_mut().for_each(|x| *x = 0);
        self.ones = 0;
    }
}

#[must_use]
const fn word_index(bit_index: usize) -> usize {
    bit_index / USIZE_BITS
}
#[must_use]
const fn bit_offset(bit_index: usize) -> usize {
    bit_index % USIZE_BITS
}

#[cfg(test)]
mod tests {
    use crate::ops::len::LenExt;

    use super::*;

    #[test]
    fn test_bit_set() {
        let mut b = BitSet::new(16);
        assert!(!b.get(1));
        assert!(b.is_empty());
        b.set(1);
        assert!(b.get(1));
        assert_eq!(b.len(), 1);
        b.set(15);
        assert!(b.get(15));
        assert_eq!(b.len(), 2);
    }
}
