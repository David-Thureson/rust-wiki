use std::collections::BTreeMap;

use crate::*;
use super::*;
use std::cmp::Ordering;

use crate::dokuwiki::*;
use proc_macro::bridge::client::ProcMacro::Attr;

pub(crate) const NAMESPACE_TOOLS: &str = "tools";
pub(crate) const NAMESPACE_HOME: &str = "home";
// const NAMESPACE_ATTRIBUTES: &str = "attr";

const ATTR_NAME_CATEGORY: &str = "Category";

pub const ATTRIBUTE_VALUE_MISSING: &str = "{missing}";

// const TOPIC_LIMIT_TOOLS: Option<usize> = None;
const TOPIC_LIMIT_TOOLS: Option<usize> = Some(100);
// const TOPIC_LIMIT_HOME: Option<usize> = None;
const TOPIC_LIMIT_HOME: Option<usize> = Some(50);

pub fn main() {
    CtGenProcess::new().gen();
}

struct CtGenProcess {
    attributes: BTreeMap<String, Attribute>,
    source_topics: BTreeMap<TopicReference, Vec<String>>,
    errors: BTreeMap<TopicReference, Vec<String>>,
}

enum AttributeType {
    String,
    Date,
    Boolean,
    Unknown,
}

struct Attribute {
    name: String,
    type_: AttributeType,
    values: BTreeMap<String, AttributeValue>,
}

struct AttributeValue {
    _name: String,
    topics: BTreeMap<TopicReference, ()>,
}

#[derive(Clone, Eq, Ord, PartialEq)]
pub struct TopicReference {
    namespace: String,
    topic_name: String,
}

impl CtGenProcess {
    pub fn new() -> Self {
        Self {
            attributes: Default::default(),
            source_topics: Default::default(),
            errors: Default::default(),
        }
    }

    pub fn gen(&mut self) {
        self.source_topics = get_topic_text_both_namespaces(TOPIC_LIMIT_TOOLS, TOPIC_LIMIT_HOME);
        self.fill_attributes();
        // self.copy_image_files();
        self.gen_sidebar_page();
        self.gen_start_page();
        self.gen_imported_pages();
        // gen_remaining_category_pages();
        self.report_errors();
    }

    fn report_errors(&self) {
        if !self.errors.is_empty() {
            println!("\nErrors:");
            for (topic_reference, messages) in self.errors.iter() {
                println!("\t{}", topic_reference.get_full_name());
                messages.iter().for_each(|msg| { println!("\t\t{}", msg); });
            }
        }
    }

    fn fill_attributes(&mut self) {
        self.attributes.clear();
        let mut source_topics = std::mem::take(&mut self.source_topics);
        for (topic_reference, lines) in source_topics.iter() {
            for line in lines.iter() {
                if let Some(category_name) = parse_line_as_category(line) {
                    self.record_attribute(&topic_reference, ATTR_NAME_CATEGORY, &category_name);
                } else {
                    if let Some((name, values)) = self.eval_result_opt(&topic_reference,parse_line_as_attribute(line)) {
                        for value in values.iter() {
                            self.record_attribute(&topic_reference, &name, value);
                        }
                    }
                }
            }
        }
        self.source_topics = std::mem::take(&mut source_topics);
        // Figure out the attribute types.
        for attribute in self.attributes.values_mut() {

        }
        self.report_attributes(0);
    }

    fn eval_result_opt<T>(&mut self, topic_reference: &TopicReference, result: Result<Option<T>, String>) -> Option<T> {
        match result {
            Ok(t_opt) => t_opt,
            Err(msg) => {
                self.record_error(topic_reference, &msg);
                None
            },
        }
    }

    fn record_attribute(&mut self, topic_reference: &TopicReference, name: &str, value: &str) {
        let attribute = self.attributes.entry(name.to_string()).or_insert(Attribute::new(name));
        let value = attribute.values.entry(value.to_string()).or_insert(AttributeValue::new(value));
        value.topics.insert(topic_reference.clone(), ());
    }

    fn record_error(&mut self, topic_reference: &TopicReference, msg: &str) {
        let entry = self.errors.entry(topic_reference.clone()).or_insert(vec![]);
        entry.push(msg.to_string());
    }

    fn report_attributes(&self, detail_level: usize) {
        println!("\nAttributes: count = {}; value count = {}; appearance count = {}",
            fc(self.attributes.len()),
            fc(self.attributes.values().map(|attr| attr.value_count()).sum::<usize>()),
            fc(self.attributes.values().map(|attr| attr.topic_count()).sum::<usize>()));
        if detail_level >= 1 {
            for attribute in self.attributes.values() {
                println!("\t{}: value count = {}, topic count = {}; {}",
                         attribute.name,
                         fc(attribute.value_count()),
                         fc(attribute.topic_count()),
                         attribute.values.keys().take(10).join(", "));
                if detail_level >= 2 {
                    println!("\t\t{}", attribute.get_topics().iter().take(5).map(|topic| topic.get_full_name()).join(", "));
                }
            }
        }
    }

