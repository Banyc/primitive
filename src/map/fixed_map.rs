use core::{
    borrow::Borrow,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

#[derive(Debug, Clone)]
pub struct FixedHashMap<K, V, H> {
    entries: Vec<Option<(K, V)>>,
    _hashing: PhantomData<H>,
}
impl<K, V, H> FixedHashMap<K, V, H> {
    #[must_use]
    pub fn new(size: usize) -> Self {
        Self {
            entries: (0..size).map(|_| None).collect(),
            _hashing: PhantomData,
        }
    }
}
impl<K, V, H> FixedHashMap<K, V, H>
where
    K: Eq + Hash,
    H: Hasher + Default,
{
    pub fn insert(&mut self, key: K, value: V) -> (usize, Option<(K, V)>) {
        let index = self.index(&key);
        let ejected = match &mut self.entries[index] {
            Some((k, v)) => {
                let k = core::mem::replace(k, key);
                let v = core::mem::replace(v, value);
                Some((k, v))
            }
            None => {
                self.entries[index] = Some((key, value));
                None
            }
        };
        (index, ejected)
    }
    #[must_use]
    pub fn entry(&self, index: usize) -> Option<(&K, &V)> {
        let (k, v) = self.entries[index].as_ref()?;
        Some((k, v))
    }
    #[must_use]
    pub fn entry_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
        let (k, v) = self.entries[index].as_mut()?;
        Some((k, v))
    }
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: Eq + Hash + ?Sized,
        K: Borrow<Q>,
    {
        let Some((k, v)) = &self.entries[self.index(key)] else {
            return None;
        };
        if k.borrow() != key {
            return None;
        }
        Some(v)
    }
    #[must_use]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: Eq + Hash + ?Sized,
        K: Borrow<Q>,
    {
        let index = self.index(key);
        let Some((k, v)) = &mut self.entries[index] else {
            return None;
        };
        let k = &*k;
        if k.borrow() != key {
            return None;
        }
        Some(v)
    }
    #[must_use]
    fn index<Q>(&self, key: &Q) -> usize
    where
        Q: Eq + Hash + ?Sized,
        K: Borrow<Q>,
    {
        let mut hasher = H::default();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        hash as usize % self.entries.len()
    }
}
