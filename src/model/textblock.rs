use std::rc::Rc;
use std::cell::RefCell;

use super::*;

pub type TextBlockRc = Rc<RefCell<TextBlock>>;

#[derive(Clone)]
pub enum TextBlock {
    Text {
        text: String,
    },
    Link {
        link: Link,
    },
}