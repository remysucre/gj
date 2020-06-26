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

// TODO make these into 1 intersect with logic
fn intersect_v_v(r: &[u32], s: &[u32]) -> Vec<u32> {
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

fn intersect_v_e<'a>(r: &'a [u32], s: &'a [(u32, Vec<u32>)]) -> Vec<(u32, &'a Vec<u32>)> {
    let mut s = s;
    if r.len() <= s.len() {
    r.iter()
        .flat_map(|x_1| {
            s = gallop(s, |(x_2, _z)| x_2 < x_1);
            if !s.is_empty() && s[0].0 == *x_1 {
                Some((*x_1, &s[0].1))
            } else {
                None
            }
        })
        .collect()
    } else {
        intersect_e_v(s, r)
    }
}
fn intersect_e_v<'a>(r: &'a [(u32, Vec<u32>)], s: &'a [u32]) -> Vec<(u32, &'a Vec<u32>)> {
    let mut s = s;
    if r.len() <= s.len() {
    r.iter()
        .flat_map(|(x_1, y)| {
            s = gallop(s, |x_2| x_2 < x_1);
            if !s.is_empty() && s[0] == *x_1 {
                Some((*x_1, y))
            } else {
                None
            }
        })
        .collect()
    } else {
        intersect_v_e(s, r)
    }
}

fn intersect_e_e<'a>(r: &'a [(u32, Vec<u32>)], s: &'a [(u32, Vec<u32>)]) -> Vec<(u32, &'a Vec<u32>, &'a Vec<u32>)> {
    let mut r = r;
    let mut s = s;
    let mut swapped = false;
    if r.len() > s.len() {
        std::mem::swap(&mut r, &mut s);
        swapped = true;
    }
    r.iter()
        .flat_map(|(x_1, y)| {
            s = gallop(s, |(x_2, _z)| x_2 < x_1);
            if !s.is_empty() && s[0].0 == *x_1 {
                Some( if swapped {
                    (*x_1, &s[0].1, y)
                } else {
                    (*x_1, y, &s[0].1)
                })
            } else {
                None
            }
        })
        .collect()
}


fn intersect_v_v_v<'a>(mut r: &'a [u32], mut s: &'a [u32], mut t: &'a[u32]) -> Vec<u32> {
    if r.len() > s.len() {
        std::mem::swap(&mut r, &mut s);
    }
    if r.len() > t.len() {
        std::mem::swap(&mut r, &mut t);
    }
    r.iter()
        .flat_map(|x_1| {
            s = gallop(s, |x_2| x_2 < x_1);
            t = gallop(t, |x_2| x_2 < x_1);

            if !s.is_empty() && !t.is_empty() && s[0] == *x_1 && t[0] == *x_1 {
                Some(*x_1)
            } else {
                None
            }
        })
        .collect()
}

fn intersect_e_e_v<'a>(r: &'a [(u32, Vec<u32>)], mut s: &'a [(u32, Vec<u32>)], mut t: &'a[u32]) -> Vec<(u32, &'a Vec<u32>, &'a Vec<u32>)> {
    r.iter()
        .flat_map(|(x_1, y)| {
            s = gallop(s, |(x_2, _z)| x_2 < x_1);
            t = gallop(t, |x_2| x_2 < x_1);

            if !s.is_empty() && s[0].0 == *x_1 && t[0] == *x_1 {
                Some((*x_1, y, &s[0].1))
            } else {
                None
            }
        })
        .collect()
}

fn intersect_v_e_v<'a>(r: &'a [u32], mut s: &'a [(u32, Vec<u32>)], mut t: &'a[u32]) -> Vec<(u32, &'a Vec<u32>)> {
    r.iter()
        .flat_map(|x_1| {
            s = gallop(s, |(x_2, _z)| x_2 < x_1);
            t = gallop(t, |x_2| x_2 < x_1);

            if !s.is_empty() && !t.is_empty() && s[0].0 == *x_1 && t[0] == *x_1 {
                Some((*x_1, &s[0].1))
            } else {
                None
            }
        })
        .collect()
}

pub fn imdb_kmc(
    k: &[u32],
    cn: &[u32],
    mc: &[(u32, Vec<u32>)],
    mk: &[(u32, Vec<u32>)],
    t: &[u32],
) {
    let mut count = 0;
    for (_k, mk_k) in intersect_v_e(k, mk) {
        for (_m, mc_m) in intersect_v_e_v(mk_k, mc, t) {
            for _cid in intersect_v_v(cn, mc_m) {
                count += 1;
            }
        }
    }
    println!("{}", count);
}

pub fn imdb_kcm(
    k: &[u32],
    cn: &[u32],
    mc: &[(u32, Vec<u32>)],
    mk: &[(u32, Vec<u32>)],
    t: &[u32],
) {
    let mut count = 0;
    for (_k, mk_k) in intersect_v_e(k, mk) {
        for (_cid, mc_c) in intersect_v_e(cn, mc) {
            for _m in intersect_v_v_v(mk_k, mc_c, t) {
                count += 1;
            }
        }
    }
    println!("{}", count);
}

pub fn imdb_mkc(
    k: &[u32],
    cn: &[u32],
    mc: &[(u32, Vec<u32>)],
    mk: &[(u32, Vec<u32>)],
    t: &[u32],
) {
    let mut count = 0;
    for (_m, mc_m, mk_m) in intersect_e_e_v(mc, mk, t) {
        for _k in intersect_v_v(k, mk_m) {
            for _cid in intersect_v_v(cn, mc_m) {
                count += 1;
            }
        }
    }
    println!("{}", count);
}

