use super::*;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::cell::{RefCell, Ref};
use chrono::NaiveDate;
use crate::dokuwiki::legal_file_name;

pub(crate) struct Topic {
    parents: Vec<LinkRc>,
    namespace: String,
    name: String,
    category: Option<String>,
    temp_attributes: BTreeMap<String, Vec<String>>,
    attributes: BTreeMap<String, AttributeInstance>,
    paragraphs: Vec<Paragraph>,
    inbound_topic_keys: Vec<TopicKey>,
    // outbound_links: Vec<Link>,
    // generated_outbound_links: Vec<Link>,
    category_tree_node: Option<Rc<RefCell<TopicTreeNode>>>,
    subtopics: Vec<LinkRc>,
    subtopic_tree_node: Option<Rc<RefCell<TopicTreeNode>>>,
    combo_subtopics: Vec<LinkRc>,
    // listed_topics: Vec<TopicKey>,
}

#[derive(Clone, Debug)]
pub(crate) struct TopicKey {
    namespace: String,
    topic_name: String,
}

#[derive(Clone, Debug)]
pub(crate) struct SectionKey {
    topic_key: TopicKey,
    section_name: String,
}

impl Topic {
    pub(crate) fn new(namespace: &str, name: &str) -> Self {
        TopicKey::assert_legal_namespace(namespace);
        Self {
            parents: vec![],
            namespace: namespace.to_string(),
            name: name.to_string(),
            category: None,
            temp_attributes: Default::default(),
            attributes: Default::default(),
            paragraphs: vec![],
            inbound_topic_keys: vec![],
            // outbound_links: vec![],
            // generated_outbound_links: vec![],
            category_tree_node: None,
            subtopics: vec![],
            subtopic_tree_node: None,
            combo_subtopics: vec![],
            // listed_topics: vec![],
            // sections: Default::default(),
        }
    }

    pub(crate) fn get_topic_key(&self) -> TopicKey {
        TopicKey::new(&self.namespace, &self.name)
    }

    pub(crate) fn add_parent(&mut self, link: LinkRc) {
        assert!(self.parents.len() < 2);
        self.parents.push(link);
    }

    pub(crate) fn set_parents(&mut self, parents: Vec<LinkRc>) {
        assert!(self.parents.is_empty());
        assert!(!parents.is_empty());
        assert!(parents.len() <= 2);
        self.parents = parents;
    }

    pub(crate) fn get_parent_count(&self) -> usize {
        self.parents.len()
    }

    pub(crate) fn get_parent(&self, index: usize) -> LinkRc {
        self.parents[index].clone()
    }

