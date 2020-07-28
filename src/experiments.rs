use std::time::Instant;
use crate::{util::*, *};
use trie::*;

use gj_macro::*;

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
    triangle(rx, sy, tx, |result: &mut u64, _| {
        *result += 1
    });
    println!("done in {}", now.elapsed().as_millis());
}

pub fn triangle<R, F>(rx: Trie, sy: Trie, tx: Trie, agg: F) -> R
where R: Default, F: Fn(&mut R, (&Val, &Val, &Val))
{
    let mut result = R::default();
    for (a, ra_ta) in Trie::inter_min(&vec![&rx, &tx]) {
        let ra = ra_ta[0];
        let ta = ra_ta[1];
        for (b, rab_sb) in Trie::inter_min(&vec![ra, &sy]) {
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
    for (_a, ct__mcX) in Trie::inter_min(&vec![ct, mc]) {
        let _ct_a = ct__mcX[0];
        let mc_a = ct__mcX[0];
        for (_b, t__mc_a__mi_idxY) in Trie::inter_min(&vec![t, mc_a, mi_idx]) {
            let t_b = t__mc_a__mi_idxY[0];
            let mc_ab = t__mc_a__mi_idxY[1];
            let mi_idx_b = t__mc_a__mi_idxY[2];
            for (_c, it__mi_idxZ) in Trie::inter_min(&vec![it, mi_idx_b]) {
                // output aggregates, hopefully it doesn't need the join attrs
                agg(&mut result, &vec![mc_ab, t_b])
            }
        }
    }
    result
}

pub fn try_macro() {
    sql!("lol");
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
