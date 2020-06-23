#![allow(unused_variables)]
use criterion::*;
use gj::util::*;

criterion_group!(benches, bench_indexed);
criterion_main!(benches);

pub fn bench_indexed(c: &mut Criterion) {
    let mut group = c.benchmark_group("triangle-indexed");

    static K: u32 = 1000;

    for i in [100*K, 300*K, 500*K].iter() {

        let (mut r, mut s, t) = gen_worst_case_relations(*i);

        {
            use gj::hashed::*;

            let (r_x, rks) = build_hash(&r, |e| e);
            let (s_y, sks) = build_hash(&s, |e| e);
            let (t_x, tks) = build_hash(&t, |(z, x)| (x, z));

            group
                .sample_size(10)
            // .measurement_time(std::time::Duration::from_secs(60))
                .bench_function(
                    BenchmarkId::new("hashed", i),
                    |b| b.iter_batched(
                        || (r_x.clone(), rks.clone(), s_y.clone(), sks.clone(), t_x.clone(), tks.clone()),
                        |(r_x, rks, s_y, sks, t_x, tks)| {
                            triangle_index(r_x, rks, s_y, sks, t_x, tks,
                                           |result: &mut u32, _| {
                                               *result += 1
                                           })
                        },
                        BatchSize::SmallInput
                    ));
        }

        {
            use gj::sorted::*;
            // sort-gj with tries
            r.sort_unstable_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
            let r_t = to_trie(&r);
            s.sort_unstable_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
            let s_t = to_trie(&s);
            let mut t: Vec<_> = t.into_iter().map(|(x, y)| (y, x)).collect();
            t.sort_unstable_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
            let t_t = to_trie(&t);

            group
                .sample_size(10)
            // .measurement_time(std::time::Duration::from_secs(60))
                .bench_function(
                    BenchmarkId::new("sorted", i),
                    |b| b.iter(|| {
                        triangle_index_xyz(&r_t, &s_t, &t_t, |n: &mut u32, _| *n += 1)
                    }));
        }
    }

    group.finish()
}
