use super::*;
use std::rc::Rc;
use std::cell::RefCell;

pub type ParagraphRc = Rc<RefCell<Paragraph>>;

// #[derive(Clone)]
pub enum Paragraph {
    Attributes,
    Breadcrumbs,
    Category,
    GenStart,
    GenEnd,
    List {
        type_: ListType,
        header: TextBlock,
        items: Vec<ListItem>,
    },
    Placeholder,
    Quote {
        text: String,
    },
    SectionHeader {
        name: String,
        depth: usize,
    },
    Text {
        text_block: TextBlock,
    },
    TextUnresolved {
        text: String,
    },
    Unknown {
        text: String,
    },
}

impl Paragraph {
    pub fn new_text(text_block: TextBlock) -> Self {
        Paragraph::Text { text_block }
    }

    pub fn new_text_unresolved(text: &str) -> Self {
        Paragraph::TextUnresolved { text: text.to_string() }
    }

    pub fn new_quote(text: &str) -> Self {
        Paragraph::Quote { text: text.to_string() }
    }

    pub fn new_unknown(text: &str) -> Self {
        Paragraph::Unknown { text: text.to_string() }
    }

    pub fn get_variant_name(&self) -> &str {
        match self {
            Paragraph::Attributes { .. } => "Attributes",
            Paragraph::Breadcrumbs { .. } => "Breadcrumbs",
            Paragraph::Category { .. } => "Category",
            Paragraph::GenStart { .. } => "GenStart",
            Paragraph::GenEnd { .. } => "GenEnd",
            Paragraph::List { .. } => "List",
            Paragraph::Placeholder { .. } => "Placeholder",
            Paragraph::Quote { .. } => "Quote",
            Paragraph::SectionHeader { .. } => "SectionHeader",
            Paragraph::Text { .. } => "Text",
            Paragraph::TextUnresolved { .. } => "TextUnresolved",
            Paragraph::Unknown { .. } => "Unknown",
        }
    }
}