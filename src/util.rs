use crate::Val;

pub fn read_es(n: usize) -> Result<Vec<Vec<Val>>, Box<dyn std::error::Error>> {
    use csv::ReaderBuilder;
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(std::io::stdin());
    let mut es = vec![];
    for rec in rdr.records().take(n) {
        let record = rec?;
        es.push(vec![
            Val::Int(record[0].parse().unwrap()),
            Val::Int(record[1].parse().unwrap()),
        ]);
    }
    Ok(es)
}
