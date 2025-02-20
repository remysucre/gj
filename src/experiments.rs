use std::time::Instant;
use crate::{util::*, *};
use trie::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;

//------------------
// Hash summary join
//------------------
pub fn community(n: usize, sample: f64, cross: f64) {
    // use rand::prelude::*;
    // create scale copies of input graph
    let es0 = read_es(n).unwrap();
    let es0: Vec<_> = es0.into_iter().map(|xy| if let (Val::Int(x), Val::Int(y)) = (&xy[0], &xy[1]) { vec![*x,*y] } else {unreachable!()}).collect();
    let mut fac = 0;
    for xy in es0.iter() {
        fac = std::cmp::max(fac, std::cmp::max(xy[0], xy[1]));
    }
    let g0: Vec<_> = es0.iter().map(|xy| vec![Val::Int(xy[0]), Val::Int(xy[1])]).collect();
    let g0_r: Vec<_> = g0.iter().map(|xy| vec![xy[1].clone(), xy[0].clone()]).collect();

    let rx0 = Trie::from_iter(g0.iter().map(|v| v.as_slice()));
    let sy0 = Trie::from_iter(g0.iter().map(|v| v.as_slice()));
    let tx0 = Trie::from_iter(g0_r.iter().map(|v| v.as_slice()));

    // NOTE breaks if scale too large due to overflow
    let scale = 100;
    let mut g = vec![];
    for i in 0..scale {
        for xy in es0.iter() {
            if rand::random::<f64>() < sample {
                g.push(
                    vec![Val::Int(xy[0] + i * fac),
                         Val::Int(xy[1] + i * fac + fac)]
                );
                if rand::random::<f64>() < cross {
                    g.push(
                        vec![Val::Int(xy[0] + i * fac),
                             Val::Int(xy[1] + i * fac + fac)]
                    );
                }
            }
        }
    }
    let g_r: Vec<_> = g.iter().map(|xy| vec![xy[1].clone(), xy[0].clone()]).collect();

    let rx = Trie::from_iter(g.iter().map(|v| v.as_slice()));
    let sy = Trie::from_iter(g.iter().map(|v| v.as_slice()));
    let tx = Trie::from_iter(g_r.iter().map(|v| v.as_slice()));

    let ts_h;
    let mut ts_s = 0;

    // Hash-based generic join
    {

        println!("hash-join starting");
        let now = Instant::now();
        ts_h = triangle(&rx, &sy, &tx, |result: &mut u64, _| {
            *result += 1
        });
        println!("generic: {}", now.elapsed().as_millis());
    }

    // summary generic join
    {
        let mut hr = HashMap::default();

        for xy in g.iter() {
            if let (Val::Int(x), Val::Int(y)) = (&xy[0], &xy[1]) {
                let m = hr.entry((x / fac, y / fac)).or_insert_with(Trie::new);
                m.add(&vec![Val::Int(*x), Val::Int(*y)])
            } else {
                unreachable!()
            }
        }

        let mut hr_r = HashMap::default();

        for xy in g.iter() {
            if let (Val::Int(x), Val::Int(y)) = (&xy[0], &xy[1]) {
                let m = hr_r.entry((x / fac, y / fac)).or_insert_with(Trie::new);
                m.add(&vec![Val::Int(*y), Val::Int(*x)])
            } else {
                unreachable!()
            }
        }
        // //let (r_x, rks) = build_hash(&es0, |e| e);
        // let r_x = Trie::from_iter(es0.iter());
        // let (s_y, sks) = (r_x.clone(), rks.clone());
        // let (t_x, tks) = build_hash(&es0, |(z, x)| (x, z));

        let now = Instant::now();
        let ts = triangle(&rx0, &sy0, &tx0, |result: &mut Vec<_>, (a, b, c)| {
            if let (Val::Int(a), Val::Int(b), Val::Int(c)) = (a, b, c) {
                result.push((*a, *b, *c));
            } else {
                unreachable!()
            }
        });

        for (a, b, c) in ts {
            if let (Some(r_0), Some(s_0), Some(t_0)) = (hr.get(&(a,b)), hr.get(&(b,c)), hr_r.get(&(c, a))) {

                let ts = triangle(r_0, s_0, t_0, |result: &mut u64, _| {
                    *result += 1;
                });

                ts_s += ts;
            }
        }
        println!("generic-summary: {}", now.elapsed().as_millis());
    }

    assert_eq!(ts_h, ts_s);
}

