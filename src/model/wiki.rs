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