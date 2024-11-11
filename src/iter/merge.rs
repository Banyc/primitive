use super::lookahead::Lookahead1;

/// # Example
///
/// ```rust
/// use primitive::iter::{merge::VecZipLookahead1, lookahead::Lookahead1};
///
/// let iterators = [vec![1, 4, 6], vec![2, 3, 5]];
/// let iterators = iterators.map(|x| Lookahead1::new(x.into_iter()));
/// let mut iter = VecZipLookahead1::new(iterators.to_vec(), |x, y| x <= y);
/// assert_eq!(&iter.next().unwrap(), &[1, 2]);
/// assert_eq!(&iter.next().unwrap(), &[4, 2]);
/// assert_eq!(&iter.next().unwrap(), &[4, 3]);
/// assert_eq!(&iter.next().unwrap(), &[4, 5]);
/// assert_eq!(&iter.next().unwrap(), &[6, 5]);
/// assert!(&iter.next().is_none());
/// ```
#[derive(Debug, Clone)]
pub struct VecZipLookahead1<I, T, F> {
    iterators: Vec<Lookahead1<I, T>>,
    choose_left: F,
}
impl<I, T, F> VecZipLookahead1<I, T, F> {
    #[must_use]
    pub const fn new(iterators: Vec<Lookahead1<I, T>>, choose_left: F) -> Self {
        Self {
            iterators,
            choose_left,
        }
    }
}
impl<I: Iterator, F> Iterator for VecZipLookahead1<I, I::Item, F>
where
    F: FnMut(I::Item, I::Item) -> bool,
    I::Item: Copy,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iterators.iter().map(|x| x.peek()).any(|x| x.is_none()) {
            return None;
        }
        let out = self
            .iterators
            .iter()
            .map(|x| *x.peek().unwrap())
            .collect::<Vec<_>>();
        let i = choose_one(out.iter().copied(), &mut self.choose_left)?;
        self.iterators[i].pop().unwrap();
        Some(out)
    }
}

/// # Example
///
/// ```rust
/// use primitive::iter::{merge::VecLookahead1, lookahead::Lookahead1};
///
/// let iterators: [Vec<i32>; 2] = [vec![1, 4, 6], vec![2, 3, 5]];
/// let iterators = iterators.map(|x| Lookahead1::new(x.into_iter()));
/// let mut iter = VecLookahead1::new(iterators.to_vec(), |x: &i32, y: &i32| *x <= *y);
/// assert_eq!(iter.next().unwrap(), 1);
/// assert_eq!(iter.next().unwrap(), 2);
/// assert_eq!(iter.next().unwrap(), 3);
/// assert_eq!(iter.next().unwrap(), 4);
/// assert_eq!(iter.next().unwrap(), 5);
/// assert_eq!(iter.next().unwrap(), 6);
/// assert!(&iter.next().is_none());
/// ```
#[derive(Debug, Clone)]
pub struct VecLookahead1<I, T, F> {
    iterators: Vec<Lookahead1<I, T>>,
    choose_left: F,
}
impl<I, T, F> VecLookahead1<I, T, F> {
    #[must_use]
    pub const fn new(iterators: Vec<Lookahead1<I, T>>, choose_left: F) -> Self {
        Self {
            iterators,
            choose_left,
        }
    }
}
impl<I: Iterator, F> Iterator for VecLookahead1<I, I::Item, F>
where
    F: FnMut(&I::Item, &I::Item) -> bool,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let iter = self.iterators.iter().filter_map(|x| x.peek());
        let i = choose_one(iter, &mut self.choose_left)?;
        let x = self
            .iterators
            .iter_mut()
            .filter(|x| x.peek().is_some())
            .nth(i)
            .unwrap()
            .pop()
            .unwrap();
        Some(x)
    }
}

/// `choose_left`: choose first arg if true
fn choose_one<T: Copy>(
    iter: impl Iterator<Item = T>,
    mut choose_left: impl FnMut(T, T) -> bool,
) -> Option<usize> {
    let mut next = None;
    for (i, x) in iter.enumerate() {
        let replace = match next {
            Some((_, so_far)) => (choose_left)(x, so_far),
            None => true,
        };
        if replace {
            next = Some((i, x));
        }
    }
    let (i, _) = next?;
    Some(i)
}