//----------
// Summaries
//----------
pub fn lj_sum(n: u64) {
    let es = read_es(n as usize).unwrap();

    let args: Vec<String> = env::args().collect();
    let f = &args[1];
    let mut file = File::open(f).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let parts: Vec<Vec<u64>> = contents.lines().map(|l| l.split_whitespace().map(|n| n.parse().unwrap()).collect()).collect();
    // homomorphism maps each vertex to its cluster
    println!("reading homs");
    let mut hom: HashMap<Val, Val> = HashMap::default();
    // for part in parts {
    //   hom.insert(Val::Int(part[0]), Val::Int(part[1]));
    // }
    for part in parts {
        let c = part[0];
        for v in &part[1..] {
            hom.insert(Val::Int(*v), Val::Int(c));
        }
    }

    // compute summary graph hom(es)
    let g_sum: HashSet<_> = es.iter().map(|xy| vec![hom[&xy[0]].clone(), hom[&xy[1]].clone()]).collect();
    let g_sum_r: HashSet<_> = g_sum.iter().map(|v| vec![v[1].clone(), v[0].clone()]).collect();
    // compute reverse mapping from summary to (tries of) edges
    let mut moh: HashMap<(&Val, &Val), Trie> = HashMap::default();
    for xy in es.iter() {
        let x = &xy[0];
        let y = &xy[1];
        let xy_ = (&hom[x], &hom[y]);
        let srcs = moh.entry(xy_).or_insert_with(Trie::new);
        srcs.add(xy);
    }
    let mut moh_r: HashMap<(&Val, &Val), Trie> = HashMap::default();
    for xy in es.iter() {
        let x = &xy[0];
        let y = &xy[1];
        let xy_ = (&hom[x], &hom[y]);
        let yx = vec![y.clone(), x.clone()];
        let srcs = moh_r.entry(xy_).or_insert_with(Trie::new);
        srcs.add(&yx);
    }

    let now = Instant::now();
    println!("building tries");
    let rx_ = Trie::from_iter(g_sum.iter().map(|v| v.as_slice()));
    let sy_ = Trie::from_iter(g_sum.iter().map(|v| v.as_slice()));
    let tx_ = Trie::from_iter(g_sum_r.iter().map(|v| v.as_slice()));
    println!("done in {}", now.elapsed().as_millis());
    println!("{:?}", (rx_.len(), sy_.len(), tx_.len()));

    println!("generic join starting");
    let now = Instant::now();
    let result = triangle(&rx_, &sy_, &tx_, |result: &mut u64, (a, b, c)| {
        // perform local join
        let (r, s, t);
        if b == c {
          r = &moh[&(a, b)];
          s = &moh[&(b, c)];
          t = &moh_r[&(c, a)];
        } else if a == c {
          // b, a, c
          r = &moh[&(b, c)];
          s = &moh[&(c, a)];
          t = &moh_r[&(a, b)];
        } else {
          // b, a, c
          r = &moh[&(c, a)];
          s = &moh[&(a, b)];
          t = &moh_r[&(b, c)];
        }

        *result += triangle(r, s, t, |r: &mut u64, _| *r += 1 );
    });
    println!("{}", result);
    println!("done in {}", now.elapsed().as_millis());
}

//---------------
// Triangle query
//---------------
pub fn live_journal(n: u64) {
    let es = read_es(n as usize).unwrap();
    let esr: Vec<_> = es.iter().map(|v| vec![v[1].clone(), v[0].clone()]).collect();

    let rx = Trie::from_iter(es.iter().map(|v| v.as_slice()));
    let sy = Trie::from_iter(es.iter().map(|v| v.as_slice()));
    let tx = Trie::from_iter(esr.iter().map(|v| v.as_slice()));

    println!("generic join starting");
    let now = Instant::now();
    let result = triangle(&rx, &sy, &tx, |result: &mut u64, _| {
        *result += 1
    });
    println!("{}", result);
    println!("done in {}", now.elapsed().as_millis());
}

