use super::*;
use std::rc::Rc;
use std::cell::RefCell;

pub type ParagraphRc = Rc<RefCell<Paragraph>>;

#[derive(Clone, Debug)]
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
    Marker {
        text: String,
    },
    Placeholder,
    SectionHeader {
        name: String,
        depth: usize,
    },
    Table {
        table: Table,
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

    pub fn new_marker(text: &str) -> Self {
        Self::Marker {
            text: text.to_string(),
        }
    }

    pub fn new_section_header(name: &str, depth: usize) -> Self {
        Self::SectionHeader {
            name: name.to_string(),
            depth
        }
    }

    pub fn new_table(table: Table) -> Self {
        Self::Table {
            table,
        }
    }

    pub fn new_text(text_block: TextBlock) -> Self {
        Self::Text {
            text_block
        }
    }

    pub fn new_text_unresolved(text: &str) -> Self {
        Self::TextUnresolved {
            text: text.to_string()
        }
    }

    pub fn new_unknown(text: &str) -> Self {
        Self::Unknown {
            text: text.to_string()
        }
    }

    pub fn get_variant_name(&self) -> &str {
        match self {
            Paragraph::Attributes { .. } => "Attributes",
            Paragraph::Breadcrumbs { .. } => "Breadcrumbs",
            Paragraph::Category { .. } => "Category",
            Paragraph::GenStart { .. } => "GenStart",
            Paragraph::GenEnd { .. } => "GenEnd",
            Paragraph::List { .. } => "List",
            Paragraph::Marker { .. } => "Marker",
            Paragraph::Placeholder { .. } => "Placeholder",
            Paragraph::SectionHeader { .. } => "SectionHeader",
            Paragraph::Table { .. } => "Table",
            Paragraph::Text { .. } => "Text",
            Paragraph::TextUnresolved { .. } => "TextUnresolved",
            Paragraph::Unknown { .. } => "Unknown",
        }
    }
}