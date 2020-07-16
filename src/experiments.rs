use std::time::Instant;
use crate::{util::*, *};

//------------------
// Hash summary join
//------------------
pub fn community(n: Value, sample: f64, cross: f64) {
    // use rand::prelude::*;
    // create scale copies of input graph
    let es0 = read_edges(n as usize).unwrap();
    let mut fac = 0;
    for (x, y) in es0.iter() {
        fac = std::cmp::max(fac, std::cmp::max(*x, *y));
    }

    // NOTE breaks if scale too large due to overflow
    let scale = 100;
    let mut es = vec![];
    for i in 0..scale {
        for (x, y) in es0.iter() {
            if rand::random::<f64>() < sample {
                es.push((*x + i * fac, *y + i * fac));
                if rand::random::<f64>() < cross {
                    es.push((*x + i * fac, *y + i * fac + fac));
                }
            }
        }
    }

    let mut ts_h;
    let mut ts_s = 0;

    // Hash-based generic join
    {
        use hashed::*;

        let (r_x, rks) = build_hash(&es, |e| e);
        let (s_y, sks) = (r_x.clone(), rks.clone());
        let (t_x, tks) = build_hash(&es, |(z, x)| (x, z));

        println!("hash-join starting");
        let now = Instant::now();
        ts_h = triangle_index(r_x, rks, s_y, sks, t_x, tks, |result: &mut Value, _| {
            *result += 1
        });
        println!("generic: {}", now.elapsed().as_millis());
    }

    // summary generic join
    {
        use hashed::*;
        let mut hr = HashMap::default();

        for (x, y) in &es {
            let m = hr.entry((x % fac, y % fac)).or_insert_with(Vec::new);
            m.push((*x, *y));
        }

        let (r_x, rks) = build_hash(&es0, |e| e);
        let (s_y, sks) = (r_x.clone(), rks.clone());
        let (t_x, tks) = build_hash(&es0, |(z, x)| (x, z));

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

            let ts = triangle_index(r0_x, r0ks, s0_y, s0ks, t0_x, t0ks, |result: &mut Value, _| {
                *result += 1;
            });


            ts_s += ts;
        }
        println!("generic-summary: {}", now.elapsed().as_millis());
    }

    assert_eq!(ts_h, ts_s);

    // pair-wise summary join
    {
        use hashed::HashMap;

        let now = Instant::now();
        let mut hr = HashMap::default();
        for (x, y) in &es {
            let m = hr.entry((x % fac, y % fac)).or_insert_with(Vec::new);
            m.push((*x, *y));
        }

        // join on summary

        let mut r_y = HashMap::default();
        for (x, y) in &es0 {
            let xs = r_y.entry(y).or_insert_with(Vec::new);
            xs.push(*x);
        }
        let mut xyz = vec![];
        for (y, z) in &es0 {
            if let Some(xs) = r_y.get(y) {
                for x in xs {
                    xyz.push((*x,*y,*z));
                }
            }
        }
        let mut rs_xz = HashMap::default();
        // join with zx
        for (x, y, z) in xyz {
            let xys = rs_xz.entry((x, z)).or_insert_with(Vec::new);
            xys.push(y);
        }
        let mut inter = vec![];
        for (z, x) in &es0 {
            if let Some(xys) = rs_xz.get(&(*x, *z)) {
                for y in xys {
                    inter.push((*x,*y,*z));
                }
            }
        }

        ts_s = 0;

        for (a, b, c) in inter {
            let r_0 = &hr[&(a, b)];
            let s_0 = &hr[&(b, c)];
            let t_0 = &hr[&(c, a)];

            // join r0 s0 t0
            let mut r0_y = HashMap::default();
            for (x, y) in r_0 {
                let xs = r0_y.entry(y).or_insert_with(Vec::new);
                xs.push(*x);
            }
            let mut xyz0 = vec![];
            for (y, z) in s_0 {
                if let Some(xs) = r0_y.get(y) {
                    for x in xs {
                        xyz0.push((*x,*y,*z));
                    }
                }
            }
            let mut rs0_xz = HashMap::default();
        // join with zx
            for (x, y, z) in xyz0 {
                let xys = rs0_xz.entry((x, z)).or_insert_with(Vec::new);
                xys.push(y);
            }

            for (z, x) in t_0 {
                if let Some(xys) = rs0_xz.get(&(*x, *z)) {
                    for y in xys {
                        ts_s += 1;
                    }
                }
            }
        }
        println!("pairwise-summary: {}", now.elapsed().as_millis());

    }
    assert_eq!(ts_h, ts_s);

    // pair-wise hash join
    {
        use hashed::HashMap;
        let now = Instant::now();
        // join xy and yz
        let mut r_y = HashMap::default();
        for (x, y) in &es {
            let xs = r_y.entry(y).or_insert_with(Vec::new);
            xs.push(*x);
        }
        let mut xyz = vec![];
        for (y, z) in &es {
            if let Some(xs) = r_y.get(y) {
                for x in xs {
                    xyz.push((*x,*y,*z));
                }
            }
        }
        let mut rs_xz = HashMap::default();
        // join with zx
        for (x, y, z) in xyz {
            let xys = rs_xz.entry((x, z)).or_insert_with(Vec::new);
            xys.push(y);
        }
        ts_h = 0;
        for (z, x) in &es {
            if let Some(xys) = rs_xz.get(&(*x, *z)) {
                for y in xys {
                    ts_h += 1;
                }
            }
        }
        println!("pairwise: {}", now.elapsed().as_millis());
    }
    assert_eq!(ts_h, ts_s);
}

