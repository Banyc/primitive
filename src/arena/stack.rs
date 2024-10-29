use core::mem::MaybeUninit;

use crate::{
    seq::{Seq, SeqMut},
    Capacity, Len, LenExt,
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
        if self.buf.len() == self.buf.capacity() {
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
impl<T> Seq<T> for DynCappedStack<T> {
    fn as_slice(&self) -> &[T] {
        &self.buf
    }
}
impl<T> SeqMut<T> for DynCappedStack<T> {
    fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.buf
    }
}

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
impl<T> Seq<T> for DynStack<T> {
    fn as_slice(&self) -> &[T] {
        match self {
            DynStack::Capped(dyn_capped_stack) => dyn_capped_stack.as_slice(),
            DynStack::Vec(vec) => vec,
        }
    }
}
impl<T> SeqMut<T> for DynStack<T> {
    fn as_slice_mut(&mut self) -> &mut [T] {
        match self {
            DynStack::Capped(dyn_capped_stack) => dyn_capped_stack.as_slice_mut(),
            DynStack::Vec(vec) => vec,
        }
    }
}

#[derive(Debug, Copy)]
pub struct StaticStack<T, const N: usize> {
    array: [MaybeUninit<T>; N],
    len: usize,
}
impl<T, const N: usize> StaticStack<T, N> {
    pub fn new() -> Self {
        Self {
            array: (0..N)
                .map(|_| MaybeUninit::uninit())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
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
        let last = if self.len() == self.capacity() {
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
    let mut s: StaticStack<usize, 5> = StaticStack::new();
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
        if self.len() == self.capacity() {
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
impl<T, const N: usize> Seq<T> for StaticStack<T, N> {
    fn as_slice(&self) -> &[T] {
        unsafe { core::mem::transmute(&self.array[..self.len]) }
    }
}
impl<T, const N: usize> SeqMut<T> for StaticStack<T, N> {
    fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { core::mem::transmute(&mut self.array[..self.len]) }
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