    pub(crate) fn get_namespace(&self) -> &str {
        &self.namespace
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn get_category(&self) -> Option<String> {
        if let Some(category) = &self.category {
            Some(category.clone())
        } else {
            None
        }
    }

    pub(crate) fn set_category(&mut self, category: &str) {
        debug_assert!(self.category.is_none());
        self.category = Some(category.to_string());
    }

    #[allow(dead_code)]
    pub(crate) fn get_temp_attributes(&self) -> &BTreeMap<String, Vec<String>> {
        &self.temp_attributes
    }

    pub(crate) fn has_temp_attribute(&self, attr_name: &str) -> bool {
        self.temp_attributes.contains_key(attr_name)
    }

    pub(crate) fn set_temp_attribute_date(&mut self, attr_type_name: &str, value: &NaiveDate) {
        self.temp_attributes.remove(attr_type_name);
        self.temp_attributes.insert(attr_type_name.to_string(), vec![AttributeType::date_to_canonical_value(value)]);
    }

    pub(crate) fn catalog_attributes(&mut self, errors: &mut TopicErrorList, attribute_types: &mut BTreeMap<String, AttributeType>, attribute_values: &mut BTreeMap<String, Vec<(TopicKey, String)>>, attribute_orders: &BTreeMap<String, usize>) {
        // At this point we may have a mixture of temp attributes and resolved attributes in this
        // topic. Any temp attributes will end up replacing resolved attributes with the same name.

        // The earlier parsing stage may have left some temp attributes that have no values,
        // either because they were empty in the source text or because they had only
        // placeholder values like "***". We want to ignore those cases.
        let temp_attributes = std::mem::replace(&mut self.temp_attributes, BTreeMap::new());
        for (temp_attr_name, temp_attr_values) in temp_attributes.iter()
                .filter(|(_name, values)| !values.is_empty()) {
            AttributeType::assert_legal_attribute_type_name(temp_attr_name);
            // The list of attribute types is shared among all topics, so we want only one
            // entry each for "Added", "Title", and so on.
            let attribute_type = attribute_types.entry(temp_attr_name.clone())
                .or_insert({
                    // We don't yet have this attribute type in the master list. Based on the
                    // string values in the temp attribute for this topic, guess the type such
                    // as a date, year, or boolean. If this ever turns out to be wrong we can
                    // use a more elaborate method to figure out the type.
                    let value_type = AttributeType::value_to_presumed_type(temp_attr_name,&*temp_attr_values[0]);
                    // The sequence determines how the attributes will be ordered in the
                    // generated wiki. For instance, Author, Title, Narrator, and Translator
                    // will appear together.
                    let sequence = attribute_orders.get(temp_attr_name).map_or_else(
                        || {
                            errors.add(&self.get_topic_key(), &format!("No sequence found for attribute type \"{}\".", temp_attr_name));
                            ATTRIBUTE_ORDER.len()
                        },
                        |sequence| { *sequence }
                    );
                    AttributeType::new(temp_attr_name, &value_type, sequence)
                });
            // This list holds the canonical forms of the attribute values for this topic. At
            // the end it will be attached to the topic through an AttributeInstance owned by
            // the topic.
            let mut values_for_topic = vec![];
            for temp_value in temp_attr_values.iter() {
                AttributeType::assert_legal_attribute_value(temp_value);
                // If this attribute type does not have the value, add it. Then either way add
                // a reference to the topic, showing that this topic has this value for this
                // attribute type.
                match attribute_type.add_value_for_topic(temp_value,&self.get_topic_key()) {
                    Ok(canonical_value) => {
                        AttributeType::assert_legal_attribute_value(&canonical_value);
                        // Don't add a topic item if the topic has itself as an attribute.
                        if self.name.ne(&canonical_value) {
                            let entry = attribute_values.entry(canonical_value.clone()).or_insert(vec![]);
                            entry.push((self.get_topic_key(), attribute_type.get_name().to_string()));
                            // model.add_attribute_value(canonical_value.clone(), topic.get_key(), attribute_type.get_name().clone());
                        }
                        values_for_topic.push(canonical_value)
                    },
                    Err(msg) => { errors.add(&self.get_topic_key(), &msg) }
                };
            }
            self.replace_attribute(AttributeInstance::new(temp_attr_name, attribute_type.get_sequence(),values_for_topic));
        }
    }

    pub(crate) fn add_temp_attribute_values(&mut self, attr_type_name: String, mut values: Vec<String>) {
        let entry = self.temp_attributes.entry(attr_type_name).or_insert(vec![]);
        entry.append(&mut values);
    }

    pub(crate) fn add_or_find_temp_attribute(&mut self, name: &str) -> &mut Vec<String> {
        self.temp_attributes.entry(name.to_string()).or_insert(vec![])
    }

    pub(crate) fn get_attribute_count(&self) -> usize {
        self.attributes.len()
    }

    #[allow(dead_code)]
    pub(crate) fn get_attribute_value_count(&self, attr_type_name: &str) -> usize {
        self.attributes.get(attr_type_name).map_or(0, |attr_instance| attr_instance.get_values().len())
    }

    pub(crate) fn get_attributes(&self) -> &BTreeMap<String, AttributeInstance> {
        &self.attributes
    }

    /*
    pub(crate) fn get_attribute(&self, attr_type_name: &str) -> &Option<&AttributeInstance> {
        &self.attributes.get(attr_type_name)
    }
     */

    /*
    pub(crate) fn has_attribute(&mut self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }
     */

    pub(crate) fn add_attribute(&mut self, attr_instance: AttributeInstance) {
        let key = attr_instance.get_attribute_type_name();
        assert!(!self.attributes.contains_key(key));
        self.attributes.insert(key.to_string(), attr_instance);
    }

    pub(crate) fn replace_attribute(&mut self, attr_instance: AttributeInstance) {
        let key = attr_instance.get_attribute_type_name();
        self.attributes.remove(key);
        self.add_attribute(attr_instance);
    }

    /*
    pub(crate) fn set_attribute_date(&mut self, attr_type_name: &str, sequence: usize, value: &NaiveDate) {
        AttributeType::assert_legal_attribute_type_name(attr_type_name);
        self.attributes.remove(attr_type_name);
        let mut attr_type = AttributeType::new(attr_type_name, &AttributeValueType::Date, sequence);
        attr_type.add_date_value(value, &self.get_topic_key()).unwrap();
        let attr_instance = AttributeInstance::new_date(attr_type_name, sequence, vec![value]);
        self.add_attribute(attr_instance);
    }
    */
    /*
    pub(crate) fn clear_attributes(&mut self) {
        self.attributes.clear()
    }
    */

    pub(crate) fn get_paragraph_count(&self) -> usize {
        self.paragraphs.len()
    }

    pub(crate) fn get_paragraphs(&self) -> &Vec<Paragraph> {
        &self.paragraphs
    }

    /*
    pub(crate) fn get_paragraphs_mut(&mut self) -> &mut Vec<Paragraph> {
        &mut self.paragraphs
    }
    */

    /*
    pub(crate) fn get_paragraph(&self, index: usize) -> &Paragraph {
        &self.paragraphs[index]
    }
    */

    pub(crate) fn replace_paragraph(&mut self, paragraph_index: usize, paragraph: Paragraph) -> Paragraph {
        std::mem::replace(&mut self.paragraphs[paragraph_index], paragraph)
    }

    pub(crate) fn replace_paragraph_with_placeholder(&mut self, paragraph_index: usize) -> Paragraph {
        self.replace_paragraph(paragraph_index,Paragraph::Placeholder)
    }

    pub(crate) fn find_list_paragraph_by_type(&self, type_name: &str) -> Option<usize> {
        for (index, paragraph) in self.paragraphs.iter().enumerate() {
            match paragraph {
                Paragraph::List { list } => {
                    if list.get_type().eq(type_name) {
                        return Some(index);
                    }
                }
                _ => {},
            }
        }
        None
    }

    pub(crate) fn remove_list_paragraph_by_type(&mut self, type_name: &str) -> Option<(usize, Paragraph)> {
        match self.find_list_paragraph_by_type(type_name) {
            Some(index) => Some((index, self.paragraphs.remove(index))),
            None => None,
        }
    }

    pub(crate) fn add_paragraph(&mut self, paragraph: Paragraph) {
        self.paragraphs.push(paragraph);
    }

    pub(crate) fn insert_paragraph(&mut self, index: usize, paragraph: Paragraph) {
        self.paragraphs.insert(index, paragraph);
    }

    pub(crate) fn get_paragraph_index_end_of_first_section(&self) -> usize {
        for (index, paragraph) in self.paragraphs.iter().enumerate() {
            match paragraph {
                Paragraph::SectionHeader { name: _, depth } => {
                    if *depth == 1 {
                        return index;
                    }
                },
                _ => {},
            }
        }
        self.paragraphs.len()
    }

    /*
    pub(crate) fn get_outbound_links(&self) -> &Vec<Link> {
        &self.outbound_links
    }
    */

    /*
    pub(crate) fn add_outbound_link(&mut self, link: Link) {
        self.outbound_links.push(link)
    }

    pub(crate) fn add_outbound_links(&mut self, mut links: Vec<Link>) {
        self.outbound_links.append(&mut links)
    }
    */

    pub(crate) fn get_inbound_topic_keys(&self) -> &Vec<TopicKey> {
        &self.inbound_topic_keys
    }

    /*
    pub(crate) fn add_inbound_topic_key(&mut self, topic_key: TopicKey) {
        self.inbound_topic_keys.push(topic_key);
    }
    *

     */
    pub(crate) fn clear_inbound_topic_keys(&mut self) {
        self.inbound_topic_keys.clear();
    }

    pub(crate) fn finalize_inbound_topic_keys(&mut self) {
        TopicKey::sort_topic_keys_by_name(&mut self.inbound_topic_keys);
        self.inbound_topic_keys.dedup();
    }

    /*
    pub(crate) fn get_inbound_topic_keys_count(&self) -> usize {
        self.inbound_topic_keys.len()
    }
    */

    pub(crate) fn set_inbound_topic_keys(&mut self, topic_keys: Vec<TopicKey>) {
        // let debug = self.name.eq("Algorithms (Rust project)");
        // if debug { //bg!(&topic_keys); }
        self.inbound_topic_keys = topic_keys;
    }

    /*
    pub(crate) fn add_inbound_topic_keys(&mut self, mut topic_keys: Vec<TopicKey>) {
        self.inbound_topic_keys.append(&mut topic_keys);
    }
     */

    /*
    pub(crate) fn clear_inbound_topic_keys(&mut self) {
        self.inbound_topic_keys.clear();
    }
    */

    pub(crate) fn get_category_tree_node(&self) -> &Option<Rc<RefCell<TopicTreeNode>>> {
        &self.category_tree_node
    }

    pub(crate) fn set_category_tree_node(&mut self, node: Option<Rc<RefCell<TopicTreeNode>>>) {
        self.category_tree_node = node
    }

    /*
    pub(crate) fn add_subtopic(&mut self, topic_key: TopicKey) {
        self.subtopics.push(topic_key);
    }
    */

    /*
    pub(crate) fn clear_subtopics(&mut self) {
        self.subtopics.clear();
    }
    */

    pub(crate) fn get_subtopic_tree_node(&self) -> &Option<Rc<RefCell<TopicTreeNode>>> {
        &self.subtopic_tree_node
    }

    pub(crate) fn get_combo_subtopics(&self) -> &Vec<LinkRc> {
        &self.combo_subtopics
    }

    /*
    pub(crate) fn add_combo_subtopic(&mut self, topic_key: TopicKey) {
        self.combo_subtopics.push(topic_key);
    }
    */

    /*
    pub(crate) fn clear_combo_subtopics(&mut self) {
        self.combo_subtopics.clear()
    }
    */

    pub(crate) fn is_category(&self) -> bool {
        self.category_tree_node.as_ref().map_or(false, |node| b!(&node).height() > 1)
    }

    /*
    pub(crate) fn direct_subcategory_nodes(&self) -> Vec<Rc<RefCell<TopicTreeNode>>> {
        // Get all of the topics corresponding to the non-leaf nodes directly under this one.
        match &self.category_tree_node {
            Some(node_rc) => {
                let node = b!(node_rc);
                // If the child topic is a category topic, it will have at least one child of its
                // own in the category tree and thus will not be a leaf.
                let filter_func = |found_node: Ref<TopicTreeNode>| !found_node.is_leaf();
                let mut child_nodes = node.get_direct_child_nodes(&filter_func);
                child_nodes.sort_by_cached_key(|child_node_rc| b!(child_node_rc).item.topic_name.clone());
                child_nodes
            },
            None => vec![],
        }
    }
    */

    pub(crate) fn direct_topics_in_category(&self) -> Vec<TopicKey> {
        match &self.category_tree_node {
            Some(node_rc) => {
                let node = b!(node_rc);
                // If the child topic is a category topic, it will have at least one child of its
                // own in the category tree and thus will not be a leaf.
                let filter_func = |found_node: Ref<TopicTreeNode>| found_node.is_leaf();
                let mut topic_keys = node.get_direct_child_items(&filter_func);
                TopicKey::sort_topic_keys_by_name(&mut topic_keys);
                topic_keys
            },
            None => vec![],
        }
    }

    pub(crate) fn indirect_topics_in_category(&self) -> Vec<TopicKey> {
        match &self.category_tree_node {
            Some(node_rc) => {
                let node = b!(node_rc);
                let filter_func = |found_node: Ref<TopicTreeNode>| found_node.is_leaf();
                let mut topic_keys = node.get_indirect_child_items(&filter_func);
                TopicKey::sort_topic_keys_by_name(&mut topic_keys);
                topic_keys
            },
            None => vec![],
        }
    }

    pub(crate) fn has_section(&self, section_name: &str) -> bool {
        let section_name = section_name.to_lowercase();
        // let debug = section_name.contains("cognitive");
        // if debug { //bg!(&self.name, &section_name); }
        for paragraph in self.paragraphs.iter() {
            match paragraph {
                Paragraph::SectionHeader { name, .. } => {
                    // if debug { //bg!(&name); }
                    if name.to_lowercase() == section_name {
                        // if debug { //bg!("found section"); }
                        return true;
                    }
                },
                _ => {},
            }
        }
        // if debug { //bg!("didn't find section"); }
        false
    }

    pub(crate) fn sort_topic_tree(tree: &mut TopicTree) {
        tree.sort_recursive(&|node: &Rc<RefCell<TopicTreeNode>>| b!(node).item.topic_name.clone());
    }

    pub(crate) fn check_subtopic_relationships(model: &Model) -> TopicErrorList {
        let mut errors = TopicErrorList::new();
        let err_msg_func = |msg: &str| format!("Topic::check_subtopic_relationships: {}", msg);
        let cat_combo = "Combinations".to_string();
        for topic in model.get_topics().values() {
            let topic_key = topic.get_topic_key();
            let parent_count = topic.parents.len();
            if topic.category.as_ref().is_none() || topic.category.as_ref().unwrap().to_string() != cat_combo {
                // Not a combination topic.
                if parent_count > 1 {
                    errors.add(&topic_key, &format!("Non-combo category, so expected 0 or 1 parents, found {}.", parent_count));
                } else {
                    for parent_link_rc in topic.parents.iter() {
                        let parent_topic_key = b!(parent_link_rc).get_topic_key().unwrap();
                        //bg!(topic.get_name(), parent_topic_key);
                        assert!(model.get_topics().contains_key(&parent_topic_key), "No topic found for parent key = \"{:?}\" in topic = \"{}\". This should have been caught earlier.", parent_topic_key, topic.get_name());
                        // if !link_list_contains_topic_key(&model.get_topics()[&parent_topic_key].listed_topics, &topic_key) {
                        //     errors.add(&parent_topic_key,&err_msg_func(&format!("[[{}]]", topic.get_name())));
                        // }
                    }
                }
            } else {
                // Combination topic.
                if parent_count != 2 {
                    errors.add(&topic_key,&err_msg_func(&format!("Combo category, so expected 2 parents, found {}.", parent_count)));
                } else {
                    for parent_link_rc in topic.parents.iter() {
                        let parent_topic_key = b!(parent_link_rc).get_topic_key().unwrap();
                        assert!(model.get_topics().contains_key(&parent_topic_key), "No topic found for parent key = \"{:?}\" in topic = \"{}\". This should have been caught earlier.", parent_topic_key, topic.get_name());
                        if !link_list_contains_topic_key(&model.get_topics()[&parent_topic_key].combo_subtopics, &topic_key) {
                            errors.add(&parent_topic_key, &err_msg_func(&format!("No combination link to child [[{}]].", topic.get_name())));
                        }
                    }
                }
            }
        }
        errors
    }

    pub(crate) fn make_subtopic_tree(model: &mut Model) -> TopicTree {
        for topic in model.get_topics_mut().values_mut() {
            topic.subtopics.clear();
            topic.combo_subtopics.clear();
        }
        let mut parent_child_pairs = vec![];
        let mut parent_combo_pairs = vec![];
        for topic in model.get_topics().values() {
            let topic_key = topic.get_topic_key();
            match topic.parents.len() {
                0 => {
                    // This is not a subtopic.
                },
                1 => {
                    // Normal (non-combo) subtopic.
                    let parent_topic_key = b!(&topic.parents[0]).get_topic_key().unwrap();
                    parent_child_pairs.push((parent_topic_key, topic_key));
                },
                2 => {
                    // Combination topic.
                    for parent_link_rc in topic.parents.iter() {
                        parent_combo_pairs.push((b!(parent_link_rc).get_topic_key().unwrap(), topic_key.clone()))
                    }
                    // Don't include combination topics in the subcategory tree.
                },
                _ => {
                    panic!("Found {} parent topics for topic \"{}\". Expected either 1 or 2.", topic.parents.len(), topic.get_name());
                }
            }
        }
        for (parent_topic_key, child_topic_key) in parent_child_pairs.iter() {
            // let parent_topic_key= b!(parent_link_rc).get_topic_key().unwrap();
            let link_rc = r!(Link::new_topic_from_key(None, child_topic_key));
            model.get_topics_mut().get_mut(&parent_topic_key).unwrap().subtopics.push(link_rc);
        }
        for (parent_topic_key, combo_topic_key) in parent_combo_pairs.iter() {
            let link_rc = r!(Link::new_topic_from_key(None, combo_topic_key));
            model.get_topics_mut().get_mut(&parent_topic_key).unwrap().combo_subtopics.push(link_rc);
        }
        let mut tree = util::tree::Tree::create(parent_child_pairs, true);
        Topic::sort_topic_tree(&mut tree);
        // Have each topic with a subtopic point to its node in the subtopic tree.
        for topic in model.get_topics_mut().values_mut() {
            topic.subtopic_tree_node = tree.get_node(&topic.get_topic_key());
        }
        // tree.print_counts_to_depth();
        // tree.print_with_items(None);
        tree
    }

    /*
    pub(crate) fn catalog_outbound_links(&mut self) {
        self.outbound_links.clear();
        self.listed_topics.clear();
        self.subtopics.clear();
        self.combo_subtopics.clear();
        for paragraph in self.paragraphs.iter() {
            match paragraph {
                Paragraph::List { list } => {
                    let (is_combos, is_subtopics) = match list.get_type() {
                        ListType::Combinations => (true, false),
                        ListType::Subtopics => (false, true),
                        _ => (false, false),
                    };
                    if let Some(header) = list.get_header() {
                        let mut text_block_links = Link::catalog_links_text_block(header);
                        self.outbound_links.append(&mut text_block_links);
                    }
                    for list_item in list.get_items().iter() {
                        // if list_item.get_depth() == 1 {
                        let mut links = Link::catalog_links_text_block(&list_item.get_text_block());
                        for link in links.iter() {
                            match &link.get_type() {
                                LinkType::Topic { topic_key } => {
                                    if !self.listed_topics.contains(topic_key) {
                                        self.listed_topics.push(topic_key.clone());
                                    }
                                    // self.add_listed_topic_optional(topic_key);
                                    if is_combos {
                                        // self.add_combo_subtopic(topic_key.clone());
                                        self.combo_subtopics.push(topic_key.clone());
                                    } else if is_subtopics {
                                        // self.add_subtopic(topic_key.clone());
                                        self.subtopics.push(topic_key.clone());
                                    }
                                    break;
                                },
                                _ => {},
                            }
                        }
                        self.outbound_links.append(&mut links);
                    }
                },
                Paragraph::Text { text_block } => {
                    let mut links = Link::catalog_links_text_block(text_block);
                    self.outbound_links.append(&mut links);
                },
                _ => {},
            }
        }

        TopicKey::sort_topic_keys_by_name(&mut self.subtopics);
        TopicKey::sort_topic_keys_by_name(&mut self.combo_subtopics);
        TopicKey::sort_topic_keys_by_name(&mut self.listed_topics);
    }
    */

    /*
    pub(crate) fn get_topic_links_as_topic_keys(&self, include_generated: bool) -> Vec<TopicKey> {
        assert!(!include_generated, "The Topic.include_generated list is not getting filled yet.");
        let mut links = self.outbound_links.clone();
        if include_generated {
            links.append(&mut self.generated_outbound_links.clone());
        }
        let mut topic_keys = links.iter()
            .filter_map(|link| {
                match link.get_type() {
                    LinkType::Topic { topic_key } => Some(topic_key.clone()),
                    LinkType::Section { section_key } => Some(section_key.get_topic_key().clone()),
                    _ => None,
                }
            })
            .collect();
        TopicKey::sort_topic_keys_by_name(&mut topic_keys);
        topic_keys.dedup();
        topic_keys
    }
    */

    pub(crate) fn get_links(&self, include_generated: bool, dependencies_are_generated: bool) -> Vec<LinkRc> {
        // let debug = self.name.eq("Criterion (Rust crate)");
        let mut links = vec![];
        // for parent in self.parents.iter() {
        //     links.push(parent.clone());
        // }
        for paragraph in self.paragraphs.iter() {
            links.append(&mut paragraph.get_links(include_generated, dependencies_are_generated));
        }
        // links.append(&mut self.subtopics.iter().map(|link_rc| link_rc.clone()).collect());
        // links.append(&mut self.combo_subtopics.iter().map(|link_rc| link_rc.clone()).collect());
        // if debug { Link::print_link_ref_list(&links, self.get_name()); //bg!(include_generated, dependencies_are_generated); }
        links
    }

    /*
    pub(crate) fn check_links(model: &Model) -> TopicErrorList {
        //bg!(model.get_topics().keys());
        let mut errors = TopicErrorList::new();
        for topic in model.get_topics().values() {
            let this_topic_key = topic.get_topic_key();
            for link in topic.outbound_links.iter() {
                match &link.get_type() {
                    LinkType::Topic { topic_key } => {
                        model.check_topic_link(&mut errors, "outbound_links", &this_topic_key, topic_key);
                    },
                    LinkType::Section { section_key } => {
                        //bg!(&section_key);
                        if !model.has_section(section_key) {
                            errors.add(&topic.get_topic_key(), &format!("wiki::check_links(): Section link {} not found.", section_key));
                        }
                    },
                    _ => {},
                }
            }
            topic.parents.iter().for_each(|ref_topic_key| { model.check_topic_link(&mut errors, "parents", &this_topic_key, ref_topic_key); } );
            topic.inbound_topic_keys.iter().for_each(|ref_topic_key| { model.check_topic_link(&mut errors, "inbound_topic_keys", &this_topic_key, ref_topic_key); } );
            topic.subtopics.iter().for_each(|ref_topic_key| { model.check_topic_link(&mut errors, "subtopics", &this_topic_key, ref_topic_key); } );
            topic.combo_subtopics.iter().for_each(|ref_topic_key| { model.check_topic_link(&mut errors, "combo_subtopics", &this_topic_key, ref_topic_key); } );
            topic.listed_topics.iter().for_each(|ref_topic_key| { model.check_topic_link(&mut errors, "listed_topics", &this_topic_key, ref_topic_key); } );
        }
        errors
    }
    */
    /*
    pub(crate) fn update_internal_links(&mut self, keys: &Vec<(TopicKey, TopicKey)>) {
        //bg!(&keys);
        // For each entry in keys, the first TopicKey is the old value and the second is the new
        // value.
        for paragraph in self.paragraphs.iter_mut() {
            match paragraph {
                Paragraph::List { type_: _, header, items} => {
                    header.update_internal_links(keys);
                    for item in items.iter_mut() {
                        item.get_text_block_mut().update_internal_links(keys);
                    }
                },
                Paragraph::Table { table} => {
                    for row in table.get_rows_mut().iter_mut() {
                        for cell in row.iter_mut() {
                            cell.update_internal_links(keys);
                        }
                    }
                },
                Paragraph::Text { text_block} => {
                    text_block.update_internal_links(keys);
                },
                _ => {},
            }
        }
        if !self.parents.is_empty() {
            let old_parents = self.parents.clone();
            self.parents.clear();
            for parent_topic_key in old_parents.iter() {
                let mut new_parent_topic_key = parent_topic_key.clone();
                for (topic_key_old, topic_key_new) in keys.iter() {
                    if parent_topic_key.eq(&topic_key_old) {
                        new_parent_topic_key = topic_key_new.clone();
                        break;
                    }
                }
                self.parents.push(new_parent_topic_key);
            }
        }
    }
     */

    pub(crate) fn assert_all_text_blocks_resolved(&self) {
        for text_block in self.get_all_text_blocks_cloned().iter() {
            match text_block {
                TextBlock::Resolved { .. } => {},
                TextBlock::Unresolved { text} => {
                    panic!("In topic \"{}\", expected all text blocks to be resolved. Found text = \"{}\".", self.name, text)
                }
            }
        }
    }

    pub(crate) fn get_all_text_blocks_cloned(&self) -> Vec<TextBlock> {
        let mut text_blocks = vec![];
        for paragraph in self.paragraphs.iter() {
            text_blocks.append(&mut paragraph.get_all_text_blocks_cloned());
        }
        text_blocks
    }

    pub(crate) fn get_file_monitor_file<'a>(&self, summary: &'a file_monitor::summary::Summary) -> Option<&'a file_monitor::summary::MonitoredFile> {
        let path = self.namespace.replace(":", "/");
        let file_name = format!("{}.txt", legal_file_name(self.get_name()));
        let key = format!("{}/{}", path, file_name);
        if self.name.eq("Yew (crate)") { dbg!(&key, summary.files.contains_key(&key)); }
        summary.files.get(&key)
    }
}

