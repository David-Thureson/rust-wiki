use std::rc::Rc;
use std::cell::RefCell;

use super::*;

pub type TopicRc = Rc<RefCell<Topic>>;

pub struct Topic {
    pub wiki: WikiRc,
    pub parent: Option<TopicRc>,
    pub namespace: NamespaceRc,
    pub title: String,
    pub file_name: String,
    pub category: Option<CategoryRc>,
    pub section: SectionRc,
}