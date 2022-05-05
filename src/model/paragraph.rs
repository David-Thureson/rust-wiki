use super::*;
use crate::dokuwiki::internal_link_name;

pub(crate) const PARAGRAPH_VARIANT_NAME_ATTRIBUTES: &str = "Attributes";
pub(crate) const PARAGRAPH_VARIANT_NAME_BREADCRUMBS: &str = "Breadcrumbs";
pub(crate) const PARAGRAPH_VARIANT_NAME_CATEGORY: &str = "Category";
pub(crate) const PARAGRAPH_VARIANT_NAME_GEN_START: &str = "GenStart";
pub(crate) const PARAGRAPH_VARIANT_NAME_GEN_END: &str = "GenEnd";
pub(crate) const PARAGRAPH_VARIANT_NAME_GLOSSARY: &str = "Glossary";
pub(crate) const PARAGRAPH_VARIANT_NAME_LIST: &str = "List";
pub(crate) const PARAGRAPH_VARIANT_NAME_MARKER: &str = "Marker";
pub(crate) const PARAGRAPH_VARIANT_NAME_PLACEHOLDER: &str = "Placeholder";
pub(crate) const PARAGRAPH_VARIANT_NAME_SECTION_HEADER: &str = "SectionHeader";
pub(crate) const PARAGRAPH_VARIANT_NAME_TABLE: &str = "Table";
pub(crate) const PARAGRAPH_VARIANT_NAME_TEXT: &str = "Text";
pub(crate) const PARAGRAPH_VARIANT_NAME_TEXT_UNRESOLVED: &str = "TextUnresolved";
pub(crate) const PARAGRAPH_VARIANT_NAME_UNKNOWN: &str = "Unknown";

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) enum Paragraph {
    Attributes,
    Breadcrumbs,
    Category,
    GenStart,
    GenEnd,
    Glossary {
        name: String,
    },
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
        link_name: String,
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
        line_index: usize,
        text: String,
    },
}

impl Paragraph {

    pub(crate) fn new_glossary(name: &str) -> Self {
        Self::Glossary {
            name: name.to_string(),
        }
    }

    pub(crate) fn new_list(list: List) -> Self {
        Self::List {
            list,
        }
    }

    pub(crate) fn new_list_of_type(list_type: &str) -> Self {
        let header = list_type_to_header(list_type);
        let text_block = TextBlock::new_resolved(vec![TextItem::new_text(&header)]);
        let list = List::new(list_type, Some(text_block));
        Paragraph::new_list(list)
    }

    pub(crate) fn new_marker(text: &str) -> Self {
        Self::Marker {
            text: text.to_string(),
        }
    }

    pub(crate) fn new_section_header(name: &str, depth: usize) -> Self {
        Self::SectionHeader {
            name: name.to_string(),
            depth,
            link_name: internal_link_name(name),
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

    /*
    pub(crate) fn new_text_unresolved(text: &str) -> Self {
        Self::TextUnresolved {
            text: text.to_string()
        }
    }
    */

    pub(crate) fn new_unknown(line_index: usize, text: &str) -> Self {
        Self::Unknown {
            line_index,
            text: text.to_string()
        }
    }

    pub(crate) fn get_variant_name(&self) -> &str {
        match self {
            Paragraph::Attributes { .. } => PARAGRAPH_VARIANT_NAME_ATTRIBUTES,
            Paragraph::Breadcrumbs { .. } => PARAGRAPH_VARIANT_NAME_BREADCRUMBS,
            Paragraph::Category { .. } => PARAGRAPH_VARIANT_NAME_CATEGORY,
            Paragraph::GenStart { .. } => PARAGRAPH_VARIANT_NAME_GEN_START,
            Paragraph::GenEnd { .. } => PARAGRAPH_VARIANT_NAME_GEN_END,
            Paragraph::Glossary { .. } => PARAGRAPH_VARIANT_NAME_GLOSSARY,
            Paragraph::List { .. } => PARAGRAPH_VARIANT_NAME_LIST,
            Paragraph::Marker { .. } => PARAGRAPH_VARIANT_NAME_MARKER,
            Paragraph::Placeholder { .. } => PARAGRAPH_VARIANT_NAME_PLACEHOLDER,
            Paragraph::SectionHeader { .. } => PARAGRAPH_VARIANT_NAME_SECTION_HEADER,
            Paragraph::Table { .. } => PARAGRAPH_VARIANT_NAME_TABLE,
            Paragraph::Text { .. } => PARAGRAPH_VARIANT_NAME_TEXT,
            Paragraph::TextUnresolved { .. } => PARAGRAPH_VARIANT_NAME_TEXT_UNRESOLVED,
            Paragraph::Unknown { .. } => PARAGRAPH_VARIANT_NAME_UNKNOWN,
        }
    }

    pub(crate) fn get_all_text_blocks_cloned(&self) -> Vec<TextBlock> {
        let mut text_blocks = vec![];
        match self {
            Paragraph::List { list } => {
                text_blocks.append(&mut list.get_all_text_blocks_cloned());
            },
            Paragraph::Table { table } => {
                text_blocks.append(&mut table.get_all_text_blocks_cloned());
            },
            Paragraph::Text { text_block } => {
                text_blocks.push(text_block.clone());
            },
            Paragraph::Attributes | Paragraph::Breadcrumbs | Paragraph::Category | Paragraph::GenStart
                | Paragraph::GenEnd | Paragraph::Glossary { .. } | Paragraph::Marker { .. } | Paragraph::Placeholder | Paragraph::SectionHeader { .. }
                | Paragraph::TextUnresolved { .. } | Paragraph::Unknown { .. } => {},
        }
        text_blocks
    }

    /*
    pub(crate) fn visit_text_blocks_mut<V>(&mut self, func: F) -> V
        where F: FnOnce()
    {
    }
     */

    pub(crate) fn get_links(&self, include_generated: bool, dependencies_are_generated: bool) -> Vec<LinkRc> {
        let mut links = vec![];
        match self {
            Paragraph::List { list } => {
                links.append(&mut list.get_links(include_generated, dependencies_are_generated));
            },
            Paragraph::Table { table } => {
                links.append(&mut table.get_links());
            },
            Paragraph::Text { text_block } => {
                links.append(&mut text_block.get_links());
            },
            Paragraph::Attributes | Paragraph::Breadcrumbs | Paragraph::Category | Paragraph::GenStart
            | Paragraph::GenEnd | Paragraph::Glossary { .. } | Paragraph::Marker { .. } | Paragraph::Placeholder | Paragraph::SectionHeader { .. }
            | Paragraph::TextUnresolved { .. } | Paragraph::Unknown { .. } => {},
        }
        links
    }

    pub(crate) fn get_list_mut(&mut self) -> &mut List {
        match self {
            Paragraph::List { list } => list,
            _ => panic!(),
        }
    }

}