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

#[cfg(feature = "nightly")]
#[cfg(test)]
mod tests {
    use core::hint::black_box;

    use test::Bencher;

    const SIZE1: usize = 1 << 10;
    const SIZE2: usize = 1 << 4;

    #[bench]
    fn bench_indirection_once(bencher: &mut Bencher) {
        let mut a = vec![];
        for _ in 0..SIZE1 {
            let b: Vec<u8> = vec![0; SIZE2];
            a.push(b);
        }
        bencher.iter(|| {
            for a in &a {
                for b in a {
                    black_box(b);
                }
            }
        });
    }
    #[bench]
    fn bench_indirection_none(bencher: &mut Bencher) {
        let a: Vec<[u8; SIZE2]> = vec![[0; SIZE2]; SIZE1];
        bencher.iter(|| {
            for a in &a {
                for b in a {
                    black_box(b);
                }
            }
        });
    }
}
