use std::collections::{HashMap, HashSet};
use std::time::Instant;

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

fn sorted(s: &[u32]) -> bool {
    let mut prev = u32::MIN;
    for n in s {
        if n < &prev {
            return false;
        } else {
            prev = *n;
        }
    }
    true
}

fn intersect(r: &[u32], s: &[u32]) -> Vec<u32> {
    debug_assert!(sorted(r) && sorted(s));
    let mut r = r;
    let mut s = s;
    if r.len() > s.len() {
        std::mem::swap(&mut r, &mut s);
    }
    r.into_iter().flat_map(|x| {
        let t = gallop(s, |y| y < x );
        if !t.is_empty() && t[0] == *x { Some(*x) } else { None }
    }).collect()
}

fn triangle_sort(mut r: &[(u32, Vec<u32>)], s: &[(u32, Vec<u32>)], mut t: &[(u32, Vec<u32>)]) ->
    Vec<(Vertex, Vertex, Vertex)>
{
    let mut result = vec![];

    let r_x: Vec<_> = r.into_iter().map(|(x, _)| *x).collect();
    let t_x: Vec<_> = t.into_iter().map(|(x, _)| *x).collect();
    let s_y: Vec<_> = s.into_iter().map(|(y, _)| *y).collect();

    let big_a = intersect(&r_x, &t_x);
    for a in big_a {
        r = gallop(r, |(x, _)| x < &a);
        let r_a = &r[0].1;
        debug_assert_eq!(&r[0].0, &a);

        let big_b = intersect(r_a, &s_y);
        t = gallop(t, |(x, _)| x < &a);
        let t_a = &t[0].1;
        debug_assert_eq!(&t[0].0, &a);

        // NOTE this should reset s
        let mut s_ = s;
        for b in big_b {
            s_ = gallop(s_, |(y, _)| y < &b);
            let s_b = &s_[0].1;
            debug_assert_eq!(&s_[0].0, &b);
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
            for c in s_b.intersection(t_a) {
                result.push((*a, *b, *c));
            }
        }
    }
    result
}

// this version builds a hashmap for s on y to save some scans
fn triangle_hash_alt<'a>(r: &'a[Edge], s: &'a[Edge], t: &'a[Edge]) ->
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
    r: HashMap<Vertex, HashSet<Vertex>>, rks: HashSet<Vertex>,
    s: HashMap<Vertex, HashSet<Vertex>>, sks: HashSet<Vertex>,
    t: HashMap<Vertex, HashSet<Vertex>>, tks: HashSet<Vertex>,
) -> Vec<(Vertex, Vertex, Vertex)>
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


// NOTE should be sorted
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

fn read_edges() -> Result<Vec<(u32, u32)>, Box<dyn std::error::Error>> {
    use csv::ReaderBuilder;
    // Build the CSV reader and iterate over each record.
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(std::io::stdin());
    let mut es = vec![];
    for result in rdr.records() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here.
        let record = result?;
        let (u, v) = (record[0].to_owned(), record[1].to_owned());
        es.push((u.parse().unwrap(), v.parse().unwrap()));
    }
    Ok(es)
}

fn main() {
    // let args: Vec<String> = std::env::args().collect();
    // let n = args[1].parse().unwrap();
    // let (mut r, mut s, mut t) = gen_worst_case_relations(n);

    let mut es = read_edges().unwrap();
    let ts_h_len;
    {
        let r = &es;
        let s = &es;
        let t = &es;

        // hash-gj without index
        // println!("hash-join starting");
        // let now = Instant::now();
        // let ts = triangle_hash_alt(&r, &s, &t);
        // let ts_h_len = ts.len();
        // println!("hash-join: {}", now.elapsed().as_millis());

        // hash-gj with index
        let mut r_x = HashMap::new();
        for (x, y) in r.iter().copied() {
            let ys = r_x.entry(x).or_insert_with(HashSet::new);
            ys.insert(y);
        }
        let rks: HashSet<u32> = r_x.keys().copied().collect();

        let mut t_x = HashMap::new();
        for (z, x) in t.iter().copied() {
            let zs = t_x.entry(x).or_insert_with(HashSet::new);
            zs.insert(z);
        }
        let tks: HashSet<u32> = t_x.keys().copied().collect();

        let mut s_y = HashMap::new();
        for (y, z) in s.iter().copied() {
            let zs = s_y.entry(y).or_insert_with(HashSet::new);
            zs.insert(z);
        }
        let sks: HashSet<u32> = s_y.keys().copied().collect();

        println!("hash-join starting");
        let now = Instant::now();
        let ts = triangle_hash_index(
            r_x, rks,
            s_y, sks,
            t_x, tks,
        );
        ts_h_len = ts.len();
        println!("hash-join: {}", now.elapsed().as_millis());
    }
    // sort-gj with tries
    es.sort_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
    let r_t = to_trie(&es);
    // s.sort_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
    let s_t = to_trie(&es);
    let mut t: Vec<_> = es.into_iter().map(|(x, y)| (y, x)).collect();
    t.sort_by(|(x_1, y_1), (x_2, y_2)| x_1.cmp(x_2).then(y_1.cmp(y_2)));
    let t_t = to_trie(&t);

    println!("sort-join starting");
    let now = Instant::now();
    let ts_s = triangle_sort(&r_t, &s_t, &t_t);
    let ts_s_len = ts_s.len();
    println!("sort-join: {}", now.elapsed().as_millis());
    assert_eq!(ts_h_len, ts_s_len);
    // println!("{:?}", ts_h_len);
}

fn gen_worst_case_relations(n: u32) -> (Vec<Edge>, Vec<Edge>, Vec<Edge>) {
    assert!(n > 0);
    let x: Vec<_> = (0..n).collect();
    let y: Vec<_> = (n..2*n).collect();
    let z: Vec<_> = (2*n..3*n).collect();

    let mut r = vec![];
    let mut s = vec![];
    let mut t = vec![];
    for i in 0..n as usize {
        r.push((x[0], y[i]));
        s.push((y[0], z[i]));
        t.push((z[0], x[i]));
    }
    for i in 1..n as usize {
        r.push((x[i], y[0]));
        s.push((y[i], z[0]));
        t.push((z[i], x[0]));
    }
    (r, s, t)
}
