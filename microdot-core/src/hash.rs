use crate::util::generate_hash;
use std::hash::Hash;

#[derive(PartialEq, Eq, Debug, Hash, Clone, Ord, PartialOrd)]
pub struct HashTag {
    tag: String,
}

impl HashTag {
    pub fn hash(&self) -> usize {
        generate_hash(&self.tag)
    }

    pub fn new(tag: impl Into<String>) -> HashTag {
        HashTag { tag: tag.into() }
    }
}

impl ToString for HashTag {
    fn to_string(&self) -> String {
        self.tag.clone()
    }
}
