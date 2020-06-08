use crate::relation::*;

// find last index < element
pub fn gallop_v<T>(v: &Vec<T>, mut cmp: impl FnMut(&T) -> bool) -> Option<usize> {
    // if empty slice, or already >= element, return None
    if !v.is_empty() && cmp(&v[0]) {
        let mut step = 1;
        let mut i = 0;
        while i + step < v.len() && cmp(&v[i + step]) {
            i += step;
            step <<= 1;
        }

        step >>= 1;
        while step > 0 {
            if i + step < v.len() && cmp(&v[step]) {
                i += step;
            }
            step >>= 1;
        }
        Some(i)
    } else {
        None
    }
}

// Taken from Frank McSherry's blog Worst-case optimal joins, in dataflow
// advances slice to the first element not less than value
pub fn gallop<T>(mut slice: &[T], mut cmp: impl FnMut(&T) -> bool) -> &[T] {
    // if empty slice, or already >= element, return
    if !slice.is_empty() && cmp(&slice[0]) {
        let mut step = 1;
        while step < slice.len() && cmp(&slice[step]) {
            slice = &slice[step..];
            step <<= 1;
        }

        step >>= 1;
        while step > 0 {
            if step < slice.len() && cmp(&slice[step]) {
                slice = &slice[step..];
            }
            step >>= 1;
        }

        slice = &slice[1..]; // advance one, as we always stayed < value
    }

    slice
}

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
    r.iter()
        .flat_map(|x| {
            s = gallop(s, |y| y < x);
            if !s.is_empty() && s[0] == *x {
                Some(*x)
            } else {
                None
            }
        })
        .collect()
}

pub fn triangle_index<R: Default, F: Fn(&mut R, (u32, u32, u32))>(
    mut r: &[(u32, Vec<u32>)],
    s: &[(u32, Vec<u32>)],
    mut t: &[(u32, Vec<u32>)],
    agg: F,
) -> R {
    let mut result = R::default();

    let r_x: Vec<_> = r.iter().map(|(x, _)| *x).collect();
    let t_x: Vec<_> = t.iter().map(|(x, _)| *x).collect();
    let s_y: Vec<_> = s.iter().map(|(y, _)| *y).collect();

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
                agg(&mut result, (a, b, c))
            }
        }
    }
    result
}

// NOTE should be sorted
pub fn to_trie(r: &[(u32, u32)]) -> Vec<(u32, Vec<u32>)> {
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

type Trie<K, R> = Vec<(K, Vec<R>)>;

impl<'t, K: Eq + Ord, R> Index for Trie<K, R> {
    type Key = K;
    type Res = R;

    fn len(&self) -> usize {
        self.len()
    }

    fn get(&self, _a: &Self::Key) -> Option<&[Self::Res]> {
        unimplemented!()
    }

    fn to_slice(&self) -> &[(Self::Key, Vec<Self::Res>)] {
        &self[..]
    }

    fn create(r: impl Iterator<Item = (Self::Key, Self::Res)>) -> Self
    {
        let mut v: Vec<(Self::Key, Self::Res)> = r.collect();
        v.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut trie: Vec<(Self::Key, Vec<Self::Res>)> = vec![];
        for (x, y) in v {
            if trie.is_empty() || trie.last().unwrap().0 != x {
                trie.push((x, vec![y]));
            } else {
                trie.last_mut().unwrap().1.push(y);
            }
        }
        trie
    }

    fn intersect
        <'a, O: Index<Key = Self::Key>>
        (&'a self, other: &'a O) ->
        Box<dyn Iterator<Item = (&'a Self::Key, &'a [Self::Res], &'a [O::Res])> + 'a>
    {
        // TODO debug_assert!(sorted(r) && sorted(s));
        if self.len() <= other.len() {
            let r = self;
            let mut s = other.to_slice();
            Box::new(
                r.iter()
                 .flat_map(move |x| {
                     s = gallop(s, |y| y.0 < x.0);
                     let y = &s[0];
                     if s.len() > 0 && y.0 == x.0 {
                         Some((&x.0, &x.1[..], &y.1[..]))
                     } else {
                         None
                     }
                 })
            )
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
    <Trie<u32, u32>,
     Trie<u32, ()>,
     R, F>(r, s, t, agg)
}
