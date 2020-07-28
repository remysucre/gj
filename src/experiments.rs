use std::time::Instant;
use crate::{util::*, *};
use trie::*;

//-----------------------------------
// Triangle query, hashing vs sorting
//-----------------------------------
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
