/// - `ACCUMULATORS`: product of operation latency and operation throughput
/// - `op`: an associative and commutative operation
pub fn acc<T, const ACCUMULATORS: usize>(
    len: usize,
    mut init: impl FnMut() -> T,
    mut get: impl FnMut(usize) -> T,
    mut op: impl FnMut(&mut T, T),
) -> T {
    let mut par_acc: [T; ACCUMULATORS] = core::array::from_fn(|_| init());
    for start in (0..len).step_by(ACCUMULATORS) {
        for (offset, acc) in par_acc.iter_mut().enumerate() {
            let i = start + offset;
            if i < len {
                op(acc, get(i));
            }
        }
    }
    let mut acc = init();
    for v in par_acc {
        op(&mut acc, v);
    }
    acc
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    use super::*;

    use test::{self, black_box};

    const N: usize = 1 << 12;
    type T = f64;
    const ZERO: T = 0.;
    const ACCUMULATORS: usize = 4;

    #[bench]
    fn bench_acc(bencher: &mut test::Bencher) {
        let arr = arr();
        let arr = black_box(&arr[..]);
        bencher.iter(|| acc::<T, ACCUMULATORS>(arr.len(), || ZERO, |i| arr[i], |a, b| *a += b));
    }
    #[bench]
    fn bench_sum(bencher: &mut test::Bencher) {
        let arr = arr();
        let arr = black_box(&arr[..]);
        bencher.iter(|| arr.iter().sum::<T>());
    }

    fn arr() -> [T; N] {
        core::array::from_fn(|i| i as _)
    }
}
