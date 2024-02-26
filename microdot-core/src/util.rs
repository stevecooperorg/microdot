use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn generate_hash(input: &str) -> usize {
    let mut s = DefaultHasher::new();
    input.hash(&mut s);
    s.finish() as usize
}
