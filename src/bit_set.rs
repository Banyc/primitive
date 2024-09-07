use crate::Len;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitSet {
    integers: Vec<usize>,
    count: usize,
}
impl BitSet {
    #[must_use]
    pub fn new(bits: usize) -> Self {
        let bytes = bits.div_ceil(bits);
        let integers = bytes.div_ceil(core::mem::size_of::<usize>());
        Self {
            integers: vec![0; integers],
            count: 0,
        }
    }
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.integers.len() * core::mem::size_of::<usize>()
    }

    pub fn clear_all(&mut self) {
        self.integers.iter_mut().for_each(|x| *x = 0);
        self.count = 0;
    }
    #[must_use]
    pub fn get(&self, index: usize) -> bool {
        let integer = self.integers[integer_index(index)];
        let pos = 1 << bit_offset(index);
        let is_empty = integer & pos == 0;
        !is_empty
    }
    fn bit_op(&mut self, bit_index: usize, op: impl Fn(usize, usize) -> usize) {
        let integer = &mut self.integers[integer_index(bit_index)];
        let prev = integer.count_ones();
        let pos = 1 << bit_offset(bit_index);
        *integer = op(*integer, pos);
        let curr = integer.count_ones();
        match prev.cmp(&curr) {
            std::cmp::Ordering::Less => self.count += usize::try_from(curr - prev).unwrap(),
            std::cmp::Ordering::Equal => (),
            std::cmp::Ordering::Greater => self.count -= usize::try_from(prev - curr).unwrap(),
        }
    }
    pub fn set(&mut self, index: usize) {
        self.bit_op(index, |integer, pos| integer | pos);
    }
    pub fn clear(&mut self, index: usize) {
        self.bit_op(index, |integer, pos| integer & !pos);
    }
    pub fn toggle(&mut self, index: usize) {
        self.bit_op(index, |integer, pos| integer ^ pos);
    }
}
impl Len for BitSet {
    fn len(&self) -> usize {
        self.count
    }
}

fn integer_index(bit_index: usize) -> usize {
    bit_index / core::mem::size_of::<usize>()
}
fn bit_offset(bit_index: usize) -> usize {
    bit_index % core::mem::size_of::<usize>()
}

#[cfg(test)]
mod tests {
    use crate::LenExt;

    use super::*;

    #[test]
    fn test_bit_set() {
        let mut b = BitSet::new(16);
        assert!(!b.get(1));
        assert!(b.is_empty());
        b.set(1);
        assert!(b.get(1));
        assert_eq!(b.len(), 1);
    }
}
