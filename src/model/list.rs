use super::*;

#[derive(Clone)]
pub enum ListType {
    Subtopic,
    Library,
    Crate,
    Dependency,
    Custom,
}

#[derive(Clone)]
pub struct ListItem {
    pub block: TextBlock,
}
