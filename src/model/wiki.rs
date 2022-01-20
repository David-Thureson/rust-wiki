use std::rc::Rc;
use std::cell::RefCell;

use super::*;
use std::collections::BTreeMap;
// use crate::connectedtext::NAMESPACE_TOOLS;

pub type WikiRc = Rc<RefCell<Wiki>>;

pub struct Wiki {
    pub name: String,
    pub main_namespace: String,
    pub namespaces: BTreeMap<String, String>,
    pub topics: BTreeMap<TopicKey, Topic>,
    pub categories: BTreeMap<String, Category>,
    category_tree: Option<TopicTree>,
    subtopic_tree: Option<TopicTree>,
    pub attributes: AttributeList,
    pub domains: DomainList,
}

impl Wiki {
    pub fn new(name: &str, main_namespace: &str) -> Self {
        TopicKey::assert_legal_namespace(main_namespace);
        let mut wiki = Self {
            name: name.to_string(),
            main_namespace: main_namespace.to_string(),
            namespaces: Default::default(),
            topics: Default::default(),
            categories: Default::default(),
            category_tree: None,
            subtopic_tree: None,
            attributes: AttributeList::new(),
            domains: DomainList::new(),
        };
        wiki.add_namespace(main_namespace);
        wiki
    }

    pub fn add_namespace(&mut self, name: &str) {
        assert!(!self.namespaces.contains_key(name));
        self.namespaces.insert(name.to_string(), name.to_string());
    }

    #[inline]
    pub fn qualify_namespace(&self, name: &str) -> String {
        if name.starts_with(":") {
            //bg!(&name, format!("{}{}", &self.main_namespace, name.to_lowercase()));
            format!("{}{}", &self.main_namespace, name.to_lowercase())
        } else {
            name.to_lowercase()
        }
    }

    pub fn namespace_attribute(&self) -> String {
        self.qualify_namespace(NAMESPACE_ATTRIBUTE)
    }

    pub fn namespace_book(&self) -> String {
        self.qualify_namespace(NAMESPACE_BOOK)
    }

    pub fn namespace_navigation(&self) -> String {
        self.qualify_namespace(NAMESPACE_NAVIGATION)
    }

    pub fn add_topic(&mut self, topic: Topic) {
        assert!(self.namespaces.contains_key(&topic.namespace));
        let key = topic.get_key();
        if self.topics.contains_key(&key) {
            panic!("We already have this topic key: {:?}", key)
        }
        assert!(!self.topics.contains_key(&key));
        self.topics.insert(key, topic);
    }

    pub fn topic_name(&self, topic_key: &TopicKey) -> String {
        assert!(self.topics.contains_key(topic_key), "Topic key {} not found.", topic_key);
        let topic = self.topics.get(topic_key).unwrap();
        topic.name.clone()
    }

    pub fn catalog_links(&mut self) {
        Link::catalog_links(self);
    }

    pub fn check_links(&self) -> TopicErrorList {
        Link::check_links(self)
    }

    pub fn check_topic_link(&self, errors: &mut TopicErrorList, list_name: &str, this_topic_key: &TopicKey, ref_topic_key: &TopicKey) {
        if !self.has_topic(ref_topic_key) {
            errors.add(this_topic_key,&format!("wiki::check_topic_link(): Topic link {} from {} list not found.", ref_topic_key, list_name));
        }
    }

    pub fn update_internal_links(&mut self, keys: &Vec<(TopicKey, TopicKey)>) {
        Link::update_internal_links(self, keys)
    }

    pub fn check_subtopic_relationships(&self) -> TopicErrorList {
        Topic::check_subtopic_relationships(self)
    }

    pub fn catalog_possible_list_types(&self) -> util::group::Grouper<String> {
        ListType::catalog_possible_list_types(self)
    }

    pub fn catalog_attributes(&mut self) -> TopicErrorList {
        AttributeType::catalog_attributes(self)
    }

    pub fn catalog_domains(&mut self) -> TopicErrorList {
        DomainList::catalog_domains(self)
    }

    pub fn interpolate_added_date(&mut self) {
        super::date::interpolate_added_date(self);
    }

