// Generate worst-case graph. See slide #13 of Hung Ngo's talk "Worst-case
// optimal join algorithms: techniques, results, and open problems"
// http://www.cse.buffalo.edu/~hungngo/papers/wcoj-gems.pptx
use rand::{thread_rng, seq::SliceRandom};

type Rel = Vec<(u32, u32)>;

pub fn gen_worst_case_relations(n: u32) -> (Rel, Rel, Rel) {
    assert!(n > 0);
    // 3N nodes
    let xs: Vec<_> = (0..n).collect();
    let ys: Vec<_> = (n..2 * n).collect();
    let zs: Vec<_> = (2 * n..3 * n).collect();

    // The edges
    let mut r = vec![];
    let mut s = vec![];
    let mut t = vec![];
    for i in 0..n as usize {
        r.push((xs[0], ys[i]));
        s.push((ys[0], zs[i]));
        t.push((zs[0], xs[i]));
    }
    for i in 1..n as usize {
        r.push((xs[i], ys[0]));
        s.push((ys[i], zs[0]));
        t.push((zs[i], xs[0]));
    }

    let mut rng = thread_rng();
    r.shuffle(&mut rng);
    s.shuffle(&mut rng);
    t.shuffle(&mut rng);

    // Graph with 3 "typed" sets of edges
    (r, s, t)
}

// Read graph from TSV file containing an edge per line
pub fn read_edges(n: usize) -> Result<Vec<(u32, u32)>, Box<dyn std::error::Error>> {
    use csv::ReaderBuilder;
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(std::io::stdin());
    let mut es = vec![];
    for rec in rdr.records().take(n) {
        let record = rec?;
        es.push((record[0].parse().unwrap(), record[1].parse().unwrap()));
    }
    Ok(es)
}

pub fn partition(es: &[(u32, u32)]) -> (Vec<(u32, u32)>, Vec<(u32, u32)>, Vec<(u32, u32)>) {
    let l = es.len();
    (es[0..l/100].to_vec(), es[l/100..l/2].to_vec(), es[l/2..].to_vec())
}
