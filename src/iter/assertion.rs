pub trait AssertIteratorItemExt {
    fn assert_item<T>(self) -> Self
    where
        Self: Iterator<Item = T> + Sized,
    {
        self
    }
}
impl<T> AssertIteratorItemExt for T {}
