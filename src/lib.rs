pub mod hashed;
pub mod sorted;
pub mod util;
pub mod experiments;

pub mod relation {
    use crate::util::Value;
    use std::collections::HashMap;

    pub trait Index {
        // NOTE Key can be a reference
        // NOTE Res can be a tuple of refs
        type Key;
        type Res;

        // TODO create should take f: record -> (key, residual)
        fn create(r: impl Iterator<Item = (Self::Key, Self::Res)>) -> Self;

        fn len(&self) -> usize;

        // HACK to support tries
        fn to_slice(&self) -> &[(Self::Key, Vec<Self::Res>)];

        fn get(&self, a: &Self::Key) -> Option<&[Self::Res]>;

        fn intersect
            <'a, O: Index<Key = Self::Key>>
            (&'a self, other: &'a O) ->
            Box<dyn Iterator<Item = (&'a Self::Key, &'a [Self::Res], &'a [O::Res])> + 'a>;
    }

    // create indexes on the fly
    pub fn triangle_otf<'a, IdxLeft, IdxUnit, R, F>(
        r: &'a [(Value, Value)],
        s: &'a [(Value, Value)],
        t: &'a [(Value, Value)],
        agg: F,
    ) -> R
    where
        IdxLeft: Index<Key = Value, Res = Value>,
        IdxUnit: Index<Key = Value, Res = ()>,
        R: Default,
        F: Fn(&mut R, (&Value, &Value, &Value)),
    {
        let mut result = R::default();
        // create indexes on r, t, s
        let r_x = IdxLeft::create(r.iter().copied());
        let t_x = IdxLeft::create(t.iter().copied());
        let s_y = IdxLeft::create(s.iter().copied());
        for (a, ra, ta) in r_x.intersect(&t_x) {
            // index residual relations r_a, t_a
            let ra_y = IdxUnit::create(ra.iter().copied().map(|n| (n, ())));
            let ta_z = IdxUnit::create(ta.iter().copied().map(|n| (n, ())));
            for (b, sb, _rab) in s_y.intersect(&ra_y) {
                // index residual relation s_b
                let sb_z = IdxUnit::create(sb.iter().copied().map(|n| (n, ())));
                for (c, _sbc, _tac) in sb_z.intersect(&ta_z) {
                    agg(&mut result, (a, b, c));
                }
            }
        }
        result
    }

    pub trait Rel {
        type K;

        fn intersect<'a>(&self, rels: &[&'a dyn Rel<K = Self::K>]) ->
            Box<dyn Iterator<Item = &'a Self::K>>;

        // fn get<'a>(&'a self, k: &'a Self::K) -> Option<&'a dyn Rel>;

        fn len(&self) -> usize;
    }

    // this is really a trie node
    // K is the type of the key
    // R is the type of each child, usually another trie or unit
    pub trait Trie<K, R> {
        fn create(r: impl Iterator<Item = (K, R)>) -> Self;

        // TODO intersect should also handle more than 2 tries
        // NOTE which cannot be done with the current structure
        // unless I generate a bunch of intersects
        fn intersect<'a, S, T: Trie<K, S>>(&'a self, other: &'a T) ->
            Box<dyn Iterator<Item = (&'a K, &'a R, &'a S)> + 'a>;

        fn get<'a>(&'a self, k: &'a K) -> Option<&'a R>;

        fn len(&self) -> usize;
    }

    // TODO refactor this?
    fn build_trie<'a, T0, T1>(r: &'a[(Value, Value)]) -> T1
    where
        T0: Trie<Value, ()>,
        T1: Trie<Value, T0>,
    {
        let mut r_m = HashMap::new();
        for (x, y) in r {
            let ys = r_m.entry(*x).or_insert_with(Vec::new);
            ys.push((*y, ()));
        }
        T1::create(r_m.iter().map(|(x, ys)| {
            (*x, T0::create(ys.iter().copied()))
        }))
    }

    pub fn triangle_idx<'a, R, F, T0, T1>(
        r: &'a [(Value, Value)],
        s: &'a [(Value, Value)],
        t: &'a [(Value, Value)],
        agg: F,
    ) -> R
    where
        T0: Trie<Value, ()>,
        T1: Trie<Value, T0>,
        R: Default,
        F: Fn(&mut R, (&Value, &Value, &Value)),
    {
        let mut result = R::default();

        let r_x: T1 = build_trie(r);
        let s_y: T1 = build_trie(s);
        let t_rev: Vec<_> = t.iter().copied().map(|(z, x)| (x, z)).collect();
        let t_x: T1 = build_trie(&t_rev);

        for (a, ra, ta) in r_x.intersect(&t_x) {
            for (b, sb, _rab) in s_y.intersect(ra) {
                for (c, _sbc,_tac) in sb.intersect(ta) {
                    agg(&mut result, (a, b, c));
                }
            }
        }
        result
    }
}
