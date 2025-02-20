use ahash::AHasher;

pub type HashMap<K, V> = std::collections::HashMap<
        K, V, std::hash::BuildHasherDefault<AHasher>>;
pub type HashSet<V> = std::collections::HashSet<
        V, std::hash::BuildHasherDefault<AHasher>>;

use crate::Val;

pub enum Trie {
    Leaf,
    Node(HashMap<Val, Trie>),
}

impl Trie {

    pub fn new() -> Self {
        Trie::Node(HashMap::default())
    }

    pub fn add(&mut self, t: &[Val]) {
        if t.len() == 0 {
            *self = Trie::Leaf
        } else if let Trie::Node(rm) = self {
            rm.entry(t[0].clone())
                .or_insert_with(Trie::new)
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

    pub fn get<'a>(&'a self, k: &Val) -> Option<&'a Trie> {
        if let Trie::Node(rm) = self {
            rm.get(k)
        } else {
            panic!("calling get on leaf")
        }
    }

    pub fn len(&self) -> usize {
        if let Trie::Node(m) = self {
            m.len()
        } else {
            panic!("calling len on leaf")
        }
    }

    pub fn inter_min<'a>(ts: &'a [&'a Trie]) ->
        Box<dyn Iterator<Item = (&'a Val, Vec<&'a Trie>)> + 'a>
    {
        ts.iter().min_by_key(|t| t.len()).unwrap().intersect(ts)
    }

    pub fn intersect<'a>(&'a self, ts: &'a [&'a Trie]) ->
        Box<dyn Iterator<Item = (&'a Val, Vec<&'a Trie>)> + 'a>
    {
        if let Trie::Node(rm) = self {
            Box::new(rm.iter().filter_map(move |(a, _y)| {
                let mut children = vec![];
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
