use std::rc::Rc;
use std::cell::RefCell;

use super::*;

pub type CategoryRc = Rc<RefCell<Category>>;

pub struct Category {
    pub wiki: WikiRc,
    pub parent: Option<CategoryRc>,
    pub name: String,
}
