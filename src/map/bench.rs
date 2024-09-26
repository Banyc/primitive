#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    use std::{collections::HashMap, hint::black_box};

    use indexmap::IndexMap;

    use crate::{
        bench::HeapRandomizer,
        map::{dense_hash_map::DenseHashMap, grow_dense_map::GrowDenseMap},
        Clear,
    };

    const N: usize = 2 << 16;
    const VALUE_SIZE: usize = 2 << 5;
    const GROW_DENSE_MAP_CHUNK_SIZE: usize = 2 << 5;

    struct Value {
        #[allow(dead_code)]
        buf: [u8; VALUE_SIZE],
    }
    impl Value {
        pub fn new() -> Self {
            Self {
                buf: [0; VALUE_SIZE],
            }
        }
    }

    macro_rules! get {
        ($m: ident, $bencher: ident) => {
            let mut heap = HeapRandomizer::new();
            for i in 0..N {
                heap.randomize();
                $m.insert(i, Value::new());
            }
            $bencher.iter(|| {
                let mut reverse = false;
                for i in 0..N {
                    let i = if reverse { N - 1 - i } else { i };
                    reverse = !reverse;
                    black_box($m.get(&i));
                }
            });
        };
    }
    #[bench]
    fn bench_get_std(bencher: &mut test::Bencher) {
        let mut m = HashMap::new();
        get!(m, bencher);
    }
    #[bench]
    fn bench_get_dense(bencher: &mut test::Bencher) {
        let mut m = DenseHashMap::new();
        get!(m, bencher);
    }
    #[bench]
    fn bench_get_index_map(bencher: &mut test::Bencher) {
        let mut m = IndexMap::new();
        get!(m, bencher);
    }
    #[bench]
    fn bench_get_grow(bencher: &mut test::Bencher) {
        let mut m = GrowDenseMap::<_, _, GROW_DENSE_MAP_CHUNK_SIZE>::new();
        get!(m, bencher);
    }

    macro_rules! iter {
        ($m: ident, $bencher: ident) => {
            for i in 0..N {
                $m.insert(i, Value::new());
            }
            $bencher.iter(|| {
                for (k, v) in $m.iter() {
                    black_box((k, v));
                }
            });
        };
    }
    #[bench]
    fn bench_iter_std(bencher: &mut test::Bencher) {
        let mut m = HashMap::new();
        iter!(m, bencher);
    }
    #[bench]
    fn bench_iter_dense(bencher: &mut test::Bencher) {
        let mut m = DenseHashMap::new();
        iter!(m, bencher);
    }
    #[bench]
    fn bench_iter_index_map(bencher: &mut test::Bencher) {
        let mut m = IndexMap::new();
        iter!(m, bencher);
    }

    macro_rules! insert_remove {
        ($m: ident, $bencher: ident) => {
            $bencher.iter(|| {
                for i in 0..N {
                    $m.insert(i, Value::new());
                }
                let mut reverse = false;
                for i in 0..N {
                    let i = if reverse { N - 1 - i } else { i };
                    reverse = !reverse;
                    #[allow(deprecated)]
                    $m.remove(&i);
                }
            });
        };
    }
    #[bench]
    fn bench_insert_remove_std(bencher: &mut test::Bencher) {
        let mut m = HashMap::new();
        insert_remove!(m, bencher);
    }
    #[bench]
    fn bench_insert_remove_dense(bencher: &mut test::Bencher) {
        let mut m = DenseHashMap::new();
        insert_remove!(m, bencher);
    }
    #[bench]
    fn bench_insert_remove_index_map(bencher: &mut test::Bencher) {
        let mut m = IndexMap::new();
        insert_remove!(m, bencher);
    }

    macro_rules! insert_iter_remove {
        ($m: ident, $bencher: ident) => {
            let n = (N as f64).sqrt().round() as usize;
            $bencher.iter(|| {
                for i in 0..n {
                    $m.insert(i, Value::new());
                }
                let mut reverse = false;
                for i in 0..n {
                    for v in $m.iter() {
                        black_box(v);
                    }
                    let i = if reverse { n - 1 - i } else { i };
                    reverse = !reverse;
                    #[allow(deprecated)]
                    $m.remove(&i);
                }
            });
        };
    }
    #[bench]
    fn bench_insert_iter_remove_std(bencher: &mut test::Bencher) {
        let mut m = HashMap::new();
        insert_iter_remove!(m, bencher);
    }
    #[bench]
    fn bench_insert_iter_remove_dense(bencher: &mut test::Bencher) {
        let mut m = DenseHashMap::new();
        insert_iter_remove!(m, bencher);
    }
    #[bench]
    fn bench_insert_iter_remove_index_map(bencher: &mut test::Bencher) {
        let mut m = IndexMap::new();
        insert_iter_remove!(m, bencher);
    }

    macro_rules! insert_clear {
        ($m: ident, $bencher: ident) => {
            $bencher.iter(|| {
                for i in 0..N {
                    $m.insert(i, Value::new());
                }
                $m.clear();
            });
        };
    }
    #[bench]
    fn bench_insert_clear_std(bencher: &mut test::Bencher) {
        let mut m = HashMap::new();
        insert_clear!(m, bencher);
    }
    #[bench]
    fn bench_insert_clear_dense(bencher: &mut test::Bencher) {
        let mut m = DenseHashMap::new();
        insert_clear!(m, bencher);
    }
    #[bench]
    fn bench_insert_clear_index_map(bencher: &mut test::Bencher) {
        let mut m = IndexMap::new();
        insert_clear!(m, bencher);
    }
    #[bench]
    fn bench_insert_clear_grow(bencher: &mut test::Bencher) {
        let mut m = GrowDenseMap::<_, _, GROW_DENSE_MAP_CHUNK_SIZE>::new();
        insert_clear!(m, bencher);
    }

    macro_rules! values {
        ($m: ident, $bencher: ident) => {
            for i in 0..N {
                $m.insert(i, Value::new());
            }
            $bencher.iter(|| {
                for v in $m.values() {
                    black_box(v);
                }
            });
        };
    }
    #[bench]
    fn bench_values_std(bencher: &mut test::Bencher) {
        let mut m = HashMap::new();
        values!(m, bencher);
    }
    #[bench]
    fn bench_values_dense(bencher: &mut test::Bencher) {
        let mut m = DenseHashMap::new();
        values!(m, bencher);
    }
    #[bench]
    fn bench_values_index_map(bencher: &mut test::Bencher) {
        let mut m = IndexMap::new();
        values!(m, bencher);
    }
}
