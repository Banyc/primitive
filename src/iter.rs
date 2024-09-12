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
pub struct VecZip<I> {
    iterators: Vec<I>,
}
impl<I> VecZip<I> {
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
