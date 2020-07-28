pub mod trie;
pub mod util;
pub mod experiments;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Val {
    Int(u64),
    Str(String),
    Boo(bool),
}
