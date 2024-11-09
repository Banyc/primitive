use crate::{ops::len::Len, Clear};

#[derive(Debug, Clone)]
pub struct DenseFreeList<T> {
    data: Vec<DenseFreeListData<T>>,
    index: SparseFreeList<usize>,
}
impl<T> DenseFreeList<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: vec![],
            index: SparseFreeList::new(),
        }
    }
}
impl<T> Default for DenseFreeList<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> FreeList<T> for DenseFreeList<T> {
    #[must_use]
    fn get(&self, index: usize) -> Option<&T> {
        let index = self.local_index(index)?;
        Some(&self.data[index].value)
    }
    #[must_use]
    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let index = self.local_index(index)?;
        Some(&mut self.data[index].value)
    }
    fn iter<'a>(&'a self) -> impl Iterator<Item = (usize, &'a T)> + Clone
    where
        T: 'a,
    {
        self.data.iter().map(|data| (data.user_index, &data.value))
    }
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (usize, &'a mut T)>
    where
        T: 'a,
    {
        self.data
            .iter_mut()
            .map(|data| (data.user_index, &mut data.value))
    }

    #[must_use]
    fn insert(&mut self, value: T) -> usize {
        let index = self.data.len();
        let user_index = self.index.insert(index);
        let data = DenseFreeListData { value, user_index };
        self.data.push(data);
        user_index
    }
    fn remove(&mut self, index: usize) -> Option<T> {
        let local_index = self.local_index(index)?;
        self.index.remove(index).unwrap();
        let DenseFreeListData { value, user_index } = self.data.swap_remove(local_index);
        assert_eq!(user_index, index);
        if let Some(data) = self.data.get(local_index) {
            let i = self.index.get_mut(data.user_index).unwrap();
            *i = local_index;
        }
        Some(value)
    }
}
impl<T> DenseFreeList<T> {
    #[must_use]
    fn local_index(&self, index: usize) -> Option<usize> {
        Some(*self.index.get(index)?)
    }
}
impl<T> Len for DenseFreeList<T> {
    fn len(&self) -> usize {
        self.data.len()
    }
}
impl<T> Clear for DenseFreeList<T> {
    fn clear(&mut self) {
        self.data.clear();
        self.index.clear();
    }
}
#[derive(Debug, Clone)]
struct DenseFreeListData<T> {
    pub value: T,
    pub user_index: usize,
}

#[derive(Debug, Clone)]
pub struct SparseFreeList<T> {
    free: Vec<usize>,
    data: Vec<Option<T>>,
    count: usize,
}
impl<T> SparseFreeList<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            free: vec![],
            data: vec![],
            count: 0,
        }
    }
}
impl<T> Default for SparseFreeList<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> FreeList<T> for SparseFreeList<T> {
    #[must_use]
    fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index).and_then(|data| data.as_ref())
    }
    #[must_use]
    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index).and_then(|data| data.as_mut())
    }
    fn iter<'a>(&'a self) -> impl Iterator<Item = (usize, &'a T)> + Clone
    where
        T: 'a,
    {
        self.data.iter().enumerate().filter_map(|(index, data)| {
            let data = data.as_ref()?;
            Some((index, data))
        })
    }
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (usize, &'a mut T)>
    where
        T: 'a,
    {
        self.data
            .iter_mut()
            .enumerate()
            .filter_map(|(index, data)| {
                let data = data.as_mut()?;
                Some((index, data))
            })
    }

    #[must_use]
    fn insert(&mut self, value: T) -> usize {
        self.count += 1;
        let Some(index) = self.free.pop() else {
            let index = self.data.len();
            self.data.push(Some(value));
            return index;
        };
        self.data[index] = Some(value);
        index
    }
    fn remove(&mut self, index: usize) -> Option<T> {
        let value = self.data.get_mut(index)?.take()?;
        self.count -= 1;
        self.free.push(index);
        Some(value)
    }
}
impl<T> Len for SparseFreeList<T> {
    fn len(&self) -> usize {
        self.count
    }
}
impl<T> Clear for SparseFreeList<T> {
    fn clear(&mut self) {
        self.data.clear();
        self.free.clear();
        self.count = 0;
    }
}

pub trait FreeList<T>: Len + Clear {
    #[must_use]
    fn get(&self, index: usize) -> Option<&T>;
    #[must_use]
    fn get_mut(&mut self, index: usize) -> Option<&mut T>;
    fn iter<'a>(&'a self) -> impl Iterator<Item = (usize, &'a T)> + Clone
    where
        T: 'a;
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (usize, &'a mut T)>
    where
        T: 'a;

    #[must_use]
    fn insert(&mut self, value: T) -> usize;
    fn remove(&mut self, index: usize) -> Option<T>;
}

#[cfg(test)]
mod tests {
    use crate::ops::len::LenExt;

    use super::*;

    #[test]
    fn test_sparse() {
        let l = SparseFreeList::new();
        test_free_list(l);
    }
    #[test]
    fn test_dense() {
        let l = DenseFreeList::new();
        test_free_list(l);
    }

    fn test_free_list(mut l: impl FreeList<usize>) {
        assert!(l.is_empty());
        let i_0 = l.insert(0);
        assert_eq!(l.len(), 1);
        let i_1 = l.insert(1);
        assert_eq!(l.iter().count(), 2);
        assert_eq!(l.len(), 2);
        assert_eq!(*l.get(i_0).unwrap(), 0);
        assert_eq!(*l.get(i_1).unwrap(), 1);
        assert_eq!(l.remove(i_0).unwrap(), 0);
        assert!(l.get(i_0).is_none());
        assert!(l.get(i_1).is_some());
        assert_eq!(l.iter().count(), 1);
        assert_eq!(l.len(), 1);
        assert_eq!(l.remove(i_1).unwrap(), 1);
        assert!(l.get(i_1).is_none());
        assert!(l.is_empty());
    }
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    use core::hint::black_box;

    use slotmap::SlotMap;

    use super::*;

    const N: usize = 2 << 16;
    const VALUE_SIZE: usize = 2 << 5;

    struct Value {
        #[allow(dead_code)]
        buf: [u8; VALUE_SIZE],
    }
    impl Value {
        pub fn new() -> Self {
            Self {
                buf: [0; VALUE_SIZE],
            }
        }
    }

    macro_rules! insert_remove {
        ($bencher: ident, $l: ident) => {
            $bencher.iter(|| {
                let mut indices = vec![];
                for _ in 0..N {
                    let index = $l.insert(Value::new());
                    indices.push(index);
                }
                let mut reverse = false;
                for i in 0..indices.len() {
                    let i = if reverse { indices.len() - 1 - i } else { i };
                    reverse = !reverse;
                    let index = indices[i];
                    $l.remove(index);
                }
            });
        };
    }
    #[bench]
    fn bench_insert_remove_sparse(bencher: &mut test::Bencher) {
        let mut l = SparseFreeList::new();
        insert_remove!(bencher, l);
    }
    #[bench]
    fn bench_insert_remove_dense(bencher: &mut test::Bencher) {
        let mut l = DenseFreeList::new();
        insert_remove!(bencher, l);
    }
    #[bench]
    fn bench_insert_remove_slot(bencher: &mut test::Bencher) {
        let mut l = SlotMap::new();
        insert_remove!(bencher, l);
    }

    macro_rules! insert_iter_remove {
        ($bencher: ident, $l: ident) => {
            let n = ((N as f64).sqrt() / 2.).round() as usize;
            $bencher.iter(|| {
                let mut indices = vec![];
                for _ in 0..n {
                    let index = $l.insert(Value::new());
                    indices.push(index);
                }
                let mut reverse = false;
                for i in 0..indices.len() {
                    for v in $l.iter() {
                        black_box(v);
                    }
                    let i = if reverse { indices.len() - 1 - i } else { i };
                    reverse = !reverse;
                    let index = indices[i];
                    $l.remove(index);
                }
            });
        };
    }
    #[bench]
    fn bench_insert_iter_remove_sparse(bencher: &mut test::Bencher) {
        let mut l = SparseFreeList::new();
        insert_iter_remove!(bencher, l);
    }
    #[bench]
    fn bench_insert_iter_remove_dense(bencher: &mut test::Bencher) {
        let mut l = DenseFreeList::new();
        insert_iter_remove!(bencher, l);
    }
    #[bench]
    fn bench_insert_iter_remove_slot(bencher: &mut test::Bencher) {
        let mut l = SlotMap::new();
        insert_iter_remove!(bencher, l);
    }

    macro_rules! insert_clear {
        ($bencher: ident, $l: ident) => {
            $bencher.iter(|| {
                let mut indices = vec![];
                for _ in 0..N {
                    let index = $l.insert(Value::new());
                    indices.push(index);
                }
                $l.clear();
            });
        };
    }
    #[bench]
    fn bench_insert_clear_sparse(bencher: &mut test::Bencher) {
        let mut l = SparseFreeList::new();
        insert_clear!(bencher, l);
    }
    #[bench]
    fn bench_insert_clear_dense(bencher: &mut test::Bencher) {
        let mut l = DenseFreeList::new();
        insert_clear!(bencher, l);
    }
    #[bench]
    fn bench_insert_clear_slot(bencher: &mut test::Bencher) {
        let mut l = SlotMap::new();
        insert_clear!(bencher, l);
    }

    macro_rules! get {
        ($bencher: ident, $l: ident) => {
            let mut indices = vec![];
            for _ in 0..N {
                let index = $l.insert(Value::new());
                indices.push(index);
            }
            $bencher.iter(|| {
                let mut reverse = false;
                for i in 0..indices.len() {
                    let i = if reverse { indices.len() - 1 - i } else { i };
                    reverse = !reverse;
                    let index = indices[i];
                    black_box($l.get(index));
                }
            });
        };
    }
    #[bench]
    fn bench_get_sparse(bencher: &mut test::Bencher) {
        let mut l = SparseFreeList::new();
        get!(bencher, l);
    }
    #[bench]
    fn bench_get_dense(bencher: &mut test::Bencher) {
        let mut l = DenseFreeList::new();
        get!(bencher, l);
    }
    #[bench]
    fn bench_get_slot(bencher: &mut test::Bencher) {
        let mut l = SlotMap::new();
        get!(bencher, l);
    }

    macro_rules! iter {
        ($bencher: ident, $l: ident) => {
            let mut indices = vec![];
            for _ in 0..N {
                let index = $l.insert(Value::new());
                indices.push(index);
            }
            $bencher.iter(|| {
                for v in $l.iter() {
                    black_box(v);
                }
            });
        };
    }
    #[bench]
    fn bench_iter_sparse(bencher: &mut test::Bencher) {
        let mut l = SparseFreeList::new();
        iter!(bencher, l);
    }
    #[bench]
    fn bench_iter_dense(bencher: &mut test::Bencher) {
        let mut l = DenseFreeList::new();
        iter!(bencher, l);
    }
    #[bench]
    fn bench_iter_slot(bencher: &mut test::Bencher) {
        let mut l = SlotMap::new();
        iter!(bencher, l);
    }
}
