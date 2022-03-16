use crate::*;
use crate::{model, Itertools};
use crate::dokuwiki as wiki;
use std::rc::Rc;
use std::cell::{RefCell, Ref};
use crate::model::{AttributeValueType, TopicKey, Topic, TableCell, LinkRc, links_to_topic_keys, ATTRIBUTE_NAME_EDITED, ATTRIBUTE_NAME_ADDED, TopicTreeNode, ATTRIBUTE_VALUE_UNKNOWN, ATTRIBUTE_NAME_VISIBILITY};
use std::collections::BTreeMap;
use crate::dokuwiki::{PAGE_NAME_ATTR_VALUE, WikiAttributeTable, PAGE_NAME_ATTR, PAGE_NAME_ATTR_DATE, PAGE_NAME_ATTR_YEAR, DELIM_TABLE_CELL_BOLD, DELIM_TABLE_CELL, WikiGenPage, HEADLINE_LINKS, RECENT_TOPICS_THRESHOLD, legal_file_name, image_ref_from_file_name};
use crate::tree::TreeNode;
use crate::dokuwiki::to_model::{make_topic_file_key, TopicFile};

//const SUBCATEGORY_TREE_MAX_SIZE: usize = 30;

pub(crate) struct GenFromModel<'a> {
    model: &'a model::Model,
    path_pages: String,
    current_topic_key: Option<model::TopicKey>,
    errors: model::TopicErrorList,
}

impl <'a> GenFromModel<'a> {

    pub(crate) fn new(model: &'a model::Model, path_pages: &str) -> Self {
        Self {
            model: model,
            path_pages: path_pages.to_string(),
            current_topic_key: None,
            errors: model::TopicErrorList::new(),
        }
    }

    pub(crate) fn get_path_pages(&self) -> &str {
        &self.path_pages
    }

