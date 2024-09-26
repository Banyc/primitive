use std::collections::LinkedList;

#[derive(Debug)]
pub struct HeapRandomizer {
    list: LinkedList<usize>,
}
impl HeapRandomizer {
    pub fn new() -> Self {
        Self {
            list: LinkedList::new(),
        }
    }

    const DEPTH: usize = 2 << 9;
    pub fn randomize(&mut self) {
        for i in 0..Self::DEPTH {
            self.list.push_back(i);
        }
    }
}
impl Default for HeapRandomizer {
    fn default() -> Self {
        Self::new()
    }
}
