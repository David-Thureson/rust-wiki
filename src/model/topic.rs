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
    pub inbound_topic_keys: Vec<TopicKey>,
    pub outbound_links: Vec<Link>,
    pub subtopics: Vec<TopicKey>,
    pub combo_subtopics: Vec<TopicKey>,
    pub listed_topics: Vec<TopicKey>,
    //pub sections: Vec<SectionRc>,
    // pub sections: BTreeMap<String, usize>,
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
        Self::make_key(&self.namespace, &self.name)
    }

    pub fn make_key(namespace_name: &str, topic_name: &str) -> TopicKey {
        (namespace_name.to_lowercase().to_string(), topic_name.to_lowercase().to_string())
    }

    pub fn make_section_key(namespace_name: &str, topic_name: &str, section_name: &str) -> SectionKey {
        (Self::make_key(namespace_name, topic_name), section_name.to_lowercase().to_string())
    }

    pub fn section_key_to_topic_key(section_key: &SectionKey) -> TopicKey {
        section_key.0.clone()
    }

    pub fn topic_key_to_string(topic_key: &TopicKey) -> String {
        // format!("[{} | {}]", topic_key.0, topic_key.1)
        format!("[{}]", topic_key.1)
    }

    pub fn section_key_to_string(section_key: &SectionKey) -> String {
        // format!("[{} | {} # {}]", &section_key.0.0, &section_key.0.1, &section_key.1)
        format!("[{} # {}]", &section_key.0.1, &section_key.1)
    }

    pub fn add_paragraph(&mut self, paragraph: Paragraph) {
        self.paragraphs.push(paragraph);
    }

    pub fn has_section(&self, section_name: &str) -> bool {
        let section_name = section_name.to_lowercase();
        // let debug = section_name == "runtime communication between a python and java app";
        // if debug { dbg!(&self.name); }
        for paragraph in self.paragraphs.iter() {
            match paragraph {
                Paragraph::SectionHeader { name, .. } => {
                    // if debug { dbg!(&name); }
                    if name.to_lowercase() == section_name {
                        return true;
                    }
                },
                _ => {},
            }
        }
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

