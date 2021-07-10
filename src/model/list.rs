use super::*;

#[derive(Clone)]
pub enum ListType {
    Crate,
    Dependency,
    General,
    Library,
    Project,
    SeeAlso,
    Settings,
    Subtopic,
}

// #[derive(Clone)]
pub struct ListItem {
    pub depth: usize,
    pub block: TextBlock,
}

impl ListType {
    pub fn from_header(header: &str) -> Self {
        match header {
            "Crates:" => Self::Crate,
            "Dependencies:" => Self::Dependency,
            "Libraries" => Self::Library,
            "Projects:" => Self::Project,
            "See also:" => Self::SeeAlso,
            "Settings:" => Self::Settings,
            "Subtopics:" => Self::Subtopic,
            _ => Self::General,
        }
    }
}

impl ListItem {
    pub fn new(depth: usize, block: TextBlock) -> Self {
        Self {
            depth,
            block,
        }
    }
}