impl PartialEq for Topic {
    fn eq(&self, other: &Self) -> bool {
        self.get_topic_key() == other.get_topic_key()
    }
}

impl Eq for Topic {
}

impl PartialOrd for Topic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_topic_key().partial_cmp(&other.get_topic_key())
    }
}

impl Ord for Topic {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_topic_key().cmp(&other.get_topic_key())
    }
}

impl TopicKey {
    pub(crate) fn new(namespace: &str, topic_name: &str) -> Self {
        if topic_name.eq("functional_programming") { panic!() }
        Self::assert_legal_namespace(namespace);
        Self::assert_legal_topic_name(topic_name);
        Self {
            namespace: namespace.to_lowercase().to_string(),
            topic_name: topic_name.to_string(),
        }
    }

    pub(crate) fn get_key(&self) -> String {
        format!("[{}:{}]", self.namespace.to_lowercase(), self.topic_name.to_lowercase())
    }

    pub(crate) fn get_namespace(&self) -> &str {
        &self.namespace
    }

    pub(crate) fn get_topic_name(&self) -> &str {
        &self.topic_name
    }

    pub(crate) fn sort_topic_keys_by_name(vec: &mut Vec<TopicKey>) {
        vec.sort_by_cached_key(|topic_key| topic_key.topic_name.to_lowercase());
    }

