use std::rc::Rc;
use std::cell::RefCell;

use super::*;

pub type NamespaceRc = Rc<RefCell<Namespace>>;

pub struct Namespace {
    pub parent: Option<NamespaceRc>,
    pub name: String,
    pub namespaces: Vec<NamespaceRc>,
    pub topics: Vec<TopicRc>,
}

impl Namespace {
    pub fn new(parent: Option<NamespaceRc>, name: &str) -> Self {
        Self {
            parent,
            name: name.to_string(),
            namespaces: vec![],
            topics: vec![]
        }
    }
}

