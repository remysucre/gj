use std::time::Instant;

use gj::{util::*, *};

fn main() {
    // let args: Vec<String> = std::env::args().collect();
    // let n = args[1].parse().unwrap();
    // let (mut r, mut s, mut t) = gen_worst_case_relations(n);
    //
    live_journal()
}

fn live_journal() {
    let mut es = read_edges().unwrap();
    let (ts_h, ts_s);
    {
        use hashed::*;

        let (r_x, rks) = build_hash(&es, |e| e);
        let (s_y, sks) = (r_x.clone(), rks.clone());
        let (t_x, tks) = build_hash(&es, |(z, x)| (x, z));

        println!("hash-join starting");
        let now = Instant::now();
        ts_h = triangle_index(r_x, rks, s_y, sks, t_x, tks, |result: &mut u32, _| {
            *result += 1
        });
        println!("hash-join: {}", now.elapsed().as_millis());
    }
    {
        use gj::sorted::*;
        // sort-gj with tries
        es.sort_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
        let r_t = to_trie(&es);
        let s_t = r_t.clone();
        let mut t: Vec<_> = es.into_iter().map(|(x, y)| (y, x)).collect();
        t.sort_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
        let t_t = to_trie(&t);

        println!("sort-join starting");
        let now = Instant::now();
        ts_s = triangle_sort(&r_t, &s_t, &t_t, |n: &mut u32, _| *n += 1);
        println!("sort-join: {}", now.elapsed().as_millis());
    }
    assert_eq!(ts_h, ts_s);
    println!("{:?}", ts_h);
}
