use gj::experiments::*;
use gj::util::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: usize = args[1].parse().unwrap();

    let es0 = read_edges(n).unwrap();

    for sample in vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
        for cross in vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
            println!("sample {} cross {}", sample, cross);
            community(es0.clone(), sample, cross)
        }
    }
}
