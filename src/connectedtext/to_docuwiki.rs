use crate::dokuwiki::gen_page::*;
use crate::dokuwiki::model::*;
use crate::connectedtext::{get_topic_text, parse_line_as_category, parse_line_as_attribute};
use std::collections::BTreeMap;

use crate::*;

const NAMESPACE_TOOLS: &str = "tools";

const ATTR_NAME_CATEGORY: &str = "Category";

const TOPIC_LIMIT: Option<usize> = None;
// const TOPIC_LIMIT: Option<usize> = Some(100);

pub fn main() {
    CtGenProcess::new(NAMESPACE_TOOLS).gen();
}

struct CtGenProcess {
    namespace: String,
    attributes: BTreeMap<String, Attribute>,
    source_topics: BTreeMap<String, Vec<String>>,
    errors: BTreeMap<String, Vec<String>>,
}

struct Attribute {
    name: String,
    values: BTreeMap<String, usize>,
}

impl CtGenProcess {
    pub fn new(namespace: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            attributes: Default::default(),
            source_topics: Default::default(),
            errors: Default::default(),
        }
    }

    pub fn gen(&mut self) {
        self.source_topics = get_topic_text(TOPIC_LIMIT);
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
            for (topic_name, messages) in self.errors.iter() {
                println!("\t{}", topic_name);
                messages.iter().for_each(|msg| { println!("\t\t{}", msg); });
            }
        }
    }

    fn fill_attributes(&mut self) {
        self.attributes.clear();
        let mut source_topics = std::mem::take(&mut self.source_topics);
        for (topic_name, lines) in source_topics.iter() {
            for line in lines.iter() {
                if let Some(category_name) = parse_line_as_category(line) {
                    self.record_attribute(ATTR_NAME_CATEGORY, &category_name);
                } else {
                    if let Some((name, values)) = self.eval_result_opt(topic_name,parse_line_as_attribute(line)) {
                        for value in values.iter() {
                            self.record_attribute(&name, value);
                        }
                    }
                }
            }
        }
        self.source_topics = std::mem::take(&mut source_topics);
        self.report_attributes(false);
    }

    fn eval_result_opt<T>(&mut self, topic_name: &str, result: Result<Option<T>, String>) -> Option<T> {
        match result {
            Ok(t_opt) => t_opt,
            Err(msg) => {
                self.record_error(topic_name, &msg);
                None
            },
        }
    }

    fn record_attribute(&mut self, name: &str, value: &str) {
        let attribute = self.attributes.entry(name.to_string()).or_insert(Attribute::new(name));
        *attribute.values.entry(value.to_string()).or_insert(0) += 1;
    }

    fn record_error(&mut self, topic_name: &str, msg: &str) {
        let entry = self.errors.entry(topic_name.to_string()).or_insert(vec![]);
        entry.push(msg.to_string());
    }

    fn report_attributes(&self, show_detail: bool) {
        println!("\nAttributes: count = {}; value count = {}; appearance count = {}",
            fc(self.attributes.len()),
            fc(self.attributes.values().map(|attr| attr.values.len()).sum::<usize>()),
            fc(self.attributes.values().map(|attr| attr.values.values().sum::<usize>()).sum::<usize>()));
        if show_detail {
            for attribute in self.attributes.values() {
                println!("\t{}: value count = {}, appearance count = {}; {}",
                         attribute.name,
                         fc(attribute.values.len()),
                         fc(attribute.values.values().sum::<usize>()),
                         attribute.values.keys().take(10).join(", "));
            }
        }
    }

    fn gen_sidebar_page(&self) {
        let mut page = WikiGenPage::new(NAMESPACE_NONE, PAGE_NAME_SIDEBAR);
        self.add_main_page_links(&mut page, false, true);
        page.write();
    }

    fn gen_start_page(&self) {
        let mut page = WikiGenPage::new(&self.namespace, PAGE_NAME_START);
        self.add_main_page_links(&mut page, false, true);
        page.write();
    }

    fn gen_imported_pages(&mut self) {
        for (topic_name, _lines) in self.source_topics.iter() {
            let page = WikiGenPage::new(&self.namespace, topic_name);

            page.write();
        }
    }

    fn add_main_page_links(&self, page: &mut WikiGenPage, use_list: bool, include_start_page: bool) {
        let mut links = vec![];
        if include_start_page {
            links.push(page_link(&self.namespace, PAGE_NAME_START, None));
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
            values: Default::default(),
        }
    }
}