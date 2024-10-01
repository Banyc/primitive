use std::collections::HashMap;

pub trait HashMapExt<K, V> {
    #[must_use]
    fn ensure(&mut self, key: &K, new_value: impl Fn() -> V) -> &mut V
    where
        K: Eq + core::hash::Hash + Clone;
}
impl<K, V> HashMapExt<K, V> for HashMap<K, V> {
    #[must_use]
    fn ensure(&mut self, key: &K, new_value: impl Fn() -> V) -> &mut V
    where
        K: Eq + core::hash::Hash + Clone,
    {
        if !self.contains_key(key) {
            self.insert(key.clone(), new_value());
        }
        self.get_mut(key).unwrap()
    }
}
