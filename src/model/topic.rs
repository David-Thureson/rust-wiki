use std::rc::Rc;
use std::cell::RefCell;

use crate::*;
use super::*;
use std::cmp::Ordering;
use std::collections::BTreeMap;

pub type TopicRc = Rc<RefCell<Topic>>;
pub type TopicKey = (String, String);

pub struct Topic {
    pub wiki: WikiRc,
    pub parents: Vec<TopicRc>,
    pub namespace: NamespaceRc,
    pub name: String,
    pub category: Option<CategoryRc>,
    pub attributes: AttributeValueList,
    pub paragraphs: Vec<ParagraphRc>,
    //pub sections: Vec<SectionRc>,
    pub sections: BTreeMap<String, usize>,
    pub errors: Vec<String>,
}

impl Topic {
    pub fn new(wiki: &WikiRc, namespace: &NamespaceRc, name: &str) -> Self {
        Self {
            wiki: wiki.clone(),
            parents: vec![],
            namespace: namespace.clone(),
            name: name.to_string(),
            category: None,
            attributes: AttributeValueList::new(),
            paragraphs: vec![],
            sections: Default::default(),
            errors: vec![]
        }
    }

    pub fn get_key(&self) -> TopicKey {
        Self::make_key(&b!(&self.namespace).name, &self.name)
    }

    pub fn make_key(namespace_name: &str, topic_name: &str) -> TopicKey {
        (namespace_name.to_lowercase().to_string(), topic_name.to_lowercase().to_string())
    }

    pub fn add_paragraph(&mut self, paragraph: Paragraph) {
        self.paragraphs.push(r!(paragraph));
    }

    pub fn add_error(&mut self, msg: &str) {
        self.errors.push(msg.to_string());
    }

    pub fn print_errors(&self) {
        if !self.errors.is_empty() {
            println!("\t{}", self.name);
            self.errors.iter().for_each(|msg| { println!("\t{}", msg)});
        }
    }
}

impl PartialEq for Topic {
    fn eq(&self, other: &Self) -> bool {
        self.get_key() == other.get_key()
    }
    /*
    fn eq(&self, other: &Self) -> bool {
        b!(&self.wiki).name == b!(&other.wiki).name
            && b!(&self.namespace).name == b!(&other.namespace).name
            && self.name == other.name
    }
     */
}

impl Eq for Topic {
}

impl PartialOrd for Topic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_key().partial_cmp(&other.get_key())
    }
    /*
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (b!(&self.wiki).name, b!(&self.namespace).name, &self.name)
            .partial_cmp(&(b!(&other.wiki).name, b!(&other.namespace).name, &other.name))
    }

     */
}

impl Ord for Topic {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_key().cmp(&other.get_key())
    }
    /*
    fn cmp(&self, other: &Self) -> Ordering {
        (b!(&self.wiki).name, b!(&self.namespace).name, &self.name)
            .cmp(&(b!(&other.wiki).name, b!(&other.namespace).name, &other.name))
    }
     */
}



