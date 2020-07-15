// use seahash::SeaHasher;
use ahash::AHasher;
use std::hash::Hash;

pub type HashMap<K, V> = std::collections::HashMap<
        K, V, std::hash::BuildHasherDefault<AHasher>>;
pub type HashSet<V> = std::collections::HashSet<
        V, std::hash::BuildHasherDefault<AHasher>>;

use crate::relation::*;

impl<K: Eq + Hash, R> Index for HashMap<K, Vec<R>> {
    type Key = K;
    type Res = R;

    fn len(&self) -> usize {
        self.len()
    }

    fn get(&self, a: &Self::Key) -> Option<&[Self::Res]> {
        self.get(a).map(Vec::as_slice)
    }

    fn to_slice(&self) -> &[(Self::Key, Vec<Self::Res>)] {
        unimplemented!()
    }

    fn create(r: impl Iterator<Item = (Self::Key, Self::Res)>) -> Self
    {
        let mut r_x = HashMap::default();
        for (x, y) in r {
            let ys = r_x.entry(x).or_insert_with(Vec::new);
            ys.push(y);
        }
        r_x
    }

    fn intersect
        <'a, O: Index<Key = Self::Key>>
        (&'a self, other: &'a O) ->
        Box<dyn Iterator<Item = (&'a Self::Key, &'a [Self::Res], &'a [O::Res])> + 'a>
    {
        if self.len() <= other.len() {
            Box::new(self.iter().filter_map(move |(a, y)| {
                other.get(a).map(|z| (a, &y[..], z))
            }))
        } else {
            Box::new(other.intersect(self).map(|(a, x, y)| (a, y, x)))
        }
    }
}

pub fn triangle<'a, R: Default, F: Fn(&mut R, (&u32, &u32, &u32))>(
    r: &'a [(u32, u32)],
    s: &'a [(u32, u32)],
    t: &'a [(u32, u32)],
    agg: F,
) -> R {
    triangle_otf::
    <HashMap<u32, Vec<u32>>,
     HashMap<u32, Vec<()>>,
     R, F>(r, s, t, agg)
}

fn lookup(m: &HashMap<u32, HashSet<u32>>, n: u32) -> &HashSet<u32> {
  // let _guard = flame::start_guard("lookup");
  m.get(&n).unwrap()
}

fn inter<'a>(x: &'a HashSet<u32>, y: &'a HashSet<u32>) -> std::collections::hash_set::Intersection<'a, u32, std::hash::BuildHasherDefault<AHasher>> {
  // let _guard = flame::start_guard("inter");
  x.intersection(y)
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
    // let big_a = r_keys.intersection(&t_keys);
    let big_a = inter(&r_keys, &t_keys);
    for a in big_a {
        // let r_a = r.get(a).unwrap();
        let r_a = lookup(&r, *a);
        // let big_b = r_a.intersection(&s_keys);
        let big_b = inter(&r_a, &s_keys);
        for b in big_b {
            // let s_b = s.get(b).unwrap();
            let s_b = lookup(&s, *b);
            // let big_c = s_b.intersection(&t[a]);
            let t_a = lookup(&t, *a);
            let big_c = inter(&s_b, &t_a);
            for c in big_c {
                agg(&mut result, (*a, *b, *c));
            }
        }
    }
    // flame::dump_html(&mut std::fs::File::create("flame-graph.html").unwrap()).unwrap();
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
