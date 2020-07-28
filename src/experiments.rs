use std::time::Instant;
use crate::{util::*, *};
use trie::*;

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

pub fn triangle<'a, R, F>(rx: Trie, sy: Trie, tx: Trie, agg: F) -> R
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
