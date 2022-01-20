use super::*;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::cell::{RefCell, Ref};
use chrono::NaiveDate;

pub struct Topic {
    pub parents: Vec<TopicKey>,
    pub namespace: String,
    pub name: String,
    pub category: Option<String>,
    pub temp_attributes: BTreeMap<String, Vec<String>>,
    pub attributes: BTreeMap<String, AttributeInstance>,
    pub paragraphs: Vec<Paragraph>,
    pub inbound_topic_keys: Vec<TopicKey>,
    pub outbound_links: Vec<Link>,
    pub category_tree_node: Option<Rc<RefCell<TopicTreeNode>>>,
    pub subtopics: Vec<TopicKey>,
    pub subtopic_tree_node: Option<Rc<RefCell<TopicTreeNode>>>,
    pub combo_subtopics: Vec<TopicKey>,
    pub listed_topics: Vec<TopicKey>,
    //pub sections: Vec<SectionRc>,
    // pub sections: BTreeMap<String, usize>,
}

#[derive(Clone, Debug)]
pub struct TopicKey {
    pub namespace: String,
    pub topic_name: String,
}

#[derive(Clone, Debug)]
pub struct SectionKey {
    pub topic_key: TopicKey,
    pub section_name: String,
}

impl Topic {
    pub fn new(namespace: &str, name: &str) -> Self {
        Self {
            parents: vec![],
            namespace: namespace.to_string(),
            name: name.to_string(),
            category: None,
            temp_attributes: Default::default(),
            attributes: Default::default(),
            paragraphs: vec![],
            inbound_topic_keys: vec![],
            outbound_links: vec![],
            category_tree_node: None,
            subtopics: vec![],
            subtopic_tree_node: None,
            combo_subtopics: vec![],
            listed_topics: vec![],
            // sections: Default::default(),
        }
    }

    pub fn get_key(&self) -> TopicKey {
        TopicKey::new(&self.namespace, &self.name)
    }

    pub fn add_paragraph(&mut self, paragraph: Paragraph) {
        self.paragraphs.push(paragraph);
    }

    pub fn is_category(&self) -> bool {
        self.category_tree_node.as_ref().map_or(false, |node| b!(&node).height() > 1)
    }

