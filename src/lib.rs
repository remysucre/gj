pub mod hashed;
pub mod sorted;
pub mod util;

pub mod relation {
    use seahash::SeaHasher;

    type HashMap<K, V> = std::collections::HashMap<
            K, V, std::hash::BuildHasherDefault<SeaHasher>>;

    pub trait IdxRel<Res: 'static> {

        fn create<'a>(r: impl Iterator<Item = &'a (u32, Res)>) -> Self;

        fn len(&self) -> usize;

        fn get(&self, a: u32) -> Option<&Vec<Res>>;

        fn intersect
            <'a, ORes: 'static>
            (&'a self, other: &'a impl IdxRel<ORes>) ->
            Box<dyn Iterator<Item = (&'a u32, &'a Vec<Res>, &'a Vec<ORes>)> + 'a>;
    }

    impl<Res: 'static + Copy> IdxRel<Res> for HashMap<u32, Vec<Res>> {
        fn intersect
            <'a, ORes: 'static>
            (&'a self, other: &'a impl IdxRel<ORes>) ->
            Box<dyn Iterator<Item = (&'a u32, &'a Vec<Res>, &'a Vec<ORes>)> + 'a>
        {
            if self.len() <= other.len() {
                Box::new(self.iter().filter_map(move |(a, y)| {
                    other.get(*a).map(|z| (a, y, z))
                }))
            } else {
                Box::new(other.intersect(self).map(|(a, x, y)| (a, y, x)))
            }
        }

        fn len(&self) -> usize {
            self.keys().len()
        }

        fn get(&self, a: u32) -> Option<&Vec<Res>> {
            self.get(&a)
        }

        fn create<'a>(r: impl Iterator<Item = &'a (u32, Res)>) -> Self
        {
            let mut r_x = HashMap::default();
            for (x, y) in r.copied() {
                let ys = r_x.entry(x).or_insert_with(Vec::new);
                ys.push(y);
            }
            r_x
        }
    }

    // pub fn triangle<'a, R: Default, F: Fn(&mut R, (&u32, &u32, &u32))>(
    //     r: &'a [(u32, u32)],
    //     s: &'a [(u32, u32)],
    //     t: &'a [(u32, u32)],
    //     agg: F,
    // ) -> R {
    //     let r_x =
    //
    //     let mut r_x = HashMap::default();

    //     for (x, y) in r {
    //         let ys = r_x.entry(x).or_insert_with(HashSet::default);
    //         ys.insert(y);
    //     }
    //     // hash-join t with r on x
    //     // t_x[a] is the residual relation t(z, a)
    //     let mut t_x = HashMap::default();
    //     for (z, x) in t {
    //         if r_x.contains_key(&x) {
    //             let zs = t_x.entry(x).or_insert_with(HashSet::default);
    //             zs.insert(z);
    //         }
    //     }
    //     // building this hash outside the loop
    //     let mut s_y = HashMap::default();
    //     let mut s_y_keys = HashSet::default();
    //     for (y, z) in s {
    //         let zs = s_y.entry(y).or_insert_with(HashSet::default);
    //         zs.insert(z);
    //         s_y_keys.insert(y);
    //     }
    //     let mut result = R::default();
    //     // now we have hash-joined r and t, and t_x.keys = intersect(r.x, t.x)
    //     for (a, t_a) in t_x.iter() {
    //         let r_a = r_x.get(a).expect("t_x.x not found in r_x");
    //         // join s and r_a
    //         // s_y[b] is the residual relation s(b, z)
    //         for b in r_a.intersection(&s_y_keys) {
    //             for c in s_y[b].intersection(t_a) {
    //                 agg(&mut result, (*a, *b, *c));
    //             }
    //         }
    //     }
    //     result
    // }
}
