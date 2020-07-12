use std::time::Instant;
use gj::{util::*, *};

fn load_table(f: String) -> Vec<Vec<String>> {
    use std::fs;
    let data = fs::read_to_string(f)
                .expect("Something went wrong reading the file");

    use csv::ReaderBuilder;
    let mut rdr = ReaderBuilder::new()
        .escape(Some(b'\\'))
        .from_reader(data.as_bytes());
    let mut es = vec![];
    for rec in rdr.records() {
        let record = rec.unwrap();
        es.push(record.into_iter().map(|s| s.to_owned()).collect());
    }
    es
}

fn job_main() -> Result<(), Box<dyn std::error::Error>>{

    let args: Vec<String> = std::env::args().collect();
    let company_name = args[1].parse().unwrap();
    let keyword = args[2].parse().unwrap();
    let movie_companies = args[3].parse().unwrap();
    let movie_keyword = args[4].parse().unwrap();
    let title = args[5].parse().unwrap();

    let k = load_table(keyword);
    let cn = load_table(company_name);
    let mc = load_table(movie_companies);
    let mk = load_table(movie_keyword);
    let t = load_table(title);

    // kid
    let mut k: Vec<u32> = k.iter().filter_map(|sr| {
        if &sr[1] == "character-name-in-title" {
            sr[0].parse().ok()
        } else {
            None
        }
    }).collect();
    // cid
    let mut cn: Vec<u32> = cn.iter().filter_map(|sr| {
        if &sr[2] == "[de]" {
            sr[0].parse().ok()
        } else {
            None
        }
    }).collect();
    // mid, cid
    let mut mc: Vec<(u32, u32)> = mc.iter().map(|sr| (sr[1].parse().unwrap(), sr[2].parse().unwrap())).collect();
    // kid, mid
    let mut mk: Vec<(u32, u32)> = mk.iter().map(|sr| (sr[2].parse().unwrap(), sr[1].parse().unwrap())).collect();
    // mid
    let mut t: Vec<u32> = t.iter().map(|sr| sr[0].parse().unwrap()).collect();

    println!("{:?}", (
        cn.len(),
        k.len(),
        mc.len(),
        mk.len(),
        t.len(),
    ));

    {
        use gj::sorted::*;
        k.sort_unstable();
        cn.sort_unstable();
        mc.sort_unstable_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
        mc.dedup();
        let mc_m = to_trie(&mc);
        let mut cm: Vec<_> = mc.into_iter().map(|(m, c)| (c, m)).collect();
        cm.sort_unstable_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
        let mc_c = to_trie(&cm);
        mk.sort_unstable_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
        mk.dedup();
        let mk_k = to_trie(&mk);
        let mut km: Vec<_> = mk.into_iter().map(|(m, c)| (c, m)).collect();
        km.sort_unstable_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
        let mk_m = to_trie(&km);
        t.sort_unstable();

        println!("sort-join starting");
        let now = Instant::now();
        imdb_kmc(&k, &cn, &mc_m, &mk_k, &t);
        println!("kmc sort-join: {}", now.elapsed().as_millis());
        println!("sort-join starting");
        let now = Instant::now();
        imdb_kcm(&k, &cn, &mc_c, &mk_k, &t);
        println!("kcm sort-join: {}", now.elapsed().as_millis());
        println!("sort-join starting");
        let now = Instant::now();
        imdb_mkc(&k, &cn, &mc_m, &mk_m, &t);
        println!("mkc sort-join: {}", now.elapsed().as_millis());
        println!("sort-join starting");
        let now = Instant::now();
        imdb_mck(&k, &cn, &mc_m, &mk_m, &t);
        println!("mck sort-join: {}", now.elapsed().as_millis());
        println!("sort-join starting");
        let now = Instant::now();
        imdb_ckm(&k, &cn, &mc_c, &mk_k, &t);
        println!("ckm sort-join: {}", now.elapsed().as_millis());
        println!("sort-join starting");
        let now = Instant::now();
        imdb_cmk(&k, &cn, &mc_c, &mk_m, &t);
        println!("cmk sort-join: {}", now.elapsed().as_millis());
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n = args[1].parse().unwrap();

    // on_the_fly(n)
    // worst_case(n)
    // live_journal(n)
    compressed(n)
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

pub fn compressed(n: u32) {
    // create scale copies of input graph
    let es0 = read_edges(n as usize).unwrap();
    let mut fac = 0;
    for (x, y) in es0.iter() {
        fac = std::cmp::max(fac, std::cmp::max(*x, *y));
    }
    let scale = 100;
    let mut es = vec![];
    for i in 0..scale {
        for (x, y) in es0.iter() {
            es.push((*x + i * fac, *y + i * fac));
        }
    }

    let ts_h;
    let mut ts_s = 0;

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

    // compress
    {
        use hashed::*;
        use std::collections::HashMap;
        // change this to a vector and address into it
        let mut hr = HashMap::new();
        for (x, y) in es {
            let m = hr.entry((x % fac, y % fac)).or_insert_with(Vec::new);
            m.push((x, y));
        }

        let (r_x, rks) = build_hash(&es0, |e| e);
        let (s_y, sks) = (r_x.clone(), rks.clone());
        let (t_x, tks) = build_hash(&es0, |(z, x)| (x, z));

        println!("compressed-join starting");
        let now = Instant::now();
        let ts = triangle_index(r_x, rks, s_y, sks, t_x, tks, |result: &mut Vec<_>, t| {
            result.push(t);
        });

        for (a, b, c) in ts {
            let r_0 = &hr[&(a, b)];
            let s_0 = &hr[&(b, c)];
            let t_0 = &hr[&(c, a)];

            let (r0_x, r0ks) = build_hash(r_0, |e| e);
            let (s0_y, s0ks) = build_hash(s_0, |e| e);
            let (t0_x, t0ks) = build_hash(t_0, |(z, x)| (x, z));

            let ts = triangle_index(r0_x, r0ks, s0_y, s0ks, t0_x, t0ks, |result: &mut u32, _| {
                *result += 1;
            });

            ts_s += ts;
        }
        println!("compressed-join: {}", now.elapsed().as_millis());
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
