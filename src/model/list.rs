use super::*;

pub(crate) const LIST_TYPE_ALL_TOPICS : &str = "All topics";
pub(crate) const LIST_TYPE_ARTICLES : &str = "Articles";
pub(crate) const LIST_TYPE_BOOKS : &str = "Books";
pub(crate) const LIST_TYPE_CLIENTS : &str = "Clients";
pub(crate) const LIST_TYPE_CODING_PROJECTS : &str = "Coding projects";
pub(crate) const LIST_TYPE_COMBINATIONS : &str = "Combinations";
pub(crate) const LIST_TYPE_COMPONENTS : &str = "Components";
pub(crate) const LIST_TYPE_COURSES : &str = "Courses";
pub(crate) const LIST_TYPE_DEPENDENCIES : &str = "Dependencies";
pub(crate) const LIST_TYPE_GENERAL : &str = "General";
pub(crate) const LIST_TYPE_IDEAS : &str = "Ideas";
pub(crate) const LIST_TYPE_PRODUCTS : &str = "Products";
pub(crate) const LIST_TYPE_PROJECTS : &str = "Projects";
pub(crate) const LIST_TYPE_RESOURCES : &str = "Resources";
pub(crate) const LIST_TYPE_SEE_ALSO : &str = "See also";
pub(crate) const LIST_TYPE_SETTINGS : &str = "Settings";
pub(crate) const LIST_TYPE_SPECS : &str = "Specs";
pub(crate) const LIST_TYPE_SUBCATEGORIES : &str = "Subcategories";
pub(crate) const LIST_TYPE_SUBTOPICS : &str = "Subtopics";
pub(crate) const LIST_TYPE_TOOLS : &str = "Tools";
pub(crate) const LIST_TYPE_TOPICS : &str = "Topics";
pub(crate) const LIST_TYPE_TO_DO : &str = "To do";
pub(crate) const LIST_TYPE_TO_READ : &str = "To read";
pub(crate) const LIST_TYPE_TO_TRY : &str = "To try";
pub(crate) const LIST_TYPE_TUTORIALS : &str = "Tutorials";
pub(crate) const LIST_TYPE_USED_BY : &str = "Used by";


#[derive(Clone, Debug)]
pub(crate) struct List {
    type_: String,
    header: Option<TextBlock>,
    items: Vec<ListItem>,
}

#[derive(Clone, Debug)]
pub(crate) struct ListItem {
    depth: usize,
    is_ordered: bool,
    text_block: TextBlock,
}

impl List {
    pub(crate) fn new(type_: &str, header: Option<TextBlock>) -> Self {
        Self {
            type_: type_.to_string(),
            header,
            items: vec![]
        }
    }

    pub fn get_type(&self) -> &str {
        &self.type_
    }

    pub fn get_header(&self) -> &Option<TextBlock> {
        &self.header
    }

    /*
    pub fn replace_header(&mut self, header: Option<TextBlock>) -> Option<TextBlock> {
        std::mem::replace(&mut self.header, header)
    }
     */

    pub fn add_item(&mut self, item: ListItem) {
        self.items.push(item);
    }

    pub fn get_items(&self) -> &Vec<ListItem> {
        &self.items
    }

    pub(crate) fn is_generated(&self) -> bool {
        GENERATED_LIST_TYPES.contains(&&*self.type_)
    }

    pub(crate) fn get_all_text_blocks_cloned(&self) -> Vec<TextBlock> {
        let mut text_blocks = vec![];
        if let Some(header) = &self.header {
            text_blocks.push(header.clone());
        }
        for list_item in self.items.iter() {
            text_blocks.push(list_item.get_text_block().clone());
        }
        text_blocks
    }

    pub(crate) fn get_links(&self) -> Vec<LinkRc> {
        let mut links = vec![];
        if let Some(header) = &self.header {
            links.append(&mut header.get_links())
        }
        for item in self.items.iter() {
            links.append(&mut item.text_block.get_links());
        }
        links
    }

    pub(crate) fn sort_items(&mut self) {
        self.items.sort_by_cached_key(|item| item.get_display_text());
    }

    pub fn header_to_type(header: &str) -> String {
        let header = util::parse::before(header, ":").trim();
        match header {
            "Crates:" | "Libraries" => LIST_TYPE_DEPENDENCIES.to_string(),
            _ => {
                if LIST_TYPES.contains(&header) {
                    header.to_string()
                } else {
                    LIST_TYPE_GENERAL.to_string()
                }
            }
        }
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

    pub(crate) fn get_display_text(&self) -> String {
        self.text_block.get_display_text()
    }
}

pub fn list_type_to_header(list_type: &str) -> String {
    format!("{}:", list_type)
}

const LIST_TYPES: [&str; 26] = [
    LIST_TYPE_ALL_TOPICS,
    LIST_TYPE_ARTICLES,
    LIST_TYPE_BOOKS,
    LIST_TYPE_CLIENTS,
    LIST_TYPE_CODING_PROJECTS,
    LIST_TYPE_COMBINATIONS,
    LIST_TYPE_COMPONENTS,
    LIST_TYPE_COURSES,
    LIST_TYPE_DEPENDENCIES,
    LIST_TYPE_GENERAL,
    LIST_TYPE_IDEAS,
    LIST_TYPE_PRODUCTS,
    LIST_TYPE_PROJECTS,
    LIST_TYPE_RESOURCES,
    LIST_TYPE_SEE_ALSO,
    LIST_TYPE_SETTINGS,
    LIST_TYPE_SPECS,
    LIST_TYPE_SUBCATEGORIES,
    LIST_TYPE_SUBTOPICS,
    LIST_TYPE_TOOLS,
    LIST_TYPE_TOPICS,
    LIST_TYPE_TO_DO,
    LIST_TYPE_TO_READ,
    LIST_TYPE_TO_TRY,
    LIST_TYPE_TUTORIALS,
    LIST_TYPE_USED_BY,
];

pub const GENERATED_LIST_TYPES: [&str; 5] = [
    LIST_TYPE_ALL_TOPICS,
    LIST_TYPE_COMBINATIONS,
    LIST_TYPE_SUBCATEGORIES,
    LIST_TYPE_SUBTOPICS,
    LIST_TYPE_TOPICS,
];

/*
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
*/
