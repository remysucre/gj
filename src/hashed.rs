use seahash::SeaHasher;
use std::hash::Hash;

type HashMap<K, V> = std::collections::HashMap<
        K, V, std::hash::BuildHasherDefault<SeaHasher>>;
type HashSet<V> = std::collections::HashSet<
        V, std::hash::BuildHasherDefault<SeaHasher>>;

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
