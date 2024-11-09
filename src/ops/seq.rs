#[must_use]
pub fn dyn_vec_init<T>(size: usize, new_value: impl Fn() -> T) -> Vec<T> {
    (0..size).map(|_| new_value()).collect()
}
#[must_use]
pub fn dyn_array_init<T, const N: usize>(new_value: impl Fn() -> T) -> [T; N] {
    let res = dyn_vec_init(N, new_value).try_into();
    let Ok(array) = res else { unreachable!() };
    array
}

pub trait Seq<T> {
    #[must_use]
    fn as_slice(&self) -> &[T];
}
pub trait SeqMut<T>: Seq<T> {
    #[must_use]
    fn as_slice_mut(&mut self) -> &mut [T];
}

impl<T> Seq<T> for Vec<T> {
    fn as_slice(&self) -> &[T] {
        self
    }
}
impl<T> SeqMut<T> for Vec<T> {
    fn as_slice_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T, const N: usize> Seq<T> for [T; N] {
    fn as_slice(&self) -> &[T] {
        self
    }
}
impl<T, const N: usize> SeqMut<T> for [T; N] {
    fn as_slice_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T> Seq<T> for &[T] {
    fn as_slice(&self) -> &[T] {
        self
    }
}
impl<T> Seq<T> for &mut [T] {
    fn as_slice(&self) -> &[T] {
        self
    }
}
impl<T> SeqMut<T> for &mut [T] {
    fn as_slice_mut(&mut self) -> &mut [T] {
        self
    }
}

pub trait LinearSearch<T>: Seq<T> {
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
impl<S, T> LinearSearch<T> for S where S: Seq<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seq() {
        let mut v = vec![1, 2];
        assert_eq!(Seq::as_slice(&v)[0], 1);
        assert_eq!(SeqMut::as_slice_mut(&mut v)[0], 1);
    }
}
