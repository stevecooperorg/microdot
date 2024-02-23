use std::collections::HashSet;
use regex::Regex;
use crate::graph::Variable;
use crate::hash::HashTag;
use crate::Label;

#[derive(Debug, PartialEq, Clone)]
pub struct NodeInfo {
    pub label: String,
    pub tags: Vec<HashTag>,
    pub variables: Vec<Variable>,
    pub subgraph: Option<HashTag>
}

impl NodeInfo {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
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

fn extract_hashtags(input: impl AsRef<str>) -> (Vec<HashTag>, String) {
    let input = input.as_ref();
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
    fn it_parses_node_label_with_no_markup() {
        let actual = NodeInfo::parse(&Label("no hashtags".to_string()));
        let expected = NodeInfo {
            label: "no hashtags".to_string(),
            tags: Vec::new(),
            variables: Vec::new(),
            subgraph: None
        };
        assert_eq!(actual, expected);

    }
    #[test]
    fn it_parses_node_label_with_inner_hashtag() {

        let actual = NodeInfo::parse(&Label("a #hashtag in the middle".to_string()));
        let expected = NodeInfo {
            label: "a #hashtag in the middle".to_string(),
            tags: vec![HashTag::new("#hashtag")],
            variables: Vec::new(),
            subgraph: None
        };
        assert_eq!(actual, expected);
    }
    #[test]
    fn it_parses_node_label_with_end_hashtag() {

        let actual = NodeInfo::parse(&Label("a #hashtag at the #end".to_string()));
        let expected = NodeInfo {
            label: "a #hashtag at the".to_string(),
            tags: vec![HashTag::new("#end"), HashTag::new("#hashtag")],
            variables: Vec::new(),
            subgraph: None
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_parses_node_label_with_end_subgraph() {

        let actual = NodeInfo::parse(&Label("a #hashtag in the middle and a #SG_SUBGRAPH".to_string()));
        let expected = NodeInfo {
            label: "a #hashtag in the middle and a".to_string(),
            tags: vec![HashTag::new("#hashtag")],
            variables: Vec::new(),
            subgraph: Some(HashTag::new("#SG_SUBGRAPH"))
        };
        assert_eq!(actual, expected);
    }
}