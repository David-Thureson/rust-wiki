use super::*;
use std::rc::Rc;
use std::cell::RefCell;

pub type ParagraphRc = Rc<RefCell<Paragraph>>;

// #[derive(Clone)]
pub enum Paragraph {
    Attributes,
    Breadcrumbs,
    Category,
    Code {
        text: String,
    },
    GenStart,
    GenEnd,
    List {
        type_: ListType,
        header: TextBlock,
        items: Vec<ListItem>,
    },
    Placeholder,
    QuoteEnd,
    QuoteStart,
    SectionHeader {
        name: String,
        depth: usize,
    },
    Table {
        has_header: bool,
        rows: Vec<Vec<TextBlock>>,
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
    pub fn new_code(text: &str) -> Self {
        Paragraph::Code { text: text.to_string() }
    }

    pub fn new_section_header(name: &str, depth: usize) -> Self {
        Paragraph::SectionHeader { name: name.to_string(), depth }
    }

    pub fn new_text(text_block: TextBlock) -> Self {
        Paragraph::Text { text_block }
    }

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
            Paragraph::Code { .. } => "Code",
            Paragraph::GenStart { .. } => "GenStart",
            Paragraph::GenEnd { .. } => "GenEnd",
            Paragraph::List { .. } => "List",
            Paragraph::Placeholder { .. } => "Placeholder",
            Paragraph::QuoteEnd { .. } => "QuoteEnd",
            Paragraph::QuoteStart { .. } => "QuoteStart",
            Paragraph::SectionHeader { .. } => "SectionHeader",
            Paragraph::Table { .. } => "Table",
            Paragraph::Text { .. } => "Text",
            Paragraph::TextUnresolved { .. } => "TextUnresolved",
            Paragraph::Unknown { .. } => "Unknown",
        }
    }
}