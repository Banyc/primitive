mod bench;
pub mod cap_map;
pub mod dense_hash_map;
pub mod expiring_map;
pub mod free_list;
pub mod grow_dense_map;
pub mod hash_map;
pub mod linear_front_btree;
pub mod weak_lru;

pub trait MapInsert<K, V> {
    type Out;
    fn insert(&mut self, key: K, value: V) -> Self::Out;
}
