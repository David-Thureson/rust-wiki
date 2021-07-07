use std::rc::Rc;
use std::cell::RefCell;

use crate::*;
use super::*;
use std::collections::BTreeMap;

pub type WikiRc = Rc<RefCell<Wiki>>;

pub struct Wiki {
    pub name: String,
    pub namespaces: BTreeMap<String, NamespaceRc>,
    pub topics: BTreeMap<TopicKey, TopicRc>,
    pub categories: BTreeMap<String, CategoryRc>,
    pub attributes: BTreeMap<String, AttributeRc>,
}

impl Wiki {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            namespaces: Default::default(),
            topics: Default::default(),
            categories: Default::default(),
            attributes: Default::default()
        }
    }

    pub fn add_namespace(&mut self, namespace_rc: &NamespaceRc) {
        let key = b!(&namespace_rc).name.clone();
        assert!(!self.namespaces.contains_key(&key));
        self.namespaces.insert(key, namespace_rc.clone());
    }

    pub fn add_topic(&mut self, topic_rc: &TopicRc) {
        // assert!(self.namespaces.contains_key(&b!(&b!(&topic_rc).namespace).name));
        assert!(self.namespaces.contains_key(&b2!(&topic_rc, namespace).name));
        let key = b!(&topic_rc).get_key();
        assert!(!self.topics.contains_key(&key));
        self.topics.insert(key, topic_rc.clone());
    }
}