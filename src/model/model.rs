use super::*;
use manage_projects::model::Model as ProjectModel;
use std::collections::BTreeMap;
use crate::model::date::{update_date_attributes_from_file_monitor, remove_edited_same_as_added};
use crate::model::glossary::Glossary;
// use crate::connectedtext::NAMESPACE_TOOLS;

const CHECK_SECTION_LINKS: bool = false;

pub(crate) type TopicRefs = BTreeMap<String, TopicKey>;
pub(crate) type NameTopicMap = BTreeMap<String, TopicKey>;
pub(crate) type GlossaryMap = BTreeMap<String, Glossary>;

pub(crate) struct Model {
    _name: String,
    main_namespace: String,
    is_filtered: bool,
    filter_is_public: bool,
    filter_root_topic_ref: Option<String>,
    namespaces: BTreeMap<String, String>,
    topics: BTreeMap<TopicKey, Topic>,
    topic_refs: TopicRefs,
    categories: Vec<String>,
    category_tree: Option<TopicTree>,
    subtopic_tree: Option<TopicTree>,
    attribute_list: AttributeList,
    domain_list: DomainList,
    projects: Option<ProjectModel>,
    projects_name_map: Option<NameTopicMap>,
    file_monitor_project: Option<file_monitor::model::Project>,
    glossaries: GlossaryMap,
    redacted_phrases: Vec<String>,
    warnings: Vec<String>,
}

impl Model {
    pub(crate) fn new(name: &str, main_namespace: &str, filter_is_public: bool, filter_root_topic_ref: Option<String>) -> Self {
        TopicKey::assert_legal_namespace(main_namespace);
        assert!(!filter_is_public || filter_root_topic_ref.is_none());
        let mut wiki = Self {
            _name: name.to_string(),
            main_namespace: main_namespace.to_string(),
            is_filtered: filter_is_public || filter_root_topic_ref.is_some(),
            filter_is_public,
            filter_root_topic_ref,
            namespaces: Default::default(),
            topics: Default::default(),
            topic_refs: Default::default(),
            categories: Default::default(),
            category_tree: None,
            subtopic_tree: None,
            attribute_list: AttributeList::new(),
            domain_list: DomainList::new(),
            projects: None,
            projects_name_map: None,
            file_monitor_project: None,
            glossaries: Default::default(),
            redacted_phrases: vec![],
            warnings: vec![],
        };
        wiki.add_namespace(main_namespace);
        wiki
    }

    pub(crate) fn get_main_namespace(&self) -> &str {
        &self.main_namespace
    }

    #[allow(dead_code)]
    pub(crate) fn get_namespace_count(&self) -> usize {
        self.namespaces.len()
    }

    pub(crate) fn add_namespace(&mut self, name: &str) {
        assert!(!self.namespaces.contains_key(name));
        self.namespaces.insert(name.to_string(), name.to_string());
    }

    #[inline]
    pub(crate) fn qualify_namespace(&self, name: &str) -> String {
        if name.starts_with(":") {
            //bg!(&name, format!("{}{}", &self.main_namespace, name.to_lowercase()));
            format!("{}{}", &self.main_namespace, name.to_lowercase())
        } else {
            name.to_lowercase()
        }
    }

    /*
    pub(crate) fn namespace_attribute(&self) -> String {
        self.qualify_namespace(NAMESPACE_ATTRIBUTE)
    }
    */

    pub(crate) fn namespace_book(&self) -> String {
        self.qualify_namespace(NAMESPACE_BOOK)
    }

    pub(crate) fn namespace_navigation(&self) -> String {
        self.qualify_namespace(NAMESPACE_NAVIGATION)
    }

    pub(crate) fn get_topics(&self) -> &BTreeMap<TopicKey, Topic> {
        &self.topics
    }

    pub(crate) fn get_topics_mut(&mut self) -> &mut BTreeMap<TopicKey, Topic> {
        &mut self.topics
    }

    pub(crate) fn find_topic_mut_opt(&mut self, topic_key: &TopicKey) -> Option<&mut Topic> {
        if self.topics.contains_key(topic_key) {
            Some(self.topics.get_mut(topic_key).unwrap())
        } else {
            let topic_ref = make_topic_ref(topic_key.get_namespace(), topic_key.get_topic_name());
            if let Some(ref_topic_key) = self.topic_refs.get(&topic_ref) {
                Some(self.topics.get_mut(ref_topic_key).unwrap())
            } else {
                None
            }
        }
    }

