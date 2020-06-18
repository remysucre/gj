use std::time::Instant;
use gj::{util::*, *};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n = args[1].parse().unwrap();

    on_the_fly(n)
    // worst_case(n)
    // live_journal(n)
    // live_journal_part(n)
}

pub fn on_the_fly(n: u32) {

    let es = read_edges(n as usize).unwrap();
    let (ts_h, ts_s);

    // Hash-based generic join
    {
        use hashed::*;

        let t: Vec<_> = es.iter().copied().map(|(x, y)| (y, x)).collect();
        println!("hash-join starting");
        let now = Instant::now();
        ts_h = triangle(&es, &es, &t, |result: &mut u32, _| {
            *result += 1
        });
        println!("hash-join: {}", now.elapsed().as_millis());
    }

    // Sort-based generic join
    {
        use gj::sorted::*;

        let t: Vec<_> = es.iter().copied().map(|(x, y)| (y, x)).collect();
        println!("sort-join starting");
        let now = Instant::now();
        ts_s = triangle(&es, &es, &t, |result: &mut u32, _| {
            *result += 1
        });
        println!("sort-join: {}", now.elapsed().as_millis());
    }
    assert_eq!(ts_h, ts_s);
    println!("{:?}", ts_h);
}

pub fn worst_case(n: u32) {
    let (mut r, mut s, t) = gen_worst_case_relations(n);
    let (ts_h, ts_s);

    // Hash-based generic join
    {
        use hashed::*;

        let (r_x, rks) = build_hash(&r, |e| e);
        let (s_y, sks) = build_hash(&s, |e| e);
        let (t_x, tks) = build_hash(&t, |(z, x)| (x, z));

        println!("hash-join starting");
        let now = Instant::now();
        ts_h = triangle_index(r_x, rks, s_y, sks, t_x, tks, |result: &mut u32, _| {
            *result += 1
        });
        println!("hash-join: {}", now.elapsed().as_millis());
    }

    // Sort-based generic join
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

        println!("sort-join starting");
        let now = Instant::now();
        ts_s = triangle_index_xyz(&r_t, &s_t, &t_t, |n: &mut u32, _| *n += 1);
        println!("sort-join: {}", now.elapsed().as_millis());
    }
    assert_eq!(ts_h, ts_s);
    println!("{:?}", ts_h);
}

pub fn live_journal(n: u32) {
    let mut es = read_edges(n as usize).unwrap();
    let (ts_h, ts_s);

    // Hash-based generic join
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

    // Sort-based generic join
    {
        use gj::sorted::*;
        // sort-gj with tries
        es.sort_unstable_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
        let r_t = to_trie(&es);
        let s_t = r_t.clone();
        let mut t: Vec<_> = es.into_iter().map(|(x, y)| (y, x)).collect();
        t.sort_unstable_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
        let t_t = to_trie(&t);

        println!("sort-join starting");
        let now = Instant::now();
        ts_s = triangle_index_xyz(&r_t, &s_t, &t_t, |n: &mut u32, _| *n += 1);
        println!("sort-join: {}", now.elapsed().as_millis());
    }
    assert_eq!(ts_h, ts_s);
    println!("{:?}", ts_h);
}

fn compare<'r, 's>((x_1, y_1): &'r (u32, u32), (x_2, y_2): &'s (u32, u32)) -> std::cmp::Ordering {
    x_1.cmp(x_2).then(y_1.cmp(y_2))
}

pub fn live_journal_part(n: u32) {
    let edges = read_edges(n as usize).unwrap();
    let (mut r, mut s, mut t) = partition(&edges);
    let mut ts_s;

    // Hash-based generic join
    // {
    //     use hashed::*;

    //     let (r_x, rks) = build_hash(&r, |e| e);
    //     let (s_y, sks) = build_hash(&s, |e| e);
    //     let (t_x, tks) = build_hash(&t, |(z, x)| (x, z));

    //     println!("hash-join starting");
    //     let now = Instant::now();
    //     ts_h = triangle_index(r_x, rks, s_y, sks, t_x, tks, |n: &mut u32, _| {
    //         *n += 1
    //     });
    //     println!("hash-join: {}", now.elapsed().as_millis());
    // }

    // Sort-based generic join
    {
        use gj::sorted::*;

        // sort-gj with tries
        r.sort_unstable_by(compare);
        s.sort_unstable_by(compare);
        t.sort_unstable_by(compare);
        let r_x = to_trie(&r);
        let s_y = to_trie(&s);
        let t_z = to_trie(&t);

        let mut rr: Vec<_> = r.into_iter().map(|(x, y)| (y, x)).collect();
        let mut sr: Vec<_> = s.into_iter().map(|(x, y)| (y, x)).collect();
        let mut tr: Vec<_> = t.into_iter().map(|(x, y)| (y, x)).collect();
        rr.sort_unstable_by(compare);
        sr.sort_unstable_by(compare);
        tr.sort_unstable_by(compare);
        let r_y = to_trie(&rr);
        let s_z = to_trie(&sr);
        let t_x = to_trie(&tr);

        println!("sort-join xyz");
        let now = Instant::now();
        ts_s = triangle_index_xyz(&r_x, &s_y, &t_x, |n: &mut u32, _| *n += 1);
        println!("{}", ts_s);
        println!("sort-join: {}", now.elapsed().as_millis());

        println!("sort-join xzy");
        let now = Instant::now();
        ts_s = triangle_index_xzy(&r_x, &s_z, &t_x, |n: &mut u32, _| *n += 1);
        println!("{}", ts_s);
        println!("sort-join: {}", now.elapsed().as_millis());

        println!("sort-join yxz");
        let now = Instant::now();
        ts_s = triangle_index_yxz(&r_y, &s_y, &t_x, |n: &mut u32, _| *n += 1);
        println!("{}", ts_s);
        println!("sort-join: {}", now.elapsed().as_millis());

        println!("sort-join zxy");
        let now = Instant::now();
        ts_s = triangle_index_zxy(&r_x, &s_z, &t_z, |n: &mut u32, _| *n += 1);
        println!("{}", ts_s);
        println!("sort-join: {}", now.elapsed().as_millis());

        println!("sort-join zyx");
        let now = Instant::now();
        ts_s = triangle_index_zyx(&r_y, &s_z, &t_z, |n: &mut u32, _| *n += 1);
        println!("{}", ts_s);
        println!("sort-join: {}", now.elapsed().as_millis());
    }
    // assert_eq!(ts_h, ts_s);
}