    pub(crate) fn assert_legal_topic_name(topic_name: &str) {
        if topic_name != topic_name.trim() {
            panic!("Topic name \"{}\" is not trimmed.", topic_name);
        }
        if topic_name.contains(":=")
            || topic_name.contains("[")
            || topic_name.contains("]")
            || topic_name.starts_with("_") {
            panic!("Topic name \"{}\" contains invalid characters.", topic_name);
        }
    }

    pub(crate) fn assert_legal_namespace(namespace: &str) {
        if namespace != namespace.trim() {
            panic!("Namespace \"{}\" is not trimmed.", namespace);
        }
        namespace.chars().for_each(|c| {
            if !(c.is_ascii_lowercase() || c == ':') {
                panic!("Namespace name \"{}\" contains invalid characters.", namespace);
            }
        });
    }

    pub(crate) fn get_display_text(&self) -> String {
        format!("{}:{}", self.namespace, self.topic_name)
    }

}

impl Display for TopicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "[{}:{}]", self.namespace, self.topic_name)
    }
}

impl PartialEq for TopicKey {
    fn eq(&self, other: &Self) -> bool {
        self.get_key() == other.get_key()
    }
}

impl Eq for TopicKey {
}

impl PartialOrd for TopicKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_key().partial_cmp(&other.get_key())
    }
}

