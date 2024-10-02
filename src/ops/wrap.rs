pub trait Map<U> {
    type Wrap<V>;
    fn map<V>(self, f: impl FnOnce(U) -> V) -> Self::Wrap<V>;
}
pub trait TransposeOption {
    type Inner;
    type Wrap<T>;
    fn transpose_option(self) -> Option<Self::Wrap<Self::Inner>>;
}
pub trait TransposeResult {
    type Inner;
    type Error;
    type Wrap<T>;
    fn transpose_result(self) -> Result<Self::Wrap<Self::Inner>, Self::Error>;
}
