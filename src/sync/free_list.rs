use std::mem::{self, MaybeUninit};

use super::{free_u32_list::FreeU32List, sync_unsafe_cell::SyncUnsafeCell};

#[derive(Debug)]
pub struct FreeList<T, const N: usize> {
    slots: [SyncUnsafeCell<MaybeUninit<T>>; N],
    free_slots: FreeU32List<N>,
}
impl<T, const N: usize> Drop for FreeList<T, N> {
    fn drop(&mut self) {
        for index in self.free_slots.iter_occupied() {
            let index_ = usize::try_from(index).unwrap();
            let slot = self.slots[index_].get_mut();
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
        }
    }
    pub fn try_insert(&self, value: T) -> Result<u32, T> {
        let Some(index) = self.free_slots.pop() else {
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
    pub unsafe fn take(&self, index: u32) -> T {
        let index_ = usize::try_from(index).unwrap();
        let slot = unsafe { &mut *self.slots[index_].get() };
        let val = mem::replace(slot, MaybeUninit::uninit());
        let val = unsafe { val.assume_init() };
        unsafe { self.free_slots.push(index) };
        val
    }
}
