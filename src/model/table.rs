use crate::model::TextBlock;

// This is a simple table abstraction used during parsing. It's not part of the model. In the
// model, use Paragraph::Table.
#[derive(Clone, Debug)]
pub struct Table {
    pub rows: Vec<Vec<TableCell>>,
    pub has_header: bool,
}

#[derive(Clone, Debug)]
pub struct TableCell {
    pub text_block: TextBlock,
    pub is_bold: bool,
    pub horizontal: HorizontalAlignment,
}

#[derive(Clone, Debug)]
pub enum HorizontalAlignment {
    Center,
    Left,
    Right,
}

impl Table {
    pub fn new(has_header: bool) -> Self{
        Self {
            rows: vec![],
            has_header,
        }
    }

    pub fn assume_has_header(&self) -> bool {
        !self.rows.is_empty() && self.rows[0].iter().all(|cell| cell.is_bold)
    }

    pub fn get_column_count(&self) -> usize {
        self.rows.iter()
            .map(|row| row.len())
            .max().unwrap()
    }

    pub fn get_row_values_as_text(&self, row_index: usize) -> Vec<String> {
        self.rows[row_index].iter()
            .map(|cell| cell.text_block.get_unresolved_text())
            .collect()
    }

    pub fn get_column_values_as_text(&self, col_index: usize) -> Vec<String> {
        self.rows.iter()
            .map(|row| row[col_index].text_block.get_unresolved_text())
            .collect()
    }
}

impl TableCell {
    pub fn new_unresolved_text(text: &str, is_bold: bool, horizontal: &HorizontalAlignment) -> Self {
        Self {
            text_block: TextBlock::new_unresolved(text),
            is_bold,
            horizontal: horizontal.clone(),
        }
    }

    pub fn new_text_block(text_block: TextBlock, is_bold: bool, horizontal: &HorizontalAlignment) -> Self {
        Self {
            text_block,
            is_bold,
            horizontal: horizontal.clone(),
        }
    }
}

impl HorizontalAlignment {
    pub fn get_variant_name(&self) -> &str {
        match self {
            Self::Center => "Center",
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
}

impl PartialEq for HorizontalAlignment {
    fn eq(&self, other: &Self) -> bool {
        self.get_variant_name().eq(other.get_variant_name())
    }
}

impl Eq for HorizontalAlignment {}
