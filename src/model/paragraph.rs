use super::*;
use std::rc::Rc;
use std::cell::RefCell;

pub type ParagraphRc = Rc<RefCell<Paragraph>>;

#[derive(Clone)]
pub enum Paragraph {
    Attributes,
    Breadcrumbs,
    Category,
    List {
        type_: ListType,
        label: Option<String>,
        items: Vec<ListItem>,
    },
    Quote {

    },
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

    pub fn get_variant_name(&self) -> &str {
        match self {
            Paragraph::Attributes { .. } => "Attributes",
            Paragraph::Breadcrumbs { .. } => "Breadcrumbs",
            Paragraph::Category { .. } => "Category",
            Paragraph::List { .. } => "List",
            Paragraph::Quote { .. } => "Quote",
            Paragraph::SectionHeader { .. } => "SectionHeader",
            Paragraph::Text { .. } => "Text",
            Paragraph::TextUnresolved { .. } => "TextUnresolved",
            Paragraph::Unknown { .. } => "Unknown",
        }
    }
}