    pub(crate) fn gen_recent_topics_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.namespace_navigation(), wiki::PAGE_NAME_RECENT_TOPICS, None);
        let date_today = util::date_time::naive_date_now();
        let mut date_map = BTreeMap::new();
        for (topic_key, date) in self.model.get_topics().values()
            .filter_map(|topic| {
                topic.get_attribute_date(ATTRIBUTE_NAME_EDITED).or(topic.get_attribute_date(ATTRIBUTE_NAME_ADDED))
                    .map(|date| ((topic.get_topic_key(), date)))
            }) {
            // We want the ordering in the map to put the most recent dates first.
            let key = date_today - date;
            let entry = date_map.entry(key).or_insert((date, vec![]));
            entry.1.push(topic_key);
        }
        let mut topic_count = 0;
        for (_key, (date, mut topic_keys)) in date_map.drain_filter(|_k, _v| true) {
            let date_display_value = model::AttributeType::date_to_display_string(&date);
            page.add_headline(&date_display_value, 1);
            TopicKey::sort_topic_keys_by_name(&mut topic_keys);
            for topic_key in topic_keys.iter() {
                let link = Self::page_link(topic_key);
                page.add_line_with_break(&link);
            }
            page.add_linefeed();
            topic_count += topic_keys.len();
            if topic_count >= RECENT_TOPICS_THRESHOLD {
                break;
            }
        }
        page.write(&self.path_pages);
    }

    pub(crate) fn gen_all_topics_page(&mut self) {
        let namespace = &self.model.qualify_namespace(&self.model.namespace_navigation());
        let mut page = wiki::WikiGenPage::new(namespace, wiki::PAGE_NAME_ALL_TOPICS,None);
        let first_letter_map = self.model.get_topics_first_letter_map();
        for (index, (map_key, topic_keys)) in first_letter_map.iter().enumerate() {
            let section_name = if map_key.eq("#") { "Number" } else { map_key };
            if index > 0 {
                page.add_linefeed();
            }
            page.add_headline(section_name, 1);
            self.gen_topic_first_letter_links(&mut page, 9);
            for topic_key in topic_keys {
                let link = Self::page_link(topic_key);
                page.add_line_with_break(&link);
            }
        }
        page.write(&self.path_pages);
    }

    pub(crate) fn gen_topic_first_letter_links(&mut self, page: &mut WikiGenPage, column_count: usize) {
        let namespace = &self.model.qualify_namespace(&self.model.namespace_navigation());
        let first_letter_map = self.model.get_topics_first_letter_map();

        let mut cells = vec![];
        for map_key in first_letter_map.keys() {
            let section_name = if map_key.eq("#") { "Number" } else { map_key };
            let link = model::Link::new_section(Some(&map_key), namespace, wiki::PAGE_NAME_ALL_TOPICS, section_name);
            let text_items = vec![model::TextItem::new_link(link)];
            let cell = TableCell::new_text_block(model::TextBlock::new_resolved(text_items), false, &model::HorizontalAlignment::Center);
            cells.push(cell);
        }
        let mut table = model::Table::new(false);
        table.add_cells_flow_layout(column_count, cells);
        self.add_table(page, &table);

        /*
        for map_key in first_letter_map.keys() {
            let section_name = if map_key.eq("#") { "Number" } else { map_key };
            page.add_section_link(namespace, wiki::PAGE_NAME_ALL_TOPICS, section_name, None);
        }
         */
    }

    pub(crate) fn gen_categories_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.namespace_navigation(), wiki::PAGE_NAME_CATEGORIES,None);
        let nodes = self.model.get_category_tree().unroll_to_depth(None);

        // Debugging:
        // for node in nodes.iter() {
        //     let topic_key = &b!(node).item;
        //     debug_assert!(self.model.has_topic(topic_key), "Topic key not found: {}", topic_key.to_string());
        // }

        self.gen_partial_topic_tree(&mut page, &nodes, true, None);
        page.write(&self.path_pages);
    }

    pub(crate) fn gen_subtopics_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.namespace_navigation(), wiki::PAGE_NAME_SUBTOPICS,None);
        let nodes = self.model.subtopic_tree().unroll_to_depth(None);
        self.gen_partial_topic_tree(&mut page, &nodes, false, None);
        page.write(&self.path_pages);
    }

    pub(crate) fn gen_attr_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.namespace_navigation(), wiki::PAGE_NAME_ATTR,None);
        for attribute_type in self.model.get_attribute_types().values()
            .filter(|attribute_type| {
                let value_type = attribute_type.get_value_type();
                AttributeValueType::Date.ne(value_type) && AttributeValueType::Year.ne(value_type)
            })
            .filter(|attribute_type| self.model.is_attribute_indexed(attribute_type.get_name())) {
            page.add_headline(attribute_type.get_name(),1);
            for (value, topic_keys) in attribute_type.get_values().iter() {
                page.add_headline(&attribute_type.get_value_display_string(value), 2);
                if let Some(link) = self.page_link_if_exists(value) {
                    page.add_paragraph(&link);
                }
                self.add_related_domains_optional(&mut page,value, false);
                page.add_line("Topics:");
                for topic_key in topic_keys.iter() {
                    let link = self.page_link_simple(&topic_key);
                    page.add_list_item_unordered(1, &link);
                }
                page.add_linefeed();
            }
        }
        page.write(&self.path_pages);
    }

    pub(crate) fn gen_attr_value_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.namespace_navigation(), wiki::PAGE_NAME_ATTR_VALUE,None);
        let mut map = BTreeMap::new();
        for attribute_type in self.model.get_attribute_types().values()
            .filter(|attribute_type| self.model.is_attribute_indexed(attribute_type.get_name())) {
            for (value, topic_keys) in attribute_type.get_values().iter() {
                let entry = map.entry(value).or_insert(vec![]);
                for topic_key in topic_keys.iter() {
                    entry.push((attribute_type.get_name().to_string(), topic_key.clone()));
                }
            }
        }
        for (value, mut list) in map.drain_filter(|_value, _list| true) {
            page.add_headline(value,1);
            if let Some(link) = self.page_link_if_exists(value) {
                page.add_paragraph(&link);
            }
            self.add_related_domains_optional(&mut page,value, true);
            // Sort by topic name, then attribute type name.
            list.sort_by(|a, b| a.1.get_topic_name().to_lowercase().cmp(&b.1.get_topic_name().to_lowercase()).then(a.0.cmp(&b.0)));
            page.add_line("Topics:");
            for (attribute_type_name, topic_key) in list.drain(..) {
                let link = self.page_link_simple(&topic_key);
                let line = format!("({}) {}", attribute_type_name.to_lowercase(), link);
                page.add_list_item_unordered(1, &line);
            }
            page.add_linefeed();
        }
        page.write(&self.path_pages);
    }

    #[allow(dead_code)]
    pub(crate) fn gen_reports_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.namespace_navigation(), wiki::PAGE_NAME_REPORTS,None);
        if !self.model.is_public() {
            // self.gen_reports_page_public_topics_by_category(&mut page);
            // self.gen_reports_page_public_ref_to_private(&mut page);
            // self.gen_reports_page_redactions(&mut page);
            self.gen_reports_page_privacy_unknown(&mut page);
        }
        page.write(&self.path_pages);
    }

    #[allow(dead_code)]
    fn gen_reports_page_public_topics_by_category(&self, page: &mut WikiGenPage) {
        let mut map = BTreeMap::new();
        for topic in self.model.get_topics().values()
                .filter(|topic| topic.is_public() && topic.get_category().is_none()) {
            let entry = map.entry("".to_string()).or_insert(vec![]);
            entry.push(topic.get_topic_key());
        }
        for node_rc in self.model.get_category_tree().top_nodes.iter() {
            self.add_public_topics_by_category(&mut map, node_rc, "".to_string());
        }
        for (category_label, topic_keys) in map.iter() {
            page.add_line(category_label);
            for topic_key in topic_keys.iter() {
                let link = self.page_link_qualified(topic_key);
                page.add_list_item_unordered(1, &link);
            }
        }
    }

    #[allow(dead_code)]
    fn gen_reports_page_public_ref_to_private(&self, page: &mut WikiGenPage) {
        let mut map = BTreeMap::new();
        for topic in self.model.get_topics().values()
            .filter(|topic| topic.is_public()) {
            for ref_topic_key in topic.get_referenced_topic_keys(true, false).drain_filter(|_x| true)
                .filter(|ref_topic_key| !self.model.get_topics().get(ref_topic_key).unwrap().is_public()) {
                let entry = map.entry(topic.get_topic_key()).or_insert(vec![]);
                entry.push(ref_topic_key);
            }
        }
        for (topic_key, ref_topic_keys) in map.iter() {
            let link = self.page_link_qualified(topic_key);
            page.add_line(&link);
            for ref_topic_key in ref_topic_keys.iter() {
                let link = self.page_link_qualified(ref_topic_key);
                page.add_list_item_unordered(1, &link);
            }
        }
    }

    #[allow(dead_code)]
    fn gen_reports_page_redactions(&self, _page: &mut WikiGenPage) {
        // While redactions should be used only for public renders, the report of redactions is
        // private. So in this case we did the redactions just to get the report.
        assert!(!self.model.is_public());
        //let redaction_record = self.model.get_redaction_record().unwrap();
    }

    #[allow(dead_code)]
    fn gen_reports_page_privacy_unknown(&self, page: &mut WikiGenPage) {
        for topic in self.model.get_topics().values()
                .filter(|topic| topic.has_attribute_value_temp_or_permanent(ATTRIBUTE_NAME_VISIBILITY, ATTRIBUTE_VALUE_UNKNOWN)) {
            let link = self.page_link_qualified(&topic.get_topic_key());
            page.add_list_item_unordered(1, &link);
        }
    }

    #[allow(dead_code)]
    fn add_public_topics_by_category(&self, map: &mut BTreeMap<String, Vec<TopicKey>>, node_rc: &Rc<RefCell<TopicTreeNode>>, category_label: String) {
        let node: Ref<TreeNode<TopicKey>> = RefCell::borrow(node_rc);
        let category_name = node.item.get_topic_name();
        let category_label = if category_label.is_empty() { category_name.to_string() } else { format!("{} - {}", category_label, category_name) };
        for topic in self.model.get_topics().values()
                .filter(|topic| topic.is_public() && topic.get_category().map_or(false, |cat| cat.eq(category_name))) {
            let entry = map.entry(category_label.clone()).or_insert(vec![]);
            entry.push(topic.get_topic_key());
        }
        for node_rc in node.child_nodes.iter() {
            self.add_public_topics_by_category(map, node_rc, category_label.clone());
        }
    }

    fn add_related_domains_optional(&self, page: &mut wiki::WikiGenPage, attribute_value_name: &str, on_attribute_value_page: bool) {
        if let Some(domain) = self.model.get_domain(attribute_value_name) {
            let related_by_count = domain.get_related_by_count();
            if !related_by_count.is_empty() {
                let related_link_list = related_by_count.iter()
                    .map(|related_name| self.domain_link(related_name, on_attribute_value_page))
                    .join(", ");
                let line = format!("Related: {}", related_link_list);
                page.add_paragraph(&line);
            }
        }
    }

    pub(crate) fn gen_attr_year_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.namespace_navigation(), wiki::PAGE_NAME_ATTR_YEAR,None);
        let values = self.model.get_distinct_attr_values(&AttributeValueType::Year);
        for value in values.iter() {
            let display_value = model::AttributeType::value_to_display_string(&AttributeValueType::Year, value);
            page.add_headline(&display_value,1);
            for topic_key in self.model.get_topics_for_attr_value(&AttributeValueType::Year, &value, None) {
                let link = self.page_link_simple(&topic_key);
                page.add_list_item_unordered(1, &link);
            }
            page.add_linefeed();
        }
        page.write(&self.path_pages);
    }

    pub(crate) fn gen_attr_date_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.namespace_navigation(), wiki::PAGE_NAME_ATTR_DATE,None);
        let values = self.model.get_distinct_attr_values(&AttributeValueType::Date);
        let dates = values.iter().map(|value| model::AttributeType::value_to_date(value)).collect::<Vec<_>>();
        let year_month_map = util::date_time::year_month_map(dates);
        for (year, month_map) in year_month_map.iter() {
            page.add_headline(&year.to_string(),1);
            for (month, dates) in month_map.iter() {
                page.add_headline(&util::date_time::year_month_to_doc_format(*year, *month), 2);
                for date in dates.iter() {
                    let display_value = model::AttributeType::date_to_display_string(&date);
                    page.add_headline(&display_value, 3);
                    let match_value = model::AttributeType::date_to_canonical_value(date);
                    for (attribute_type_name, topic_key) in self.model.get_typed_topics_for_attr_value(&AttributeValueType::Date, &match_value, None) {
                        let link = self.page_link_simple(&topic_key);
                        page.add_list_item_unordered(1, &format!("({}) {}", attribute_type_name.to_lowercase(), link));
                    }
                    page.add_linefeed();
                }
            }
        }
        page.write(&self.path_pages);
    }

    fn page_link_if_exists(&self, topic_name: &str) -> Option<String> {
        TopicKey::assert_legal_topic_name(topic_name);
        let possible_topic_key = TopicKey::new(&self.model.get_main_namespace(), topic_name);
        if self.model.has_topic(&possible_topic_key) {
            Some(self.page_link_simple(&possible_topic_key))
        } else {
            None
        }
    }

    /*
    pub(crate) fn gen_attr_date_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.qualify_namespace(model::NAMESPACE_NAVIGATION), wiki::PAGE_NAME_ATTR_DATE,None);
        let values = self.model.get_distinct_attr_values(&AttributeValueType::Date);
        let dates = values.iter().map(|value| model::AttributeType::value_to_date(value)).collect::<Vec<_>>();
        let year_map = util::date_time::year_map(dates);
        for (year, values) in year_map.iter() {
            page.add_headline(&year.to_string(), 1);
            for date in values.iter() {
                let display_value = model::AttributeType::date_to_display_string(&date);
                page.add_headline(&display_value, 2);
                let match_value = model::AttributeType::date_to_canonical_value(date);
                for topic_key in self.model.get_topics_for_attr_value(&AttributeValueType::Date, &match_value, None) {
                    let link = wiki::page_link(&self.model.qualify_namespace(topic_key.get_namespace()), topic_key.get_topic_name(), None);
                    page.add_list_item_unordered(1, &link);
                }
            }
        }
        page.write_if_changed();
    }
    */

    pub(crate) fn gen(&mut self) -> BTreeMap<String, TopicFile> {
        let mut map = BTreeMap::new();
        for topic in self.model.get_topics().values() {
            // let debug = topic.get_name().eq("Terms");
            // let debug = false;
            // if debug { dbg!(topic.get_paragraphs().len()); }
            self.current_topic_key = Some(topic.get_topic_key());
            //bg!(&self.current_topic_key);
            let mut page = wiki::WikiGenPage::new(&self.model.qualify_namespace(topic.get_namespace()), topic.get_name(), None);
            self.add_breadcrumbs_optional(&mut page, &topic);
            self.add_category_optional(&mut page, &topic);
            self.add_attributes_optional(&mut page, &topic);
            self.add_paragraphs(&mut page, &topic);
            self.add_inbound_links_section_optional(&mut page,  &topic);
            page.fix_content_before_write();
            let topic_file_name = legal_file_name(topic.get_name());
            let topic_file_key = make_topic_file_key(topic.get_namespace(), &topic_file_name);
            let topic_file = TopicFile::new(topic.get_namespace(), &topic_file_name, topic.get_name(), page.content);
            map.insert(topic_file_key, topic_file);
            // page.write_if_changed(&self.path_pages, self.model.get_original_pages());
            // if debug { dbg!(&page); panic!(); }
        }
        self.errors.print(Some("GenFromModel::gen()"));
        map
    }

    fn add_breadcrumbs_optional(&mut self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        // if topic.get_name().starts_with("Test ") {
        //     /bg!(topic.get_name(), &topic.parents);
        // }
        match topic.get_parent_count() {
            0 => {},
            1 => {
                //bg!(topic.get_name(), &topic.parents);
                let mut topic_keys = vec![];
                let mut parent_topic_key = b!(&topic.get_parent(0)).get_topic_key().unwrap();
                loop {
                    //bg!(&parent_topic_key);
                    topic_keys.push(parent_topic_key.clone());
                    let parent_topic = self.model.get_topics().get(&parent_topic_key).unwrap();
                    //bg!(&parent_topic.get_name(), &parent_topic.parents);
                    match parent_topic.get_parent_count() {
                        0 => {
                            break;
                        },
                        1 => {
                            parent_topic_key = b!(&parent_topic.get_parent(0)).get_topic_key().unwrap();
                        },
                        _ => {
                            panic!("Unexpected number of parent topics for topic \"{}\".", parent_topic.get_name());
                        }
                    }
                }
                //bg!(&topic_keys);
                topic_keys.reverse();
                let breadcrumbs = topic_keys.iter()
                    .map(|topic_key| self.page_link_simple(topic_key))
                    .join(&format!(" {} ", wiki::DELIM_BREADCRUMB_RIGHT));
                let breadcrumbs = format!("{}{} {} {}{}", wiki::DELIM_BOLD, breadcrumbs, wiki::DELIM_BREADCRUMB_RIGHT, topic.get_name(), wiki::DELIM_BOLD);
                page.add_paragraph(&breadcrumbs);
            },
            2 => {
                // Combination topic.
                let link_a = self.page_link_simple_from_link(&topic.get_parent(0));
                let link_b = self.page_link_simple_from_link(&topic.get_parent(1));
                let breadcrumbs = format!("{}{} {} {} {} {}{}", wiki::DELIM_BOLD, link_a, wiki::DELIM_BREADCRUMB_RIGHT, topic.get_name(), wiki::DELIM_BREADCRUMB_LEFT, link_b, wiki::DELIM_BOLD);
                page.add_paragraph(&breadcrumbs);
            },
            _ => {
                panic!("Unexpected number of parent topics for topic \"{}\".", topic.get_name());
            },
        }
    }

    fn add_category_optional(&mut self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        if let Some(category) = topic.get_category() {
            page.add_category(&self.model.get_main_namespace(),&category);
        }
        // if page.topic_name.contains("10,000") { //bg!(&page.content); }
    }

    fn add_attributes_optional(&mut self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        if !topic.get_attribute_count() > 0 {
            let namespace_navigation = &self.model.namespace_navigation();
            let mut table = WikiAttributeTable::new();
            for attr_instance in topic.get_attributes().values()
                    .sorted_by_key(|attr_instance| attr_instance.get_sequence()) {
                let attr_type = self.model.get_attribute_type(attr_instance.get_attribute_type_name()).unwrap();
                let attr_type_name = attr_type.get_name();
                let attr_type_link = match attr_type.get_value_type() {
                    AttributeValueType::Date => wiki::page_link(&namespace_navigation, PAGE_NAME_ATTR_DATE,Some(attr_type_name)),
                    AttributeValueType::Year => wiki::page_link(&namespace_navigation, PAGE_NAME_ATTR_YEAR,Some(attr_type_name)),
                    _ => if self.model.is_attribute_indexed(attr_type_name) {
                        wiki::section_link(&namespace_navigation, PAGE_NAME_ATTR, attr_type_name, Some(attr_type_name))
                    } else {
                        attr_type_name.to_string()
                    },
                };
                let value_list = attr_instance.get_values().iter()
                    .map(|value| {
                        let label = attr_type.get_value_display_string(value);
                        match attr_type.get_value_type() {
                            AttributeValueType::Date => wiki::section_link(&namespace_navigation, PAGE_NAME_ATTR_DATE,&label,Some(&label)),
                            AttributeValueType::Year => wiki::section_link(&namespace_navigation, PAGE_NAME_ATTR_YEAR,&label,Some(&label)),
                            _ => if self.model.is_attribute_indexed(attr_type_name) {
                                wiki::section_link(&namespace_navigation, PAGE_NAME_ATTR_VALUE, &label, Some(&label))
                            } else {
                                // A raw, unindexed attribute value such as a book title may
                                // contain commas, which will complicate the round trip since it's
                                // unclear whether the string represents multiple values or a
                                // single value with commas. So put quotes around values that have
                                // commas.
                                if label.contains(",") {
                                    format!("\"{}\"", label)
                                } else {
                                    label
                                }
                            },
                        }})
                    .join(", ");
                table.add_row(&attr_type_link, &value_list);
            }
            page.add_text(&table.get_markup());
            page.add_linefeed();
        }
    }

    fn add_paragraphs(&mut self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        //let debug = topic.get_name().eq("Terms");
        let debug = false;
        let msg_func_unexpected = |variant_name: &str| format!("In dokuwiki::gen_from_model::add_paragraphs(), unexpected Paragraph variant = \"{}\"", variant_name);
        // let add_error_unexpected = |paragraph_variant: &str| self.add_error(&msg_func_unexpected(paragraph_variant));
        let mut generated_navigation_paragraphs_added = false;
        for paragraph in topic.get_paragraphs().iter() {
            // First see if it's necessary to add generated navigation paragraphs like subtopics
            // and subcategories.
            if debug { dbg!(paragraph.get_variant_name()); }
            match paragraph {
                // model::Paragraph::List { .. } | model::Paragraph::SectionHeader { .. } => {
                model::Paragraph::SectionHeader { .. } => {
                    // We've gotten past the initial text, if any, so it's a good place to add
                    // the navigation paragraphs before getting into more detail.
                    if !generated_navigation_paragraphs_added {
                        self.add_generated_navigation_paragraphs(page, topic);
                        generated_navigation_paragraphs_added = true;
                    }
                },
                _ => {},
            }
            match paragraph {
                model::Paragraph::Attributes => {}, // This was already added to the page.
                model::Paragraph::Breadcrumbs => {}, // This was already added to the page.
                model::Paragraph::Category => {}, // This was already added to the page.
                model::Paragraph::GenStart => {},
                model::Paragraph::GenEnd => {},
                model::Paragraph::List { list} => {
                    self.add_list(page, list);
                },
                model::Paragraph::Marker { text } => {
                    page.add_paragraph(text);
                },
                model::Paragraph::Placeholder => {
                    // This is OK. It means while creating the model we came across a raw paragraph
                    // that turned out to be something like bookmarks or an attribute table. We
                    // dealt with that in some way that meant we no longer needed the paragraph.
                    // self.add_error(&msg_func_unexpected("Placeholder"));
                },
                model::Paragraph::SectionHeader { name, depth, .. } => {
                    page.add_headline(name, *depth);
                }
                model::Paragraph::Table { table} => {
                    //bg!(topic.get_name());
                    if debug { dbg!(table); }
                    self.add_table(page, table);
                }
                model::Paragraph::Text { text_block} => {
                    let markup = self.text_block_to_markup(text_block);
                    page.add(&markup);
                    page.end_paragraph();
                }
                model::Paragraph::TextUnresolved { .. } => {
                    self.add_error(&msg_func_unexpected("TextUnresolved"));
                }
                model::Paragraph::Unknown { .. } => {
                    dbg!(topic.get_name(), &paragraph);
                    self.add_error(&msg_func_unexpected("Unknown"));
                }
            }
        }
        // We've gotten to the end of the topic without running into the kind of paragraph that
        // signals we're about to get into the detail parts of the page, so we haven't yet added
        // the navigation paragraphs.
        if !generated_navigation_paragraphs_added {
            self.add_generated_navigation_paragraphs(page, topic);
        }
    }

    fn add_generated_navigation_paragraphs(&mut self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        // These would be things like lists of subtopics, combinations, subcategories, and topics
        // within a given category.
        if topic.is_category() {
            // Self::add_category_list(page, &topic.direct_subcategory_nodes(), model::LIST_LABEL_SUBCATEGORIES);
            self.add_subcategory_tree(page, topic);
            let direct_topics = topic.direct_topics_in_category();
            let indirect_topics = topic.indirect_topics_in_category();
            self.add_topic_list(page, &direct_topics, &model::list_type_to_header(model::LIST_TYPE_TOPICS));
            if indirect_topics.len() > direct_topics.len() {
                self.add_topic_list(page, &indirect_topics, &model::list_type_to_header(model::LIST_TYPE_ALL_TOPICS));
            }
        }
        // Self::add_topic_list(page, &topic.subtopics,model::LIST_LABEL_SUBTOPICS);
        self.add_subtopic_tree(page, topic);
        // Combination topics.
        self.add_topic_list(page,&links_to_topic_keys(topic.get_combo_subtopics()),&model::list_type_to_header(model::LIST_TYPE_COMBINATIONS));
    }

    fn add_topic_list(&self, page: &mut wiki::WikiGenPage, topic_keys: &Vec<model::TopicKey>, label: &str) {
        if !topic_keys.is_empty() {
            page.add_line(label);
            for topic_key in topic_keys.iter() {
                page.add_list_item_unordered(1, &self.page_link_simple(&topic_key));
            }
            page.add_linefeed();
        }
    }

    /*
    fn add_category_list(page: &mut wiki::WikiGenPage, nodes: &Vec<Rc<RefCell<model::TopicTreeNode>>>, label: &str) {
        if !nodes.is_empty() {
            page.add_line(label);
            for node_rc in nodes.iter() {
                let node = b!(node_rc);
                let topic_count = node.subtree_leaf_count();
                let topic_key = &node.item;
                let page_link = page_link(topic_key.get_namespace(), topic_key.get_topic_name(), None);
                page.add_list_item_unordered(1, &format!("{} ({})", &page_link, topic_count));
            }
            page.add_linefeed();
        }
    }
     */

    fn add_subcategory_tree(&self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        let node_rc = topic.get_category_tree_node().as_ref().unwrap();
        let node = b!(&node_rc);
        if node.height() > 2 {
            // let filter_func = |node: Ref<TopicTreeNode>| node.height() > 1;
            // let max_depth = node.max_depth_for_max_count_filtered(SUBCATEGORY_TREE_MAX_SIZE, &filter_func);
            let nodes = node.unroll_to_depth(None, None);
            //bg!(topic.get_name(), node.description_line(), max_depth, nodes.len());
            self.gen_partial_topic_tree(page, &nodes, true, Some(&model::list_type_to_header(model::LIST_TYPE_SUBCATEGORIES)));
        }
    }

    fn add_subtopic_tree(&self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        if let Some(node_rc) = &topic.get_subtopic_tree_node() {
            let node = b!(&node_rc);
            if node.height() > 1 {
                let nodes = node.unroll_to_depth(None, None);
                //bg!(topic.get_name(), node.description_line(), max_depth, nodes.len());
                self.gen_partial_topic_tree(page, &nodes, false, Some(&model::list_type_to_header(model::LIST_TYPE_SUBTOPICS)));
            }
        }
    }

    pub(crate) fn gen_partial_topic_tree(&self, page: &mut wiki::WikiGenPage, nodes: &Vec<Rc<RefCell<model::TopicTreeNode>>>, is_category: bool, label: Option<&str>) {
        if !nodes.is_empty() {
            if let Some(label) = label {
                page.add_line(label);
            }
            // Presumably the first item is at the highest level of the tree. That is, we're not
            // going to find any subsequent items that should be outdented compared to this one.
            let base_depth = b!(&nodes[0]).depth();
            for node_rc in nodes.iter() {
                // for node_rc in nodes.iter().sorted_by_key(|node| b!(node).item.get_topic_name().to_lowercase()) {
                let node = b!(node_rc);
                let use_this_node = if is_category { !node.is_leaf() } else { true };
                if use_this_node {
                    let depth = (node.depth() - base_depth) + 1;
                    let link = self.page_link_simple(&node.item);
                    let topic_count_label = if is_category {
                        let topic_count = node.subtree_leaf_count();
                        format!(" ({})", util::format::format_count(topic_count))
                    } else {
                        "".to_string()
                    };
                    let line = format!("{}{}", link, topic_count_label);
                    page.add_list_item_unordered(depth, &line);
                }
            }
            page.add_linefeed();
        }
    }

    fn text_block_to_markup(&mut self, text_block: &model::TextBlock) -> String {
        let mut markup = "".to_string();
        match text_block {
            model::TextBlock::Resolved { items} => {
                for text_item in items.iter() {
                    match text_item {
                        model::TextItem::Text { text } => {
                            markup.push_str(text);
                        },
                        model::TextItem::Link { link } => {
                            markup.push_str(&self.link_to_markup(link));
                        }
                    }
                }
            },
            model::TextBlock::Unresolved { text } => {
                panic!("Text block should be resolved by this point. Text = \"{}\".", text)
            }
        }
        markup
    }

    fn add_list(&mut self, page: &mut wiki::WikiGenPage, list: &model::List) {
        if list.is_generated() {
            return;
        }
        if let Some(header) = list.get_header() {
            match header {
                model::TextBlock::Unresolved { text } => {
                    panic!("Text block should be resolved by this point. Page = \"{}\"; text = \"{}\".", page.topic_name, text)
                }
                _ => {},
            }
            page.add(&self.text_block_to_markup(header));
            page.add_linefeed();
        }
        for list_item in list.get_items().iter() {
            let markup = &self.text_block_to_markup(list_item.get_text_block());
            page.add_list_item(list_item.get_depth(),list_item.is_ordered(), markup);
        }
        page.add_linefeed();
        // if page.topic_name.contains("10,000") { //bg!(&page.content); }
    }

    fn add_table(&mut self, page: &mut wiki::WikiGenPage, table: &model::Table) {
        for (row_index, cells) in table.get_rows().iter().enumerate() {
            let cells_as_markup = cells.iter()
                .map(|cell| self.text_block_to_markup(cell.get_text_block()))
                .collect::<Vec<_>>();
            self.add_table_row(page, table, row_index, &cells_as_markup);
        }
        page.end_paragraph();
    }

    pub(crate) fn add_table_row(&mut self, page: &mut wiki::WikiGenPage, table: &model::Table, row_index: usize, cells: &Vec<String>) {
        // A table header row should look something like:
        //   ^ Color ^ Blue ^
        // A regular table row should look something like:
        //   | Color | Blue |
        let last_delimiter = if table.has_header() && row_index == 0 { DELIM_TABLE_CELL_BOLD } else { DELIM_TABLE_CELL };
        let markup = format!("{}{}\n", cells.iter().enumerate()
            .map(|(col_index, cell_text)| {
                let cell_info = table.get_cell(row_index, col_index);
                let delimiter = if cell_info.is_bold() { DELIM_TABLE_CELL_BOLD } else { DELIM_TABLE_CELL };
                match cell_info.get_horizontal() {
                    model::HorizontalAlignment::Center => {
                        format!("{}  {}  ", delimiter, cell_text.trim())
                    },
                    model::HorizontalAlignment::Left => {
                        format!("{} {} ", delimiter, cell_text.trim())
                    },
                    model::HorizontalAlignment::Right => {
                        format!("{}  {} ", delimiter, cell_text.trim())
                    },
                }
            })
            .join(""),
                             last_delimiter
        );
        page.add_text(&markup);
    }

    fn link_to_markup(&mut self, link: &model::LinkRc) -> String {
        let msg_func_unexpected = |type_, variant: &str| format!("In gen_from_model::add_link(), unexpected {} variant = \"{}\"", type_, variant);
        let link = b!(link);
        let label = link.get_label().map(|label| label.to_string());
        match link.get_type() {
            model::LinkType::External { url } => {
                let text = wiki::gen::external_link_from_string_label(&url, &label);
                text
            },
            model::LinkType::File { file_ref } => {
                let text = wiki::gen::file_ref(&file_ref, &label);
                text
            },
            model::LinkType::Image { source, alignment: _, size: _, type_: _ } => {
                // For now ignore alignment, size, and type (what happens when you click on the image).
                // pub(crate) fn image_part(image_namespace: &str, image_file_name: &str, image_link_type: &WikiImageLinkType, image_size: &WikiImageSize) -> String {
                match source {
                    model::ImageSource::Internal { namespace, file_name } => {
                        let image_ref = image_ref_from_file_name(&namespace, &file_name);
                        let link_type = wiki::gen::WikiImageLinkType::Direct;
                        let size = wiki::gen::WikiImageSize::Original;
                        let text = wiki::gen::image_part(&image_ref, &link_type, &size);
                        text
                    }
                    model::ImageSource::External { url } => {
                        let link_type = wiki::gen::WikiImageLinkType::Direct;
                        let size = wiki::gen::WikiImageSize::Original;
                        let text = wiki::gen::image_part(&url, &link_type, &size);
                        text
                        // self.add_error(&msg_func_unexpected("ImageSource", "External"));
                        //"".to_string()
                    }
                }
            },
            model::LinkType::InternalUnresolved { .. } => {
                self.add_error(&msg_func_unexpected("LinkType", "InternalUnresolved"));
                "".to_string()
            }
            model::LinkType::Section { section_key } => {
                let text = wiki::gen::section_link_from_string_label(&self.model.qualify_namespace(section_key.get_namespace()),section_key.get_topic_name(), &section_key.get_section_name(), &label);
                //bg!(&text);
                text
            },
            model::LinkType::Topic { topic_key } => {
                let page_name = self.model.get_topic_name(&topic_key);
                let text = wiki::gen::page_link_from_string_label(&self.model.qualify_namespace(topic_key.get_namespace()), &page_name, &label);
                text
            },
        }
    }

    pub(crate) fn add_inbound_links_section_optional(&self, page: &mut wiki::WikiGenPage, topic: &Topic) {
        let has_attribute_links = self.model.has_attribute_links(&page.topic_name);
        let has_inbound_links = !topic.get_inbound_topic_keys().is_empty();
        if has_attribute_links || has_inbound_links {
            page.add_headline(HEADLINE_LINKS, 1);
            self.add_attribute_value_topics_list_optional(page);
            self.add_inbound_links_optional(page, topic);
        }
    }

    pub(crate) fn add_attribute_value_topics_list_optional(&self, page: &mut wiki::WikiGenPage) {
        let list = self.model.get_topics_with_attribute_value(&page.topic_name);
        if !list.is_empty() {
            page.add_line("Topics with this attribute:");
            for (topic_key, attribute_type_name) in list.iter() {
                let link = self.page_link_simple(&topic_key);
                let line = format!("({}) {}", attribute_type_name.to_lowercase(), link);
                page.add_list_item_unordered(1, &line);
            }
            page.add_linefeed();
        }
    }

    pub(crate) fn add_inbound_links_optional(&self, page: &mut wiki::WikiGenPage, topic: &Topic) {
        if !topic.get_inbound_topic_keys().is_empty() {
            page.add_line(&model::list::list_type_to_header(model::LIST_TYPE_INBOUND_LINKS));
            for topic_key in topic.get_inbound_topic_keys().iter() {
                let link = self.page_link_simple(&topic_key);
                page.add_list_item_unordered(1, &link);
            }
            page.add_linefeed();
        }
    }

    fn add_error(&mut self, msg: &str) {
        self.errors.add(&self.current_topic_key.as_ref().unwrap(),msg);
    }

    pub(crate) fn page_link_simple(&self, topic_key: &model::TopicKey) -> String {
        //ebug_assert!(self.model.has_topic(topic_key), "Topic key not found: {}", topic_key.to_string());
        Self::page_link(topic_key)
    }

    pub(crate) fn page_link_simple_from_link(&self, link_rc: &LinkRc) -> String {
        let topic_key = b!(link_rc).get_topic_key().unwrap();
        self.page_link_simple(&topic_key)
    }

    #[allow(dead_code)]
    pub(crate) fn section_link_simple(&self, topic_key: &model::TopicKey, section_name: &str) -> String {
        //ebug_assert!(self.model.has_topic(topic_key), "Topic key not found: {}", topic_key.to_string());
        Self::section_link(topic_key, section_name)
    }

    pub(crate) fn domain_link(&self, domain_name: &str, on_attribute_value_page: bool) -> String {
        if on_attribute_value_page {
            wiki::section_link_same_page(&domain_name, None)
        } else {
            wiki::section_link(&self.model.namespace_navigation(), PAGE_NAME_ATTR_VALUE, domain_name, Some(domain_name))
        }
    }

    pub(crate) fn page_link_qualified(&self, topic_key: &model::TopicKey) -> String {
        wiki::page_link(&self.model.qualify_namespace(topic_key.get_namespace()), topic_key.get_topic_name(), None)
    }

    pub(crate) fn page_link(topic_key: &model::TopicKey) -> String {
        let link = wiki::page_link(&topic_key.get_namespace(), &topic_key.get_topic_name(), None);
        link
    }

    #[allow(dead_code)]
    pub(crate) fn section_link(topic_key: &model::TopicKey, section_name: &str) -> String {
        let link = wiki::section_link(&topic_key.get_namespace(), &topic_key.get_topic_name(), section_name, None);
        link
    }

    /*
    pub(crate) fn copy_image_files(path_from: &str, path_to: &str, print: bool) {
        dbg!(&path_from, &path_to);
        for path in fs::read_dir(path_from).unwrap() {
            let dir_entry = path.as_ref().unwrap();
            let file_name_from = util::file::dir_entry_to_file_name(dir_entry);
            let full_file_name_from = format!("{}/{}", path_from, file_name_from);
            let full_file_name_to = format!("{}/{}", path_to, wiki::gen::legal_file_name(&file_name_from));
            if print { println!("{} => {}", full_file_name_from, full_file_name_to); }
            std::fs::copy(&full_file_name_from, full_file_name_to).unwrap();
        }
    }
     */

}
