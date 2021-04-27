use std::rc::Rc;
use std::cell::RefCell;

use super::*;

pub type AttributeRc = Rc<RefCell<Attribute>>;
pub type AttributeValueRc = Rc<RefCell<AttributeValue>>;

pub enum AttributeType {
    Date,
    Choice,
}

pub struct Attribute {
    pub wiki: WikiRc,
    pub type_: AttributeType,
    pub name: String,
    pub values: Vec<AttributeValueRc>,
}

pub struct AttributeValue {
    pub name: String,
}