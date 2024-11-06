use std::collections::HashMap;

use crate::{Clear, Len};

use super::{
    free_list::{DenseFreeList, FreeList},
    hash_map::{HashGet, HashGetMut, HashRemove},
    MapInsert,
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
impl<K, V> MapInsert<K, V> for DenseHashMap<K, V>
where
    K: Eq + core::hash::Hash,
{
    type Out = Option<V>;
    /// slower than [`std::collections::HashMap::insert()`]
    fn insert(&mut self, key: K, value: V) -> Option<V> {
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
}
impl<K, V> HashRemove<K, V> for DenseHashMap<K, V>
where
    K: Eq + core::hash::Hash,
{
    /// slower than [`std::collections::HashMap::remove()`]:
    fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: ?Sized + core::hash::Hash + Eq,
        K: core::borrow::Borrow<Q>,
    {
        let index = self.index.remove(key)?;
        self.data.remove(index)
    }
}
impl<K, V> HashGet<K, V> for DenseHashMap<K, V>
where
    K: Eq + core::hash::Hash,
{
    /// slower than [`std::collections::HashMap::get()`]:
    #[must_use]
    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: ?Sized + core::hash::Hash + Eq,
        K: core::borrow::Borrow<Q>,
    {
        let index = *self.index.get(key)?;
        Some(self.data.get(index).unwrap())
    }
}
impl<K, V> HashGetMut<K, V> for DenseHashMap<K, V>
where
    K: Eq + core::hash::Hash,
{
    /// slower than [`std::collections::HashMap::get_mut()`]:
    #[must_use]
    fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: ?Sized + core::hash::Hash + Eq,
        K: core::borrow::Borrow<Q>,
    {
        let index = *self.index.get(key)?;
        Some(self.data.get_mut(index).unwrap())
    }
}
impl<K, V> DenseHashMap<K, V>
where
    K: Eq + core::hash::Hash,
{
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
