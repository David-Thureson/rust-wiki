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
                        let is_subtopics = match type_ {
                            ListType::Subtopics => true,
                            _ => false,
                        };
                        topic.outbound_links.append(&mut Self::catalog_links_text_block(header));
                        for list_item in items.iter() {
                            let mut links = Self::catalog_links_text_block(&list_item.block);
                            if is_subtopics {
                                for link in links.iter() {
                                    match &link.type_ {
                                        LinkType::Topic { topic_key } => {
                                            topic.subtopics.push(topic_key.clone());
                                            break;
                                        },
                                        _ => {},
                                    }
                                }
                            }
                            topic.outbound_links.append(&mut links);
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
                            entry.push(format!("Topic link [{} | {}] not found.", &topic_key.0, &topic_key.1));
                        }
                    },
                    LinkType::Section { section_key } => {
                        if !self.has_section(section_key) {
                            let entry = errors.entry(topic.get_key()).or_insert(vec![]);
                            entry.push(format!("Section link [{} | {} | {}] not found.", &section_key.0.0, &section_key.0.1, &section_key.1));
                        }
                    },
                    _ => {},
                }
            }
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