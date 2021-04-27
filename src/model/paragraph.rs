use super::*;
use std::rc::Rc;
use std::cell::RefCell;

pub type ParagraphRc = Rc<RefCell<Paragraph>>;

pub enum Paragraph {
    Text {
        blocks: Vec<TextBlock>
    },
    List {
        type_: ListType,
        label: Option<String>,
        items: Vec<ListItem>,
    },
    Category {
        category: CategoryRc
    },
    Breadcrumbs {
        breadcrumbs: Breadcrumbs,
    }

}
