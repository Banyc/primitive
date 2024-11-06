use core::{borrow::Borrow, hash::Hash};
use std::collections::HashMap;

use super::MapInsert;

pub trait HashGet<K, V> {
    #[must_use]
    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: Eq + Hash + ?Sized,
        K: Borrow<Q>;
}
pub trait HashGetMut<K, V> {
    #[must_use]
    fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: Eq + Hash + ?Sized,
        K: Borrow<Q>;
}
pub trait HashRemove<K, V> {
    fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: Eq + Hash + ?Sized,
        K: Borrow<Q>;
}

pub trait HashEnsure<K, V>: HashGetMut<K, V> + MapInsert<K, V> {
    #[must_use]
    fn ensure(&mut self, key: &K, new_value: impl Fn() -> V) -> &mut V
    where
        K: Eq + core::hash::Hash + Clone,
    {
        if self.get_mut(key).is_none() {
            self.insert(key.clone(), new_value());
        }
        self.get_mut(key).unwrap()
    }
}
impl<K, V, T> HashEnsure<K, V> for T where T: HashGetMut<K, V> + MapInsert<K, V> {}

impl<K, V> HashGet<K, V> for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: Eq + Hash + ?Sized,
        K: Borrow<Q>,
    {
        HashMap::get(self, key)
    }
}
impl<K, V> HashGetMut<K, V> for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: Eq + Hash + ?Sized,
        K: Borrow<Q>,
    {
        HashMap::get_mut(self, key)
    }
}
impl<K, V> HashRemove<K, V> for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: Eq + Hash + ?Sized,
        K: Borrow<Q>,
    {
        HashMap::remove(self, key)
    }
}
impl<K, V> MapInsert<K, V> for HashMap<K, V>
where
    K: Eq + Hash,
{
    type Out = Option<V>;
    fn insert(&mut self, key: K, value: V) -> Self::Out {
        HashMap::insert(self, key, value)
    }
}
