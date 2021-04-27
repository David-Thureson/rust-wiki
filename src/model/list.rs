use super::*;

pub enum ListType {
    Subtopic,
    Library,
    Crate,
    Dependency,
    Custom,
}

pub struct ListItem {
    pub block: TextBlock,
}