pub fn triangle<R, F>(rx: &Trie, sy: &Trie, tx: &Trie, agg: F) -> R
where R: Default, F: Fn(&mut R, (&Val, &Val, &Val))
{
    let mut result = R::default();
    for (a, ra_ta) in Trie::inter_min(&vec![rx, tx]) {
        let ra = ra_ta[0];
        let ta = ra_ta[1];
        for (b, rab_sb) in Trie::inter_min(&vec![ra, sy]) {
            let sb = rab_sb[1];
            for (c, _sbc_tac) in Trie::inter_min(&vec![sb, ta]) {
                agg(&mut result, (a, b, c))
            }
        }
    }
    result
}

//-----------------
// Building a query
//-----------------

// SELECT MIN(mc.note) AS production_note,
//        MIN(t.title) AS movie_title,
//        MIN(t.production_year) AS movie_year
// FROM company_type AS ct,
//      info_type AS it,
//      movie_companies AS mc,
//      movie_info_idx AS mi_idx,
//      title AS t
// WHERE ct.kind = 'production companies'
//   AND it.info = 'top 250 rank'
//   AND mc.note NOT LIKE '%(as Metro-Goldwyn-Mayer Pictures)%'
//   AND (mc.note LIKE '%(co-production)%'
//        OR mc.note LIKE '%(presents)%')
//
//   AND ct.id = mc.company_type_id
//   AND t.id = mc.movie_id
//   AND t.id = mi_idx.movie_id
//   AND mc.movie_id = mi_idx.movie_id
//   AND it.id = mi_idx.info_type_id;
//
// JOINED ATTRIBUTES
// ct.id mc.company_type_id
// t.id mc.movie_id mi_idx.movie_id
// it.id mi_idx.info_type_id

// may also want output aggs to determine variable order
// FROM clause determine arguments
pub fn query<R, F>(
    ct: &Trie, it: &Trie, mc: &Trie, mi_idx: &Trie, t: &Trie,
    agg: F
) -> R
// output aggregates determine # of args to F
where R: Default, F: Fn(&mut R, &[&Trie])
{
    let mut result = R::default();
    for (_a, ct_mc_x) in Trie::inter_min(&vec![ct, mc]) {
        let _ct_a = ct_mc_x[0];
        let mc_a = ct_mc_x[0];
        for (_b, t_mc_a_mi_idx_yt) in Trie::inter_min(&vec![t, mc_a, mi_idx]) {
            let t_b = t_mc_a_mi_idx_yt[0];
            let mc_ab = t_mc_a_mi_idx_yt[1];
            let mi_idx_b = t_mc_a_mi_idx_yt[2];
            for (_c, _it_mi_idx_z) in Trie::inter_min(&vec![it, mi_idx_b]) {
                // output aggregates, hopefully it doesn't need the join attrs
                agg(&mut result, &vec![mc_ab, t_b])
            }
        }
    }
    result
}

// 1. Decide on variable order
// 2. Build tries on each relation according to variable order
// 3. Generate nested loops implementing generic join

// let mut result = R::default();
// for (a, a_rels) in Trie::inter_min(&vec![r, s, t]) {
//   let ra = a_rels[0];
//   let sa = a_rels[1];
//   let ra = a_rels[2];
//   for (b, b_rels) in Trie::inter_min(&vec![u, v, w]) {
//     let ub = b_rels[0];
//     let vb = b_rels[1];
//     let wb = b_rels[2];
//     for (c, c_rels) in Trie::inter_min(&vec![x, y, z])
//       let xc = c_rels[0];
//       let yc = c_rels[1];
//       let zc = c_rels[2];
//       agg(&mut result, (a, b, c))
//   }
// }
// result
