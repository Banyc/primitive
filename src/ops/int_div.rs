//! `f64` format: 1-bit sign, 11-bit exponent, 52-bit fraction

use std::num::NonZeroI32;

use super::unsigned::{NonZeroU52, U52};

#[cfg(test)]
fn a() -> [U52; 9] {
    let max = u64::from(U52::MAX);
    [0, 1, 2, 3, 4, 5, max / 2, max - 1, max].map(|x| U52::new(x).unwrap())
}
#[cfg(test)]
fn b() -> [U52; 9] {
    let max = u64::from(U52::MAX);
    [1, 2, 3, 4, 5, 6, max / 2, max - 1, max].map(|x| U52::new(x).unwrap())
}

pub fn u52_div(a: U52, b: NonZeroU52) -> U52 {
    let a: u64 = a.into();
    let b: u64 = b.get().into();
    let quotient = a as f64 / b as f64;
    unsafe { U52::new_unchecked(quotient as u64) }
}
pub fn i32_div(a: i32, b: NonZeroI32) -> i32 {
    let quotient = a as f64 / b.get() as f64;
    quotient as i32
}
#[cfg(test)]
#[test]
fn test_int_div() {
    use crate::ops::unsigned::U52;

    let u52_max: u64 = U52::MAX.into();
    assert_eq!(u52_max, (u52_max as f64) as u64);

    let a = a();
    let b = b();
    for b in b {
        let b = NonZeroU52::new(b).unwrap();
        for a in a {
            assert_eq!(u64::from(a) / u64::from(b.get()), u64::from(u52_div(a, b)));
        }
    }
}
#[cfg(feature = "nightly")]
#[cfg(test)]
#[bench]
fn bench_int_div_int(bencher: &mut test::Bencher) {
    bench_int_div(bencher, |a, b| unsafe {
        U52::new_unchecked(u64::from(a) / u64::from(b.get()))
    });
}
#[cfg(feature = "nightly")]
#[cfg(test)]
#[bench]
fn bench_int_div_float(bencher: &mut test::Bencher) {
    bench_int_div(bencher, u52_div);
}
#[cfg(feature = "nightly")]
#[cfg(test)]
fn bench_int_div(bencher: &mut test::Bencher, mut f: impl FnMut(U52, NonZeroU52) -> U52) {
    use test::black_box;
    let a = a();
    let b = b();
    bencher.iter(|| {
        for b in b {
            let b = NonZeroU52::new(b).unwrap();
            for a in a {
                black_box(f(a, b));
            }
        }
    });
}

pub fn u52_modulo(a: U52, b: NonZeroU52) -> U52 {
    let quotient = u52_div(a, b);
    unsafe { U52::new_unchecked(u64::from(a) - u64::from(quotient) * u64::from(b.get())) }
}
#[cfg(feature = "nightly")]
#[cfg(test)]
#[test]
fn test_int_modulo() {
    let a = a();
    let b = b();
    for b in b {
        let b = NonZeroU52::new(b).unwrap();
        for a in a {
            assert_eq!(
                u64::from(a) % u64::from(b.get()),
                u64::from(u52_modulo(a, b))
            );
        }
    }
}
#[cfg(feature = "nightly")]
#[cfg(test)]
#[bench]
fn bench_int_modulo_int(bencher: &mut test::Bencher) {
    bench_int_div(bencher, |a, b| unsafe {
        U52::new_unchecked(u64::from(a) % u64::from(b.get()))
    });
}
#[cfg(feature = "nightly")]
#[cfg(test)]
#[bench]
fn bench_int_modulo_float(bencher: &mut test::Bencher) {
    bench_int_div(bencher, u52_modulo);
}
