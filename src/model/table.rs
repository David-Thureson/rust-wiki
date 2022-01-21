use crate::model::TextBlock;

// This is a simple table abstraction used during parsing. It's not part of the model. In the
// model, use Paragraph::Table.
#[derive(Clone, Debug)]
pub struct Table {
    rows: Vec<Vec<TableCell>>,
    has_header: bool,
}

#[derive(Clone, Debug)]
pub struct TableCell {
    text_block: TextBlock,
    is_bold: bool,
    horizontal: HorizontalAlignment,
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

    pub fn add_row(&mut self, row: Vec<TableCell>) {
        self.rows.push(row);
    }

    pub fn get_rows(&self) -> &Vec<Vec<TableCell>> {
        &self.rows
    }

    pub fn has_header(&self) -> bool {
        self.has_header
    }

    pub fn set_has_header(&mut self, has_header: bool) {
        self.has_header = has_header;
    }

    pub fn get_cell(&self, row_index: usize, col_index: usize) -> &TableCell {
        &self.rows[row_index][col_index]
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

    pub fn add_cells_flow_layout(&mut self, column_count: usize, mut cells: Vec<TableCell>) {
        let mut row_index = 0;
        let mut col_index = 0;
        self.rows.push(vec![]);
        for cell in cells.drain(..) {
            if col_index >= column_count {
                row_index += 1;
                col_index = 0;
                self.rows.push(vec![]);
            }
            self.rows[row_index].push(cell);
            col_index += 1;
        }
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

    pub fn is_bold(&self) -> bool {
        self.is_bold
    }

    pub fn get_horizontal(&self) -> &HorizontalAlignment {
        &self.horizontal
    }

    pub fn get_text_block(&self) -> &TextBlock {
        &self.text_block
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
