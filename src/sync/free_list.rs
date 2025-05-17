use std::mem::{self, MaybeUninit};

use crate::set::bit_set::BitSet;

use super::{free_u32_list::FreeU32List, sync_unsafe_cell::SyncUnsafeCell};

/// - ref: <https://yeet.cx/blog/lock-free-rust/>
#[derive(Debug)]
pub struct FreeList<T, const N: usize> {
    slots: [SyncUnsafeCell<MaybeUninit<T>>; N],
    free_slots: FreeU32List<N>,
    leak_on_drop: bool,
}
impl<T, const N: usize> Drop for FreeList<T, N> {
    fn drop(&mut self) {
        if self.leak_on_drop {
            return;
        }
        let mut vacant_slots = BitSet::new(N);
        for index in self.free_slots.iter_vacant() {
            let index_ = usize::try_from(index).unwrap();
            vacant_slots.set(index_);
        }
        for (i, vacant) in vacant_slots.iter().enumerate() {
            if vacant {
                continue;
            }
            let slot = self.slots[i].get_mut();
            let obj = mem::replace(slot, MaybeUninit::uninit());
            unsafe { obj.assume_init() };
        }
    }
}
impl<T, const N: usize> Default for FreeList<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T, const N: usize> FreeList<T, N> {
    pub fn new() -> Self {
        Self {
            slots: [const { SyncUnsafeCell::new(MaybeUninit::uninit()) }; N],
            free_slots: FreeU32List::new(),
            leak_on_drop: false,
        }
    }
    pub fn try_insert(&self, value: T) -> Result<u32, T> {
        let Some(index) = self.free_slots.alloc() else {
            return Err(value);
        };
        let index_ = usize::try_from(index).unwrap();
        let slot = unsafe { &mut *self.slots[index_].get() };
        *slot = MaybeUninit::new(value);
        Ok(index)
    }
    /// # Safety
    ///
    /// - `index` is from [`Self::try_insert()`]
    /// - once taken after [`Self::try_insert()`], `index` will not be taken anymore unless it has been pulled from [`Self::try_insert()`] later
    pub unsafe fn assume_take(&self, index: u32) -> T {
        let index_ = usize::try_from(index).unwrap();
        let slot = unsafe { &mut *self.slots[index_].get() };
        let val = mem::replace(slot, MaybeUninit::uninit());
        let val = unsafe { val.assume_init() };
        unsafe { self.free_slots.free(index) };
        val
    }
    pub fn leak(mut self) {
        self.leak_on_drop = true;
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, sync::Arc};

    use super::*;

    #[cfg(miri)]
    const N: usize = 1 << 2;
    #[cfg(not(miri))]
    const N: usize = 1 << 15;

    #[test]
    fn basics() {
        let l: Arc<FreeList<usize, N>> = Arc::new(FreeList::new());
        std::thread::scope(|s| {
            for _ in 0..16 {
                s.spawn(|| {
                    let mut written = VecDeque::new();
                    let pop = |written: &mut VecDeque<(u32, usize)>| {
                        let (index, val) = written.pop_front().unwrap();
                        let val_ = unsafe { l.assume_take(index) };
                        assert_eq!(val, val_);
                    };

                    for i in 0.. {
                        let Ok(index) = l.try_insert(i) else {
                            break;
                        };
                        written.push_back((index, i));
                        if i % 2 == 0 {
                            pop(&mut written);
                        }
                    }
                    while !written.is_empty() {
                        pop(&mut written);
                    }
                });
            }
        });
    }
}
