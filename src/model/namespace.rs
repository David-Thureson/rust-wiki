use std::rc::Rc;
use std::cell::RefCell;

use super::*;
use std::collections::BTreeMap;

pub type NamespaceRc = Rc<RefCell<Namespace>>;

pub struct Namespace {
    pub wiki: WikiRc,
    pub parent: Option<NamespaceRc>,
    pub name: String,
    pub namespaces: BTreeMap<String, NamespaceRc>,
    // pub topics: BTreeMap<TopicKey, TopicRc>,
}

impl Namespace {
    pub fn new(wiki: &WikiRc, parent: Option<&NamespaceRc>, name: &str) -> Self {
        Self {
            wiki: wiki.clone(),
            parent: parent.map(|namespace_rc| namespace_rc.clone()),
            name: name.to_string(),
            namespaces: Default::default(),
            // topics: Default::default(),
        }
    }
}

