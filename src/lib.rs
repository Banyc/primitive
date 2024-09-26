#![cfg_attr(feature = "nightly", feature(test))]
#[cfg(feature = "nightly")]
extern crate test;

pub mod bench;
pub mod bit_set;
pub mod dense_hash_map;
pub mod dep_inj;
pub mod fixed_queue;
pub mod float;
pub mod free_list;
pub mod grow_dense_map;
pub mod indexed_queue;
pub mod iter;
pub mod mut_cell;
pub mod non_max;
pub mod notify;
pub mod obj_pool;
pub mod priority;
pub mod ring_seq;
pub mod seq;
pub mod sparse_set;
pub mod stable_vec;
pub mod stacked_state;
pub mod static_borrow_vec;
pub mod vec_seg;

pub trait Capacity: Len {
    #[must_use]
    fn capacity(&self) -> usize;
}

#[allow(clippy::len_without_is_empty)]
pub trait Len {
    #[must_use]
    fn len(&self) -> usize;
}
pub trait LenExt: Len {
    #[must_use]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<T: Len> LenExt for T {}

pub trait Clear {
    fn clear(&mut self);
}