    fn gen_sidebar_page(&self) {
        let mut page = WikiGenPage::new(NAMESPACE_NONE, PAGE_NAME_SIDEBAR);
        self.add_main_page_links(&mut page, false, true);
        page.write();
    }

    fn gen_start_page(&self) {
        let mut page = WikiGenPage::new(NAMESPACE_TOOLS, PAGE_NAME_START);
        self.add_main_page_links(&mut page, false, true);
        page.write();
    }

    fn gen_imported_pages(&mut self) {
        for (topic_reference, lines) in self.source_topics.iter() {
            let (namespace, topic_name) = (&topic_reference.namespace, &topic_reference.topic_name);
            let mut page = WikiGenPage::new(namespace, topic_name);
            page.add_headline(topic_name, 0);
            let mut table = None;
            for line in lines.iter() {
                let line = line.trim();
                if line.is_empty() {
                    if let Some(table) = table {
                        page.add_line(table.get_markup());
                    }
                    table = None;
                    page.add_blank_line();
                }
                if line.starts_with("{|") {
                    table = Some(WikiAttributeTable::new());
                    continue;
                }
                if let Some(category_name) = parse_line_as_category(line) {
                    debug_assert_eq!(false, in_table);
                    let link = page_link(NAMESPACE_TOOLS, &category_name, Some(&category_name));
                    page.add_line(&format!("Category: {}", link));
                } else {
                    if let Some((name, values)) = self.eval_result_opt(&topic_reference,parse_line_as_attribute(line)) {
                        match table {
                            Some(table) => {
                                let value_markup = values.iter()
                            }
                        }
                        debug_assert_eq!(true, in_table);

                        for value in values.iter() {
                            self.record_attribute(&topic_reference, &name, value);
                        }
                    }
                }

            }
            page.write();
        }
    }

    fn add_main_page_links(&self, page: &mut WikiGenPage, use_list: bool, include_start_page: bool) {
        let mut links = vec![];
        if include_start_page {
            links.push(page_link(NAMESPACE_TOOLS, PAGE_NAME_START, None));
        };
        /*
        links.append(&mut vec![
            wiki::page_link(NAMESPACE_NAVIGATION, PAGE_NAME_GALLERIES, Some("Galleries")),
            wiki::page_link(NAMESPACE_NAVIGATION, PAGE_NAME_KEYWORDS, Some("Tags and Themes")),
            wiki::page_link(NAMESPACE_NAVIGATION, PAGE_NAME_BRANDS, Some("Brands")),
            //wiki::page_link(NAMESPACE_NAVIGATION, PAGE_NAME_CATEGORIES, Some("Categories")),
            wiki::page_link(NAMESPACE_NAVIGATION, PAGE_NAME_PERFORMANCE, Some("Performance")),
        ]);
        */
        if use_list {
            let mut list = WikiList::new();
            for link in links.iter() {
                list.add_item(link);
            }
            page.add_paragraph(&list.get_markup(None));
        } else {
            for link in links.iter() {
                page.add_paragraph(link);
            }
        }
    }

    /*
    fn source_lines(&self) -> Vec<String> {
        self.source_topics.values().map(|lines| lines.iter()).flatten().map(|x| x.to_string()).collect()
    }
    */
}

impl Attribute {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            type_: AttributeType::Unknown,
            values: Default::default(),
        }
    }

    pub fn value_count(&self) -> usize {
        self.values.len()
    }

    pub fn topic_count(&self) -> usize {
        self.values.values().map(|value| value.topic_count()).sum()
    }

    pub fn get_topics(&self) -> Vec<TopicReference> {
        let mut topics = self.values.values()
            .map(|value| value.topics.keys())
            .flatten()
            .map(|topic| topic.clone())
            .collect::<Vec<_>>();
        topics.sort();
        topics.dedup();
        topics
    }
}

impl AttributeValue {
    pub fn new(name: &str) -> Self {
        Self {
            _name: name.to_string(),
            topics: Default::default()
        }
    }

    pub fn topic_count(&self) -> usize {
        self.topics.len()
    }
}

impl TopicReference {
    pub(crate) fn new(namespace: &str, topic_name: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            topic_name: topic_name.to_string()
        }
    }

    fn get_full_name(&self) -> String {
        format!("{{{}: {}}}", self.namespace, self.topic_name)
    }
}

impl PartialOrd for TopicReference {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (&self.namespace, &self.topic_name).partial_cmp(&(&other.namespace, &other.topic_name))
    }
}

