use core::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

use crate::ops::{
    len::{Capacity, Full, Len, LenExt},
    list::{List, ListMut},
    slice::{AsSlice, AsSliceMut},
};

pub trait Stack<T> {
    /// Return [`Some`] if the stack is in full capacity
    fn push(&mut self, obj: T) -> Option<T>;
    fn pop(&mut self) -> Option<T>;
}

#[derive(Debug, Clone)]
pub struct DynCappedStack<T> {
    buf: Vec<T>,
}
impl<T> DynCappedStack<T> {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }
    pub fn push(&mut self, obj: T) -> Option<T> {
        if self.is_full() {
            return Some(obj);
        }
        self.buf.push(obj);
        None
    }
    pub fn pop(&mut self) -> Option<T> {
        self.buf.pop()
    }
}
impl<T> Len for DynCappedStack<T> {
    fn len(&self) -> usize {
        self.buf.len()
    }
}
impl<T> Capacity for DynCappedStack<T> {
    fn capacity(&self) -> usize {
        self.buf.capacity()
    }
}
impl<T> AsSlice<T> for DynCappedStack<T> {
    fn as_slice(&self) -> &[T] {
        &self.buf
    }
}
impl<T> AsSliceMut<T> for DynCappedStack<T> {
    fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.buf
    }
}
impl<T> Index<usize> for DynCappedStack<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}
impl<T> IndexMut<usize> for DynCappedStack<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_slice_mut()[index]
    }
}
impl<T> List<T> for DynCappedStack<T> {}
impl<T> ListMut<T> for DynCappedStack<T> {}

#[derive(Debug, Clone)]
pub enum DynStack<T> {
    Capped(DynCappedStack<T>),
    Vec(Vec<T>),
}
impl<T> DynStack<T> {
    pub fn new(capacity: Option<usize>) -> Self {
        match capacity {
            Some(capacity) => Self::Capped(DynCappedStack::new(capacity)),
            None => Self::Vec(vec![]),
        }
    }
}
impl<T> Stack<T> for DynStack<T> {
    fn push(&mut self, obj: T) -> Option<T> {
        match self {
            DynStack::Capped(capped_stack) => capped_stack.push(obj),
            DynStack::Vec(vec) => {
                vec.push(obj);
                None
            }
        }
    }
    fn pop(&mut self) -> Option<T> {
        match self {
            DynStack::Capped(capped_stack) => capped_stack.pop(),
            DynStack::Vec(vec) => vec.pop(),
        }
    }
}
impl<T> AsSlice<T> for DynStack<T> {
    fn as_slice(&self) -> &[T] {
        match self {
            DynStack::Capped(dyn_capped_stack) => dyn_capped_stack.as_slice(),
            DynStack::Vec(vec) => vec,
        }
    }
}
impl<T> AsSliceMut<T> for DynStack<T> {
    fn as_slice_mut(&mut self) -> &mut [T] {
        match self {
            DynStack::Capped(dyn_capped_stack) => dyn_capped_stack.as_slice_mut(),
            DynStack::Vec(vec) => vec,
        }
    }
}
impl<T> Index<usize> for DynStack<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}
impl<T> IndexMut<usize> for DynStack<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_slice_mut()[index]
    }
}
impl<T> Len for DynStack<T> {
    fn len(&self) -> usize {
        match self {
            DynStack::Capped(dyn_capped_stack) => dyn_capped_stack.len(),
            DynStack::Vec(vec) => vec.len(),
        }
    }
}
impl<T> List<T> for DynStack<T> {}
impl<T> ListMut<T> for DynStack<T> {}

