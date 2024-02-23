use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct HashTag {
    tag: String,
}

impl HashTag {
    pub fn hash(&self) -> usize {
        generate_hash(&self.tag)
    }

    pub fn new(tag: impl Into<String>) -> HashTag {
        HashTag {
            tag: tag.into(),
        }
    }
}

impl ToString for HashTag {
    fn to_string(&self) -> String {
        self.tag.clone()
    }
}


fn generate_hash(input: &str) -> usize {
    let mut s = DefaultHasher::new();
    input.hash(&mut s);
    s.finish() as usize
}
