#![cfg_attr(feature = "nightly", feature(test))]
#[cfg(feature = "nightly")]
extern crate test;

pub mod arena;
pub mod bench;
pub mod dep_inj;
pub mod fail;
pub mod io;
pub mod iter;
pub mod map;
pub mod mut_cell;
pub mod non_max;
pub mod ops;
pub mod queue;
pub mod set;
pub mod stacked_state;
pub mod sync;
pub mod time;

pub trait Clear {
    fn clear(&mut self);
}
