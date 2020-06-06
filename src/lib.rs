pub mod hashed;
pub mod sorted;
pub mod util;

pub mod relation {
    use seahash::SeaHasher;

    type HashMap<K, V> = std::collections::HashMap<
            K, V, std::hash::BuildHasherDefault<SeaHasher>>;

    pub trait IdxRel<T: 'static, Res: 'static> {

        fn create<'a, I, F>(r: I, index_on: F) -> Self
        where I: Iterator<Item = &'a T>,
              F: Fn(T) -> (u32, Res);

        fn len(&self) -> usize;

        fn get(&self, a: T) -> Option<&Vec<Res>>;

        fn intersect
            <'a, ORes: 'static>
            (&'a self, other: &'a impl IdxRel<T, ORes>) ->
            Box<dyn Iterator<Item = (&'a u32, &'a Vec<Res>, &'a Vec<ORes>)> + 'a>;
    }

    impl<Res: 'static> IdxRel<u32, Res> for HashMap<u32, Vec<Res>> {
        fn intersect
            <'a, ORes: 'static>
            (&'a self, other: &'a impl IdxRel<u32, ORes>) ->
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

        fn create<'a, I, F>(r: I, index_on: F) -> Self
        where I: Iterator<Item = &'a u32>,
              F: Fn(u32) -> (u32, Res)
        {
            let mut r_x = HashMap::default();
            for e in r.copied() {
                let (x, y) = index_on(e);
                let ys = r_x.entry(x).or_insert_with(Vec::new);
                ys.push(y);
            }
            r_x
        }
    }
}
