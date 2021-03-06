use std::rc::Rc;
use std::cell::RefCell;

use super::*;
use std::collections::BTreeMap;

pub(crate) type NamespaceRc = Rc<RefCell<Namespace>>;

#[derive(Clone)]
pub(crate) struct Namespace {
    model: ModelRc,
    parent: Option<NamespaceRc>,
    name: String,
    namespaces: BTreeMap<String, NamespaceRc>,
    // pub(crate) topics: BTreeMap<TopicKey, TopicRc>,
}

impl Namespace {
    pub(crate) fn new(wiki: &WikiRc, parent: Option<&NamespaceRc>, name: &str) -> Self {
        Self {
            model: wiki.clone(),
            parent: parent.map(|namespace_rc| namespace_rc.clone()),
            name: name.to_string(),
            namespaces: Default::default(),
            // topics: Default::default(),
        }
    }
}

