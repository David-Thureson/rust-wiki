use super::*;

#[derive(Clone)]
pub enum ListType {
    Crates,
    Dependencies,
    General,
    Ideas,
    Libraries,
    Projects,
    Resources,
    SeeAlso,
    Settings,
    Specs,
    Subtopics,
    Tools,
    ToDo,
    ToRead,
    ToTry,
}

// #[derive(Clone)]
pub struct ListItem {
    pub depth: usize,
    pub block: TextBlock,
}

impl ListType {
    pub fn from_header(header: &str) -> Self {
        match header {
            "Crates:" => Self::Crates,
            "Dependencies:" => Self::Dependencies,
            "Ideas:" => Self::Ideas,
            "Libraries" => Self::Libraries,
            "Projects:" => Self::Projects,
            "Resources:" => Self::Resources,
            "See also:" => Self::SeeAlso,
            "Settings:" => Self::Settings,
            "Specs:" => Self::Specs,
            "Subtopics:" => Self::Subtopics,
            "Tools:" => Self::Tools,
            "To do:" => Self::ToDo,
            "To read:" => Self::ToRead,
            "To try:" => Self::ToTry,
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