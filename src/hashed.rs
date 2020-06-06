use seahash::SeaHasher;

type HashMap<K, V> = std::collections::HashMap<
        K, V, std::hash::BuildHasherDefault<SeaHasher>>;
type HashSet<V> = std::collections::HashSet<
        V, std::hash::BuildHasherDefault<SeaHasher>>;

// Compute the triangle query Q(x,y,z) = R(x, y), S(y, z), T(z, x) using generic join.
// This version scans the entire S when intersecting it with R(a, y), therefore is slow.
pub fn triangle<'a, R: Default, F: Fn(&mut R, (&u32, &u32, &u32))>(
    r: &'a [(u32, u32)],
    s: &'a [(u32, u32)],
    t: &'a [(u32, u32)],
    agg: F,
) -> R {
    // hash r on x to be joined with t
    // r_x[a] is the residual relation r(a, y)
    let mut r_x = HashMap::default();
    for (x, y) in r {
        let ys = r_x.entry(x).or_insert_with(HashSet::default);
        ys.insert(y);
    }
    // hash-join t with r on x
    // t_x[a] is the residual relation t(z, a)
    let mut t_x = HashMap::default();
    for (z, x) in t {
        if r_x.contains_key(&x) {
            let zs = t_x.entry(x).or_insert_with(HashSet::default);
            zs.insert(z);
        }
    }
    // building this hash outside the loop
    let mut s_y = HashMap::default();
    let mut s_y_keys = HashSet::default();
    for (y, z) in s {
        let zs = s_y.entry(y).or_insert_with(HashSet::default);
        zs.insert(z);
        s_y_keys.insert(y);
    }
    let mut result = R::default();
    // now we have hash-joined r and t, and t_x.keys = intersect(r.x, t.x)
    for (a, t_a) in t_x.iter() {
        let r_a = r_x.get(a).expect("t_x.x not found in r_x");
        // join s and r_a
        // s_y[b] is the residual relation s(b, z)
        for b in r_a.intersection(&s_y_keys) {
            for c in s_y[b].intersection(t_a) {
                agg(&mut result, (*a, *b, *c));
            }
        }
    }
    result
}

// This version takes hash indexes for r, s, t.
pub fn triangle_index<R: Default, F: Fn(&mut R, (u32, u32, u32))>(
    r: HashMap<u32, HashSet<u32>>,
    r_keys: HashSet<u32>,
    s: HashMap<u32, HashSet<u32>>,
    s_keys: HashSet<u32>,
    t: HashMap<u32, HashSet<u32>>,
    t_keys: HashSet<u32>,
    agg: F,
) -> R {
    let mut result = R::default();
    for a in r_keys.intersection(&t_keys) {
        for b in r[a].intersection(&s_keys) {
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
    let mut r_x = HashMap::default();
    for e in r.iter().copied() {
        let (x, y) = order(e);
        let ys = r_x.entry(x).or_insert_with(HashSet::default);
        ys.insert(y);
    }
    let r_keys: HashSet<u32> = r_x.keys().copied().collect();
    (r_x, r_keys)
}
