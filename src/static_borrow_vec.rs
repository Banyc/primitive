pub struct EmptyBorrowVec<T: 'static> {
    empty: Option<Vec<&'static T>>,
}
impl<T: 'static> EmptyBorrowVec<T> {
    pub fn new() -> Self {
        Self {
            empty: Some(vec![]),
        }
    }

    pub fn take<'t>(self) -> BorrowVec<'t, T> {
        BorrowVec {
            vec: self.empty.unwrap(),
        }
    }

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

pub struct BorrowVecGuard<'guard, 't, T: 'static> {
    parent: &'guard mut EmptyBorrowVec<T>,
    vec: Option<Vec<&'t T>>,
}
impl<'guard, 't, T> BorrowVecGuard<'guard, 't, T> {
    pub fn get(&self) -> &Vec<&T> {
        self.vec.as_ref().unwrap()
    }

    pub fn get_mut(&mut self) -> &mut Vec<&'t T> {
        self.vec.as_mut().unwrap()
    }
}
impl<'guard, 't, T> Drop for BorrowVecGuard<'guard, 't, T> {
    fn drop(&mut self) {
        let vec = self.vec.take().unwrap();
        let empty = Some(empty(vec));
        self.parent.empty = empty;
    }
}

pub struct BorrowVec<'t, T> {
    vec: Vec<&'t T>,
}
impl<'t, T> BorrowVec<'t, T> {
    /// Erase the lifetime on `T` and reuse the inner [`Vec`] in the future.
    pub fn clear(self) -> EmptyBorrowVec<T> {
        let empty = Some(empty(self.vec));
        EmptyBorrowVec { empty }
    }

    pub fn get(&self) -> &Vec<&T> {
        &self.vec
    }

    pub fn get_mut(&mut self) -> &mut Vec<&'t T> {
        &mut self.vec
    }
}

/// Ref: <https://users.rust-lang.org/t/cast-empty-vec-a-t-to-vec-static-t/66687/17>
fn empty<T>(mut v: Vec<&T>) -> Vec<&'static T> {
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
    use std::hint::black_box;

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
