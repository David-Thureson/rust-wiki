use std::rc::Rc;
use std::cell::RefCell;

use super::*;
use std::collections::BTreeMap;
// use crate::connectedtext::NAMESPACE_TOOLS;

pub type ModelRc = Rc<RefCell<Model>>;

pub struct Model {
    _name: String,
    main_namespace: String,
    namespaces: BTreeMap<String, String>,
    topics: BTreeMap<TopicKey, Topic>,
    categories: Vec<String>,
    category_tree: Option<TopicTree>,
    subtopic_tree: Option<TopicTree>,
    attribute_list: AttributeList,
    domain_list: DomainList,
}

impl Model {
    pub fn new(name: &str, main_namespace: &str) -> Self {
        TopicKey::assert_legal_namespace(main_namespace);
        let mut wiki = Self {
            _name: name.to_string(),
            main_namespace: main_namespace.to_string(),
            namespaces: Default::default(),
            topics: Default::default(),
            categories: Default::default(),
            category_tree: None,
            subtopic_tree: None,
            attribute_list: AttributeList::new(),
            domain_list: DomainList::new(),
        };
        wiki.add_namespace(main_namespace);
        wiki
    }

    pub fn get_main_namespace(&self) -> &str {
        &self.main_namespace
    }

    pub fn get_namespaces(&self) -> &BTreeMap<String, String> {
        &self.namespaces
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

    pub fn get_topics(&self) -> &BTreeMap<TopicKey, Topic> {
        &self.topics
    }

    pub fn get_topics_mut(&mut self) -> &mut BTreeMap<TopicKey, Topic> {
        &mut self.topics
    }

    /*
    pub fn get_topic_mut(&mut self, topic_key: &TopicKey) -> &Option<&mut Topic> {
        self.topics.get_mut(topic_key)
    }
    */

    pub fn add_topic(&mut self, topic: Topic) {
        assert!(self.namespaces.contains_key(topic.get_namespace()));
        let key = topic.get_key();
        if self.topics.contains_key(&key) {
            panic!("We already have this topic key: {:?}", key)
        }
        assert!(!self.topics.contains_key(&key));
        self.topics.insert(key, topic);
    }

    pub fn remove_topic(&mut self, topic_key: &TopicKey) -> Option<Topic> {
        self.topics.remove(topic_key)
    }

    pub fn get_topic_name(&self, topic_key: &TopicKey) -> &str {
        assert!(self.topics.contains_key(topic_key), "Topic key {} not found.", topic_key);
        let topic = self.topics.get(topic_key).unwrap();
        topic.get_name()
    }

    pub fn set_attributes_to_index(&mut self, attr: Vec<String>) {
        self.attribute_list.set_attributes_to_index(attr);
    }

    pub fn is_attribute_indexed(&self, name: &str) -> bool {
        self.attribute_list.is_attribute_indexed(name)
    }

    pub fn get_attribute_types(&self) -> &BTreeMap<String, AttributeType> {
        self.attribute_list.get_attribute_types()
    }

    pub fn get_attribute_types_mut(&mut self) -> &mut BTreeMap<String, AttributeType> {
        self.attribute_list.get_attribute_types_mut()
    }

    // In the values map, each entry is a list of pairs of topic keys and attribute type names.
    // Sort each of these lists by topic name first, then attribute type name.
    pub fn sort_attribute_topic_lists(&mut self) {
        self.attribute_list.sort_attribute_topic_lists();
    }

    pub fn get_attribute_type(&self, name: &str) -> Option<&AttributeType> {
        self.attribute_list.get_attribute_type(name)
    }

    pub fn clear_attribute_orders(&mut self) {
        self.attribute_list.clear_attribute_orders();
    }

    pub fn add_attribute_order(&mut self, type_name: String, sequence: usize) {
        self.attribute_list.add_attribute_order(type_name, sequence);
    }

    pub fn get_attribute_orders(&self) -> &BTreeMap<String, usize> {
        self.attribute_list.get_attribute_orders()
    }

    pub fn add_attribute_value(&mut self, value: String, topic_key: TopicKey, value_type_name: String) {
        self.attribute_list.add_attribute_value(value, topic_key, value_type_name);
    }

    pub fn has_attribute_links(&self, value: &str) -> bool {
        self.attribute_list.has_attribute_links(value)
    }

    pub fn clear_attribute_types_and_values(&mut self) {
        self.attribute_list.clear_attribute_types_and_values();
    }

    pub fn get_attribute_list(&self) -> &AttributeList {
        &self.attribute_list
    }

    pub fn get_topics_with_attribute_value(&self, value: &str) -> Vec<(TopicKey, String)> {
        self.attribute_list.get_topics_with_attribute_value(value)
    }

    pub fn get_domains(&self) -> &BTreeMap<String, Domain> {
        self.domain_list.get_domains()
    }

    pub fn get_domain(&self, name: &str) -> Option<&Domain> {
        self.domain_list.get_domain(name)
    }

    pub fn catalog_links(&mut self) {
        for topic in self.topics.values_mut() {
            topic.catalog_outbound_links();
        }
        self.catalog_inbound_links();
    }

    fn catalog_inbound_links(&mut self) {
        let mut map = BTreeMap::new();
        for topic in self.topics.values() {
            let topic_key = topic.get_key();
            for outbound_topic_key in topic.get_topic_links_as_topic_keys().drain(..) {
                let entry = map.entry(outbound_topic_key.clone()).or_insert(vec![]);
                if !entry.contains(&topic_key) {
                    entry.push(topic_key.clone());
                }
            }
        }
        for (topic_key, mut inbound_topic_keys) in map.drain_filter(|_k, _v| true) {
            TopicKey::sort_topic_keys_by_name(&mut inbound_topic_keys);
            if let Some(topic) = self.get_topics_mut().get_mut(&topic_key) {
                topic.add_inbound_topic_keys(inbound_topic_keys);
            }
        }
    }

    pub fn check_links(&self) -> TopicErrorList {
        Topic::check_links(self)
    }

    pub fn check_topic_link(&self, errors: &mut TopicErrorList, list_name: &str, this_topic_key: &TopicKey, ref_topic_key: &TopicKey) {
        if !self.has_topic(ref_topic_key) {
            errors.add(this_topic_key,&format!("wiki::check_topic_link(): Topic link {} from {} list not found.", ref_topic_key, list_name));
        }
    }

    pub fn update_internal_links(&mut self, keys: &Vec<(TopicKey, TopicKey)>) {
        for topic in self.topics.values_mut() {
            topic.update_internal_links(keys);
        }
    }

    pub fn check_subtopic_relationships(&self) -> TopicErrorList {
        Topic::check_subtopic_relationships(self)
    }

    pub fn catalog_possible_list_types(&self) -> util::group::Grouper<String> {
        ListType::catalog_possible_list_types(self)
    }

    pub fn catalog_attributes(&mut self) -> TopicErrorList {
        // At this point each topic has a list of temp attributes which are simply named sets of
        // string values, with no sense of their type. Go through the temp attributes in all of the
        // topics and use them to create a master list of attribute types such as "Added" or
        // "Domain" which are held by the model to be shared among topics. Within each topic,
        // replace the temp attributes with references to these shared attribute types.
        let mut errors = TopicErrorList::new();
        AttributeType::fill_attribute_orders(self);
        let attribute_orders = self.get_attribute_orders().clone();
        // Presumably we haven't yet created the real attribute types held by the model.
        assert!(self.attribute_list.get_attribute_types().is_empty());
        assert!(self.attribute_list.get_attribute_values().is_empty());
        // model.clear_attributes();
        // This map holds the real attribute types. We build it while looping through the topics,
        // then attach it to the model at the end to avoid attempting more than one mutable
        // reference to the model.
        let mut attribute_types = BTreeMap::new();
        // Similarly, this map holds the master list of attribute values encountered within the
        // topics. It will be attached to the model at the end.
        let mut attribute_values = BTreeMap::new();
        for topic in self.get_topics_mut().values_mut() {
            topic.catalog_attributes(&mut errors, &mut attribute_types, &mut attribute_values, &attribute_orders);
        }
        self.attribute_list.set_attribute_types_and_values(attribute_types, attribute_values);
        // In the values map, each entry is a list of pairs of topic keys and attribute type names.
        // Sort each of these lists by topic name first, then attribute type name.
        self.sort_attribute_topic_lists();
        // Self::list_attribute_types(model);
        errors
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
        self.topics.keys().sorted_by_key(|topic_key| topic_key.get_topic_name().to_lowercase()).map(|x| x.clone()).collect()
    }

    pub fn add_category_optional(&mut self, name: String) {
        if !self.categories.contains(&name) {
            self.categories.push(name);
        }
    }

    pub fn get_categories(&self) -> &Vec<String> {
        &self.categories
    }

    pub fn get_attribute_order(&self, attr_type_name: &str) -> Result<usize, String> {
        match self.attribute_list.get_attribute_orders().get(attr_type_name) {
            Some(sequence) => Ok(*sequence),
            None => Err(format!("No sequence found for attribute type \"{}\".", attr_type_name)),
        }
    }

    pub fn set_domain_list(&mut self, domain_list: DomainList) {
        self.domain_list = domain_list;
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
        if !self.has_topic(&section_key.get_topic_key()) {
            return false;
        }
        self.topics[&section_key.get_topic_key()].has_section(&section_key.get_section_name())
    }

    pub fn add_missing_category_topics(&mut self) {
        category::add_missing_category_topics(self)
    }

    /*
    pub fn move_topics_to_namespace_by_category(&mut self, category_name: &str, namespace_name: &str) {
        TopicKey::assert_legal_namespace(namespace_name);
        Category::move_topics_to_namespace_by_category(self, category_name, namespace_name)
    }
    */

    pub fn make_category_tree(&mut self) {
        self.category_tree = Some(make_category_tree(self));
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
        for topic_key in self.topic_keys_alphabetical_by_topic_name() {
            let first_char = topic_key.get_topic_name().to_uppercase().chars().next().unwrap();
            let map_key = if first_char.is_numeric() {
                '#'.to_string()
            } else if first_char.is_ascii_alphabetic() {
                first_char.to_string()
            } else {
                panic!("Topic name \"{}\" does not start with a number or ASCII letter.", topic_key.get_topic_name())
            };
            let entry = map.entry(map_key).or_insert(vec![]);
            entry.push(topic_key);
        }
        map
    }

}