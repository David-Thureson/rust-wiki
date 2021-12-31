use super::*;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

pub struct Topic {
    pub parents: Vec<TopicKey>,
    pub namespace: String,
    pub name: String,
    pub category: Option<String>,
    pub attributes: BTreeMap<String, Vec<String>>,
    pub paragraphs: Vec<Paragraph>,
    pub inbound_topic_keys: Vec<TopicKey>,
    pub outbound_links: Vec<Link>,
    pub subtopics: Vec<TopicKey>,
    pub combo_subtopics: Vec<TopicKey>,
    pub listed_topics: Vec<TopicKey>,
    //pub sections: Vec<SectionRc>,
    // pub sections: BTreeMap<String, usize>,
}

#[derive(Clone, Debug)]
pub struct TopicKey {
    pub namespace: String,
    pub topic_name: String,
}

#[derive(Clone, Debug)]
pub struct SectionKey {
    pub topic_key: TopicKey,
    pub section_name: String,
}

impl Topic {
    pub fn new(namespace: &str, name: &str) -> Self {
        Self {
            parents: vec![],
            namespace: namespace.to_string(),
            name: name.to_string(),
            category: None,
            attributes: Default::default(),
            paragraphs: vec![],
            inbound_topic_keys: vec![],
            outbound_links: vec![],
            subtopics: vec![],
            combo_subtopics: vec![],
            listed_topics: vec![],
            // sections: Default::default(),
        }
    }

    pub fn get_key(&self) -> TopicKey {
        TopicKey::new(&self.namespace, &self.name)
    }

    pub fn add_paragraph(&mut self, paragraph: Paragraph) {
        self.paragraphs.push(paragraph);
    }

    pub fn has_section(&self, section_name: &str) -> bool {
        let section_name = section_name.to_lowercase();
        // let debug = section_name.contains("cognitive");
        // if debug { //bg!(&self.name, &section_name); }
        for paragraph in self.paragraphs.iter() {
            match paragraph {
                Paragraph::SectionHeader { name, .. } => {
                    // if debug { //bg!(&name); }
                    if name.to_lowercase() == section_name {
                        // if debug { //bg!("found section"); }
                        return true;
                    }
                },
                _ => {},
            }
        }
        // if debug { //bg!("didn't find section"); }
        false
    }

}

impl PartialEq for Topic {
    fn eq(&self, other: &Self) -> bool {
        self.get_key() == other.get_key()
    }
}

impl Eq for Topic {
}

impl PartialOrd for Topic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_key().partial_cmp(&other.get_key())
    }
}

impl Ord for Topic {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_key().cmp(&other.get_key())
    }
}

impl TopicKey {
    pub fn new(namespace: &str, topic_name: &str) -> Self {
        assert!(!topic_name.contains(":="), "Topic name \"{}\" has a \":=\"", topic_name);
        assert!(!topic_name.starts_with("_"), "Topic name \"{}\" starts with \"_\"", topic_name);
        Self {
            namespace: namespace.to_lowercase().to_string(),
            topic_name: topic_name.to_string(),
        }
    }

    pub fn get_key(&self) -> String {
        self.to_string().to_lowercase()
    }
}

impl Display for TopicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "[{}:{}]", self.namespace, self.topic_name)
    }
}

impl PartialEq for TopicKey {
    fn eq(&self, other: &Self) -> bool {
        self.get_key() == other.get_key()
    }
}

impl Eq for TopicKey {
}

impl PartialOrd for TopicKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_key().partial_cmp(&other.get_key())
    }
}

impl Ord for TopicKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_key().cmp(&other.get_key())
    }
}

impl SectionKey {
    pub fn new(namespace: &str, topic_name: &str, section_name: &str) -> Self {
        assert!(!topic_name.contains(":="), "Topic name \"{}\" has a \":=\"", topic_name);
        assert!(!topic_name.starts_with("_"), "Topic name \"{}\" starts with \"_\"", topic_name);

        Self {
            topic_key: TopicKey::new(namespace, topic_name),
            section_name: section_name.to_string(),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.topic_key.namespace
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_key.topic_name
    }
}

impl Display for SectionKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "[{}:{}#{}]", self.topic_key.namespace, self.topic_key.topic_name, self.section_name)
    }
}

