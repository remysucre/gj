use crate::relation::*;

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
            let t = gallop(s, |y| y < x);
            if !t.is_empty() && t[0] == *x {
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

// type Trie<K, R> = Vec<(K, Vec<R>)>;
//
// impl<K: Eq + Hash, R> Index for Trie<K, R> {
//     type Key = K;
//     type Res = R;
//
//     fn len(&self) -> usize {
//         self.len()
//     }
//
//     fn get(&self, a: &Self::Key) -> Option<&[Self::Res]> {
//         self.get(a).map(Vec::as_slice)
//     }
//
//     fn create(r: impl Iterator<Item = (Self::Key, Self::Res)>) -> Self
//     {
//         let mut r_x = HashMap::default();
//         for (x, y) in r {
//             let ys = r_x.entry(x).or_insert_with(Vec::new);
//             ys.push(y);
//         }
//         r_x
//     }
//
//     fn intersect
//         <'a, O: Index<Key = Self::Key>>
//         (&'a self, other: &'a O) ->
//         Box<dyn Iterator<Item = (&'a Self::Key, &'a [Self::Res], &'a [O::Res])> + 'a>
//     {
//         if self.len() <= other.len() {
//             Box::new(self.iter().filter_map(move |(a, y)| {
//                 other.get(a).map(|z| (a, &y[..], z))
//             }))
//         } else {
//             Box::new(other.intersect(self).map(|(a, x, y)| (a, y, x)))
//         }
//     }
//
// }