    pub(crate) fn find_topic_mut(&mut self, topic_key: &TopicKey, context: &str) -> &mut Topic {
        self.find_topic_mut_opt(topic_key).expect(&format!("{}: Not able to find topic with topic_key = {},", context, topic_key))
    }

    /*
    pub(crate) fn get_topic_mut(&mut self, topic_key: &TopicKey) -> &Option<&mut Topic> {
        self.topics.get_mut(topic_key)
    }
    */

    pub(crate) fn add_topic(&mut self, topic: Topic) {
        assert!(self.namespaces.contains_key(topic.get_namespace()));
        let topic_key = topic.get_topic_key();

        let topic_ref = make_topic_ref(topic_key.get_namespace(), topic_key.get_topic_name());
        if self.topic_refs.contains_key(&topic_ref) {
            panic!("We already have this topic ref: \"{}\".", topic_ref)
        }
        self.topic_refs.insert(topic_ref, topic_key.clone());

        if self.topics.contains_key(&topic_key) {
            panic!("We already have this topic key: {:?}.", topic_key)
        }
        self.topics.insert(topic_key, topic);
    }

    /*
    pub(crate) fn remove_topic(&mut self, topic_key: &TopicKey) -> Option<Topic> {
        self.topics.remove(topic_key)
    }
    */

    pub(crate) fn get_topic_name(&self, topic_key: &TopicKey) -> &str {
        assert!(self.topics.contains_key(topic_key), "Topic key {} not found.", topic_key);
        let topic = self.topics.get(topic_key).unwrap();
        topic.get_name()
    }

    pub(crate) fn get_topic_refs(&self) -> &TopicRefs {
        &self.topic_refs
    }

    pub(crate) fn get_corrected_topic_key(topic_refs: &TopicRefs, namespace: &str, topic_name: &str) -> Result<TopicKey, String> {
        // The topic name will be in either the title form like "Functional Programming" or the
        // canonical file name form found in links. Either way we want to end up with a topic key
        // that uses the title form.
        // if topic_name.eq("//www.alteryx.com/") { panic!(); }
        let topic_ref = make_topic_ref(namespace, topic_name);
        //et debug = topic_ref.eq("tools:open_data_protocol_odata");
        //f debug { //bg!(&topic_ref, topic_refs.get(&topic_ref), topic_refs.keys()); panic!() }
        match topic_refs.get(&topic_ref) {
            Some(topic_key) => Ok(topic_key.clone()),
            None => {
                //f debug { //bg!(topic_refs); panic!(); }
                Err(format!("Model::get_corrected_topic_key(): Corrected topic key not found for namespace = \"{}\", topic_name = \"{}\", topic_ref = \"{}\".", namespace, topic_name, topic_ref))
            },
        }
    }

    pub(crate) fn catalog_links(&mut self) {
        for topic in self.topics.values_mut() {
            topic.clear_inbound_topic_keys();
        }
        let mut map = BTreeMap::new();
        for topic in self.topics.values() {
            let dependencies_are_generated = topic.get_category().map_or(false, |cat| cat.eq(CATEGORY_RUST_PROJECTS) || cat.eq(CATEGORY_RUST_CRATES));
            for dest_topic_key in topic.get_links(false, dependencies_are_generated).iter()
                .filter_map(|link_rc| b!(link_rc).get_topic_key()) {
                let entry = map.entry(dest_topic_key).or_insert(vec![]);
                entry.push(topic.get_topic_key());
            }
        }
        for glossary in self.glossaries.values() {
            if let Some(glossary_topic_key) = &glossary.topic_key {
                for dest_topic_key in glossary.get_links().iter()
                    .filter_map(|link_rc| b!(link_rc).get_topic_key()) {
                    let entry = map.entry(dest_topic_key).or_insert(vec![]);
                    entry.push(glossary_topic_key.clone());
                }
            }
        }
        for (topic_key, inbound_topic_keys) in map.drain_filter(|_, _| true) {
            let topic = self.topics.get_mut(&topic_key).unwrap();
            topic.set_inbound_topic_keys(inbound_topic_keys);
            topic.finalize_inbound_topic_keys();
        }
    }

    /*
    fn get_links(&self) -> Vec<LinkRc> {
        let mut links = vec![];
        for topic in self.topics.values() {
            links.append(&mut topic.get_links());
        }
        links
    }
    */