impl Ord for TopicKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_key().cmp(&other.get_key())
    }
}

impl SectionKey {
    pub(crate) fn new(namespace: &str, topic_name: &str, section_name: &str) -> Self {
        TopicKey::assert_legal_namespace(namespace);
        TopicKey::assert_legal_topic_name(topic_name);

        Self {
            topic_key: TopicKey::new(namespace, topic_name),
            section_name: section_name.to_string(),
        }
    }

    pub(crate) fn get_topic_key(&self) -> &TopicKey {
        &self.topic_key
    }

    pub(crate) fn get_namespace(&self) -> &str {
        self.topic_key.get_namespace()
    }

    pub(crate) fn get_topic_name(&self) -> &str {
        self.topic_key.get_topic_name()
    }

    pub(crate) fn get_section_name(&self) -> &str {
        &self.section_name
    }

    pub(crate) fn get_display_text(&self) -> String {
        format!("{}#{}", self.get_topic_key().get_display_text(), self.section_name)
    }

}

impl Display for SectionKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "[{}:{}#{}]", self.topic_key.namespace, self.topic_key.topic_name, self.section_name)
    }
}

pub(crate) fn make_topic_ref(namespace: &str, topic_name: &str) -> String {
    // For now use the DokuWiki conventions. If we later need to generate a different wiki format,
    // create a function like this for each wiki engine and pass it to the gen process.
    let canonical_topic_name = super::dokuwiki::gen::legal_file_name(topic_name);
    let topic_ref = format!("{}{}{}", namespace, super::dokuwiki::DELIM_NAMESPACE, canonical_topic_name);
    topic_ref
}