    pub fn direct_subcategory_nodes(&self) -> Vec<Rc<RefCell<TopicTreeNode>>> {
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

    pub fn direct_topics_in_category(&self) -> Vec<TopicKey> {
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

    pub fn indirect_topics_in_category(&self) -> Vec<TopicKey> {
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

    pub fn has_section(&self, section_name: &str) -> bool {
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

    pub fn sort_topic_tree(tree: &mut TopicTree) {
        tree.sort_recursive(&|node: &Rc<RefCell<TopicTreeNode>>| b!(node).item.topic_name.clone());
    }

    pub fn check_subtopic_relationships(model: &Wiki) -> TopicErrorList {
        let mut errors = TopicErrorList::new();
        let err_msg_func = |msg: &str| format!("Topic::check_subtopic_relationships: {}", msg);
        let cat_combo = "Combinations".to_string();
        for topic in model.topics.values() {
            let topic_key = topic.get_key();
            let parent_count = topic.parents.len();
            if topic.category.as_ref().is_none() || topic.category.as_ref().unwrap().to_string() != cat_combo {
                // Not a combination topic.
                if parent_count > 1 {
                    errors.add(&topic_key, &format!("Non-combo category, so expected 0 or 1 parents, found {}.", parent_count));
                } else {
                    for parent_topic_key in topic.parents.iter() {
                        //bg!(&topic.name, parent_topic_key);
                        assert!(model.topics.contains_key(parent_topic_key), "No topic found for parent key = \"{:?}\" in topic = \"{}\". This should have been caught earlier.", parent_topic_key, topic.name);
                        if !model.topics[parent_topic_key].listed_topics.contains(&topic_key) {
                            errors.add(&parent_topic_key,&err_msg_func(&format!("[[{}]]", topic.name)));
                        }
                    }
                }
            } else {
                // Combination topic.
                if parent_count != 2 {
                    errors.add(&topic_key,&err_msg_func(&format!("Combo category, so expected 2 parents, found {}.", parent_count)));
                } else {
                    for parent_topic_key in topic.parents.iter() {
                        assert!(model.topics.contains_key(parent_topic_key), "No topic found for parent key = \"{:?}\" in topic = \"{}\". This should have been caught earlier.", parent_topic_key, topic.name);
                        if !model.topics[parent_topic_key].combo_subtopics.contains(&topic_key) {
                            errors.add(&parent_topic_key, &err_msg_func(&format!("No combination link to child [[{}]].", topic.name)));
                        }
                    }
                }
            }
        }
        errors
    }

    pub fn make_subtopic_tree(model: &mut Wiki) -> TopicTree {
        for topic in model.topics.values_mut() {
            topic.subtopics.clear();
            topic.combo_subtopics.clear();
        }
        let mut parent_child_pairs = vec![];
        let mut parent_combo_pairs = vec![];
        for topic in model.topics.values() {
            let topic_key = topic.get_key();
            match topic.parents.len() {
                0 => {
                    // This is not a subtopic.
                },
                1 => {
                    // Normal (non-combo) subtopic.
                    let parent_topic_key = topic.parents[0].clone();
                    parent_child_pairs.push((parent_topic_key, topic_key));
                },
                2 => {
                    // Combination topic.
                    for parent_topic_key in topic.parents.iter() {
                        parent_combo_pairs.push((parent_topic_key.clone(), topic_key.clone()))
                    }
                    // Don't include combination topics in the subcategory tree.
                },
                _ => {
                    panic!("Found {} parent topics for topic \"{}\". Expected either 1 or 2.", topic.parents.len(), topic.name);
                }
            }
        }
        for (parent_topic_key, child_topic_key) in parent_child_pairs.iter() {
            model.topics.get_mut(&parent_topic_key).unwrap().subtopics.push(child_topic_key.clone());
        }
        for (parent_topic_key, combo_topic_key) in parent_combo_pairs.iter() {
            model.topics.get_mut(&parent_topic_key).unwrap().combo_subtopics.push(combo_topic_key.clone());
        }
        for topic in model.topics.values_mut() {
            topic.subtopics.sort_by_cached_key(|topic_key| topic_key.topic_name.clone());
            topic.combo_subtopics.sort_by_cached_key(|topic_key| topic_key.topic_name.clone());
        }
        let mut tree = util::tree::Tree::create(parent_child_pairs, true);
        Topic::sort_topic_tree(&mut tree);
        // Have each topic with a subtopic point to its node in the subtopic tree.
        for topic in model.topics.values_mut() {
            topic.subtopic_tree_node = tree.get_node(&topic.get_key());
        }
        // tree.print_counts_to_depth();
        // tree.print_with_items(None);
        tree
    }

    pub fn set_attribute_date(&mut self, attr_type_name: &str, sequence: usize, value: &NaiveDate) {
        self.attributes.remove(attr_type_name);
        let mut attr_type = AttributeType::new(attr_type_name, &AttributeValueType::Date, sequence);
        attr_type.add_date_value(value, &self.get_key()).unwrap();
    }

}

impl PartialEq for Topic {
    fn eq(&self, other: &Self) -> bool {
        self.get_key() == other.get_key()
    }
}

impl Eq for Topic {
}

impl PartialOrd for Topic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_key().partial_cmp(&other.get_key())
    }
}

impl Ord for Topic {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_key().cmp(&other.get_key())
    }
}

impl TopicKey {
    pub fn new(namespace: &str, topic_name: &str) -> Self {
        assert!(!topic_name.contains(":="), "Topic name \"{}\" has a \":=\"", topic_name);
        assert!(!topic_name.starts_with("_"), "Topic name \"{}\" starts with \"_\"", topic_name);
        Self {
            namespace: namespace.to_lowercase().to_string(),
            topic_name: topic_name.to_string(),
        }
    }

    pub fn get_key(&self) -> String {
        format!("[{}:{}]", self.namespace.to_lowercase(), self.topic_name.to_lowercase())
    }

    pub fn sort_topic_keys_by_name(vec: &mut Vec<TopicKey>) {
        vec.sort_by_cached_key(|topic_key| topic_key.topic_name.to_lowercase());
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
    pub fn new(namespace: &str, topic_name: &str, section_name: &str) -> Self {
        assert!(!topic_name.contains(":="), "Topic name \"{}\" has a \":=\"", topic_name);
        assert!(!topic_name.starts_with("_"), "Topic name \"{}\" starts with \"_\"", topic_name);

        Self {
            topic_key: TopicKey::new(namespace, topic_name),
            section_name: section_name.to_string(),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.topic_key.namespace
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_key.topic_name
    }
}

impl Display for SectionKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "[{}:{}#{}]", self.topic_key.namespace, self.topic_key.topic_name, self.section_name)
    }
}