    /*
    pub(crate) fn get_inbound_link_topic_keys(&self, topic_key: &TopicKey) -> Vec<TopicKey> {
        let mut topic_keys = self.get_links().iter()
            .filter_map(|link_rc| b!(link_rc).get_topic_key())
            .filter(|link_topic_key| link_topic_key.eq(topic_key))
            .collect::<Vec<_>>();
        TopicKey::sort_topic_keys_by_name(&mut topic_keys);
        topic_keys.dedup();
        topic_keys
    }
    */
    pub(crate) fn is_attribute_indexed(&self, name: &str) -> bool {
        self.attribute_list.is_attribute_indexed(name)
    }

    pub(crate) fn get_attribute_types(&self) -> &BTreeMap<String, AttributeType> {
        self.attribute_list.get_attribute_types()
    }

    /*
    pub(crate) fn get_attribute_types_mut(&mut self) -> &mut BTreeMap<String, AttributeType> {
        self.attribute_list.get_attribute_types_mut()
    }
    */

    // In the values map, each entry is a list of pairs of topic keys and attribute type names.
    // Sort each of these lists by topic name first, then attribute type name.
    pub(crate) fn sort_attribute_topic_lists(&mut self) {
        self.attribute_list.sort_attribute_topic_lists();
    }

    pub(crate) fn get_attribute_type(&self, name: &str) -> Option<&AttributeType> {
        self.attribute_list.get_attribute_type(name)
    }

    pub(crate) fn clear_attribute_orders(&mut self) {
        self.attribute_list.clear_attribute_orders();
    }

    pub(crate) fn add_attribute_order(&mut self, type_name: String, sequence: usize) {
        self.attribute_list.add_attribute_order(type_name, sequence);
    }

    pub(crate) fn get_attribute_orders(&self) -> &BTreeMap<String, usize> {
        self.attribute_list.get_attribute_orders()
    }

    /*
    pub(crate) fn add_attribute_value(&mut self, value: String, topic_key: TopicKey, value_type_name: String) {
        self.attribute_list.add_attribute_value(value, topic_key, value_type_name);
    }
    */

    pub(crate) fn has_attribute_links(&self, value: &str) -> bool {
        self.attribute_list.has_attribute_links(value)
    }

    pub(crate) fn get_attribute_list(&self) -> &AttributeList {
        &self.attribute_list
    }

    pub(crate) fn get_topics_with_attribute_value(&self, value: &str) -> Vec<(TopicKey, String)> {
        self.attribute_list.get_topics_with_attribute_value(value)
    }

    pub(crate) fn get_domain(&self, name: &str) -> Option<&Domain> {
        self.domain_list.get_domain(name)
    }

    /*
    pub(crate) fn catalog_links(&mut self) {
        for topic in self.topics.values_mut() {
            topic.catalog_outbound_links();
        }
        self.catalog_inbound_links();
    }

    fn catalog_inbound_links(&mut self) {
        let include_generated = false;
        let mut map = BTreeMap::new();
        for topic in self.topics.values() {
            let topic_key = topic.get_topic_key();
            for outbound_topic_key in topic.get_topic_links_as_topic_keys(include_generated).drain(..) {
                let entry = map.entry(outbound_topic_key.clone()).or_insert(vec![]);
                if !entry.contains(&topic_key) {
                    entry.push(topic_key.clone());
                }
            }
        }
        for (topic_key, mut inbound_topic_keys) in map.drain_filter(|_k, _v| true) {
            TopicKey::sort_topic_keys_by_name(&mut inbound_topic_keys);
            if let Some(topic) = self.get_topics_mut().get_mut(&topic_key) {
                topic.set_inbound_topic_keys(inbound_topic_keys);
            }
        }
    }
    */

    pub(crate) fn check_links(&self) -> TopicErrorList {
        let mut errors = TopicErrorList::new();
        for topic in self.topics.values() {
            let this_topic_key = topic.get_topic_key();
            for link_rc in topic.get_links(true, false).iter() {
                let link = b!(link_rc);
                match &link.get_type() {
                    LinkType::Topic { topic_key } => {
                        self.check_topic_link(&mut errors, "links", &this_topic_key, topic_key);
                    },
                    LinkType::Section { section_key } => {
                        if CHECK_SECTION_LINKS && !self.has_section(section_key) {
                            self.print_sections(section_key.get_topic_key());
                            errors.add(&topic.get_topic_key(), &format!("Model::check_links(): Section link {} not found.", section_key));
                        }
                    },
                    _ => {},
                }
            }
        }
        errors
    }

