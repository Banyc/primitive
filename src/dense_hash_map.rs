use std::collections::HashMap;

use crate::{
    free_list::{DenseFreeList, FreeList},
    Clear, Len,
};

/// vs. [`indexmap::IndexMap`]:
/// - [`Self::values()`]: basically the same
#[derive(Debug, Clone)]
pub struct DenseHashMap<K, V> {
    data: DenseFreeList<V>,
    index: HashMap<K, usize>,
}
impl<K, V> DenseHashMap<K, V> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: DenseFreeList::new(),
            index: HashMap::new(),
        }
    }
}
impl<K, V> Default for DenseHashMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K, V> DenseHashMap<K, V>
where
    K: Eq + core::hash::Hash,
{
    /// often slower than [`std::collections::HashMap::insert()`]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let Some(&index) = self.index.get(&key) else {
            let index = self.data.insert(value);
            self.index.insert(key, index);
            return None;
        };
        let prev = self.data.remove(index).unwrap();
        let index = self.data.insert(value);
        self.index.insert(key, index);
        Some(prev)
    }
    /// vs. [`std::collections::HashMap::remove()`]:
    /// - small `V`: slower
    /// - big `V`: faster, lower variance
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: ?Sized + core::hash::Hash + Eq,
        K: core::borrow::Borrow<Q>,
    {
        let index = self.index.remove(key)?;
        self.data.remove(index)
    }

    /// vs. [`std::collections::HashMap::get()`]:
    /// - small `V`: slower
    /// - big `V`: faster, lower variance
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: ?Sized + core::hash::Hash + Eq,
        K: core::borrow::Borrow<Q>,
    {
        let index = *self.index.get(key)?;
        Some(self.data.get(index).unwrap())
    }
    /// vs. [`std::collections::HashMap::get_mut()`]:
    /// - small `V`: slower
    /// - big `V`: faster, lower variance
    #[must_use]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: ?Sized + core::hash::Hash + Eq,
        K: core::borrow::Borrow<Q>,
    {
        let index = *self.index.get(key)?;
        Some(self.data.get_mut(index).unwrap())
    }
    /// always faster than [`std::collections::HashMap::values()`]
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter().map(|(_, value)| value)
    }
    /// always faster than [`std::collections::HashMap::values_mut()`]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.iter_mut().map(|(_, value)| value)
    }
    pub fn keys(&self) -> impl Iterator<Item = &K> + Clone {
        self.index.keys()
    }
    /// always slower than [`std::collections::HashMap::iter()`]
    ///
    /// Consider using [`Self::values()`] instead.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> + Clone {
        let indices = self.index.iter();
        indices.map(|(k, &index)| (k, self.data.get(index).unwrap()))
    }
}
impl<K, V> Len for DenseHashMap<K, V> {
    fn len(&self) -> usize {
        assert_eq!(self.data.len(), self.index.len());
        self.data.len()
    }
}
impl<K, V> Clear for DenseHashMap<K, V> {
    fn clear(&mut self) {
        self.data.clear();
        self.index.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::LenExt;

    use super::*;

    #[test]
    fn test_dense_hash_map() {
        let mut m = DenseHashMap::new();
        m.insert(0, 0);
        assert_eq!(*m.get(&0).unwrap(), 0);
        m.insert(0, 1);
        assert_eq!(*m.get(&0).unwrap(), 1);
        assert_eq!(m.remove(&0).unwrap(), 1);
        assert!(m.is_empty());
    }
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    use std::hint::black_box;

    use indexmap::IndexMap;

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

    macro_rules! get {
        ($m: ident, $bencher: ident) => {
            for i in 0..N {
                $m.insert(i, Value::new());
            }
            $bencher.iter(|| {
                for i in 0..N {
                    black_box($m.get(&i));
                }
            });
        };
    }
    #[bench]
    fn bench_get_std(bencher: &mut test::Bencher) {
        let mut m = HashMap::new();
        get!(m, bencher);
    }
    #[bench]
    fn bench_get_dense(bencher: &mut test::Bencher) {
        let mut m = DenseHashMap::new();
        get!(m, bencher);
    }
    #[bench]
    fn bench_get_index_map(bencher: &mut test::Bencher) {
        let mut m = IndexMap::new();
        get!(m, bencher);
    }

    macro_rules! iter {
        ($m: ident, $bencher: ident) => {
            for i in 0..N {
                $m.insert(i, Value::new());
            }
            $bencher.iter(|| {
                for (k, v) in $m.iter() {
                    black_box((k, v));
                }
            });
        };
    }
    #[bench]
    fn bench_iter_std(bencher: &mut test::Bencher) {
        let mut m = HashMap::new();
        iter!(m, bencher);
    }
    #[bench]
    fn bench_iter_dense(bencher: &mut test::Bencher) {
        let mut m = DenseHashMap::new();
        iter!(m, bencher);
    }
    #[bench]
    fn bench_iter_index_map(bencher: &mut test::Bencher) {
        let mut m = IndexMap::new();
        iter!(m, bencher);
    }

    macro_rules! insert_remove {
        ($m: ident, $bencher: ident) => {
            $bencher.iter(|| {
                for i in 0..N {
                    $m.insert(i, Value::new());
                }
                for i in 0..N {
                    #[allow(deprecated)]
                    $m.remove(&i);
                }
            });
        };
    }
    #[bench]
    fn bench_insert_remove_std(bencher: &mut test::Bencher) {
        let mut m = HashMap::new();
        insert_remove!(m, bencher);
    }
    #[bench]
    fn bench_insert_remove_dense(bencher: &mut test::Bencher) {
        let mut m = DenseHashMap::new();
        insert_remove!(m, bencher);
    }
    #[bench]
    fn bench_insert_remove_index_map(bencher: &mut test::Bencher) {
        let mut m = IndexMap::new();
        insert_remove!(m, bencher);
    }

    macro_rules! insert_clear {
        ($m: ident, $bencher: ident) => {
            $bencher.iter(|| {
                for i in 0..N {
                    $m.insert(i, Value::new());
                }
                $m.clear();
            });
        };
    }
    #[bench]
    fn bench_insert_clear_std(bencher: &mut test::Bencher) {
        let mut m = HashMap::new();
        insert_clear!(m, bencher);
    }
    #[bench]
    fn bench_insert_clear_dense(bencher: &mut test::Bencher) {
        let mut m = DenseHashMap::new();
        insert_clear!(m, bencher);
    }
    #[bench]
    fn bench_insert_clear_index_map(bencher: &mut test::Bencher) {
        let mut m = IndexMap::new();
        insert_clear!(m, bencher);
    }

    macro_rules! values {
        ($m: ident, $bencher: ident) => {
            for i in 0..N {
                $m.insert(i, Value::new());
            }
            $bencher.iter(|| {
                for v in $m.values() {
                    black_box(v);
                }
            });
        };
    }
    #[bench]
    fn bench_values_std(bencher: &mut test::Bencher) {
        let mut m = HashMap::new();
        values!(m, bencher);
    }
    #[bench]
    fn bench_values_dense(bencher: &mut test::Bencher) {
        let mut m = DenseHashMap::new();
        values!(m, bencher);
    }
    #[bench]
    fn bench_values_index_map(bencher: &mut test::Bencher) {
        let mut m = IndexMap::new();
        values!(m, bencher);
    }
}
