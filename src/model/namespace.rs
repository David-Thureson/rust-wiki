use std::rc::Rc;
use std::cell::RefCell;

//use super::*;

pub type NamespaceRc = Rc<RefCell<Namespace>>;

pub struct Namespace {
    pub parent: Option<NamespaceRc>,
    pub name: String,
    pub namespaces: Vec<NamespaceRc>,
}