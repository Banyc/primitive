#[must_use]
pub fn dyn_vec_init<T>(size: usize, new_value: impl Fn() -> T) -> Vec<T> {
    (0..size).map(|_| new_value()).collect()
}

pub trait AsSlice<T> {
    #[must_use]
    fn as_slice(&self) -> &[T];
}
pub trait AsSliceMut<T>: AsSlice<T> {
    #[must_use]
    fn as_slice_mut(&mut self) -> &mut [T];
}

impl<T> AsSlice<T> for Vec<T> {
    fn as_slice(&self) -> &[T] {
        self
    }
}
impl<T> AsSliceMut<T> for Vec<T> {
    fn as_slice_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T, const N: usize> AsSlice<T> for [T; N] {
    fn as_slice(&self) -> &[T] {
        self
    }
}
impl<T, const N: usize> AsSliceMut<T> for [T; N] {
    fn as_slice_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T> AsSlice<T> for &[T] {
    fn as_slice(&self) -> &[T] {
        self
    }
}
impl<T> AsSlice<T> for &mut [T] {
    fn as_slice(&self) -> &[T] {
        self
    }
}
impl<T> AsSliceMut<T> for &mut [T] {
    fn as_slice_mut(&mut self) -> &mut [T] {
        self
    }
}

pub trait LinearSearch<T>: AsSlice<T> {
    /// If the slice is not sorted or if the comparator function does not
    /// implement an order consistent with the sort order of the underlying
    /// slice, the returned result is unspecified and meaningless.
    ///
    /// If the value is found then [`Result::Ok`] is returned, containing the
    /// index of the matching element. If there are multiple matches, then any
    /// one of the matches could be returned.
    ///
    /// If the value is not found then [`Result::Err`] is returned, containing
    /// the index where a matching element could be inserted while maintaining
    /// sorted order.
    fn linear_search_by(
        &self,
        mut cmp: impl FnMut(&T) -> core::cmp::Ordering,
    ) -> Result<usize, usize> {
        for (i, value) in self.as_slice().iter().enumerate() {
            match cmp(value) {
                core::cmp::Ordering::Less => {
                    continue;
                }
                core::cmp::Ordering::Equal => {
                    return Ok(i);
                }
                core::cmp::Ordering::Greater => {
                    return Err(i);
                }
            }
        }
        Err(self.as_slice().len())
    }

    /// ref: <https://curiouscoding.nl/posts/static-search-tree/#auto-vectorization>
    fn linear_search_branchless_by(&self, mut cmp: impl FnMut(&T) -> core::cmp::Ordering) -> usize {
        let mut count = 0;
        for value in self.as_slice().iter() {
            match cmp(value) {
                core::cmp::Ordering::Less => {
                    count += 1;
                }
                core::cmp::Ordering::Equal | core::cmp::Ordering::Greater => (),
            }
        }
        count
    }
}
impl<S, T> LinearSearch<T> for S where S: AsSlice<T> {}

pub trait LossyCpy: AsSliceMut<u8> {
    fn lossy_copy_from_slice(&mut self, src: &[u8]) -> usize {
        let this = self.as_slice_mut();
        let len = this.len().min(src.len());
        this[..len].copy_from_slice(&src[..len]);
        len
    }
}
impl<T> LossyCpy for T where T: AsSliceMut<u8> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice() {
        let mut v = vec![1, 2];
        assert_eq!(AsSlice::as_slice(&v)[0], 1);
        assert_eq!(AsSliceMut::as_slice_mut(&mut v)[0], 1);
    }

    #[test]
    fn test_linear_search() {
        let v: Vec<u32> = vec![1, 2, 4];
        {
            let mut cmp = |x: &u32| x.cmp(&3);
            assert_eq!(
                v.linear_search_by(&mut cmp).unwrap_err(),
                v.linear_search_branchless_by(&mut cmp)
            );
        }
        {
            let mut cmp = |x: &u32| x.cmp(&2);
            assert_eq!(
                v.linear_search_by(&mut cmp).unwrap(),
                v.linear_search_branchless_by(&mut cmp)
            );
        }
    }

    #[test]
    fn test_lossy_cpy() {
        let mut v: Vec<u8> = vec![1, 2];
        let len = v.lossy_copy_from_slice(&[3]);
        assert_eq!(len, 1);
        assert_eq!(v, &[3, 2]);
    }
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    use std::hint::black_box;

    use test::Bencher;

    use super::*;

    #[bench]
    fn bench_linear_search(b: &mut Bencher) {
        let v = (0..16).collect::<Vec<u32>>();
        b.iter(|| {
            for i in 0..16 {
                let a = v.linear_search_by(|x| x.cmp(&i));
                let _ = black_box(a);
            }
        });
    }
    #[bench]
    fn bench_linear_search_branchless(b: &mut Bencher) {
        let v = (0..16).collect::<Vec<u32>>();
        b.iter(|| {
            for i in 0..16 {
                let a = v.linear_search_branchless_by(|x| x.cmp(&i));
                let _ = black_box(a);
            }
        });
    }
}
