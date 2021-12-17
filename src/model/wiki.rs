use std::rc::Rc;
use std::cell::RefCell;

use super::*;
use std::collections::BTreeMap;

pub type WikiRc = Rc<RefCell<Wiki>>;

pub struct Wiki {
    pub name: String,
    pub namespaces: BTreeMap<String, String>,
    pub topics: BTreeMap<TopicKey, Topic>,
    pub categories: BTreeMap<String, Category>,
    pub attributes: BTreeMap<String, Attribute>,
}

impl Wiki {
    pub fn new(name: &str) -> Self {
        let wiki = Self {
            name: name.to_string(),
            namespaces: Default::default(),
            topics: Default::default(),
            categories: Default::default(),
            attributes: Default::default(),
        };
        wiki
    }

    pub fn add_namespace(&mut self, name: &str) {
        let key = name.to_lowercase();
        assert!(!self.namespaces.contains_key(&key));
        self.namespaces.insert(key, name.to_string());
    }

    pub fn add_topic(&mut self, topic: Topic) {
        assert!(self.namespaces.contains_key(&topic.namespace));
        let key = topic.get_key();
        assert!(!self.topics.contains_key(&key));
        self.topics.insert(key, topic);
    }

    pub fn catalog_links(&mut self) -> BTreeMap<TopicKey, Vec<String>> {
        for topic in self.topics.values_mut() {
            for paragraph in topic.paragraphs.iter() {
                match paragraph {
                    Paragraph::List { type_, header, items } => {
                        let (is_combos, is_subtopics) = match type_ {
                            ListType::Combinations => (true, false),
                            ListType::Subtopics => (false, true),
                            _ => (false, false),
                        };
                        topic.outbound_links.append(&mut Self::catalog_links_text_block(header));
                        for list_item in items.iter() {
                            if list_item.depth == 1 {
                                let mut links = Self::catalog_links_text_block(&list_item.block);
                                for link in links.iter() {
                                    match &link.type_ {
                                        LinkType::Topic { topic_key } => {
                                            if !topic.listed_topics.contains(&topic_key) {
                                                topic.listed_topics.push(topic_key.clone());
                                            }
                                            if is_combos {
                                                topic.combo_subtopics.push(topic_key.clone());
                                            } else if is_subtopics {
                                                topic.subtopics.push(topic_key.clone());
                                            }
                                            break;
                                        },
                                        _ => {},
                                    }
                                }
                                topic.outbound_links.append(&mut links);
                            }
                        }
                    },
                    Paragraph::Text { text_block} => {
                        topic.outbound_links.append(&mut Self::catalog_links_text_block(text_block));
                    },
                    _ => {},
                }
            }
        }

        let mut errors = BTreeMap::new();
        for topic in self.topics.values() {
            for link in topic.outbound_links.iter() {
                match &link.type_ {
                    LinkType::Topic { topic_key } => {
                        if !self.has_topic(topic_key) {
                            let entry = errors.entry(topic.get_key()).or_insert(vec![]);
                            entry.push(format!("Topic link {} not found.", Topic::topic_key_to_string(topic_key)));
                        }
                    },
                    LinkType::Section { section_key } => {
                        if !self.has_section(section_key) {
                            let entry = errors.entry(topic.get_key()).or_insert(vec![]);
                            entry.push(format!("Section link {} not found.", Topic::section_key_to_string(section_key)));
                        }
                    },
                    _ => {},
                }
            }
        }

        // Set inbound links.
        let mut map = BTreeMap::new();
        for topic in self.topics.values() {
            let topic_key = topic.get_key();
            for link in topic.outbound_links.iter() {
                let outbound_topic_key = match &link.type_ {
                    LinkType::Topic { topic_key } => Some(topic_key.clone()),
                    LinkType::Section { section_key } => Some(Topic::section_key_to_topic_key(section_key)),
                    _ => None,
                };
                if let Some(outbound_topic_key) = outbound_topic_key {
                    let entry = map.entry(outbound_topic_key.clone()).or_insert(vec![]);
                    if !entry.contains(&topic_key) {
                        entry.push(topic_key.clone());
                    }
                }
            }
        }
        for (topic_key, mut inbound_topic_keys) in map.drain_filter(|_k, _v| true) {
            if let Some(topic) = self.topics.get_mut(&topic_key) {
                topic.inbound_topic_keys.append(&mut inbound_topic_keys);
            }
        }

        // Sort all of the vectors of TopicKeys.
        for topic in self.topics.values_mut() {
            topic.inbound_topic_keys.sort();
            topic.subtopics.sort();
            topic.combo_subtopics.sort();
            topic.listed_topics.sort();
        }

        errors
    }

