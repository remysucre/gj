// use seahash::SeaHasher;
use ahash::AHasher;
use std::hash::Hash;

pub type HashMap<K, V> = std::collections::HashMap<
        K, V, std::hash::BuildHasherDefault<AHasher>>;
pub type HashSet<V> = std::collections::HashSet<
        V, std::hash::BuildHasherDefault<AHasher>>;

use crate::relation::*;
use crate::util::Value;

impl<K: Eq + Hash, R> Trie<K, R> for HashMap<K, R> {
    fn create(r: impl Iterator<Item = (K, R)>) -> Self {
        r.collect()
    }

    fn intersect<'a, S, T: Trie<K, S>>(&'a self, other: &'a T) ->
        Box<dyn Iterator<Item = (&'a K, &'a R, &'a S)> + 'a>
    {
        if self.len() <= other.len() {
            Box::new(self.iter().filter_map(move |(a, y)| {
                other.get(a).map(|z| (a, y, z))
            }))
        } else {
            Box::new(other.intersect(self).map(|(a, x, y)| (a, y, x)))
        }
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn get<'a>(&'a self, k: &'a K) -> Option<&'a R> {
        self.get(k)
    }
}

fn len(r: &HTrie) -> usize {
    if let HTrie::Node(m) = r {
        m.len()
    } else {
        panic!("calling len on leaf")
    }
}

fn intersect<'a>(r: &'a HTrie, s: &'a HTrie) ->
    Box<dyn Iterator<Item = (&'a Val, &'a HTrie, &'a HTrie)> + 'a>
{
    if let (HTrie::Node(rm), HTrie::Node(sm)) = (r, s) {
        if rm.len() <= sm.len() {
            Box::new(rm.iter().filter_map(move |(a, y)| {
                sm.get(a).map(|z| (a, y, z))
            }))
        } else {
            Box::new(intersect(s, r).map(|(a, x, y)| (a, y, x)))
        }
    } else {
        panic!("intersecting leaves")
    }
}

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

pub fn triangle<'a, R: Default, F: Fn(&mut R, (&Value, &Value, &Value))>(
    r: &'a [(Value, Value)],
    s: &'a [(Value, Value)],
    t: &'a [(Value, Value)],
    agg: F,
) -> R {
    triangle_otf::
    <HashMap<Value, Vec<Value>>,
     HashMap<Value, Vec<()>>,
     R, F>(r, s, t, agg)
}

fn lookup(m: &HashMap<Value, HashSet<Value>>, n: Value) -> &HashSet<Value> {
  // let _guard = flame::start_guard("lookup");
  m.get(&n).unwrap()
}

fn inter<'a>(x: &'a HashSet<Value>, y: &'a HashSet<Value>) -> std::collections::hash_set::Intersection<'a, Value, std::hash::BuildHasherDefault<AHasher>> {
  // let _guard = flame::start_guard("inter");
  x.intersection(y)
}

// This version takes hash indexes for r, s, t.
pub fn triangle_index<R: Default,F: Fn(&mut R, (Value, Value, Value))>(
    r: HashMap<Value, HashSet<Value>>,
    r_keys: HashSet<Value>,
    s: HashMap<Value, HashSet<Value>>,
    s_keys: HashSet<Value>,
    t: HashMap<Value, HashSet<Value>>,
    t_keys: HashSet<Value>,
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

pub fn triangle_trie<'a, R: Default, F: Fn(&mut R, (&Value, &Value, &Value))>(
        r: &'a [(Value, Value)],
        s: &'a [(Value, Value)],
        t: &'a [(Value, Value)],
        agg: F,
) -> R {
    triangle_idx::<R, F, HashMap<Value, ()>, HashMap<Value, HashMap<Value, ()>>>(r,s,t,agg)
}

pub fn build_hash<F: Fn((Value, Value)) -> (Value, Value)>(
    r: &[(Value, Value)],
    order: F,
) -> (HashMap<Value, HashSet<Value>>, HashSet<Value>) {
    let mut r_x = HashMap::default();
    for e in r.iter().copied() {
        let (x, y) = order(e);
        let ys = r_x.entry(x).or_insert_with(HashSet::default);
        ys.insert(y);
    }
    let r_keys: HashSet<Value> = r_x.keys().copied().collect();
    (r_x, r_keys)
}

pub enum HTrie {
    Leaf,
    Node(HashMap<Val, HTrie>),
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Val {
    Int(u64),
    Str(String),
    Boo(bool),
}
