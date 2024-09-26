use core::{mem::MaybeUninit, ptr::NonNull};
use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::{Clear, Len};

#[derive(Debug)]
pub struct StableVec<T, const CHUNK_SIZE: usize> {
    chunks: Vec<Box<[MaybeUninit<T>; CHUNK_SIZE]>>,
    size: usize,
}
impl<T, const CHUNK_SIZE: usize> StableVec<T, CHUNK_SIZE> {
    pub fn new() -> Self {
        assert_eq!(CHUNK_SIZE % 2, 0);
        Self {
            chunks: vec![],
            size: 0,
        }
    }

    fn indices(index: usize) -> (usize, usize) {
        (index / CHUNK_SIZE, index % CHUNK_SIZE)
    }
    pub fn push(&mut self, value: T) -> NonNull<T> {
        let (chunk, offset) = Self::indices(self.size);
        self.size += 1;
        if self.chunks.len() == chunk {
            self.chunks
                .push(Box::new([const { MaybeUninit::uninit() }; CHUNK_SIZE]));
        }
        let chunk = &mut self.chunks[chunk];
        chunk[offset] = MaybeUninit::new(value);
        let ptr = unsafe { chunk[offset].assume_init_mut() };
        NonNull::from(ptr)
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }
        let (chunk, offset) = Self::indices(self.size);
        self.size -= 1;
        let chunk = &mut self.chunks[chunk];
        let item = core::mem::replace(&mut chunk[offset], MaybeUninit::uninit());
        Some(unsafe { item.assume_init() })
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.size)
            .map(|i| Self::indices(i))
            .map(|(chunk, offset)| unsafe { self.chunks[chunk][offset].assume_init_ref() })
    }
}
impl<T, const CHUNK_SIZE: usize> Default for StableVec<T, CHUNK_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T, const CHUNK_SIZE: usize> Len for StableVec<T, CHUNK_SIZE> {
    fn len(&self) -> usize {
        self.size
    }
}
impl<T, const CHUNK_SIZE: usize> Clear for StableVec<T, CHUNK_SIZE> {
    fn clear(&mut self) {
        self.chunks.clear();
    }
}

type StorePtr<T, const CHUNK_SIZE: usize> = Arc<UnsafeCell<StableVec<T, CHUNK_SIZE>>>;
#[derive(Debug)]
pub struct SafePtr<T: 'static> {
    ptr: NonNull<T>,
    _store: Arc<dyn core::any::Any>,
}
impl<T> Deref for SafePtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}
impl<T> DerefMut for SafePtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}
/// Dropping won't invalidate created [`SafePtr`]s
#[derive(Debug)]
pub struct SafeStableVec<T, const CHUNK_SIZE: usize> {
    vec: StorePtr<T, CHUNK_SIZE>,
}
impl<T, const CHUNK_SIZE: usize> SafeStableVec<T, CHUNK_SIZE> {
    pub fn new() -> Self {
        let vec = Arc::new(UnsafeCell::new(StableVec::<T, CHUNK_SIZE>::new()));
        Self { vec }
    }

    pub fn push(&mut self, value: T) -> SafePtr<T> {
        let vec = unsafe { self.vec.as_ref().get().as_mut() }.unwrap();
        let ptr = vec.push(value);
        let vec: Arc<dyn core::any::Any> = Arc::clone(&self.vec) as _;
        SafePtr { ptr, _store: vec }
    }
}
impl<T, const CHUNK_SIZE: usize> Default for SafeStableVec<T, CHUNK_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T, const CHUNK_SIZE: usize> Len for SafeStableVec<T, CHUNK_SIZE> {
    fn len(&self) -> usize {
        unsafe { self.vec.get().as_ref() }.unwrap().len()
    }
}
#[cfg(test)]
#[test]
fn test_safe_stable_vec() {
    let mut vec = SafeStableVec::<_, 2>::new();
    let p0 = vec.push(0);
    assert_eq!(*p0, 0);
    struct S {
        vec: SafeStableVec<usize, 2>,
        ptr: Vec<SafePtr<usize>>,
    }

    let mut s = S { vec, ptr: vec![p0] };
    let p1 = s.vec.push(1);
    assert_eq!(*s.ptr[0], 0);
    assert_eq!(*p1, 1);
}