    pub fn has_topic(&self, topic_key: &TopicKey) -> bool {
        self.topics.contains_key(topic_key)
    }

    pub fn topic_keys_alphabetical_by_topic_name(&self) -> Vec<TopicKey> {
        self.topics.keys().sorted_by_key(|topic_key| topic_key.topic_name.to_lowercase()).map(|x| x.clone()).collect()
    }

    pub fn get_attribute_order(&self, attr_type_name: &str) -> Result<usize, String> {
        match self.attributes.attribute_orders.get(attr_type_name) {
            Some(sequence) => Ok(*sequence),
            None => Err(format!("No sequence found for attribute type \"{}\".", attr_type_name)),
        }
    }

    /*
    pub fn topic_keys_alphabetical_by_topic_name(&self) -> Vec<TopicKey> {
        let mut map = BTreeMap::new();
        for topic_key in self.topics.keys() {
            //bg!(topic_key);
            let key_new = topic_key.topic_name.clone();
            map.insert(key_new, topic_key.clone());
        }
        //bg!(&map);
        map.values().map(|topic_key| topic_key.clone()).collect::<Vec<_>>()
    }
    */

    pub fn has_section(&self, section_key: &SectionKey) -> bool {
        if !self.has_topic(&section_key.topic_key) {
            return false;
        }
        self.topics[&section_key.topic_key].has_section(&section_key.section_name)
    }

    pub fn add_missing_category_topics(&mut self) {
        Category::add_missing_category_topics(self)
    }

    pub fn move_topics_to_namespace_by_category(&mut self, category_name: &str, namespace_name: &str) {
        TopicKey::assert_legal_namespace(namespace_name);
        Category::move_topics_to_namespace_by_category(self, category_name, namespace_name)
    }

    pub fn make_category_tree(&mut self) {
        self.category_tree = Some(Category::make_category_tree(self));
    }

    pub fn make_subtopic_tree(&mut self) {
        self.subtopic_tree = Some(Topic::make_subtopic_tree(self));
    }

    pub fn category_tree(&self) -> &TopicTree {
        match &self.category_tree {
            Some(tree) => tree,
            None => panic!("The wiki model has no category tree. Call make_category_tree() after loading all of the topics."),
        }
    }

    pub fn subtopic_tree(&self) -> &TopicTree {
        match &self.subtopic_tree {
            Some(tree) => tree,
            None => panic!("The wiki model has no subtopic tree. Call make_subtopic_tree() after loading all of the topics."),
        }
    }

    pub fn get_distinct_attr_values(&self, value_type: &AttributeValueType) -> Vec<String> {
        AttributeType::get_distinct_attr_values(self, value_type)
    }

    pub fn get_topics_for_attr_value(&self, value_type: &AttributeValueType, match_value: &str, included_attr_names: Option<Vec<&str>>) -> Vec<TopicKey> {
        AttributeType::get_topics_for_attr_value(self, value_type, match_value, included_attr_names)
    }

    // Create a list of pairs of the attribute type name and the topic key.
    pub fn get_typed_topics_for_attr_value(&self, value_type: &AttributeValueType, match_value: &str, included_attr_names: Option<Vec<&str>>) -> Vec<(String, TopicKey)> {
        AttributeType::get_typed_topics_for_attr_value(self, value_type, match_value, included_attr_names)
    }

    pub fn get_topics_first_letter_map(&self) -> BTreeMap<String, Vec<TopicKey>> {
        let mut map = BTreeMap::new();
        for topic_key in self.topics.values().map(|topic| topic.get_key()) {
            let first_char = topic_key.topic_name.to_uppercase().chars().next().unwrap();
            let map_key = if first_char.is_numeric() {
                '#'.to_string()
            } else if first_char.is_ascii_alphabetic() {
                first_char.to_string()
            } else {
                panic!("Topic name \"{}\" does not start with a number or ASCII letter.", topic_key.topic_name)
            };
            let entry = map.entry(map_key).or_insert(vec![]);
            entry.push(topic_key);
        }
        map
    }

}