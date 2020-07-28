use gj::experiments::*;
// use gj::util::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: usize = args[1].parse().unwrap();

    live_journal(n as u64)
}
