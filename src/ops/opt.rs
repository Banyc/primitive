pub trait Opt<T>: From<Option<T>> + Into<Option<T>> {
    type GetOut;
    fn none() -> Self;
    fn some(v: T) -> Self;
    fn get(&self) -> Option<Self::GetOut>;
    fn take(&mut self) -> Option<T>;
    fn map<U>(mut self, f: impl FnOnce(T) -> U) -> Option<U> {
        let o = self.take();
        o.map(f)
    }
}
