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
    pub attributes: BTreeMap<String, AttributeType>,
    pub domains: DomainList,
}

impl Wiki {
    pub fn new(name: &str, main_namespace: &str) -> Self {
        let mut wiki = Self {
            name: name.to_string(),
            main_namespace: main_namespace.to_string(),
            namespaces: Default::default(),
            topics: Default::default(),
            categories: Default::default(),
            category_tree: None,
            subtopic_tree: None,
            attributes: Default::default(),
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
        assert!(!self.topics.contains_key(&key));
        self.topics.insert(key, topic);
    }

    pub fn topic_name(&self, topic_key: &TopicKey) -> String {
        assert!(self.topics.contains_key(topic_key), "Topic key {} not found.", topic_key);
        let topic = self.topics.get(topic_key).unwrap();
        topic.name.clone()
    }

    pub fn catalog_links(&mut self) {
        for topic in self.topics.values_mut() {
            topic.outbound_links.clear();
            topic.inbound_topic_keys.clear();
            topic.listed_topics.clear();
            topic.subtopics.clear();
            topic.combo_subtopics.clear();
        }
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

        // Set inbound links.
        let mut map = BTreeMap::new();
        for topic in self.topics.values() {
            let topic_key = topic.get_key();
            for link in topic.outbound_links.iter() {
                let outbound_topic_key = match &link.type_ {
                    LinkType::Topic { topic_key } => Some(topic_key.clone()),
                    LinkType::Section { section_key } => Some(section_key.topic_key.clone()),
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

    pub fn check_links(&self) -> TopicErrorList {
        //bg!(self.topics.keys());
        let mut errors = TopicErrorList::new();
        for topic in self.topics.values() {
            let this_topic_key = topic.get_key();
            for link in topic.outbound_links.iter() {
                match &link.type_ {
                    LinkType::Topic { topic_key } => {
                        self.check_topic_link(&mut errors, "outbound_links", &this_topic_key, topic_key);
                    },
                    LinkType::Section { section_key } => {
                        //bg!(&section_key);
                        if !self.has_section(section_key) {
                            errors.add(&topic.get_key(), &format!("wiki::check_links(): Section link {} not found.", section_key));
                        }
                    },
                    _ => {},
                }
            }
            topic.parents.iter().for_each(|ref_topic_key| { self.check_topic_link(&mut errors, "parents", &this_topic_key, ref_topic_key); } );
            topic.inbound_topic_keys.iter().for_each(|ref_topic_key| { self.check_topic_link(&mut errors, "inbound_topic_keys", &this_topic_key, ref_topic_key); } );
            topic.subtopics.iter().for_each(|ref_topic_key| { self.check_topic_link(&mut errors, "subtopics", &this_topic_key, ref_topic_key); } );
            topic.combo_subtopics.iter().for_each(|ref_topic_key| { self.check_topic_link(&mut errors, "combo_subtopics", &this_topic_key, ref_topic_key); } );
            topic.listed_topics.iter().for_each(|ref_topic_key| { self.check_topic_link(&mut errors, "listed_topics", &this_topic_key, ref_topic_key); } );
        }
        errors
    }

    fn check_topic_link(&self, errors: &mut TopicErrorList, list_name: &str, this_topic_key: &TopicKey, ref_topic_key: &TopicKey) {
        if !self.has_topic(ref_topic_key) {
            errors.add(this_topic_key,&format!("wiki::check_topic_link(): Topic link {} from {} list not found.", ref_topic_key, list_name));
        }
    }

    pub fn update_internal_links(&mut self, keys: &Vec<(TopicKey, TopicKey)>) {
        //bg!(&keys);
        // For each entry in keys, the first TopicKey is the old value and the second is the new
        // value.
        for topic in self.topics.values_mut() {
            for paragraph in topic.paragraphs.iter_mut() {
                match paragraph {
                    Paragraph::List { type_: _, header, items} => {
                        header.update_internal_links(keys);
                        for item in items.iter_mut() {
                            item.block.update_internal_links(keys);
                        }
                    },
                    Paragraph::Table { has_header: _, rows} => {
                        for row in rows.iter_mut() {
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
            if !topic.parents.is_empty() {
                let old_parents = topic.parents.clone();
                topic.parents.clear();
                for parent_topic_key in old_parents.iter() {
                    let mut new_parent_topic_key = parent_topic_key.clone();
                    for (topic_key_old, topic_key_new) in keys.iter() {
                        if parent_topic_key.eq(&topic_key_old) {
                            new_parent_topic_key = topic_key_new.clone();
                            break;
                        }
                    }
                    topic.parents.push(new_parent_topic_key);
                }
            }
        }
    }

    pub fn check_subtopic_relationships(&self) -> TopicErrorList {
        let mut errors = TopicErrorList::new();
        let err_msg_func = |msg: &str| format!("Wiki::check_subtopic_relationships: {}", msg);
        let cat_combo = "Combinations".to_string();
        for topic in self.topics.values() {
            let topic_key = topic.get_key();
            let parent_count = topic.parents.len();
            if topic.category.as_ref().is_none() || topic.category.as_ref().unwrap().to_string() != cat_combo {
                // Not a combination topic.
                if parent_count > 1 {
                    errors.add(&topic_key, &format!("Non-combo category, so expected 0 or 1 parents, found {}.", parent_count));
                } else {
                    for parent_topic_key in topic.parents.iter() {
                        //bg!(&topic.name, parent_topic_key);
                        if !self.topics[parent_topic_key].listed_topics.contains(&topic_key) {
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
                        if !self.topics[parent_topic_key].combo_subtopics.contains(&topic_key) {
                            errors.add(&parent_topic_key, &err_msg_func(&format!("No combination link to child [[{}]].", topic.name)));
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

    pub fn catalog_attributes(&mut self) -> TopicErrorList {
        let mut errors = TopicErrorList::new();
        self.attributes.clear();
        for topic in self.topics.values_mut() {
            topic.attributes.clear();
            for (temp_attr_name, temp_attr_values) in topic.temp_attributes.iter()
                .filter(|(_name, values)| !values.is_empty()) {
                let attribute_type = self.attributes.entry(temp_attr_name.clone())
                    .or_insert({
                        let value_type = AttributeType::value_to_presumed_type(temp_attr_name,&*temp_attr_values[0]);
                        AttributeType::new(temp_attr_name, &value_type)
                    });
                let mut values_for_topic = vec![];
                for temp_value in temp_attr_values.iter() {
                    match attribute_type.add_value(temp_value,&topic.get_key()) {
                        Ok(canonical_value) => { values_for_topic.push(canonical_value) },
                        Err(msg) => { errors.add(&topic.get_key(), &msg)}
                    };
                }
                topic.attributes.insert(temp_attr_name.clone(),AttributeInstance::new(temp_attr_name, values_for_topic));
            }
        }
        errors
    }

    pub fn catalog_domains(&mut self) -> TopicErrorList {
        DomainList::catalog_domains(self)
    }

    pub fn has_topic(&self, topic_key: &TopicKey) -> bool {
        self.topics.contains_key(topic_key)
    }

    pub fn topic_keys_alphabetical_by_topic_name(&self) -> Vec<TopicKey> {
        let mut map = BTreeMap::new();
        for topic_key in self.topics.keys() {
            //bg!(topic_key);
            map.insert(topic_key.topic_name.clone(), topic_key.clone());
        }
        //bg!(&map);
        map.values().map(|topic_key| topic_key.clone()).collect::<Vec<_>>()
    }

    pub fn has_section(&self, section_key: &SectionKey) -> bool {
        if section_key.section_name.to_lowercase().contains("cognitive") {
            //bg!(section_key, self.has_topic(&section_key.topic_key));
        }
        if !self.has_topic(&section_key.topic_key) {
            return false;
        }
        self.topics[&section_key.topic_key].has_section(&section_key.section_name)
    }

    pub fn add_missing_category_topics(&mut self) {
        // First make sure we have a category entry for each category referenced in a topic.
        let mut category_names = self.topics.values()
            .filter_map(|topic| topic.category.as_ref())
            .map(|category_name| category_name.clone())
            .collect::<Vec<_>>();
        category_names.sort();
        category_names.dedup();
        for category_name in category_names.iter() {
            if !self.categories.contains_key(category_name) {
                self.categories.insert(category_name.to_string(), Category::new(None, category_name) );
            }
        }

        // Make sure that there's a topic for each category, and where there's already a topic,
        // change its namespace.
        let category_names = self.categories.values()
            .map(|category| category.name.clone())
            .collect::<Vec<_>>();
        let category_namespace = self.main_namespace.clone();
        let mut topic_keys = vec![];
        for category_name in category_names.iter() {
            let topic_key_old = TopicKey::new(&self.main_namespace, category_name);
            let found = self.topics.contains_key(&topic_key_old);
            if found {
                // Move the topic from the main to the category namespace.
                //rintln!("\t\t\tMoving topic {}", &category_name);
                let mut topic = self.topics.remove(&topic_key_old).unwrap();
                let topic_key_new = TopicKey::new(&category_namespace, &topic.name);
                topic_keys.push((topic_key_old, topic_key_new));
                topic.namespace = category_namespace.to_string();
                self.add_topic(topic);
            } else {
                //rintln!("Creating topic for {:?}.", &topic_key_new);
                self.add_topic(Topic::new(&category_namespace, category_name));
            }
        }
        self.update_internal_links(&topic_keys);
    }

    pub fn move_topics_to_namespace_by_category(&mut self, category_name: &str, namespace_name: &str) {
        let topic_names = self.topics.values()
            .filter(|topic| topic.category.as_ref().map_or(false,|cat| cat.eq_ignore_ascii_case(category_name)))
            .map(|topic| topic.name.clone())
            .collect::<Vec<_>>();
        //bg!(category_name, namespace_name, &topic_names);
        let mut topic_keys = vec![];
        for topic_name in topic_names {
            //rintln!("Moving topic {} to namespace {}.", &topic_name, &new_namespace);
            let topic_key_old = TopicKey::new(&self.main_namespace, &topic_name);
            let topic_key_new = TopicKey::new(namespace_name, &topic_name);
            let mut topic = self.topics.remove(&topic_key_old).unwrap();
            topic.namespace = namespace_name.to_string();
            self.add_topic(topic);
            topic_keys.push((topic_key_old, topic_key_new));
        }
        self.update_internal_links(&topic_keys);
    }

    pub fn make_category_tree(&mut self) {
        let mut parent_child_pairs = vec![];
        for topic in self.topics.values() {
            if let Some(category_name) = &topic.category {
                debug_assert!(!category_name.eq("Terms"), "Topic is {}", topic.name);
                let category_topic_key = TopicKey::new(&self.main_namespace, category_name);
                parent_child_pairs.push((category_topic_key, topic.get_key()));
            }
        }
        let mut tree = util::tree::Tree::create(parent_child_pairs, true);
        sort_topic_tree(&mut tree);
        // Have each category topic point to its node in the category tree.
        for topic in self.topics.values_mut() {
            topic.category_tree_node = tree.get_node(&topic.get_key());
        }
        self.category_tree = Some(tree);
    }

    pub fn make_subtopic_tree(&mut self) {
        for topic in self.topics.values_mut() {
            topic.subtopics.clear();
            topic.combo_subtopics.clear();
        }
        let mut parent_child_pairs = vec![];
        let mut parent_combo_pairs = vec![];
        for topic in self.topics.values() {
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
            self.topics.get_mut(&parent_topic_key).unwrap().subtopics.push(child_topic_key.clone());
        }
        for (parent_topic_key, combo_topic_key) in parent_combo_pairs.iter() {
            self.topics.get_mut(&parent_topic_key).unwrap().combo_subtopics.push(combo_topic_key.clone());
        }
        for topic in self.topics.values_mut() {
            topic.subtopics.sort_by_cached_key(|topic_key| topic_key.topic_name.clone());
            topic.combo_subtopics.sort_by_cached_key(|topic_key| topic_key.topic_name.clone());
        }
        let mut tree = util::tree::Tree::create(parent_child_pairs, true);
        sort_topic_tree(&mut tree);
        // Have each topic with a subtopic point to its node in the subtopic tree.
        for topic in self.topics.values_mut() {
            topic.subtopic_tree_node = tree.get_node(&topic.get_key());
        }
        // tree.print_counts_to_depth();
        // tree.print_with_items(None);
        self.subtopic_tree = Some(tree);
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
        let mut values = vec![];
        for attribute_type in self.attributes.values()
                .filter(|attribute_type| attribute_type.value_type.eq(value_type)) {
            for value in attribute_type.values.keys() {
                values.push(value.clone());
            }
        }
        values.sort();
        values.dedup();
        values
    }

    pub fn get_topics_for_attr_value(&self, value_type: &AttributeValueType, match_value: &str, included_attr_names: Option<Vec<&str>>) -> Vec<TopicKey> {
        let mut topic_keys = vec![];
        for attribute_type in self.attributes.values()
            .filter(|attribute_type| attribute_type.value_type.eq(value_type))
            .filter(|attribute_type| included_attr_names.as_ref().map_or(true, |included| included.contains(&&*attribute_type.name))) {
            for (_found_value, found_topic_keys) in attribute_type.values.iter()
                .filter(|(found_value, _found_topic_keys)| found_value.as_str() == match_value) {
                for found_topic_key in found_topic_keys.iter() {
                    topic_keys.push(found_topic_key.clone());
                }
            }
        }
        topic_keys.sort();
        topic_keys.dedup();
        sort_topic_keys_by_name(&mut topic_keys);
        topic_keys
    }

    // Create a list of pairs of the attribute type name and the topic key.
    pub fn get_typed_topics_for_attr_value(&self, value_type: &AttributeValueType, match_value: &str, included_attr_names: Option<Vec<&str>>) -> Vec<(String, TopicKey)> {
        let mut list = vec![];
        for attribute_type in self.attributes.values()
            .filter(|attribute_type| attribute_type.value_type.eq(value_type))
            .filter(|attribute_type| included_attr_names.as_ref().map_or(true, |included| included.contains(&&*attribute_type.name))) {
            for (_found_value, found_topic_keys) in attribute_type.values.iter()
                .filter(|(found_value, _found_topic_keys)| found_value.as_str() == match_value) {
                for found_topic_key in found_topic_keys.iter() {
                    list.push((attribute_type.name.clone(), found_topic_key.clone()));
                }
            }
        }
        list.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
        list
    }

    /*
    pub fn find_topic_rc(&self, namespace_name: &str, topic_name: &str, context: &str) -> Result<TopicRc, String> {
        let key = TopicKey::new(namespace_name, topic_name);
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