use std::collections::{HashMap, HashSet};

// Taken from Frank McSherry's blog Worst-case optimal joins, in dataflow
// advances slice to the first element not less than value
pub fn gallop<T>(mut slice: &[T], mut cmp: impl FnMut(&T)->bool) -> &[T] {
    // if empty slice, or already >= element, return
    if slice.len() > 0 && cmp(&slice[0]) {
        let mut step = 1;
        while step < slice.len() && cmp(&slice[step]) {
            slice = &slice[step..];
            step = step << 1;
        }

        step = step >> 1;
        while step > 0 {
            if step < slice.len() && cmp(&slice[step]) {
                slice = &slice[step..];
            }
            step = step >> 1;
        }

        slice = &slice[1..]; // advance one, as we always stayed < value
    }

    return slice;
}

type Vertex = u32;
type Edge = (Vertex, Vertex);

fn intersect(r: &[u32], s: &[u32]) -> Vec<u32> {
    let mut r = r;
    let mut s = s;
    if r.len() > s.len() {
        std::mem::swap(&mut r, &mut s);
    }
    r.into_iter().flat_map(|x| {
        let t = gallop(s, |y| y < x );
        if t[0] == *x { Some(*x) } else { None }
    }).collect()
}

fn triangle_sort(mut r: &[(u32, Vec<u32>)], s: &[(u32, Vec<u32>)], mut t: &[(u32, Vec<u32>)]) ->
    Vec<(Vertex, Vertex, Vertex)>
{
    let mut result = vec![];

    let r_x: Vec<_> = r.into_iter().map(|(x, _)| *x).collect();
    let t_x: Vec<_> = t.into_iter().map(|(x, _)| *x).collect();
    let big_a = intersect(&r_x, &t_x);
    for a in big_a {
        r = gallop(r, |(x, _)| x < &a);
        let r_a = &r[0].1;
        t = gallop(t, |(x, _)| x < &a);
        let t_a = &t[0].1;
        let s_y: Vec<_> = s.into_iter().map(|(y, _)| *y).collect();
        let big_b = intersect(r_a, &s_y);
        // NOTE this should reset s?
        let mut s = s;
        for b in big_b {
            s = gallop(s, |(y, _)| y < &b);
            let s_b = &s[0].1;
            let big_c = intersect(s_b, t_a);
            for c in big_c {
                result.push((a, b, c));
            }
        }
    }
    result
}


// compute the triangle query Q(x,y,z) = R(x, y), S(y, z), T(z, x)
// using generic join
fn triangle_hash<'a>(r: &'a[Edge], s: &'a[Edge], t: &'a[Edge]) ->
    Vec<(&'a Vertex, &'a Vertex, &'a Vertex)>
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
            for c in s_b.intersection(&t_a) {
                result.push((*a, *b, *c));
            }
        }
    }
    result
}

fn to_trie(r: &[(u32, u32)]) -> Vec<(u32, Vec<u32>)> {
    let mut result: Vec<(u32, Vec<u32>)> = vec![];
    for (x, y) in r {
        if result.is_empty() || result.last().unwrap().0 != *x {
            result.push((*x, vec![*y]));
        } else {
            result.last_mut().unwrap().1.push(*y);
        }
    }
    result
}

fn main() {
    let mut es = vec![];
    triangle_hash(&es, &es, &es);
    // Q(x,y,z) = R(x, y), S(y, z), T(z, x)
    // variable order: x, y, z
    es.sort_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
    let r = &es;
    let s = &es;
    let mut t: Vec<(u32, u32)> = es.iter().map(|(x, y)| (*y, *x)).collect();
    t.sort_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
    triangle_sort(&to_trie(r), &to_trie(s), &to_trie(&t));
}
