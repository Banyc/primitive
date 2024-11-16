#[derive(Debug, Clone, Copy)]
pub struct OrdEntry<K, V> {
    pub key: K,
    pub value: V,
}
impl<K, V> OrdEntry<K, V> {
    pub fn into_flatten(self) -> (K, V) {
        (self.key, self.value)
    }
    pub fn flatten(&self) -> (&K, &V) {
        (&self.key, &self.value)
    }
}
impl<K: PartialEq, V> PartialEq for OrdEntry<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl<K: Eq, V> Eq for OrdEntry<K, V> {}
impl<K: PartialOrd, V> PartialOrd for OrdEntry<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}
impl<K: Ord, V> Ord for OrdEntry<K, V> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}
