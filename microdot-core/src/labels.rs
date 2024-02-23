use std::collections::HashSet;
use regex::Regex;
use crate::graph::Variable;
use crate::hash::HashTag;
use crate::Label;

pub struct NodeInfo {
    pub label: String,
    pub tags: Vec<HashTag>,
    pub variables: Vec<Variable>,
    pub subgraph: Option<HashTag>
}

impl NodeInfo {
    pub fn new(label: String) -> Self {
        NodeInfo {
            label,
            tags: Vec::new(),
            variables: Vec::new(),
            subgraph: None
        }
    }
    pub fn parse(label: &Label) -> Self {
        let base_label = &label.to_string();

        let (tags, label) = extract_hashtags(base_label);

        let subgraph: Option<HashTag> = tags
            .iter()
            .find(|t| t.to_string().starts_with("#SG_"))
            .cloned();

        let tags: Vec<_> = tags
            .into_iter()
            .filter(|t| !t.to_string().starts_with("#SG_"))
            .collect();


        NodeInfo {
            label,
            tags,
            variables: Vec::new(),
            subgraph
        }
    }
}

fn extract_hashtags(input: &str) -> (Vec<HashTag>, String) {
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

    let hashtags = hashes.into_iter().map(HashTag::new).collect();
    (hashtags, new_label)
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
                vec![HashTag::new("#hashtag")],
                "a #hashtag in the middle".to_string()
            )
        );

        let actual = extract_hashtags("a #hashtag at the #end");
        assert_eq!(
            actual,
            (
                vec![
                    HashTag::new("#end"),
                    HashTag::new("#hashtag")
                ],
                "a #hashtag at the".to_string()
            )
        );
    }
}