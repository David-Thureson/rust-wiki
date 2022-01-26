use super::*;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) enum Paragraph {
    Attributes,
    Breadcrumbs,
    Category,
    GenStart,
    GenEnd,
    List {
        list: List,
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

    pub(crate) fn new_list(list: List) -> Self {
        Self::List {
            list,
        }
    }

    pub(crate) fn new_marker(text: &str) -> Self {
        Self::Marker {
            text: text.to_string(),
        }
    }

    pub(crate) fn new_section_header(name: &str, depth: usize) -> Self {
        Self::SectionHeader {
            name: name.to_string(),
            depth
        }
    }

    pub(crate) fn new_table(table: Table) -> Self {
        Self::Table {
            table,
        }
    }

    pub(crate) fn new_text(text_block: TextBlock) -> Self {
        Self::Text {
            text_block
        }
    }

    pub(crate) fn new_text_unresolved(text: &str) -> Self {
        Self::TextUnresolved {
            text: text.to_string()
        }
    }

    pub(crate) fn new_unknown(text: &str) -> Self {
        Self::Unknown {
            text: text.to_string()
        }
    }

    pub(crate) fn get_variant_name(&self) -> &str {
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