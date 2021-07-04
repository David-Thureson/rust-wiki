use std::rc::Rc;
use std::cell::RefCell;

use crate::*;
use super::*;
use std::cmp::Ordering;

pub type TopicRc = Rc<RefCell<Topic>>;
pub type TopicKey = (String, String, String);

pub struct Topic {
    pub wiki: WikiRc,
    pub parent: Option<TopicRc>,
    pub namespace: NamespaceRc,
    pub name: String,
    pub file_name: String,
    pub category: Option<CategoryRc>,
    pub section: SectionRc,
}

impl Topic {
    pub fn get_key(&self) -> TopicKey {
        (b!(&self.wiki).name.clone(), b!(&self.namespace).name.clone(), self.name.clone())
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



