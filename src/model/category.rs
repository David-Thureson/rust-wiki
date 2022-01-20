use std::rc::Rc;
use std::cell::RefCell;
use crate::model::{Wiki, TopicKey, Topic, TopicTree};

// use super::*;

pub type CategoryRc = Rc<RefCell<Category>>;

pub struct Category {
    // pub wiki: WikiRc,
    pub parent: Option<CategoryRc>,
    pub name: String,
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

    pub fn add_missing_category_topics(model: &mut Wiki) {
        // First make sure we have a category entry for each category referenced in a topic.
        let mut category_names = model.topics.values()
            .filter_map(|topic| topic.category.as_ref())
            .map(|category_name| category_name.clone())
            .collect::<Vec<_>>();
        category_names.sort();
        category_names.dedup();
        for category_name in category_names.iter() {
            if !model.categories.contains_key(category_name) {
                model.categories.insert(category_name.to_string(), Category::new(None, category_name) );
            }
        }

        // Make sure that there's a topic for each category, and where there's already a topic,
        // change its namespace.
        let category_names = model.categories.values()
            .map(|category| category.name.clone())
            .collect::<Vec<_>>();
        let category_namespace = model.main_namespace.clone();
        let mut topic_keys = vec![];
        for category_name in category_names.iter() {
            let topic_key_old = TopicKey::new(&model.main_namespace, category_name);
            let found = model.topics.contains_key(&topic_key_old);
            if found {
                // Move the topic from the main to the category namespace.
                //rintln!("\t\t\tMoving topic {}", &category_name);
                let mut topic = model.topics.remove(&topic_key_old).unwrap();
                let topic_key_new = TopicKey::new(&category_namespace, &topic.name);
                topic_keys.push((topic_key_old, topic_key_new));
                topic.namespace = category_namespace.to_string();
                model.add_topic(topic);
            } else {
                //rintln!("Creating topic for {:?}.", &topic_key_new);
                model.add_topic(Topic::new(&category_namespace, category_name));
            }
        }
        model.update_internal_links(&topic_keys);
    }

    pub fn move_topics_to_namespace_by_category(model: &mut Wiki, category_name: &str, namespace_name: &str) {
        TopicKey::assert_legal_namespace(namespace_name);
        let topic_names = model.topics.values()
            .filter(|topic| topic.category.as_ref().map_or(false,|cat| cat.eq_ignore_ascii_case(category_name)))
            .map(|topic| topic.name.clone())
            .collect::<Vec<_>>();
        //bg!(category_name, namespace_name, &topic_names);
        let mut topic_keys = vec![];
        for topic_name in topic_names {
            //rintln!("Moving topic {} to namespace {}.", &topic_name, &new_namespace);
            let topic_key_old = TopicKey::new(&model.main_namespace, &topic_name);
            let topic_key_new = TopicKey::new(namespace_name, &topic_name);
            let mut topic = model.topics.remove(&topic_key_old).unwrap();
            topic.namespace = namespace_name.to_string();
            model.add_topic(topic);
            topic_keys.push((topic_key_old, topic_key_new));
        }
        model.update_internal_links(&topic_keys);
    }

    pub fn make_category_tree(model: &mut Wiki) -> TopicTree {
        let mut parent_child_pairs = vec![];
        for topic in model.topics.values() {
            if let Some(category_name) = &topic.category {
                debug_assert!(!category_name.eq("Terms"), "Topic is {}", topic.name);
                let category_topic_key = TopicKey::new(&model.main_namespace, category_name);
                parent_child_pairs.push((category_topic_key, topic.get_key()));
            }
        }
        let mut tree = util::tree::Tree::create(parent_child_pairs, true);
        Topic::sort_topic_tree(&mut tree);
        // Have each category topic point to its node in the category tree.
        for topic in model.topics.values_mut() {
            topic.category_tree_node = tree.get_node(&topic.get_key());
        }
        tree
    }
}