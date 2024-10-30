#[derive(Debug, Clone)]
pub struct EmptyBorrowVec<T: 'static> {
    empty: Option<Vec<&'static T>>,
}
impl<T: 'static> EmptyBorrowVec<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            empty: Some(vec![]),
        }
    }

    #[must_use]
    pub fn take<'t>(self) -> BorrowVec<'t, T> {
        BorrowVec {
            vec: self.empty.unwrap(),
        }
    }

    #[must_use]
    pub fn get_mut<'t>(&mut self) -> BorrowVecGuard<'_, 't, T> {
        let vec = Some(self.empty.take().unwrap());
        BorrowVecGuard { parent: self, vec }
    }
}
impl<T: 'static> Default for EmptyBorrowVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct BorrowVecGuard<'guard, 't, T: 'static> {
    parent: &'guard mut EmptyBorrowVec<T>,
    vec: Option<Vec<&'t T>>,
}
impl<'t, T> BorrowVecGuard<'_, 't, T> {
    #[must_use]
    pub fn get(&self) -> &Vec<&T> {
        self.vec.as_ref().unwrap()
    }
    #[must_use]
    pub fn get_mut(&mut self) -> &mut Vec<&'t T> {
        self.vec.as_mut().unwrap()
    }
}
impl<T> Drop for BorrowVecGuard<'_, '_, T> {
    fn drop(&mut self) {
        let vec = self.vec.take().unwrap();
        let empty = Some(erase_vec_lifetime(vec));
        self.parent.empty = empty;
    }
}

#[derive(Debug, Clone)]
pub struct BorrowVec<'t, T> {
    vec: Vec<&'t T>,
}
impl<'t, T> BorrowVec<'t, T> {
    /// Erase the lifetime on `T` and reuse the inner [`Vec`] in the future.
    #[must_use]
    pub fn clear(self) -> EmptyBorrowVec<T> {
        let empty = Some(erase_vec_lifetime(self.vec));
        EmptyBorrowVec { empty }
    }

    #[must_use]
    pub fn get(&self) -> &Vec<&T> {
        &self.vec
    }
    #[must_use]
    pub fn get_mut(&mut self) -> &mut Vec<&'t T> {
        &mut self.vec
    }
}

/// Ref: <https://users.rust-lang.org/t/cast-empty-vec-a-t-to-vec-static-t/66687/17>
#[must_use]
pub fn erase_vec_lifetime<T>(mut v: Vec<&T>) -> Vec<&'static T> {
    v.clear();
    v.into_iter()
        .map(|_| -> &'static T { unreachable!() })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct S<'a> {
        s: &'a str,
    }

    #[test]
    fn test() {
        let mut v = EmptyBorrowVec::new().take();
        let s = S { s: "hello" };
        v.get_mut().push(&s);
        v.get_mut().push(&s);
        let s = S { s: "world" };
        v.get_mut().push(&s);
        assert_eq!(
            v.get().iter().map(|s| s.s).collect::<Vec<&str>>(),
            ["hello", "hello", "world"]
        );
        let v = v.clear();
        let v = v.take();
        assert!(v.get().is_empty());
    }
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    use core::hint::black_box;

    use super::*;

    #[bench]
    fn bench_vec(b: &mut test::Bencher) {
        b.iter(|| {
            for i in 0..1024 {
                let n = i;
                let v = vec![&n];
                black_box(v);
            }
        });
    }

    #[bench]
    fn bench_reuse_vec(b: &mut test::Bencher) {
        b.iter(|| {
            let mut v = EmptyBorrowVec::new();
            for i in 0..1024 {
                let mut v = v.get_mut();
                let n = i;
                v.get_mut().push(&n);
                black_box(v);
            }
        });
    }
}
