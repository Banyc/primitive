#![cfg_attr(feature = "nightly", feature(test))]
#[cfg(feature = "nightly")]
extern crate test;

pub mod bit_set;
pub mod free_list;
pub mod indexed_queue;
pub mod notify;
pub mod priority;
pub mod static_borrow_vec;
pub mod vec_seg;

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