    fn catalog_links_text_block(text_block: &TextBlock) -> Vec<Link> {
        let mut links = vec![];
        for item in text_block.items.iter() {
            match item {
                TextItem::Link { link } => {
                    links.push(link.clone());
                },
                _ => {},
            }
        }
        links
    }

    pub fn check_subtopic_relatioships(&self) -> BTreeMap<TopicKey, Vec<String>> {
        let mut errors = BTreeMap::new();
        let err_msg_func = |msg: &str| format!("Wiki::check_subtopic_relatioships: {}", msg);
        let cat_combo = "Combinations".to_string();
        for topic in self.topics.values() {
            let topic_key = topic.get_key();
            let parent_count = topic.parents.len();
            if topic.category.as_ref().is_none() || topic.category.as_ref().unwrap().to_string() != cat_combo {
                // Not a combination topic.
                if parent_count > 1 {
                    let entry = errors.entry(topic_key.clone()).or_insert(vec![]);
                    entry.push(err_msg_func(&format!("Non-combo category, so expected 0 or 1 parents, found {}.", parent_count)));
                } else {
                    for parent_topic_key in topic.parents.iter() {
                        if !self.topics[parent_topic_key].listed_topics.contains(&topic_key) {
                            let entry = errors.entry(parent_topic_key.clone()).or_insert(vec![]);
                            // entry.push(err_msg_func(&format!("No subtopic link to child {}.", Topic::topic_key_to_string(&topic_key))));
                            entry.push(format!("[[{}]]", topic.name));
                        }
                    }
                }
            } else {
                // Combination topic.
                if parent_count != 2 {
                    let entry = errors.entry(topic_key.clone()).or_insert(vec![]);
                    entry.push(err_msg_func(&format!("Combo category, so expected 2 parents, found {}.", parent_count)));
                } else {
                    for parent_topic_key in topic.parents.iter() {
                        if !self.topics[parent_topic_key].combo_subtopics.contains(&topic_key) {
                            let entry = errors.entry(parent_topic_key.clone()).or_insert(vec![]);
                            entry.push(err_msg_func(&format!("No combination link to child [[{}]].", topic.name)));
                        }
                    }
                }
            }
        }
        errors
    }

    pub fn catalog_possible_list_types(&self) -> util::group::Grouper<String> {
        let mut group = util::group::Grouper::new("Possible List Types");
        for topic in self.topics.values() {
            for paragraph in topic.paragraphs.iter() {
                match paragraph {
                    Paragraph::List { type_, header, .. } => {
                        match type_ {
                            ListType::General => {
                                if header.items.len() == 1 {
                                    match &header.items[0] {
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

    pub fn has_topic(&self, topic_key: &TopicKey) -> bool {
        self.topics.contains_key(topic_key)
    }

    pub fn has_section(&self, section_key: &SectionKey) -> bool {
        let (topic_key, section_name) = section_key;
        if !self.has_topic(topic_key) {
            return false;
        }
        self.topics[topic_key].has_section(section_name)
    }

    /*
    pub fn find_topic_rc(&self, namespace_name: &str, topic_name: &str, context: &str) -> Result<TopicRc, String> {
        let key = Topic::make_key(namespace_name, topic_name);
        match self.topics.get(&key) {
            Some(topic_rc) => Ok(topic_rc.clone()),
            None => Err(format!("{}: Unable to find topic \"{}\" -> \"{}\"", context, namespace_name, topic_name)),
        }
    }
    */

    /*
    pub fn get_or_create_category(&mut self, wiki_rc: WikiRc, name: &str) -> CategoryRc {
        let key = name.to_lowercase();
        if self.categories.contains_key(&key) {
            self.categories[&key].clone()
        } else {
            let category = Category::new(&wiki_rc, None, name);
            let category_rc = r!(category);
            self.categories.insert(name.to_lowercase(), category_rc.clone());
            category_rc
        }
    }
     */

    /*
    pub fn get_paragraphs(&self) -> Vec<ParagraphRc> {
        self.topics.values().map(|topic_rc| b!(topic_rc).paragraphs.clone()).flatten().collect()
    }
    */

}