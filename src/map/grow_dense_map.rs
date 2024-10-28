use core::{borrow::Borrow, ptr::NonNull};
use std::collections::HashMap;

use crate::{arena::stable_vec::StableVec, Clear, Len};

#[derive(Debug)]
pub struct GrowDenseMap<K, V, const CHUNK_SIZE: usize> {
    stable_vec: StableVec<V, CHUNK_SIZE>,
    lookup: HashMap<K, NonNull<V>>,
}
impl<K, V, const CHUNK_SIZE: usize> GrowDenseMap<K, V, CHUNK_SIZE> {
    pub fn new() -> Self {
        Self {
            stable_vec: StableVec::new(),
            lookup: HashMap::new(),
        }
    }
}
impl<K, V, const CHUNK_SIZE: usize> Default for GrowDenseMap<K, V, CHUNK_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K, V, const CHUNK_SIZE: usize> GrowDenseMap<K, V, CHUNK_SIZE>
where
    K: core::hash::Hash + Eq,
{
    /// fast
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: ?Sized + core::hash::Hash + Eq,
        K: Borrow<Q>,
    {
        let ptr = self.lookup.get(key)?;
        Some(unsafe { ptr.as_ref() })
    }
    /// fast
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: ?Sized + core::hash::Hash + Eq,
        K: Borrow<Q>,
    {
        let ptr = self.lookup.get_mut(key)?;
        Some(unsafe { ptr.as_mut() })
    }
    /// slow
    pub fn insert(&mut self, key: K, value: V) {
        if let Some(ptr) = self.lookup.get_mut(&key) {
            *unsafe { ptr.as_mut() } = value;
            return;
        }
        let ptr = self.stable_vec.push(value);
        self.lookup.insert(key, ptr);
    }
}
impl<K, V, const CHUNK_SIZE: usize> Len for GrowDenseMap<K, V, CHUNK_SIZE> {
    fn len(&self) -> usize {
        self.lookup.len()
    }
}
impl<K, V, const CHUNK_SIZE: usize> Clear for GrowDenseMap<K, V, CHUNK_SIZE> {
    fn clear(&mut self) {
        self.stable_vec.clear();
        self.lookup.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grow_dense_map() {
        let mut m = GrowDenseMap::<_, _, 2>::new();
        m.insert(0, 0);
        assert_eq!(*m.get(&0).unwrap(), 0);
        m.insert(1, 1);
        assert_eq!(*m.get(&1).unwrap(), 1);
        m.insert(2, 2);
        assert_eq!(*m.get(&2).unwrap(), 2);
        m.insert(1, usize::MAX);
        assert_eq!(*m.get(&1).unwrap(), usize::MAX);
        assert_eq!(*m.get(&0).unwrap(), 0);
        m.insert(0, usize::MAX);
        assert_eq!(*m.get(&0).unwrap(), usize::MAX);
    }
}