    pub(crate) fn check_topic_link(&self, errors: &mut TopicErrorList, list_name: &str, this_topic_key: &TopicKey, ref_topic_key: &TopicKey) {
        if !self.has_topic(ref_topic_key) {
            errors.add(this_topic_key, &format!("wiki::check_topic_link(): Topic link {} from {} list not found.", ref_topic_key, list_name));
        }
    }

    /*
    pub(crate) fn update_internal_links(&mut self, keys: &Vec<(TopicKey, TopicKey)>) {
        for topic in self.topics.values_mut() {
            topic.update_internal_links(keys);
        }
    }
    */

    /*
    pub(crate) fn check_subtopic_relationships(&self) -> TopicErrorList {
        Topic::check_subtopic_relationships(self)
    }
    */

    //#[allow(dead_code)]
    // pub(crate) fn catalog_possible_list_types(&self) -> util::group::Grouper<String> {
        //ListType::catalog_possible_list_types(self)
    // }

    pub(crate) fn catalog_attributes(&mut self) -> TopicErrorList {
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
        self.attribute_list.set_types_and_values(attribute_types, attribute_values);
        // In the values map, each entry is a list of pairs of topic keys and attribute type names.
        // Sort each of these lists by topic name first, then attribute type name.
        self.sort_attribute_topic_lists();
        // Self::list_attribute_types(model);
        errors
    }

    pub(crate) fn update_attributes(&mut self) -> TopicErrorList {
        let mut errors = TopicErrorList::new();
        AttributeType::fill_attribute_orders(self);
        let attribute_orders = self.get_attribute_orders().clone();
        let (mut attribute_types, mut attribute_values) = self.attribute_list.take_types_and_values();
        for topic in self.get_topics_mut().values_mut() {
            topic.catalog_attributes(&mut errors, &mut attribute_types, &mut attribute_values, &attribute_orders);
        }
        self.attribute_list.set_types_and_values(attribute_types, attribute_values);
        // In the values map, each entry is a list of pairs of topic keys and attribute type names.
        // Sort each of these lists by topic name first, then attribute type name.
        self.sort_attribute_topic_lists();
        // Self::list_attribute_types(model);
        errors
    }

    pub(crate) fn catalog_domains(&mut self) -> TopicErrorList {
        DomainList::catalog_domains(self)
    }

    /*
    pub(crate) fn interpolate_added_date(&mut self) {
        super::date::interpolate_added_date(self);
    }
    */

    pub(crate) fn update_attributes_from_file_monitor(&mut self) {
        // We're still working with the temp attributes.
        AttributeType::fill_attribute_orders(self);
        update_date_attributes_from_file_monitor(self);
    }

    #[allow(dead_code)]
    pub(crate) fn remove_edited_same_as_added(&mut self) {
        // One-time cleanup. Remove Edited attributes that have the same date as Added.
        remove_edited_same_as_added(self);
    }