pub fn compressed(n: Value) {
    // create scale copies of input graph
    let es0 = read_edges(n as usize).unwrap();
    let mut fac = 0;
    for (x, y) in es0.iter() {
        fac = std::cmp::max(fac, std::cmp::max(*x, *y));
    }

    // NOTE breaks if scale too large due to overflow
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
        ts_h = triangle_index(r_x, rks, s_y, sks, t_x, tks, |result: &mut Value, _| {
            *result += 1
        });
        println!("hash-join: {}", now.elapsed().as_millis());
    }

    // compress
    {
        use hashed::*;
        let mut hr = HashMap::default();

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

            let ts = triangle_index(r0_x, r0ks, s0_y, s0ks, t0_x, t0ks, |result: &mut Value, _| {
                *result += 1;
            });


            ts_s += ts;
        }
        println!("compressed-join: {}", now.elapsed().as_millis());
    }

    assert_eq!(ts_h, ts_s);
    println!("{:?}", ts_h);
}

//-----------------------------------
// Triangle query, hashing vs sorting
//-----------------------------------
pub fn live_journal(n: Value) {
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
        ts_h = triangle_index(r_x, rks, s_y, sks, t_x, tks, |result: &mut Value, _| {
            *result += 1
        });
        println!("hash-join: {}", now.elapsed().as_millis());
    }

    // Sort-based generic join
    {
        use sorted::*;
        // sort-gj with tries
        es.sort_unstable_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
        let r_t = to_trie(&es);
        let s_t = r_t.clone();
        let mut t: Vec<_> = es.into_iter().map(|(x, y)| (y, x)).collect();
        t.sort_unstable_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
        let t_t = to_trie(&t);

        println!("sort-join starting");
        let now = Instant::now();
        ts_s = triangle_index_xyz(&r_t, &s_t, &t_t, |n: &mut Value, _| *n += 1);
        println!("sort-join: {}", now.elapsed().as_millis());
    }
    assert_eq!(ts_h, ts_s);
    println!("{:?}", ts_h);
}

pub fn worst_case(n: Value) {
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
        ts_h = triangle_index(r_x, rks, s_y, sks, t_x, tks, |result: &mut Value, _| {
            *result += 1
        });
        println!("hash-join: {}", now.elapsed().as_millis());
    }

    // Sort-based generic join
    {
        use crate::sorted::*;
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
        ts_s = triangle_index_xyz(&r_t, &s_t, &t_t, |n: &mut Value, _| *n += 1);
        println!("sort-join: {}", now.elapsed().as_millis());
    }
    assert_eq!(ts_h, ts_s);
    println!("{:?}", ts_h);
}

