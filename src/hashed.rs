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

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Val {
    Int(u64),
    Str(String),
    Boo(bool),
}

impl HTrie {

    pub fn new() -> Self {
        HTrie::Node(HashMap::default())
    }

    pub fn add(&mut self, t: &[Val]) {
        if t.len() == 0 {
            *self = HTrie::Leaf
        } else if let HTrie::Node(rm) = self {
            rm.entry(t[0].clone())
                .or_insert_with(HTrie::new)
                .add(&t[1..]);
        } else {
            panic!("inserting into leaf")
        }
    }

    pub fn from_iter<'a>(ts: impl Iterator<Item = &'a [Val]>) -> Self {
        let mut r = Self::new();
        for t in ts { r.add(t) }
        r
    }

    pub fn get<'a>(&'a self, k: &Val) -> Option<&'a HTrie> {
        if let HTrie::Node(rm) = self {
            rm.get(k)
        } else {
            panic!("calling get on leaf")
        }
    }

    pub fn len(&self) -> usize {
        if let HTrie::Node(m) = self {
            m.len()
        } else {
            panic!("calling len on leaf")
        }
    }

    pub fn inter_min<'a>(ts: &'a mut [&'a HTrie]) ->
        Box<dyn Iterator<Item = (&'a Val, Vec<&'a HTrie>)> + 'a>
    {
        ts.sort_by_key(|t| t.len());
        ts[0].intersect(&ts[1..])
    }

    pub fn intersect<'a>(&'a self, ts: &'a [&'a HTrie]) ->
        Box<dyn Iterator<Item = (&'a Val, Vec<&'a HTrie>)> + 'a>
    {
        if let HTrie::Node(rm) = self {
            Box::new(rm.iter().filter_map(move |(a, y)| {
                let mut children = vec![y];
                for t in ts {
                    let c = t.get(a)?;
                    children.push(c);
                }
                Some((a, children))
            }))
        } else {
            panic!("intersecting leaves")
        }
    }
}

pub fn triangle_ht<'a, R: Default, F: Fn(&mut R, (&Value, &Value, &Value))>(
        r: &'a [(Value, Value)],
        s: &'a [(Value, Value)],
        t: &'a [(Value, Value)],
        agg: F,
) -> R {
    // first build indexes
    let r: Vec<_> = r.iter().map(|(x, y)| vec![Val::Int(x.clone()), Val::Int(y.clone())]).collect();
    let s: Vec<_> = s.iter().map(|(y, z)| vec![Val::Int(y.clone()), Val::Int(z.clone())]).collect();
    let t: Vec<_> = t.iter().map(|(z, x)| vec![Val::Int(x.clone()), Val::Int(z.clone())]).collect();
    let rx = HTrie::from_iter(r.iter().map(|v| &v[..]));
    let sy = HTrie::from_iter(s.iter().map(|v| &v[..]));
    let tx = HTrie::from_iter(t.iter().map(|v| &v[..]));

    let mut result = R::default();

    for (a, ra_ta) in HTrie::inter_min(&mut vec![&rx, &tx]) {
        let ra = ra_ta[0];
        let ta = ra_ta[1];
        for (b, sb_rab) in HTrie::inter_min(&mut vec![&sy, ra]) {
            let sb = sb_rab[0];
            for (c, _sbc_tac) in HTrie::inter_min(&mut vec![sb, ta]) {
                if let (Val::Int(a), Val::Int(b), Val::Int(c)) = (a,b,c) {
                    agg(&mut result, (a, b, c))
                } else {
                    panic!("type error")
                }
            }
        }
    }

    result
}
