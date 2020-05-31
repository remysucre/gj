use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;

// Compute the triangle query Q(x,y,z) = R(x, y), S(y, z), T(z, x) using generic join.
// This version scans the entire S when intersecting it with R(a, y), therefore is slow.
pub fn triangle_slow<'a, R: Default, F: Fn(&mut R, (&u32, &u32, &u32))>(
    r: &'a [(u32, u32)],
    s: &'a [(u32, u32)],
    t: &'a [(u32, u32)],
    agg: F,
) -> R {
    // hash r on x to be joined with t
    // r_x[a] is the residual relation r(a, y)
    let mut r_x = HashMap::new();
    for (x, y) in r {
        // ys points to r_x[x]
        let ys = r_x.entry(x).or_insert_with(HashSet::new);
        ys.insert(y);
    }
    // hash-join t with r on x
    // t_x[a] is the residual relation t(z, a)
    let mut t_x = HashMap::new();
    for (z, x) in t {
        if r_x.contains_key(&x) {
            // zs points to t_x(x)
            let zs = t_x.entry(x).or_insert_with(HashSet::new);
            zs.insert(z);
        }
    }

    let mut result = R::default();
    // now we have hash-joined r and t, and t_x.keys = intersect(r.x, t.x)
    for (a, t_a) in t_x.iter() {
        let mut s_y = HashMap::new();
        // join s and r_a
        // s_y[b] is the residual relation s(b, z)
        let r_a = r_x.get(a).expect("t_x.x not found in r_x");
        for (y, z) in s {
            if r_a.contains(&y) {
                // zs points to s_y[y]
                let zs = s_y.entry(y).or_insert_with(HashSet::new);
                zs.insert(z);
            }
        }
        // now we have hash-joined s and r_a, and s_y.keys = intersect(s.y, r_a.y)
        for (b, s_b) in s_y.iter() {
            // intersect s_b.z and t_a.z
            for c in s_b.intersection(t_a) {
                agg(&mut result, (*a, *b, *c));
            }
        }
    }
    result
}

// This version builds a hashmap for s on y to save the scans.
pub fn triangle_fast<'a, R: Default, F: Fn(&mut R, (&u32, &u32, &u32))>(
    r: &'a [(u32, u32)],
    s: &'a [(u32, u32)],
    t: &'a [(u32, u32)],
    agg: F,
) -> R {
    let mut r_x = HashMap::new();
    for (x, y) in r {
        let ys = r_x.entry(x).or_insert_with(HashSet::new);
        ys.insert(y);
    }
    let mut t_x = HashMap::new();
    for (z, x) in t {
        if r_x.contains_key(&x) {
            let zs = t_x.entry(x).or_insert_with(HashSet::new);
            zs.insert(z);
        }
    }
    let mut s_y = HashMap::new();
    let mut s_y_keys = HashSet::new();
    for (y, z) in s {
        let zs = s_y.entry(y).or_insert_with(HashSet::new);
        zs.insert(z);
        s_y_keys.insert(y);
    }
    let mut result = R::default();
    for (a, t_a) in t_x.iter() {
        let r_a = r_x.get(a).expect("t_x.x not found in r_x");
        for b in r_a.intersection(&s_y_keys) {
            for c in s_y[b].intersection(t_a) {
                agg(&mut result, (*a, *b, *c));
            }
        }
    }
    result
}

// This version takes hash indexes for r, s, t.
pub fn triangle_index<H: BuildHasher, R: Default, F: Fn(&mut R, (u32, u32, u32))>(
    r: HashMap<u32, HashSet<u32, H>, H>,
    rks: HashSet<u32, H>,
    s: HashMap<u32, HashSet<u32, H>, H>,
    sks: HashSet<u32, H>,
    t: HashMap<u32, HashSet<u32, H>, H>,
    tks: HashSet<u32, H>,
    agg: F,
) -> R {
    let mut result = R::default();
    for a in rks.intersection(&tks) {
        for b in r[a].intersection(&sks) {
            for c in s[b].intersection(&t[a]) {
                agg(&mut result, (*a, *b, *c));
            }
        }
    }
    result
}

pub fn build_hash<F: Fn((u32, u32)) -> (u32, u32)>(
    r: &[(u32, u32)],
    order: F,
) -> (HashMap<u32, HashSet<u32>>, HashSet<u32>) {
    let mut r_x = HashMap::new();
    for e in r.iter().copied() {
        let (x, y) = order(e);
        let ys = r_x.entry(x).or_insert_with(HashSet::new);
        ys.insert(y);
    }
    let rks: HashSet<u32> = r_x.keys().copied().collect();
    (r_x, rks)
}
