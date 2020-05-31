// Generate worst-case graph. See slide #13 of Hung Ngo's talk "Worst-case
// optimal join algorithms: techniques, results, and open problems"
// http://www.cse.buffalo.edu/~hungngo/papers/wcoj-gems.pptx
pub fn gen_worst_case_relations(n: u32) ->
    (Vec<(u32, u32)>, Vec<(u32, u32)>, Vec<(u32, u32)>)
{
    assert!(n > 0);
    // 3N nodes
    let x: Vec<_> = (0..n).collect();
    let y: Vec<_> = (n..2*n).collect();
    let z: Vec<_> = (2*n..3*n).collect();

    // The edges
    let mut r = vec![];
    let mut s = vec![];
    let mut t = vec![];
    for i in 0..n as usize {
        r.push((x[0], y[i]));
        s.push((y[0], z[i]));
        t.push((z[0], x[i]));
    }
    for i in 1..n as usize {
        r.push((x[i], y[0]));
        s.push((y[i], z[0]));
        t.push((z[i], x[0]));
    }

    // Graph with 3 "typed" sets of edges
    (r, s, t)
}

// Read graph from TSV file containing an edge per line
pub fn read_edges() -> Result<Vec<(u32, u32)>, Box<dyn std::error::Error>> {
    use csv::ReaderBuilder;
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(std::io::stdin());
    let mut es = vec![];
    for rec in rdr.records() {
        let record = rec?;
        es.push((
            record[0].parse().unwrap(),
            record[1].parse().unwrap()
        ));
    }
    Ok(es)
}
