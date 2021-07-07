use super::*;
use std::rc::Rc;
use std::cell::RefCell;

pub type ParagraphRc = Rc<RefCell<Paragraph>>;

pub enum Paragraph {
    SectionHeader {
        name: String,
        depth: usize,
    },
    Text {
        lines: Vec<Vec<TextBlock>>,
    },
    List {
        type_: ListType,
        label: Option<String>,
        items: Vec<ListItem>,
    },
    Category {
        category: CategoryRc,
    },
    Breadcrumbs {
        breadcrumbs: Breadcrumbs,
    },
    Attributes {
        attributes: Vec<AttributeRc>,
    },
    Quote {

    },
    Unknown {
        lines: Vec<String>,
    },
}

