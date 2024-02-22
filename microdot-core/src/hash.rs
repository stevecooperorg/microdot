use regex::Regex;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct HashTag {
    tag: String,
}

impl HashTag {
    pub fn hash(&self) -> usize {
        generate_hash(&self.tag)
    }
}

impl ToString for HashTag {
    fn to_string(&self) -> String {
        self.tag.clone()
    }
}

pub fn extract_hashtags(input: &str) -> (Vec<HashTag>, String) {
    let rx = Regex::new("#[A-Za-z][A-Za-z0-9_-]*").expect("not a regex");
    let mut hashes = HashSet::new();
    for hash in rx.captures_iter(input) {
        let hash = hash.get(0).unwrap().as_str().to_string();
        hashes.insert(hash);
    }

    // trim any trailing hashtags, since they'll be immediately displayed underneath.
    let mut work_done = true;
    let mut new_label = input.to_string();

    while work_done {
        new_label = new_label.trim().to_string();
        work_done = false;
        for hash in hashes.iter() {
            if new_label.ends_with(hash) {
                let split_at = new_label.len() - hash.len();
                //println!("found '{}' at the end of '{}', splitting from {}", hash, new_label,
                // split_at);
                new_label = new_label[..split_at].to_string();
                work_done = true;
            }
        }
    }
    let mut hashes: Vec<_> = hashes.into_iter().collect();
    hashes.sort();

    let hashtags = hashes.into_iter().map(|tag| HashTag { tag }).collect();
    (hashtags, new_label)
}

fn generate_hash(input: &str) -> usize {
    let mut s = DefaultHasher::new();
    input.hash(&mut s);
    s.finish() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_extracts_hashtags() {
        let actual = extract_hashtags("no hashtags");
        assert_eq!(actual, (vec![], "no hashtags".to_string()));

        let actual = extract_hashtags("a #hashtag in the middle");
        assert_eq!(
            actual,
            (
                vec![HashTag {
                    tag: "#hashtag".to_string()
                }],
                "a #hashtag in the middle".to_string()
            )
        );

        let actual = extract_hashtags("a #hashtag at the #end");
        assert_eq!(
            actual,
            (
                vec![
                    HashTag {
                        tag: "#end".to_string()
                    },
                    HashTag {
                        tag: "#hashtag".to_string()
                    }
                ],
                "a #hashtag at the".to_string()
            )
        );
    }
}
