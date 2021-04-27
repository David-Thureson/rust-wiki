use std::rc::Rc;
use std::cell::RefCell;

use super::*;

pub type WikiRc = Rc<RefCell<Wiki>>;

pub struct Wiki {
    pub name: String,
    pub namespaces: Vec<NamespaceRc>,
    pub topics: Vec<TopicRc>,
    pub categories: Vec<CategoryRc>,
    pub attributes: Vec<AttributeRc>,
}