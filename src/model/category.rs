use std::rc::Rc;
use std::cell::RefCell;

// use super::*;

pub type CategoryRc = Rc<RefCell<Category>>;

pub struct Category {
    // pub wiki: WikiRc,
    pub parent: Option<CategoryRc>,
    pub name: String,
}

impl Category {
    // pub fn new(wiki: &WikiRc, parent: Option<&CategoryRc>, name: &str) -> Self {
    pub fn new(parent: Option<&CategoryRc>, name: &str) -> Self {
        Self {
            // wiki: wiki.clone(),
            parent: parent.map(|category_rc| category_rc.clone()),
            name: name.to_string(),
        }
    }
}