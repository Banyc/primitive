#[derive(Debug)]
pub struct ObjectPool<T> {
    buf: Vec<T>,
    max: usize,
    alloc: fn() -> T,
    clear: fn(&mut T),
}
impl<T> ObjectPool<T> {
    pub fn new(max: usize, alloc: fn() -> T, clear: fn(&mut T)) -> Self {
        Self {
            buf: vec![],
            max,
            alloc,
            clear,
        }
    }

    pub fn take(&mut self) -> T {
        self.buf.pop().unwrap_or_else(|| (self.alloc)())
    }
    pub fn put(&mut self, mut obj: T) {
        if self.buf.len() == self.max {
            return;
        }
        (self.clear)(&mut obj);
        self.buf.push(obj);
    }
}

pub fn buf_pool<T>(max: usize) -> ObjectPool<Vec<T>> {
    ObjectPool::new(max, Vec::new, |b| b.clear())
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    use super::*;

    const N: usize = 2 << 18;
    const DATA_SIZE: usize = 2;

    #[derive(Default)]
    struct Data {
        _buf: [u8; DATA_SIZE],
    }

    #[bench]
    fn bench_pool(bencher: &mut test::Bencher) {
        let mut in_used = vec![];
        let mut pool = buf_pool(usize::MAX);
        bencher.iter(|| {
            for _ in 0..N {
                let mut buf = pool.take();
                buf.push(Data::default());
                in_used.push(buf);
            }
            for _ in 0..N {
                let buf = in_used.pop().unwrap();
                pool.put(buf);
            }
        });
    }

    #[bench]
    fn bench_alloc(bencher: &mut test::Bencher) {
        let mut in_used = vec![];
        bencher.iter(|| {
            for _ in 0..N {
                let buf = vec![Data::default()];
                in_used.push(buf);
            }
            for _ in 0..N {
                in_used.pop().unwrap();
            }
        });
    }
}
