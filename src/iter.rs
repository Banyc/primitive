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
