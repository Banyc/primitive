#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitSet {
    integers: Vec<usize>,
}
impl BitSet {
    pub fn new(bits: usize) -> Self {
        let bytes = bits.div_ceil(bits);
        let integers = bytes.div_ceil(core::mem::size_of::<usize>());
        Self {
            integers: vec![0; integers],
        }
    }
    pub fn capacity(&self) -> usize {
        self.integers.len() * core::mem::size_of::<usize>()
    }

    pub fn clear_all(&mut self) {
        self.integers.iter_mut().for_each(|x| *x = 0);
    }
    pub fn get(&self, index: usize) -> bool {
        let integer = self.integers[integer_index(index)];
        let pos = 1 << bit_offset(index);
        let is_empty = integer & pos == 0;
        !is_empty
    }
    fn bit_op(&mut self, bit_index: usize, op: impl Fn(usize, usize) -> usize) {
        let integer = &mut self.integers[integer_index(bit_index)];
        let pos = 1 << bit_offset(bit_index);
        *integer = op(*integer, pos);
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

fn integer_index(bit_index: usize) -> usize {
    bit_index / core::mem::size_of::<usize>()
}
fn bit_offset(bit_index: usize) -> usize {
    bit_index % core::mem::size_of::<usize>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_set() {
        let mut ba = BitSet::new(16);
        assert!(!ba.get(1));
        ba.set(1);
        assert!(ba.get(1));
    }
}
