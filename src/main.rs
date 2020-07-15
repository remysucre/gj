use gj::experiments::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n = args[1].parse().unwrap();

    community(n)
}
