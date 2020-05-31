use std::collections::{HashMap, HashSet};
use std::time::Instant;

use gj::{util::*, hashed::*, sorted::*};

fn main() {
    // let args: Vec<String> = std::env::args().collect();
    // let n = args[1].parse().unwrap();
    // let (mut r, mut s, mut t) = gen_worst_case_relations(n);

    let mut es = read_edges().unwrap();
    let ts_h;
    {
        let r = &es;
        let s = &es;
        let t = &es;

        // hash-gj without index
        // println!("hash-join starting");
        // let now = Instant::now();
        // let ts = triangle_hash_alt(&r, &s, &t);
        // let ts_h_len = ts.len();
        // println!("hash-join: {}", now.elapsed().as_millis());

        // hash-gj with index
        let mut r_x = HashMap::new();
        for (x, y) in r.iter().copied() {
            let ys = r_x.entry(x).or_insert_with(HashSet::new);
            ys.insert(y);
        }
        let rks: HashSet<u32> = r_x.keys().copied().collect();

        let mut t_x = HashMap::new();
        for (z, x) in t.iter().copied() {
            let zs = t_x.entry(x).or_insert_with(HashSet::new);
            zs.insert(z);
        }
        let tks: HashSet<u32> = t_x.keys().copied().collect();

        let mut s_y = HashMap::new();
        for (y, z) in s.iter().copied() {
            let zs = s_y.entry(y).or_insert_with(HashSet::new);
            zs.insert(z);
        }
        let sks: HashSet<u32> = s_y.keys().copied().collect();

        println!("hash-join starting");
        let now = Instant::now();
        ts_h = triangle_hash_index_cnt(
            r_x, rks,
            s_y, sks,
            t_x, tks,
        );
        // ts_h_len = ts.len();
        println!("hash-join: {}", now.elapsed().as_millis());
    }
    // sort-gj with tries
    es.sort_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
    let r_t = to_trie(&es);
    // s.sort_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
    let s_t = to_trie(&es);
    let mut t: Vec<_> = es.into_iter().map(|(x, y)| (y, x)).collect();
    t.sort_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
    let t_t = to_trie(&t);

    println!("sort-join starting");
    let now = Instant::now();
    let ts_s = triangle_sort_cnt(&r_t, &s_t, &t_t);
    // let ts_s_len = ts_s.len();
    println!("sort-join: {}", now.elapsed().as_millis());
    assert_eq!(ts_h, ts_s);
    println!("{:?}", ts_h);
}
