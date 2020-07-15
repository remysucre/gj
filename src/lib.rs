pub mod hashed;
pub mod sorted;
pub mod util;
pub mod experiments;

pub mod relation {
    use crate::util::Value;

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
}