#[derive(Debug, Copy)]
pub struct StaticStack<T, const N: usize> {
    array: [MaybeUninit<T>; N],
    len: usize,
}
impl<T, const N: usize> StaticStack<T, N> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            array: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }
    pub fn swap_remove(&mut self, index: usize) -> T {
        assert!(index < self.len());
        let removed = core::mem::replace(&mut self.array[index], MaybeUninit::uninit());
        let last = core::mem::replace(&mut self.array[self.len - 1], MaybeUninit::uninit());
        self.array[index] = last;
        self.len -= 1;
        unsafe { removed.assume_init() }
    }
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len());
        let removed = core::mem::replace(&mut self.array[index], MaybeUninit::uninit());
        for i in index..self.len - 1 {
            let next = core::mem::replace(&mut self.array[i + 1], MaybeUninit::uninit());
            self.array[i] = next;
        }
        self.len -= 1;
        unsafe { removed.assume_init() }
    }
    pub fn insert(&mut self, index: usize, value: T) -> Option<T> {
        assert!(index <= self.len());
        assert!(index < self.capacity());
        let last = if self.is_full() {
            let last = core::mem::replace(&mut self.array[self.len - 1], MaybeUninit::uninit());
            self.len -= 1;
            Some(unsafe { last.assume_init() })
        } else {
            None
        };
        for i in (index + 1..=self.len).rev() {
            let prev = core::mem::replace(&mut self.array[i - 1], MaybeUninit::uninit());
            self.array[i] = prev;
        }
        self.array[index] = MaybeUninit::new(value);
        self.len += 1;
        last
    }
}
#[cfg(test)]
#[test]
fn test_static_stack() {
    let mut s: StaticStack<usize, 0> = StaticStack::new();
    assert_eq!(s.push(2).unwrap(), 2);

    let mut s: StaticStack<usize, 5> = StaticStack::new();
    assert_eq!(s.as_slice(), []);
    s.push(3);
    assert_eq!(s.as_slice(), [3]);
    s.push(4);
    assert_eq!(s.as_slice(), [3, 4]);
    s.insert(0, 1);
    assert_eq!(s.as_slice(), [1, 3, 4]);
    s.insert(1, 2);
    assert_eq!(s.as_slice(), [1, 2, 3, 4]);
    s.insert(4, 6);
    assert_eq!(s.as_slice(), [1, 2, 3, 4, 6]);
    s.insert(4, 5);
    assert_eq!(s.as_slice(), [1, 2, 3, 4, 5]);
    s.remove(0);
    assert_eq!(s.as_slice(), [2, 3, 4, 5]);
    s.swap_remove(0);
    assert_eq!(s.as_slice(), [5, 3, 4]);
}
impl<T, const N: usize> Stack<T> for StaticStack<T, N> {
    fn push(&mut self, obj: T) -> Option<T> {
        if self.is_full() {
            return Some(obj);
        }
        self.array[self.len] = MaybeUninit::new(obj);
        self.len += 1;
        None
    }
    fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let top = core::mem::replace(&mut self.array[self.len - 1], MaybeUninit::uninit());
        self.len -= 1;
        Some(unsafe { top.assume_init() })
    }
}
impl<T, const N: usize> Len for StaticStack<T, N> {
    fn len(&self) -> usize {
        self.len
    }
}
impl<T, const N: usize> Capacity for StaticStack<T, N> {
    fn capacity(&self) -> usize {
        N
    }
}
impl<T, const N: usize> AsSlice<T> for StaticStack<T, N> {
    fn as_slice(&self) -> &[T] {
        unsafe { core::mem::transmute(&self.array[..self.len]) }
    }
}
impl<T, const N: usize> AsSliceMut<T> for StaticStack<T, N> {
    fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { core::mem::transmute(&mut self.array[..self.len]) }
    }
}
impl<T, const N: usize> Index<usize> for StaticStack<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}
impl<T, const N: usize> IndexMut<usize> for StaticStack<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_slice_mut()[index]
    }
}
impl<T, const N: usize> Default for StaticStack<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: Clone, const N: usize> Clone for StaticStack<T, N> {
    fn clone(&self) -> Self {
        let mut new = Self::new();
        for item in self.as_slice() {
            new.push(item.clone());
        }
        new
    }
}
impl<T, const N: usize> List<T> for StaticStack<T, N> {}
impl<T, const N: usize> ListMut<T> for StaticStack<T, N> {}

