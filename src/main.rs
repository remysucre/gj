use std::collections::{HashMap, HashSet};

type Vertex = u32;
type Edge = (Vertex, Vertex);

// compute the triangle query Q(x,y,z) = R(x, y), S(y, z), T(z, x)
// using generic join
fn triangle<'a>(r: &'a[Edge], s: &'a[Edge], t: &'a[Edge]) ->
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

fn main() {
    let es = vec![];
    triangle(&es, &es, &es);
}
