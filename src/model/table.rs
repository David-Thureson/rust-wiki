// This is a simple table abstraction used during parsing. It's not part of the model. In the
// model, use Paragraph::Table.
#[derive(Clone, Debug)]
pub struct TextTable {
    pub rows: Vec<Vec<TextTableCell>>,
}

#[derive(Clone, Debug)]
pub struct TextTableCell {
    pub text: String,
    pub is_bold: bool,
}

impl TextTable {
    pub fn new() -> Self{
        Self {
            rows: vec![],
        }
    }

    pub fn has_header(&self) -> bool {
        !self.rows.is_empty() && self.rows[0].iter().all(|cell| cell.is_bold)
    }

    pub fn has_label_column(&self) -> bool {
        !self.rows.is_empty() && self.rows.iter().all(|row| !row.is_empty() && row[0].is_bold)
    }

    pub fn get_row_values(&self, row_index: usize) -> Vec<String> {
        self.rows[row_index].iter()
            .map(|cell| cell.text.clone())
            .collect()
    }

    pub fn get_column_values(&self, col_index: usize) -> Vec<String> {
        self.rows.iter()
            .map(|row| row[col_index].text.clone())
            .collect()
    }
}

impl TextTableCell {
    pub fn new(text: &str, is_bold: bool) -> Self {
        Self {
            text: text.to_string(),
            is_bold,
        }
    }
}
