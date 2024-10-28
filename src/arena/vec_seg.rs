use crate::Len;

#[derive(Debug, Clone, Copy)]
pub struct SegKey {
    start: usize,
    end: usize,
}
impl SegKey {
    #[must_use]
    pub fn empty_slice() -> Self {
        Self { start: 0, end: 0 }
    }
}
impl Len for SegKey {
    fn len(&self) -> usize {
        self.end - self.start
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OpenSegKey {
    start: usize,
}

/// Extension only arena for slices of same items
#[derive(Debug, Clone)]
pub struct VecSeg<T> {
    arena: Vec<T>,
}
impl<T> VecSeg<T> {
    #[must_use]
    pub fn from_vec(buf: Vec<T>) -> (Self, SegKey) {
        let start = 0;
        let end = buf.len();
        let key = SegKey { start, end };
        let this = Self { arena: buf };
        (this, key)
    }
    #[must_use]
    pub fn into_vec(self) -> Vec<T> {
        self.arena
    }
    #[must_use]
    pub fn new() -> Self {
        Self { arena: vec![] }
    }

    #[must_use]
    pub fn extend(&mut self, iter: impl Iterator<Item = T>) -> SegKey {
        let start = self.arena.len();
        self.arena.extend(iter);
        let end = self.arena.len();
        SegKey { start, end }
    }
    pub fn push(&mut self, item: T) {
        self.arena.push(item);
    }
    #[must_use]
    pub fn open_seg(&self) -> OpenSegKey {
        let start = self.arena.len();
        OpenSegKey { start }
    }
    #[must_use]
    pub fn seal_seg(&self, open_key: OpenSegKey) -> SegKey {
        let start = open_key.start;
        let end = self.arena.len();
        SegKey { start, end }
    }

    #[must_use]
    pub fn slice(&self, key: SegKey) -> &[T] {
        &self.arena[key.start..key.end]
    }
    #[must_use]
    pub fn slice_mut(&mut self, key: SegKey) -> &mut [T] {
        &mut self.arena[key.start..key.end]
    }
}
impl<T> Default for VecSeg<T> {
    fn default() -> Self {
        Self::new()
    }
}