pub fn imdb_mck(
    k: &[u32],
    cn: &[u32],
    mc: &[(u32, Vec<u32>)],
    mk: &[(u32, Vec<u32>)],
    t: &[u32],
) {
    let mut count = 0;
    for (_m, mc_m, mk_m) in intersect_e_e_v(mc, mk, t) {
        for _cid in intersect_v_v(cn, mc_m) {
            for _k in intersect_v_v(k, mk_m) {
                count += 1;
            }
        }
    }
    println!("{}", count);
}

pub fn imdb_cmk(
    k: &[u32],
    cn: &[u32],
    mc: &[(u32, Vec<u32>)],
    mk: &[(u32, Vec<u32>)],
    t: &[u32],
) {
    let mut count = 0;
    for (_cid, mc_c) in intersect_v_e(cn, mc) {
        for (_m, mk_m) in intersect_v_e_v(mc_c, mk, t) {
            for _k in intersect_v_v(k, mk_m) {
                count += 1;
            }
        }
    }
    println!("{}", count);
}

pub fn imdb_ckm(
    k: &[u32],
    cn: &[u32],
    mc: &[(u32, Vec<u32>)],
    mk: &[(u32, Vec<u32>)],
    t: &[u32],
) {
    let mut count = 0;
    for (_cid, mc_c) in intersect_v_e(cn, mc) {
        for (_k, mk_k) in intersect_v_e(k, mk) {
            for _m in intersect_v_v_v(mk_k, mc_c, t) {
                count += 1;
            }
        }
    }
    println!("{}", count);
}

pub fn triangle_index_xyz<R: Default, F: Fn(&mut R, (u32, u32, u32))>(
    r: &[(u32, Vec<u32>)],
    s: &[(u32, Vec<u32>)],
    t: &[(u32, Vec<u32>)],
    agg: F,
) -> R {
    let mut result = R::default();

    for (a, r_a, t_a) in intersect_e_e(r, t) {
        for (b, s_b) in intersect_e_v(s, r_a) {
            for c in intersect_v_v(s_b, t_a) {
                agg(&mut result, (a, b, c))
            }
        }
    }
    result
}

// r, t indexed on x, s indexed on z
pub fn triangle_index_xzy<R: Default, F: Fn(&mut R, (u32, u32, u32))>(
    r: &[(u32, Vec<u32>)],
    s: &[(u32, Vec<u32>)],
    t: &[(u32, Vec<u32>)],
    agg: F,
) -> R {
    let mut result = R::default();

    for (a, r_a, t_a) in intersect_e_e(r, t) {
        for (c, s_c) in intersect_e_v(s, t_a) {
            for b in intersect_v_v(s_c, r_a) {
                agg(&mut result, (a, b, c))
            }
        }
    }
    result
}

// r, s indexed on y, t indexed on x
pub fn triangle_index_yxz<R: Default, F: Fn(&mut R, (u32, u32, u32))>(
    r: &[(u32, Vec<u32>)],
    s: &[(u32, Vec<u32>)],
    t: &[(u32, Vec<u32>)],
    agg: F,
) -> R {
    let mut result = R::default();

    for (b, r_b, s_b) in intersect_e_e(r, s) {
        for (a, t_a) in intersect_e_v(t, r_b) {
            for c in intersect_v_v(s_b, t_a) {
                agg(&mut result, (a, b, c))
            }
        }
    }
    result
}

// r, t indexed on x, s indexed on z
pub fn triangle_index_yzx<R: Default, F: Fn(&mut R, (u32, u32, u32))>(
    r: &[(u32, Vec<u32>)],
    s: &[(u32, Vec<u32>)],
    t: &[(u32, Vec<u32>)],
    agg: F,
) -> R {
    let mut result = R::default();

    for (a, r_a, t_a) in intersect_e_e(r, t) {
        for (c, s_c) in intersect_e_v(s, t_a) {
            for b in intersect_v_v(s_c, r_a) {
                agg(&mut result, (a, b, c))
            }
        }
    }
    result
}

// r, t indexed on x, s indexed on z
pub fn triangle_index_zxy<R: Default, F: Fn(&mut R, (u32, u32, u32))>(
    r: &[(u32, Vec<u32>)],
    s: &[(u32, Vec<u32>)],
    t: &[(u32, Vec<u32>)],
    agg: F,
) -> R {
    let mut result = R::default();

    for (c, s_c, t_c) in intersect_e_e(s, t) {
        for (a, r_a) in intersect_e_v(r, t_c) {
            for b in intersect_v_v(s_c, r_a) {
                agg(&mut result, (a, b, c))
            }
        }
    }
    result
}

// r, t indexed on x, s indexed on z
pub fn triangle_index_zyx<R: Default, F: Fn(&mut R, (u32, u32, u32))>(
    r: &[(u32, Vec<u32>)],
    s: &[(u32, Vec<u32>)],
    t: &[(u32, Vec<u32>)],
    agg: F,
) -> R {
    let mut result = R::default();

    for (c, s_c, t_c) in intersect_e_e(s, t) {
        for (b, r_b) in intersect_e_v(r, s_c) {
            for a in intersect_v_v(t_c, r_b) {
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
                r.iter() // for each x in r
                 .flat_map(move |x| {
                     // forward-bin search for x in s
                     s = gallop(s, |y| y.0 < x.0);
                     if s.len() > 0 && s[0].0 == x.0 {
                         Some((&x.0, &x.1[..], &s[0].1[..]))
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