#[derive(Debug, Copy)]
pub struct StaticRevStack<T, const N: usize> {
    len: usize,
    array: [MaybeUninit<T>; N],
}
impl<T, const N: usize> StaticRevStack<T, N> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            array: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }
    fn start(&self) -> usize {
        N - self.len
    }
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len());
        let removed =
            core::mem::replace(&mut self.array[index + self.start()], MaybeUninit::uninit());
        for i in (1..=index).rev() {
            let i = i + self.start();
            let prev = core::mem::replace(&mut self.array[i - 1], MaybeUninit::uninit());
            self.array[i] = prev;
        }
        self.len -= 1;
        unsafe { removed.assume_init() }
    }
    pub fn insert(&mut self, index: usize, value: T) -> Option<T> {
        assert!(index <= self.len());
        assert!(index < self.capacity());
        if self.is_full() {
            let last = core::mem::replace(&mut self.array[self.len - 1], MaybeUninit::uninit());
            for i in (index + 1..self.len).rev() {
                let i = i + self.start();
                let prev = core::mem::replace(&mut self.array[i - 1], MaybeUninit::uninit());
                self.array[i] = prev;
            }
            self.array[index + self.start()] = MaybeUninit::new(value);
            Some(unsafe { last.assume_init() })
        } else {
            if !self.is_empty() {
                for i in 0..index {
                    let i = i + self.start();
                    let curr = core::mem::replace(&mut self.array[i], MaybeUninit::uninit());
                    self.array[i - 1] = curr;
                }
            }
            self.array[index + self.start() - 1] = MaybeUninit::new(value);
            self.len += 1;
            None
        }
    }
}
#[cfg(test)]
#[test]
fn test_static_rev_stack() {
    let mut s: StaticRevStack<usize, 0> = StaticRevStack::new();
    assert_eq!(s.push(2).unwrap(), 2);

    let mut s: StaticRevStack<usize, 5> = StaticRevStack::new();
    assert_eq!(s.as_slice(), []);
    s.insert(0, 3);
    assert_eq!(s.as_slice(), [3]);
    s.insert(1, 4);
    assert_eq!(s.as_slice(), [3, 4]);
    s.insert(0, 1);
    assert_eq!(s.as_slice(), [1, 3, 4]);
    s.insert(1, 2);
    assert_eq!(s.as_slice(), [1, 2, 3, 4]);
    s.insert(4, 6);
    assert_eq!(s.as_slice(), [1, 2, 3, 4, 6]);
    s.insert(4, 5);
    assert_eq!(s.as_slice(), [1, 2, 3, 4, 5]);
    assert_eq!(s.clone().as_slice(), [1, 2, 3, 4, 5]);
    s.insert(0, 0);
    assert_eq!(s.as_slice(), [0, 1, 2, 3, 4]);
    s.remove(0);
    assert_eq!(s.as_slice(), [1, 2, 3, 4]);
    s.remove(1);
    assert_eq!(s.as_slice(), [1, 3, 4]);
}
impl<T, const N: usize> Stack<T> for StaticRevStack<T, N> {
    fn push(&mut self, obj: T) -> Option<T> {
        if self.is_full() {
            return Some(obj);
        }
        self.insert(self.len(), obj);
        None
    }
    fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        Some(self.remove(self.len()))
    }
}
impl<T, const N: usize> Len for StaticRevStack<T, N> {
    fn len(&self) -> usize {
        self.len
    }
}
impl<T, const N: usize> Capacity for StaticRevStack<T, N> {
    fn capacity(&self) -> usize {
        N
    }
}
impl<T, const N: usize> AsSlice<T> for StaticRevStack<T, N> {
    fn as_slice(&self) -> &[T] {
        unsafe { core::mem::transmute(&self.array[self.start()..]) }
    }
}
impl<T, const N: usize> AsSliceMut<T> for StaticRevStack<T, N> {
    fn as_slice_mut(&mut self) -> &mut [T] {
        let start = self.start();
        unsafe { core::mem::transmute(&mut self.array[start..]) }
    }
}
impl<T, const N: usize> Index<usize> for StaticRevStack<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}
impl<T, const N: usize> IndexMut<usize> for StaticRevStack<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_slice_mut()[index]
    }
}
impl<T, const N: usize> Default for StaticRevStack<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: Clone, const N: usize> Clone for StaticRevStack<T, N> {
    fn clone(&self) -> Self {
        let mut new = Self::new();
        for item in self.as_slice().iter().rev() {
            new.insert(0, item.clone());
        }
        new
    }
}
impl<T, const N: usize> List<T> for StaticRevStack<T, N> {}
impl<T, const N: usize> ListMut<T> for StaticRevStack<T, N> {}
