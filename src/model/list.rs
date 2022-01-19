use super::*;

#[derive(Clone, Debug)]
pub enum ListType {
    Articles,
    Books,
    Clients,
    CodingProjects,
    Combinations,
    Components,
    Courses,
    Crates,
    Dependencies,
    General,
    Ideas,
    Libraries,
    Products,
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
    Tutorials,
}

#[derive(Clone, Debug)]
pub struct ListItem {
    pub depth: usize,
    pub block: TextBlock,
}

impl ListType {
    pub fn from_header(header: &str) -> Self {
        match header {
            "Articles:" => Self::Articles,
            "Books:" => Self::Books,
            "Clients:" => Self::Clients,
            "Coding projects:" => Self::CodingProjects,
            "Combinations:" => Self::Combinations,
            "Components:" => Self::Components,
            "Courses:" => Self::Courses,
            "Crates:" => Self::Crates,
            "Dependencies:" => Self::Dependencies,
            "Ideas:" => Self::Ideas,
            "Libraries:" => Self::Libraries,
            "Products:" => Self::Products,
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
            "Tutorials:" => Self::Tutorials,
            _ => Self::General,
        }
    }

    pub fn get_variant_name(&self) -> &str {
        match self {
            Self::Articles => "Articles",
            Self::Books => "Books",
            Self::Clients => "Clients",
            Self::CodingProjects => "CodingProjects",
            Self::Combinations => "Combinations",
            Self::Components => "Components",
            Self::Courses => "Courses",
            Self::Crates => "Crates",
            Self::Dependencies => "Dependencies",
            Self::General => "General",
            Self::Ideas => "Ideas",
            Self::Libraries => "Libraries",
            Self::Products => "Products",
            Self::Projects => "Projects",
            Self::Resources => "Resources",
            Self::SeeAlso => "SeeAlso",
            Self::Settings => "Settings",
            Self::Specs => "Specs",
            Self::Subtopics => "Subtopics",
            Self::Tools => "Tools",
            Self::ToDo => "ToDo",
            Self::ToRead => "ToRead",
            Self::ToTry => "ToTry",
            Self::Tutorials => "Tutorials",
        }
    }

    pub fn catalog_possible_list_types(model: &Wiki) -> util::group::Grouper<String> {
        let mut group = util::group::Grouper::new("Possible List Types");
        for topic in model.topics.values() {
            for paragraph in topic.paragraphs.iter() {
                match paragraph {
                    Paragraph::List { type_, header, .. } => {
                        match type_ {
                            ListType::General => {
                                let items = header.get_resolved_items();
                                if items.len() == 1 {
                                    match &items[0] {
                                        TextItem::Text { text } => {
                                            group.record_entry(text);
                                        },
                                        _ => {},
                                    }
                                }
                            },
                            _ => {},
                        }
                    }
                    _ => {},
                }
            }
        }
        group
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