pub fn on_the_fly(n: Value) {

    let es = read_edges(n as usize).unwrap();
    let (ts_h, ts_s);

    // Hash-based generic join
    {
        use hashed::*;

        let t: Vec<_> = es.iter().copied().map(|(x, y)| (y, x)).collect();
        println!("hash-join starting");
        let now = Instant::now();
        ts_h = triangle(&es, &es, &t, |result: &mut Value, _| {
            *result += 1
        });
        println!("hash-join: {}", now.elapsed().as_millis());
    }

    // Sort-based generic join
    {
        use sorted::*;

        let t: Vec<_> = es.iter().copied().map(|(x, y)| (y, x)).collect();
        println!("sort-join starting");
        let now = Instant::now();
        ts_s = triangle(&es, &es, &t, |result: &mut Value, _| {
            *result += 1
        });
        println!("sort-join: {}", now.elapsed().as_millis());
    }
    assert_eq!(ts_h, ts_s);
    println!("{:?}", ts_h);
}

//----------------------------------
// Triangle query, variable ordering
//----------------------------------
pub fn live_journal_part(n: Value) {
    let edges = read_edges(n as usize).unwrap();
    let (mut r, mut s, mut t) = partition(&edges);
    let mut ts_s;

    // Sort-based generic join
    {
        use sorted::*;

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
        ts_s = triangle_index_xyz(&r_x, &s_y, &t_x, |n: &mut Value, _| *n += 1);
        println!("{}", ts_s);
        println!("sort-join: {}", now.elapsed().as_millis());

        println!("sort-join xzy");
        let now = Instant::now();
        ts_s = triangle_index_xzy(&r_x, &s_z, &t_x, |n: &mut Value, _| *n += 1);
        println!("{}", ts_s);
        println!("sort-join: {}", now.elapsed().as_millis());

        println!("sort-join yxz");
        let now = Instant::now();
        ts_s = triangle_index_yxz(&r_y, &s_y, &t_x, |n: &mut Value, _| *n += 1);
        println!("{}", ts_s);
        println!("sort-join: {}", now.elapsed().as_millis());

        println!("sort-join zxy");
        let now = Instant::now();
        ts_s = triangle_index_zxy(&r_x, &s_z, &t_z, |n: &mut Value, _| *n += 1);
        println!("{}", ts_s);
        println!("sort-join: {}", now.elapsed().as_millis());

        println!("sort-join zyx");
        let now = Instant::now();
        ts_s = triangle_index_zyx(&r_y, &s_z, &t_z, |n: &mut Value, _| *n += 1);
        println!("{}", ts_s);
        println!("sort-join: {}", now.elapsed().as_millis());
    }
}

//--------------------------
// IMDB join order benchmark
//--------------------------
pub fn job_main() -> Result<(), Box<dyn std::error::Error>>{

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
    let mut k: Vec<Value> = k.iter().filter_map(|sr| {
        if &sr[1] == "character-name-in-title" {
            sr[0].parse().ok()
        } else {
            None
        }
    }).collect();
    // cid
    let mut cn: Vec<Value> = cn.iter().filter_map(|sr| {
        if &sr[2] == "[de]" {
            sr[0].parse().ok()
        } else {
            None
        }
    }).collect();
    // mid, cid
    let mut mc: Vec<(Value, Value)> = mc.iter().map(|sr| (sr[1].parse().unwrap(), sr[2].parse().unwrap())).collect();
    // kid, mid
    let mut mk: Vec<(Value, Value)> = mk.iter().map(|sr| (sr[2].parse().unwrap(), sr[1].parse().unwrap())).collect();
    // mid
    let mut t: Vec<Value> = t.iter().map(|sr| sr[0].parse().unwrap()).collect();

    println!("{:?}", (
        cn.len(),
        k.len(),
        mc.len(),
        mk.len(),
        t.len(),
    ));

    {
        use sorted::*;
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


fn compare<'r, 's>((x_1, y_1): &'r (Value, Value), (x_2, y_2): &'s (Value, Value)) -> std::cmp::Ordering {
    x_1.cmp(x_2).then(y_1.cmp(y_2))
}