    pub(crate) fn add_visibility_attributes(&mut self) {
        for topic in self.get_topics_mut().values_mut() {
            // topic.add_temp_attribute_values(ATTRIBUTE_NAME_VISIBILITY.to_string(), vec![ATTRIBUTE_VALUE_PUBLIC.to_string()]);
            if !topic.has_temp_attribute(ATTRIBUTE_NAME_VISIBILITY) {
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_VISIBILITY.to_string(), vec![ATTRIBUTE_VALUE_UNKNOWN.to_string()]);
            }
        }
    }

    pub(crate) fn has_topic(&self, topic_key: &TopicKey) -> bool {
        self.topics.contains_key(topic_key)
    }

    pub(crate) fn topic_keys_alphabetical_by_topic_name(&self) -> Vec<TopicKey> {
        self.topics.keys().sorted_by_key(|topic_key| topic_key.get_topic_name().to_lowercase()).map(|x| x.clone()).collect()
    }

    #[allow(dead_code)]
    pub(crate) fn get_topic_names(&self) -> Vec<String> {
        self.topic_keys_alphabetical_by_topic_name().iter().map(|topic_key| topic_key.get_topic_name().to_string()).collect()
    }

    pub(crate) fn add_category_optional(&mut self, name: String) {
        if !self.categories.contains(&name) {
            self.categories.push(name);
        }
    }

    pub(crate) fn get_categories(&self) -> &Vec<String> {
        &self.categories
    }

    /*
    pub(crate) fn get_attribute_order(&self, attr_type_name: &str) -> Result<usize, String> {
        match self.attribute_list.get_attribute_orders().get(attr_type_name) {
            Some(sequence) => Ok(*sequence),
            None => Err(format!("No sequence found for attribute type \"{}\".", attr_type_name)),
        }
    }
    */

    pub(crate) fn set_domain_list(&mut self, domain_list: DomainList) {
        self.domain_list = domain_list;
    }

    /*
    pub(crate) fn topic_keys_alphabetical_by_topic_name(&self) -> Vec<TopicKey> {
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

    pub(crate) fn has_section(&self, section_key: &SectionKey) -> bool {
        //et debug = section_key.get_section_name().eq("free_account");
        //f debug { dbg!(section_key); }
        if !self.has_topic(&section_key.get_topic_key()) {
            //f debug { println!("Model::has_section(): topic not found."); }
            return false;
        }
        self.topics[&section_key.get_topic_key()].has_section(&section_key.get_section_name())
    }

    pub(crate) fn print_sections(&self, topic_key: &TopicKey) {
        if !self.has_topic(topic_key) {
            panic!("Can't find topic {} to print its sections.", topic_key);
        }
        self.topics[topic_key].print_sections();
    }

    pub(crate) fn add_missing_category_topics(&mut self) {
        category::add_missing_category_topics(self)
    }

    /*
    pub(crate) fn move_topics_to_namespace_by_category(&mut self, category_name: &str, namespace_name: &str) {
        TopicKey::assert_legal_namespace(namespace_name);
        Category::move_topics_to_namespace_by_category(self, category_name, namespace_name)
    }
    */

    pub(crate) fn make_category_tree(&mut self) {
        self.category_tree = Some(make_category_tree(self));
    }

    pub(crate) fn make_subtopic_tree(&mut self) {
        self.subtopic_tree = Some(Topic::make_subtopic_tree(self));
    }

    pub(crate) fn get_category_tree(&self) -> &TopicTree {
        match &self.category_tree {
            Some(tree) => tree,
            None => panic!("The wiki model has no category tree. Call make_category_tree() after loading all of the topics."),
        }
    }

    pub(crate) fn subtopic_tree(&self) -> &TopicTree {
        match &self.subtopic_tree {
            Some(tree) => tree,
            None => panic!("The wiki model has no subtopic tree. Call make_subtopic_tree() after loading all of the topics."),
        }
    }

    pub(crate) fn get_distinct_attr_values(&self, value_type: &AttributeValueType) -> Vec<String> {
        AttributeType::get_distinct_attr_values(self, value_type)
    }

    pub(crate) fn get_topics_for_attr_value(&self, value_type: &AttributeValueType, match_value: &str, included_attr_names: Option<Vec<&str>>) -> Vec<TopicKey> {
        AttributeType::get_topics_for_attr_value(self, value_type, match_value, included_attr_names)
    }

    // Create a list of pairs of the attribute type name and the topic key.
    pub(crate) fn get_typed_topics_for_attr_value(&self, value_type: &AttributeValueType, match_value: &str, included_attr_names: Option<Vec<&str>>) -> Vec<(String, TopicKey)> {
        AttributeType::get_typed_topics_for_attr_value(self, value_type, match_value, included_attr_names)
    }

    pub(crate) fn get_topics_first_letter_map(&self) -> BTreeMap<String, Vec<TopicKey>> {
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

    pub(crate) fn get_projects(&self) -> &Option<ProjectModel> {
        &self.projects
    }

    pub(crate) fn get_projects_name_map(&self) -> &Option<NameTopicMap> {
        &self.projects_name_map
    }

    pub(crate) fn set_projects(&mut self, projects: ProjectModel) {
        self.projects = Some(projects);
   }

    pub(crate) fn set_projects_name_map(&mut self, projects_name_map: NameTopicMap) {
        self.projects_name_map = Some(projects_name_map);
    }

    pub(crate) fn set_file_monitor_project(&mut self, file_monitor_project: file_monitor::model::Project) {
        self.file_monitor_project = Some(file_monitor_project);
    }

    pub(crate) fn get_file_monitor_project(&self) -> &Option<file_monitor::model::Project> {
        &self.file_monitor_project
    }

    /*
    #[allow(dead_code)]
    pub(crate) fn add_used_by(&mut self, dependency: TopicKey, user: TopicKey) {
        let entry = self.used_by_map.entry(dependency).or_insert(vec![]);
        if !entry.contains(&user) {
            entry.push(user);
        }
    }
     */

    pub(crate) fn is_filtered(&self) -> bool {
        self.is_filtered
    }

    pub(crate) fn filter_is_public(&self) -> bool {
        self.filter_is_public
    }

    pub fn filter_root_topic_ref(&self) -> &Option<String> {
        &self.filter_root_topic_ref
    }

    #[allow(dead_code)]
    pub(crate) fn topic_is_public(&self, topic_key: &TopicKey) -> bool {
        self.topics.get(topic_key).unwrap().is_public()
    }

    pub fn add_redacted_phrase(&mut self, phrase: String) {
        self.redacted_phrases.push(phrase);
    }

    /*
    pub(crate) fn add_redacted_phrases(&mut self, phrases: &mut Vec<String>) {
        self.redacted_phrases.append(phrases);
    }
    */

    pub(crate) fn get_redacted_phrases(&self) -> &Vec<String> {
        &self.redacted_phrases
    }

    #[allow(dead_code)]
    pub(crate) fn print_redacted_phrases(&self) {
        println!("\nModel::print_redacted_phrases():");
        for phrase in self.redacted_phrases.iter() {
            println!("\t\"{}\"", phrase);
        }
        println!();
    }

    pub(crate) fn filter_set_topics_include_and_redacted(&mut self) {
        // FILTER UNCOMMENT
        /*
        let mut topic_is_included = true;
        let mut topic_is_redacted = false;
        for topic in self.topics.values_mut() {
            let topic_is_public = topic.is_public();
            if let Some(filter_root_topic_ref) = &self.filter_root_topic_ref {
                let root_topic_ref = topic.get_root_topic_ref();
                topic_is_included = root_topic_ref == filter_root_topic_ref;
                topic_is_redacted = !topic_is_included && !topic_is_public;
            } else {
                assert!(self.filter_is_public);
                topic_is_included = topic_is_public;
                topic_is_redacted = !topic_is_included;
            }
            topic.is_included = topic_is_included;
            topic.is_redacted = topic_is_redacted;
            if !topic_is_included {
                let topic_key = topic.get_topic_key();
                assert!(!self.topic_files_to_delete.contains(&topic_key));
                self.topic_files_to_delete.push(topic_key);
            }
            if topic_is_redacted {
                self.add_redacted_phrase(topic.get_name().to_string());
                self.add_redacted_phrase(topic.get_topic_ref());
            }
        }
         */
    }

    pub(crate) fn finalize_redacted_phrases(&mut self) {
        // Take the value from self.redacted_phrases so that we avoid cloning it.
        let phrases = std::mem::replace(&mut self.redacted_phrases, vec![]);
        self.redacted_phrases = redaction::finalize_redacted_phrases(phrases);
    }

    pub(crate) fn get_glossaries(&self) -> &GlossaryMap {
        &self.glossaries
    }

    pub(crate) fn take_glossaries(&mut self) -> GlossaryMap {
        std::mem::replace(&mut self.glossaries, BTreeMap::new())
    }

    pub(crate) fn set_glossaries(&mut self, glossaries: GlossaryMap) {
        self.glossaries = glossaries;
    }

    pub(crate) fn build_glossaries(&mut self) {
        let mut glossaries = self.take_glossaries();
        for glossary in glossaries.values_mut() {
            self.warnings.append(&mut glossary.build_from_raw_list(self));
        }
        self.glossaries = glossaries;
    }

    pub(crate) fn print_warnings(&self) {
        if self.warnings.is_empty() {
            println!("\nNo model warnings.\n");
        } else {
            println!("\nModel warnings:");
            for warning in self.warnings.iter() {
                println!("\t{}", warning);
            }
            println!();
        }
    }

    /*
    pub(crate) fn redact(&mut self, compare_only: bool, redaction_preview_only: bool) {
        self.redaction_record = Some(redaction::RedactionRecord::redact(self, compare_only, redaction_preview_only));
    }
    */

    /*
    pub(crate) fn remove_non_public_parent_topic_refs(&mut self) {
        assert!(self.is_public);
        let mut delete_topic_keys = vec![];
        for topic in self.topics.values_mut() {
            let parent_count = topic.get_parent_count();
            match get_parent_count {
                0 => {},
                1 => {
                    if
                },
                2 => {
                    // Combo category.
                    if topic.get_
                },
                _ => {
                    panic!("In {}, unexpected number of parents = {}", topic.get_topic_key(), parent_count)
                }
            }
        }
    }

     */
}
