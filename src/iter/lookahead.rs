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
    pub const fn peek(&self) -> Option<&T> {
        self.next.as_ref()
    }
    pub fn pop(&mut self) -> Option<T> {
        let next = self.iter.next();
        core::mem::replace(&mut self.next, next)
    }
}

#[derive(Debug)]
pub struct Lookahead1Mut<'a, I, T> {
    iter: I,
    next: Option<&'a mut T>,
}
impl<'a, I, T> Lookahead1Mut<'a, I, T>
where
    I: Iterator<Item = &'a mut T>,
{
    #[must_use]
    pub fn new(mut iter: I) -> Self {
        let next = iter.next();
        Self { iter, next }
    }

    #[must_use]
    pub fn peek(&mut self) -> Option<&mut T> {
        self.next.as_deref_mut()
    }
    pub fn pop(&mut self) -> Option<&'a mut T> {
        let next = self.iter.next();
        core::mem::replace(&mut self.next, next)
    }
}
#[cfg(test)]
#[test]
fn test_lookahead1_mut() {
    let mut vec = vec![1, 2, 3];
    let mut iter = Lookahead1Mut::new(vec.iter_mut());
    loop {
        let Some(int) = iter.peek() else {
            break;
        };
        *int = 0;
        iter.pop();
    }
    assert_eq!(vec, [0, 0, 0]);
}
