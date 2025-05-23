#![allow(unused)]

use core::{
    cell::{RefCell, UnsafeCell},
    ops::{Deref, DerefMut},
};
use std::fmt::Debug;

#[derive(Debug)]
#[repr(transparent)]
pub struct MutCell<T> {
    #[cfg(debug_assertions)]
    cell: RefCell<T>,
    #[cfg(not(debug_assertions))]
    cell: UnsafeCell<T>,
}
impl<T> MutCell<T> {
    pub const fn new(value: T) -> Self {
        #[cfg(debug_assertions)]
        let cell = RefCell::new(value);
        #[cfg(not(debug_assertions))]
        let cell = UnsafeCell::new(value);
        Self { cell }
    }

    /// # Safety
    ///
    /// the value must not be currently borrowed
    pub unsafe fn borrow_mut(&self) -> impl DerefMut<Target = T> + '_ {
        #[cfg(debug_assertions)]
        return self.cell.borrow_mut();
        #[cfg(not(debug_assertions))]
        {
            let value = unsafe { &mut *self.cell.get() };
            ThinWrapMut::new(value)
        }
    }

    /// # Safety
    ///
    /// the value must not be currently mutably borrowed
    pub unsafe fn borrow(&self) -> impl Deref<Target = T> + '_ {
        #[cfg(debug_assertions)]
        return self.cell.borrow();
        #[cfg(not(debug_assertions))]
        {
            let value = unsafe { &*self.cell.get() };
            ThinWrap::new(value)
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct ThinWrap<'a, T> {
    value: &'a T,
}
impl<'a, T> ThinWrap<'a, T> {
    pub const fn new(value: &'a T) -> Self {
        Self { value }
    }
}
impl<T> Deref for ThinWrap<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct ThinWrapMut<'a, T> {
    value: &'a mut T,
}
impl<'a, T> ThinWrapMut<'a, T> {
    pub const fn new(value: &'a mut T) -> Self {
        Self { value }
    }
}
impl<T> Deref for ThinWrapMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}
impl<T> DerefMut for ThinWrapMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}
