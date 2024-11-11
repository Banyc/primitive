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
}
impl<S, T> LinearSearch<T> for S where S: AsSlice<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice() {
        let mut v = vec![1, 2];
        assert_eq!(AsSlice::as_slice(&v)[0], 1);
        assert_eq!(AsSliceMut::as_slice_mut(&mut v)[0], 1);
    }
}
