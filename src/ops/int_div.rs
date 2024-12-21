//! `f64` format: 1-bit sign, 11-bit exponent, 52-bit fraction

use std::num::{NonZeroI32, NonZeroU32};

#[cfg(test)]
fn a() -> [u32; 9] {
    [0, 1, 2, 3, 4, 5, u32::MAX / 2, u32::MAX - 1, u32::MAX]
}
#[cfg(test)]
fn b() -> [u32; 9] {
    [1, 2, 3, 4, 5, 6, u32::MAX / 2, u32::MAX - 1, u32::MAX]
}

pub fn u32_div(a: u32, b: NonZeroU32) -> u32 {
    let quotient = a as f64 / b.get() as f64;
    quotient as u32
}
pub fn i32_div(a: i32, b: NonZeroI32) -> i32 {
    let quotient = a as f64 / b.get() as f64;
    quotient as i32
}
#[cfg(test)]
#[test]
fn test_int_div() {
    let a = a();
    let b = b();
    for b in b {
        let b = NonZeroU32::new(b).unwrap();
        for a in a {
            assert_eq!(a / b, u32_div(a, b));
        }
    }
}
#[cfg(feature = "nightly")]
#[cfg(test)]
#[bench]
fn bench_int_div_int(bencher: &mut test::Bencher) {
    bench_int_div(bencher, |a, b| a / b);
}
#[cfg(feature = "nightly")]
#[cfg(test)]
#[bench]
fn bench_int_div_float(bencher: &mut test::Bencher) {
    bench_int_div(bencher, u32_div);
}
#[cfg(feature = "nightly")]
#[cfg(test)]
fn bench_int_div(bencher: &mut test::Bencher, mut f: impl FnMut(u32, NonZeroU32) -> u32) {
    use test::black_box;
    let a = a();
    let b = b();
    bencher.iter(|| {
        for b in b {
            let b = NonZeroU32::new(b).unwrap();
            for a in a {
                black_box(f(a, b));
            }
        }
    });
}

pub fn u32_modulo(a: u32, b: NonZeroU32) -> u32 {
    let quotient = u32_div(a, b);
    a - quotient * b.get()
}
#[cfg(feature = "nightly")]
#[cfg(test)]
#[test]
fn test_int_modulo() {
    let a = a();
    let b = b();
    for b in b {
        let b = NonZeroU32::new(b).unwrap();
        for a in a {
            assert_eq!(a % b, u32_modulo(a, b));
        }
    }
}
#[cfg(feature = "nightly")]
#[cfg(test)]
#[bench]
fn bench_int_modulo_int(bencher: &mut test::Bencher) {
    bench_int_div(bencher, |a, b| a % b);
}
#[cfg(feature = "nightly")]
#[cfg(test)]
#[bench]
fn bench_int_modulo_float(bencher: &mut test::Bencher) {
    bench_int_div(bencher, u32_modulo);
}
