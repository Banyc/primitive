pub trait Clear {
    fn clear(&mut self);
}
impl<T> Clear for Vec<T> {
    fn clear(&mut self) {
        self.clear();
    }
}
