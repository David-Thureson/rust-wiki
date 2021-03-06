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
pub(crate) const LIST_TYPE_INBOUND_LINKS : &str = "Inbound links";
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

    #[allow(dead_code)]
    pub fn set_type(&mut self, type_: &str) {
        self.type_ = type_.to_string();
    }

    pub fn set_type_and_header(&mut self, type_: &str) {
        self.type_ = type_.to_string();
        self.header = Some(TextBlock::new_resolved_text(&list_type_to_header(type_)));
    }

    pub fn get_header(&self) -> &Option<TextBlock> {
        &self.header
    }

    pub fn set_is_ordered(&mut self, is_ordered: bool) {
        for item in self.items.iter_mut() {
            item.is_ordered = is_ordered;
        }
    }

    /*
    pub fn replace_header(&mut self, header: Option<TextBlock>) -> Option<TextBlock> {
        std::mem::replace(&mut self.header, header)
    }
     */

    pub fn add_item(&mut self, item: ListItem) {
        self.items.push(item);
    }

    pub fn add_item_topic_link(&mut self, depth: usize, is_ordered: bool, topic_key: &TopicKey) {
        self.add_item(ListItem::new(depth, is_ordered, TextBlock::new_topic_link(topic_key)));
    }

    pub fn add_item_topic_link_if_missing(&mut self, depth: usize, is_ordered: bool, topic_key: &TopicKey) {
        if !self.contains_topic_link(topic_key) {
            self.add_item_topic_link(depth, is_ordered, topic_key);
        }
    }

    pub fn get_items(&self) -> &Vec<ListItem> {
        &self.items
    }

    #[allow(dead_code)]
    pub fn get_items_mut(&mut self) -> &mut Vec<ListItem> {
        &mut self.items
    }

    #[allow(dead_code)]
    pub fn set_header(&mut self, text_block: Option<TextBlock>) {
        self.header = text_block;
    }

    #[allow(dead_code)]
    pub fn set_item_text_block(&mut self, index: usize, text_block: TextBlock) {
        self.items[index].text_block = text_block;
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

    pub(crate) fn get_links(&self, include_generated: bool, dependencies_are_generated: bool) -> Vec<LinkRc> {
        let mut links = vec![];
        if !include_generated {
            if self.is_generated() {
                return links;
            }
            if dependencies_are_generated && (self.type_.eq(LIST_TYPE_DEPENDENCIES) || self.type_.eq(LIST_TYPE_USED_BY)) {
                return links;
            }
        }
        if let Some(header) = &self.header {
            links.append(&mut header.get_links())
        }
        for item in self.items.iter() {
            links.append(&mut item.text_block.get_links());
        }
        links
    }

    pub(crate) fn sort_items(&mut self) {
        self.items.sort_by_cached_key(|item| item.get_display_text().to_lowercase());
    }

    pub fn header_to_type(header: &str) -> String {
        let header_trim_lower = util::parse::before(header, ":").trim().to_lowercase();
        // if header_trim_lower.contains("all topics") { dbg!(&header_trim_lower); }
        match header_trim_lower.as_str() {
            "crates" | "libraries" => LIST_TYPE_DEPENDENCIES.to_string(),
            "projects" => LIST_TYPE_USED_BY.to_string(),
            _ => {
                for list_type in LIST_TYPES.iter() {
                    // bg!(&header_trim_lower, &list_type.to_lowercase());
                    if header_trim_lower.eq(&list_type.to_lowercase()) {
                        //bg!(&list_type);
                        return list_type.to_string();
                    }
                }
                LIST_TYPE_GENERAL.to_string()
            }
        }
    }

    pub fn contains_topic_link(&self, topic_key: &TopicKey) -> bool {
        if self.header.as_ref().map_or(false, |header| header.contains_topic_link(topic_key)) {
            return true;
        }
        self.items.iter().any(|item| item.contains_topic_link(topic_key))
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

    #[allow(dead_code)]
    pub(crate) fn set_text_block(&mut self, text_block: TextBlock) {
        self.text_block = text_block;
    }

    pub(crate) fn get_display_text(&self) -> String {
        self.text_block.get_display_text()
    }

    pub fn contains_topic_link(&self, topic_key: &TopicKey) -> bool {
        self.text_block.contains_topic_link(topic_key)
    }
}

pub fn list_type_to_header(list_type: &str) -> String {
    format!("{}:", list_type)
}

const LIST_TYPES: [&str; 27] = [
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
    LIST_TYPE_INBOUND_LINKS,
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

pub const GENERATED_LIST_TYPES: [&str; 6] = [
    LIST_TYPE_ALL_TOPICS,
    LIST_TYPE_COMBINATIONS,
    LIST_TYPE_INBOUND_LINKS,
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
