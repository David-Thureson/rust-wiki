use super::*;
use std::cmp::Ordering;
use std::collections::BTreeMap;

// pub type TopicRc = Rc<RefCell<Topic>>;
pub type TopicKey = (String, String);
pub type SectionKey = (TopicKey, String);

pub struct Topic {
    pub parents: Vec<TopicKey>,
    pub namespace: String,
    pub name: String,
    pub category: Option<String>,
    pub attributes: BTreeMap<String, Vec<String>>,
    pub paragraphs: Vec<Paragraph>,
    //pub sections: Vec<SectionRc>,
    pub sections: BTreeMap<String, usize>,
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
            sections: Default::default(),
        }
    }

    pub fn get_key(&self) -> TopicKey {
        Self::make_key(&self.namespace, &self.name)
    }

    pub fn make_key(namespace_name: &str, topic_name: &str) -> TopicKey {
        (namespace_name.to_lowercase().to_string(), topic_name.to_lowercase().to_string())
    }

    pub fn make_section_key(namespace_name: &str, topic_name: &str, section_name: &str) -> SectionKey {
        (Self::make_key(namespace_name, topic_name), section_name.to_lowercase().to_string())
    }

    pub fn add_paragraph(&mut self, paragraph: Paragraph) {
        self.paragraphs.push(paragraph);
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



