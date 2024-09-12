pub trait Seq<T> {
    fn as_slice(&self) -> &[T];
    fn as_slice_mut(&mut self) -> &mut [T];
}

impl<T> Seq<T> for Vec<T> {
    fn as_slice(&self) -> &[T] {
        self
    }
    fn as_slice_mut(&mut self) -> &mut [T] {
        self
    }
}
impl<T, const N: usize> Seq<T> for [T; N] {
    fn as_slice(&self) -> &[T] {
        self
    }
    fn as_slice_mut(&mut self) -> &mut [T] {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seq() {
        let mut v = vec![1, 2];
        assert_eq!(Seq::as_slice(&v)[0], 1);
        assert_eq!(Seq::as_slice_mut(&mut v)[0], 1);
    }
}
