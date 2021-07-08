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
    TextUnresolved {
        text: String,
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
        text: String,
    },
}

impl Paragraph {
    pub fn new_text_unresolved(text: &str) -> Self {
        Paragraph::TextUnresolved { text: text.to_string() }
    }

    pub fn new_unknown(text: &str) -> Self {
        Paragraph::Unknown { text: text.to_string() }
    }
}