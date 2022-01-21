use std::rc::Rc;
use std::cell::RefCell;
use crate::model::{Model, TopicKey, Topic, TopicTree};

// use super::*;

pub type CategoryRc = Rc<RefCell<Category>>;

pub struct Category {
    // pub wiki: WikiRc,
    parent: Option<CategoryRc>,
    name: String,
}

impl Category {
    // pub fn new(wiki: &WikiRc, parent: Option<&CategoryRc>, name: &str) -> Self {
    pub fn new(parent: Option<&CategoryRc>, name: &str) -> Self {
        Self {
            // wiki: wiki.clone(),
            parent: parent.map(|category_rc| category_rc.clone()),
            name: name.to_string(),
        }
    }

    pub fn add_missing_category_topics(model: &mut Model) {
        // First make sure we have a category entry for each category referenced in a topic.
        let mut category_names = model.get_topics().values()
            .filter_map(|topic| topic.get_category())
            .map(|category_name| category_name.to_string())
            .collect::<Vec<_>>();
        category_names.sort();
        category_names.dedup();
        for category_name in category_names.iter() {
            model.add_category_optional(category_name);
        }

        // Make sure that there's a topic for each category, and where there's already a topic,
        // change its namespace.
        let category_names = model.get_categories().values()
            .map(|category| category.name.clone())
            .collect::<Vec<_>>();
        let category_namespace = model.get_main_namespace().clone();
        let mut topic_keys = vec![];
        for category_name in category_names.iter() {
            let topic_key_old = TopicKey::new(&model.get_main_namespace(), category_name);
            let found = model.get_topics().contains_key(&topic_key_old);
            if found {
                // Move the topic from the main to the category namespace.
                //rintln!("\t\t\tMoving topic {}", &category_name);
                let mut topic = model.remove_topic(&topic_key_old).unwrap();
                let topic_key_new = TopicKey::new(&category_namespace, topic.get_name());
                topic_keys.push((topic_key_old, topic_key_new));
                topic.set_namespace(category_namespace.to_string());
                model.add_topic(topic);
            } else {
                //rintln!("Creating topic for {:?}.", &topic_key_new);
                model.add_topic(Topic::new(&category_namespace, category_name));
            }
        }
        model.update_internal_links(&topic_keys);
    }

    pub fn move_topics_to_namespace_by_category(model: &mut Model, category_name: &str, namespace_name: &str) {
        TopicKey::assert_legal_namespace(namespace_name);
        let topic_names = model.get_topics().values()
            .filter(|topic| topic.get_category().map_or(false,|cat| cat.eq_ignore_ascii_case(category_name)))
            .map(|topic| topic.get_name().clone())
            .collect::<Vec<_>>();
        //bg!(category_name, namespace_name, &topic_names);
        let mut topic_keys = vec![];
        for topic_name in topic_names {
            //rintln!("Moving topic {} to namespace {}.", &topic_name, &new_namespace);
            let topic_key_old = TopicKey::new(&model.get_main_namespace(), &topic_name);
            let topic_key_new = TopicKey::new(namespace_name, &topic_name);
            let mut topic = model.remove_topic(&topic_key_old).unwrap();
            topic.set_namespace(namespace_name.to_string());
            model.add_topic(topic);
            topic_keys.push((topic_key_old, topic_key_new));
        }
        model.update_internal_links(&topic_keys);
    }

    pub fn make_category_tree(model: &mut Model) -> TopicTree {
        let mut parent_child_pairs = vec![];
        for topic in model.get_topics().values() {
            if let Some(category_name) = topic.get_category() {
                debug_assert!(!category_name.eq("Terms"), "Topic is {}", topic.get_name());
                let category_topic_key = TopicKey::new(&model.get_main_namespace(), category_name);
                parent_child_pairs.push((category_topic_key, topic.get_key()));
            }
        }
        let mut tree = util::tree::Tree::create(parent_child_pairs, true);
        Topic::sort_topic_tree(&mut tree);
        // Have each category topic point to its node in the category tree.
        for topic in model.get_topics_mut().values_mut() {
            topic.set_category_tree_node(tree.get_node(&topic.get_key()));
        }
        tree
    }
}