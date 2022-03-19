use crate::model::{TextBlock, LinkRc};

// This is a simple table abstraction used during parsing. It's not part of the model. In the
// model, use Paragraph::Table.
#[derive(Clone, Debug)]
pub(crate) struct Table {
    rows: Vec<Vec<TableCell>>,
    has_header: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct TableCell {
    text_block: TextBlock,
    is_bold: bool,
    horizontal: HorizontalAlignment,
}

#[derive(Clone, Debug)]
pub(crate) enum HorizontalAlignment {
    Center,
    Left,
    Right,
}

impl Table {
    pub(crate) fn new(has_header: bool) -> Self{
        Self {
            rows: vec![],
            has_header,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub(crate) fn add_row(&mut self, row: Vec<TableCell>) {
        self.rows.push(row);
    }

    pub(crate) fn get_rows(&self) -> &Vec<Vec<TableCell>> {
        &self.rows
    }

    #[allow(dead_code)]
    pub(crate) fn get_rows_mut(&mut self) -> &mut Vec<Vec<TableCell>> {
        &mut self.rows
    }

    pub(crate) fn has_header(&self) -> bool {
        self.has_header
    }

    pub(crate) fn set_has_header(&mut self, has_header: bool) {
        self.has_header = has_header;
    }

    pub(crate) fn get_cell(&self, row_index: usize, col_index: usize) -> &TableCell {
        &self.rows[row_index][col_index]
    }

    pub(crate) fn assume_has_header(&self) -> bool {
        !self.rows.is_empty() && self.rows[0].iter().all(|cell| cell.is_bold)
    }

    pub(crate) fn get_column_count(&self) -> usize {
        self.rows.iter()
            .map(|row| row.len())
            .max().unwrap()
    }

    /*
    pub(crate) fn get_row_values_as_text(&self, row_index: usize) -> Vec<String> {
        self.rows[row_index].iter()
            .map(|cell| cell.text_block.get_unresolved_text())
            .collect()
    }

    pub(crate) fn get_column_values_as_text(&self, col_index: usize) -> Vec<String> {
        self.rows.iter()
            .map(|row| row[col_index].text_block.get_unresolved_text())
            .collect()
    }
    */

    pub(crate) fn add_cells_flow_layout(&mut self, column_count: usize, mut cells: Vec<TableCell>) {
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

    pub(crate) fn get_all_text_blocks_cloned(&self) -> Vec<TextBlock> {
        let mut text_blocks = vec![];
        for row in self.rows.iter() {
            for cell in row.iter() {
                text_blocks.push(cell.text_block.clone());
            }
        }
        text_blocks
    }

    pub(crate) fn get_links(&self) -> Vec<LinkRc> {
        let mut links = vec![];
        for row in self.rows.iter() {
            for cell in row.iter() {
                links.append(&mut cell.text_block.get_links());
            }
        }
        links
    }

    pub(crate) fn trim(&mut self) {
        for row in self.rows.iter_mut() {
            for cell in row.iter_mut() {
                cell.text_block.trim();
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn set_cell_text_block(&mut self, row_index: usize, col_index: usize, text_block: TextBlock) {
        self.rows[row_index][col_index].text_block = text_block;
    }

}


impl TableCell {
    pub(crate) fn new_unresolved_text(text: &str, is_bold: bool, horizontal: &HorizontalAlignment) -> Self {
        Self {
            text_block: TextBlock::new_unresolved(text),
            is_bold,
            horizontal: horizontal.clone(),
        }
    }

    pub(crate) fn new_text_block(text_block: TextBlock, is_bold: bool, horizontal: &HorizontalAlignment) -> Self {
        Self {
            text_block,
            is_bold,
            horizontal: horizontal.clone(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn set_text_block(&mut self, text_block: TextBlock) {
        self.text_block = text_block;
    }

    pub(crate) fn is_bold(&self) -> bool {
        self.is_bold
    }

    pub(crate) fn get_horizontal(&self) -> &HorizontalAlignment {
        &self.horizontal
    }

    pub(crate) fn get_text_block(&self) -> &TextBlock {
        &self.text_block
    }

    /*
    pub(crate) fn update_internal_links(&mut self, keys: &Vec<(TopicKey, TopicKey)>) {
        self.text_block.update_internal_links(keys);
    }
     */

}

impl HorizontalAlignment {
    pub(crate) fn get_variant_name(&self) -> &str {
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
