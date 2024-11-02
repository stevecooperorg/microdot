use crate::util::generate_hash;
use std::fmt::Display;
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

impl Display for HashTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tag)
    }
}
