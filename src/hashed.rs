use std::collections::{HashMap, HashSet};

// compute the triangle query Q(x,y,z) = R(x, y), S(y, z), T(z, x)
// using generic join
fn triangle_hash<'a>(r: &'a[(u32, u32)], s: &'a[(u32, u32)], t: &'a[(u32, u32)]) ->
    Vec<(&'a u32, &'a u32, &'a u32)>
{
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

    let mut result = vec![];
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
                result.push((*a, *b, *c));
            }
        }
    }
    result
}

// this version builds a hashmap for s on y to save some scans
fn triangle_hash_alt<'a>(r: &'a[(u32, u32)], s: &'a[(u32, u32)], t: &'a[(u32, u32)]) ->
    Vec<(&'a u32, &'a u32, &'a u32)>
{
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

    // build hashmap for s on y
    let mut s_y = HashMap::new();
    let mut s_y_keys = HashSet::new();
    for (y, z) in s {
        let zs = s_y.entry(y).or_insert_with(HashSet::new);
        zs.insert(z);
        s_y_keys.insert(y);
    }

    let mut result = vec![];
    // now we have hash-joined r and t, and t_x.keys = intersect(r.x, t.x)
    for (a, t_a) in t_x.iter() {
        // join s and r_a
        // s_y[b] is the residual relation s(b, z)
        let r_a = r_x.get(a).expect("t_x.x not found in r_x");
        for b in r_a.intersection(&s_y_keys) {
            for c in s_y[b].intersection(t_a) {
                result.push((*a, *b, *c));
            }
        }
    }
    result
}

// this version takes hash indexes for r, s, t
fn triangle_hash_index(
    r: HashMap<u32, HashSet<u32>>, rks: HashSet<u32>,
    s: HashMap<u32, HashSet<u32>>, sks: HashSet<u32>,
    t: HashMap<u32, HashSet<u32>>, tks: HashSet<u32>,
) -> Vec<(u32, u32, u32)>
{
    let mut result = vec![];
    for a in rks.intersection(&tks) {
        for b in r[a].intersection(&sks) {
            for c in s[b].intersection(&t[a]) {
                result.push((*a, *b, *c));
            }
        }
    }
    result
}

pub fn triangle_hash_index_cnt(
    r: HashMap<u32, HashSet<u32>>, rks: HashSet<u32>,
    s: HashMap<u32, HashSet<u32>>, sks: HashSet<u32>,
    t: HashMap<u32, HashSet<u32>>, tks: HashSet<u32>,
) -> u32
{
    let mut result = 0;
    for a in rks.intersection(&tks) {
        for b in r[a].intersection(&sks) {
            for _c in s[b].intersection(&t[a]) {
                result += 1;
            }
        }
    }
    result
}
