use core::mem::MaybeUninit;

use crate::seq::{dyn_array_init, dyn_vec_init};

/// # Example
///
/// ```rust
/// use primitive::iter::VecZip;
///
/// let data = vec![
///     vec![1, 2],
///     vec![3, 4],
/// ];
/// let data = data.into_iter().map(|column| column.into_iter()).collect::<Vec<_>>();
/// let zip = VecZip::new(data);
/// let data = zip.collect::<Vec<Vec<usize>>>();
/// assert_eq!(data, vec![
///     vec![1, 3],
///     vec![2, 4],
/// ]);
/// ```
#[derive(Debug, Clone)]
pub struct VecZip<I> {
    iterators: Vec<I>,
}
impl<I> VecZip<I> {
    #[must_use]
    pub fn new(iterators: Vec<I>) -> Self {
        Self { iterators }
    }
}
impl<I: Iterator> Iterator for VecZip<I> {
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterators.iter_mut().map(Iterator::next).collect()
    }
}

pub trait AssertIteratorItemExt {
    fn assert_item<T>(self) -> Self
    where
        Self: Iterator<Item = T> + Sized,
    {
        self
    }
}
impl<T> AssertIteratorItemExt for T {}

pub trait Chunks: Iterator + Sized {
    fn static_chunks<T, const CHUNK_SIZE: usize>(self, for_each: impl FnMut(&[T]))
    where
        Self: Iterator<Item = T>,
    {
        let mut tray = dyn_array_init::<_, CHUNK_SIZE>(|| MaybeUninit::uninit());
        self.chunks(&mut tray, for_each);
    }
    fn dyn_chunks<T, const N: usize>(self, chunk_size: usize, for_each: impl FnMut(&[T]))
    where
        Self: Iterator<Item = T>,
    {
        let mut tray = dyn_vec_init(chunk_size, || MaybeUninit::uninit());
        self.chunks(&mut tray, for_each);
    }
    fn chunks<T>(mut self, tray: &mut [MaybeUninit<T>], mut for_each: impl FnMut(&[T]))
    where
        Self: Iterator<Item = T>,
    {
        let mut i = 0;
        loop {
            let v = self.next();
            let is_end = v.is_none();
            if let Some(v) = v {
                tray[i] = MaybeUninit::new(v);
                i += 1;
                if i < tray.len() {
                    continue;
                }
            }
            if i != 0 {
                let tray = &tray[..i];
                let tray = unsafe { core::mem::transmute::<&[MaybeUninit<T>], &[T]>(tray) };
                for_each(tray);
                i = 0;
            }
            if is_end {
                break;
            }
        }
    }
}
impl<T> Chunks for T where T: Iterator {}

#[derive(Debug, Clone)]
pub struct Lookahead1<I, T> {
    iter: I,
    next: Option<T>,
}
impl<I, T> Lookahead1<I, T>
where
    I: Iterator<Item = T>,
{
    #[must_use]
    pub fn new(mut iter: I) -> Self {
        let next = iter.next();
        Self { iter, next }
    }

    #[must_use]
    pub fn peek(&self) -> Option<&T> {
        self.next.as_ref()
    }
    pub fn pop(&mut self) -> Option<T> {
        let next = self.iter.next();
        core::mem::replace(&mut self.next, next)
    }
}

#[derive(Debug)]
pub struct Lookahead1Mut<'a, I, T> {
    iter: I,
    next: Option<&'a mut T>,
}
impl<'a, I, T> Lookahead1Mut<'a, I, T>
where
    I: Iterator<Item = &'a mut T>,
{
    #[must_use]
    pub fn new(mut iter: I) -> Self {
        let next = iter.next();
        Self { iter, next }
    }

    #[must_use]
    pub fn peek(&mut self) -> Option<&mut T> {
        self.next.as_deref_mut()
    }
    pub fn pop(&mut self) -> Option<&'a mut T> {
        let next = self.iter.next();
        core::mem::replace(&mut self.next, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookahead1_mut() {
        let mut vec = vec![1, 2, 3];
        let mut iter = Lookahead1Mut::new(vec.iter_mut());
        loop {
            let Some(int) = iter.peek() else {
                break;
            };
            *int = 0;
            iter.pop();
        }
        assert_eq!(vec, [0, 0, 0]);
    }

    #[test]
    fn test_chunks() {
        {
            let mut buf = vec![];
            let a: [usize; 3] = [0, 1, 2];
            a.iter()
                .static_chunks::<_, 2>(|tray| buf.push(tray.iter().copied().sum::<usize>()));
            assert_eq!(&buf, &[1, 2]);
        }
        {
            let mut buf = vec![];
            let a: [usize; 4] = [0, 1, 2, 3];
            a.iter()
                .static_chunks::<_, 2>(|tray| buf.push(tray.iter().copied().sum::<usize>()));
            assert_eq!(&buf, &[1, 5]);
        }
    }
}
