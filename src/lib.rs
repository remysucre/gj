pub mod hashed;
pub mod sorted;
pub mod util;

pub mod relation {
    use seahash::SeaHasher;

    type HashMap<K, V> = std::collections::HashMap<
            K, V, std::hash::BuildHasherDefault<SeaHasher>>;

    pub trait IdxRel {
        type Res: 'static;

        fn create(r: impl Iterator<Item = (u32, Self::Res)>) -> Self;

        fn len(&self) -> usize;

        fn get(&self, a: u32) -> Option<&Vec<Self::Res>>;

        fn intersect
            <'a, O: IdxRel>
            (&'a self, other: &'a O) ->
            Box<dyn Iterator<Item = (&'a u32, &'a Vec<Self::Res>, &'a Vec<O::Res>)> + 'a>;
    }

    impl<R: 'static + Copy> IdxRel for HashMap<u32, Vec<R>> {
        type Res = R;

        fn len(&self) -> usize {
            self.keys().len()
        }

        fn get(&self, a: u32) -> Option<&Vec<Self::Res>> {
            self.get(&a)
        }

        fn create(r: impl Iterator<Item = (u32, Self::Res)>) -> Self
        {
            let mut r_x = HashMap::default();
            for (x, y) in r {
                let ys = r_x.entry(x).or_insert_with(Vec::new);
                ys.push(y);
            }
            r_x
        }

        fn intersect
            <'a, O: IdxRel>
            (&'a self, other: &'a O) ->
            Box<dyn Iterator<Item = (&'a u32, &'a Vec<Self::Res>, &'a Vec<O::Res>)> + 'a>
        {
            if self.len() <= other.len() {
                Box::new(self.iter().filter_map(move |(a, y)| {
                    other.get(*a).map(|z| (a, y, z))
                }))
            } else {
                Box::new(other.intersect(self).map(|(a, x, y)| (a, y, x)))
            }
        }

    }

    pub fn triangle<'a, Rx, Ry, Sy, Sz, Tz, Tx, O, F>(
        r: &'a [(u32, u32)],
        s: &'a [(u32, u32)],
        t: &'a [(u32, u32)],
        agg: F,
    ) -> O
    where
        Rx: IdxRel<Res = u32>,
        Ry: IdxRel<Res = ()>,
        Sy: IdxRel<Res = u32>,
        Sz: IdxRel<Res = ()>,
        Tz: IdxRel<Res = ()>,
        Tx: IdxRel<Res = u32>,
        O: Default,
        F: Fn(&mut O, (&u32, &u32, &u32)),
    {
        let mut result = O::default();
        let r_x = Rx::create(r.iter().copied());
        let t_x = Tx::create(t.iter().copied());
        let s_y = Sy::create(s.iter().copied());
        for (a, ra, _ta) in r_x.intersect(&t_x) {
            let ra_y = Ry::create(ra.iter().copied().map(|n| (n, ())));
            let ta_z = Tz::create(t_x.get(*a).unwrap_or(&vec![]).iter().copied().map(|n| (n, ())));
            for (b, sb, _rab) in s_y.intersect(&ra_y) {
                let sb_z = Sz::create(sb.iter().copied().map(|n| (n, ())));
                for (c, _sbc, _tac) in sb_z.intersect(&ta_z) {
                    agg(&mut result, (a, b, c));
                }
            }
        }
        result
    }

    pub fn triangle_hash<'a, R: Default, F: Fn(&mut R, (&u32, &u32, &u32))>(
    r: &'a [(u32, u32)],
    s: &'a [(u32, u32)],
    t: &'a [(u32, u32)],
    agg: F,
    ) -> R {
        triangle::
        <HashMap<u32, Vec<u32>>,
         HashMap<u32, Vec<()>>,
         HashMap<u32, Vec<u32>>,
         HashMap<u32, Vec<()>>,
         HashMap<u32, Vec<()>>,
         HashMap<u32, Vec<u32>>,
         R, F>(r, s, t, agg)
    }
}
