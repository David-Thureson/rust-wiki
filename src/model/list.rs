use super::*;

#[derive(Clone, Debug)]
pub(crate) struct List {
    type_: ListType,
    header: Option<TextBlock>,
    items: Vec<ListItem>,
}

#[derive(Clone, Debug)]
pub(crate) struct ListItem {
    depth: usize,
    is_ordered: bool,
    text_block: TextBlock,
}

#[derive(Clone, Debug)]
pub(crate) enum ListType {
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

impl List {
    pub(crate) fn new(type_: ListType, header: Option<TextBlock>) -> Self {
        Self {
            type_,
            header,
            items: vec![]
        }
    }

    pub fn get_type(&self) -> &ListType {
        &self.type_
    }

    pub fn get_header(&self) -> &Option<TextBlock> {
        &self.header
    }

    pub fn add_item(&mut self, item: ListItem) {
        self.items.push(item);
    }

    pub fn get_items(&self) -> &Vec<ListItem> {
        &self.items
    }

}

impl ListItem {
    pub(crate) fn new(depth: usize, is_ordered: bool, block: TextBlock) -> Self {
        assert!(depth > 0);
        Self {
            depth,
            is_ordered,
            text_block: block,
        }
    }

    pub(crate) fn get_depth(&self) -> usize {
        self.depth
    }

    pub(crate) fn is_ordered(&self) -> bool {
        self.is_ordered
    }

    pub(crate) fn get_text_block(&self) -> &TextBlock {
        &self.text_block
    }
}

impl ListType {
    pub(crate) fn from_header(header: &str) -> Self {
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

    pub(crate) fn get_variant_name(&self) -> &str {
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

    pub(crate) fn catalog_possible_list_types(model: &Model) -> util::group::Grouper<String> {
        let mut group = util::group::Grouper::new("Possible List Types");
        for topic in model.get_topics().values() {
            for paragraph in topic.get_paragraphs().iter() {
                match paragraph {
                    Paragraph::List { list } => {
                        match list.get_type() {
                            ListType::General => {
                                if let Some(header) = list.get_header() {
                                    let items = header.get_resolved_items();
                                    if items.len() == 1 {
                                        match &items[0] {
                                            TextItem::Text { text } => {
                                                group.record_entry(text);
                                            },
                                            _ => {},
                                        }
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

