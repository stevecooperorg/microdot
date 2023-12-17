use regex::Regex;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq, Debug)]
pub enum HashState {
    None,
    Hashed(usize),
}

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

#[allow(dead_code)]
fn hashtag_signature(input: &str) -> HashState {
    let (hashes, _) = extract_hashtags(input);
    if hashes.is_empty() {
        return HashState::None;
    }

    let combo = hashes
        .into_iter()
        .map(|hash| hash.tag)
        .collect::<Vec<_>>()
        .join("");
    let hash = generate_hash(&combo);
    HashState::Hashed(hash)
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

    #[test]
    fn it_calculates_hashtag_signatures() {
        fn eq(a: &str, b: &str) {
            let ai = hashtag_signature(a);
            let bi = hashtag_signature(b);
            assert_eq!(
                ai, bi,
                "signatures are not the same for '{}' and '{}'",
                a, b
            )
        }

        fn ne(a: &str, b: &str) {
            let ai = hashtag_signature(a);
            let bi = hashtag_signature(b);
            assert_ne!(ai, bi, "signatures are the same for '{}' and '{}'", a, b)
        }

        assert_eq!(HashState::None, hashtag_signature("no hashtags here"));
        assert_ne!(HashState::None, hashtag_signature("hashtag! #HASH"));
        eq("a #HASH", "b #HASH");
        eq("#HASH a", "a #HASH");
        eq("#A #B", "#B #A");
        eq("#A #A", "#A");
        ne("#HASHA a", "a #HASHB");
    }
}
