use crate::*;
use crate::{model, Itertools};
use crate::dokuwiki as wiki;
use std::rc::Rc;
use std::cell::RefCell;
use crate::model::{AttributeValueType, TopicKey, Topic};
use std::collections::BTreeMap;
use crate::dokuwiki::{PAGE_NAME_ATTR_VALUE, WikiAttributeTable, PAGE_NAME_ATTR, PAGE_NAME_ATTR_DATE, PAGE_NAME_ATTR_YEAR};
use std::fs;

//const SUBCATEGORY_TREE_MAX_SIZE: usize = 30;

pub struct GenFromModel<'a> {
    model: &'a model::Wiki,
    current_topic_key: Option<model::TopicKey>,
    errors: model::TopicErrorList,
}

impl <'a> GenFromModel<'a> {

    pub fn new(model: &'a model::Wiki) -> Self {
        Self {
            model: model,
            current_topic_key: None,
            errors: model::TopicErrorList::new(),
        }
    }

    pub fn gen_categories_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.namespace_navigation(), wiki::PAGE_NAME_CATEGORIES,None);
        let nodes = self.model.category_tree().unroll_to_depth(None);

        // Debugging:
        // for node in nodes.iter() {
        //     let topic_key = &b!(node).item;
        //     debug_assert!(self.model.has_topic(topic_key), "Topic key not found: {}", topic_key.to_string());
        // }

        self.gen_partial_topic_tree(&mut page, &nodes, 0, true, None);
        page.write();
    }

    pub fn gen_subtopics_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.namespace_navigation(), wiki::PAGE_NAME_SUBTOPICS,None);
        let nodes = self.model.subtopic_tree().unroll_to_depth(None);
        self.gen_partial_topic_tree(&mut page, &nodes, 0, false, None);
        page.write();
    }

    pub fn gen_attr_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.namespace_navigation(), wiki::PAGE_NAME_ATTR,None);
        for attribute_type in self.model.attributes.attributes.values()
            .filter(|attribute_type| attribute_type.value_type != AttributeValueType::Date && attribute_type.value_type != AttributeValueType::Year)
            .filter(|attribute_type| self.model.attributes.attributes_to_index.contains(&attribute_type.name)) {
            page.add_headline(&attribute_type.name,1);
            for (value, topic_keys) in attribute_type.values.iter() {
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
            }
        }
        page.write();
    }

    pub fn gen_attr_value_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.namespace_navigation(), wiki::PAGE_NAME_ATTR_VALUE,None);
        let mut map = BTreeMap::new();
        for attribute_type in self.model.attributes.attributes.values()
                .filter(|attribute_type| self.model.attributes.attributes_to_index.contains(&attribute_type.name)) {
            for (value, topic_keys) in attribute_type.values.iter() {
                let entry = map.entry(value).or_insert(vec![]);
                for topic_key in topic_keys.iter() {
                    entry.push((attribute_type.name.clone(), topic_key.clone()));
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
            list.sort_by(|a, b| a.1.topic_name.cmp(&b.1.topic_name).then(a.0.cmp(&b.0)));
            page.add_line("Topics:");
            for (attribute_type_name, topic_key) in list.drain(..) {
                let link = self.page_link_simple(&topic_key);
                let line = format!("({}) {}", attribute_type_name.to_lowercase(), link);
                page.add_list_item_unordered(1, &line);
            }
        }
        page.write();
    }

    fn add_related_domains_optional(&self, page: &mut wiki::WikiGenPage, attribute_value_name: &str, on_attribute_value_page: bool) {
        if let Some(domain) = self.model.domains.domains.get(attribute_value_name) {
            if !domain.related_by_count.is_empty() {
                let related_link_list = domain.related_by_count.iter()
                    .map(|related_name| self.domain_link(related_name, on_attribute_value_page))
                    .join(", ");
                let line = format!("Related: {}", related_link_list);
                page.add_paragraph(&line);
            }
        }
    }

    pub fn gen_attr_year_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.namespace_navigation(), wiki::PAGE_NAME_ATTR_YEAR,None);
        let values = self.model.get_distinct_attr_values(&AttributeValueType::Year);
        for value in values.iter() {
            let display_value = model::AttributeType::value_to_display_string(&AttributeValueType::Year, value);
            page.add_headline(&display_value,1);
            for topic_key in self.model.get_topics_for_attr_value(&AttributeValueType::Year, &value, None) {
                let link = self.page_link_simple(&topic_key);
                page.add_list_item_unordered(1, &link);
            }
        }
        page.write();
    }

    pub fn gen_attr_date_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.namespace_navigation(), wiki::PAGE_NAME_ATTR_DATE,None);
        let values = self.model.get_distinct_attr_values(&AttributeValueType::Date);
        let dates = values.iter().map(|value| model::AttributeType::value_to_date(value)).collect::<Vec<_>>();
        let year_month_map = util::date_time::year_month_map(dates);
        for (year, month_map) in year_month_map.iter() {
            page.add_headline(&year.to_string(),1);
            for (month, dates) in month_map.iter() {
                page.add_headline(&util::date_time::year_month_to_mon_format(*year, *month), 2);
                for date in dates.iter() {
                    let display_value = model::AttributeType::date_to_display_string(&date);
                    page.add_headline(&display_value, 3);
                    let match_value = model::AttributeType::date_to_canonical_value(date);
                    for (attribute_type_name, topic_key) in self.model.get_typed_topics_for_attr_value(&AttributeValueType::Date, &match_value, None) {
                        let link = self.page_link_simple(&topic_key);
                        page.add_list_item_unordered(1, &format!("({}) {}", attribute_type_name.to_lowercase(), link));
                    }
                }
            }
        }
        page.write();
    }

    fn page_link_if_exists(&self, topic_name: &str) -> Option<String> {
        let possible_topic_key = TopicKey::new(&self.model.main_namespace, topic_name);
        if self.model.has_topic(&possible_topic_key) {
            Some(self.page_link_simple(&possible_topic_key))
        } else {
            None
        }
    }

    /*
    pub fn gen_attr_date_page(&self) {
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
                    let link = wiki::page_link(&self.model.qualify_namespace(&topic_key.namespace), &topic_key.topic_name, None);
                    page.add_list_item_unordered(1, &link);
                }
            }
        }
        page.write();
    }
    */

    pub fn gen(&mut self) {
        for topic in self.model.topics.values() {
            self.current_topic_key = Some(topic.get_key());
            //bg!(&self.current_topic_key);
            let mut page = wiki::WikiGenPage::new(&self.model.qualify_namespace(&topic.namespace), &topic.name, None);
            self.add_breadcrumbs_optional(&mut page, &topic);
            self.add_category_optional(&mut page, &topic);
            self.add_attributes_optional(&mut page, &topic);
            self.add_paragraphs(&mut page, &topic);
            self.add_inbound_links_section_optional(&mut page, &topic);
            page.write();
        }
        self.errors.print(Some("GenFromModel::gen()"));
    }

    fn add_breadcrumbs_optional(&mut self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        // if topic.name.starts_with("Test ") {
        //     /bg!(&topic.name, &topic.parents);
        // }
        match topic.parents.len() {
            0 => {},
            1 => {
                //bg!(&topic.name, &topic.parents);
                let mut topic_keys = vec![];
                let mut parent_topic_key = topic.parents[0].clone();
                loop {
                    //bg!(&parent_topic_key);
                    topic_keys.push(parent_topic_key.clone());
                    let parent_topic = self.model.topics.get(&parent_topic_key).unwrap();
                    //bg!(&parent_topic.name, &parent_topic.parents);
                    match parent_topic.parents.len() {
                        0 => {
                            break;
                        },
                        1 => {
                            parent_topic_key = parent_topic.parents[0].clone();
                        },
                        _ => {
                            panic!("Unexpected number of parent topics for topic \"{}\".", parent_topic.name);
                        }
                    }
                }
                //bg!(&topic_keys);
                topic_keys.reverse();
                let breadcrumbs = topic_keys.iter()
                    .map(|topic_key| self.page_link_simple(topic_key))
                    .join(&format!(" {} ", wiki::DELIM_BOOKMARK_RIGHT));
                let breadcrumbs = format!("{}{} {} {}{}", wiki::DELIM_BOLD, breadcrumbs, wiki::DELIM_BOOKMARK_RIGHT, topic.name, wiki::DELIM_BOLD);
                page.add_paragraph(&breadcrumbs);
            },
            2 => {
                // Combination topic.
                let link_a = self.page_link_simple(&topic.parents[0]);
                let link_b = self.page_link_simple(&topic.parents[1]);
                let breadcrumbs = format!("{}{} {} {} {} {}{}", wiki::DELIM_BOLD, link_a, wiki::DELIM_BOOKMARK_RIGHT, topic.name, wiki::DELIM_BOOKMARK_LEFT, link_b, wiki::DELIM_BOLD);
                page.add_paragraph(&breadcrumbs);
            },
            _ => {
                panic!("Unexpected number of parent topics for topic \"{}\".", topic.name);
            },
        }
    }

    fn add_category_optional(&mut self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        if let Some(category) = topic.category.as_ref() {
            page.add_category(&self.model.main_namespace,category);
        }
    }

    fn add_attributes_optional(&mut self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        if !topic.attributes.is_empty() {
            let namespace_navigation = &self.model.namespace_navigation();
            let mut table = WikiAttributeTable::new();
            for attr_instance in topic.attributes.values()
                    .sorted_by_key(|attr_instance| attr_instance.sequence) {
                let attr_type = self.model.attributes.attributes.get(&attr_instance.attribute_type_name).unwrap();
                let attr_type_link = match attr_type.value_type {
                    AttributeValueType::Date => wiki::page_link(&namespace_navigation, PAGE_NAME_ATTR_DATE,Some(&attr_type.name)),
                    AttributeValueType::Year => wiki::page_link(&namespace_navigation, PAGE_NAME_ATTR_YEAR,Some(&attr_type.name)),
                    _ => if self.model.attributes.attributes_to_index.contains(&attr_type.name) {
                        wiki::section_link(&namespace_navigation, PAGE_NAME_ATTR, &attr_type.name, Some(&attr_type.name))
                    } else {
                        attr_type.name.clone()
                    },
                };
                let value_list = attr_instance.values.iter()
                    .map(|value| {
                        let label = attr_type.get_value_display_string(value);
                        match attr_type.value_type {
                            AttributeValueType::Date => wiki::section_link(&namespace_navigation, PAGE_NAME_ATTR_DATE,&label,Some(&label)),
                            AttributeValueType::Year => wiki::section_link(&namespace_navigation, PAGE_NAME_ATTR_YEAR,&label,Some(&label)),
                            _ => if self.model.attributes.attributes_to_index.contains(&attr_type.name) {
                                wiki::section_link(&namespace_navigation, PAGE_NAME_ATTR_VALUE, &label, Some(&label))
                            } else {
                                label
                            },
                        }})
                    .join(", ");
                table.add_row(&attr_type_link, &value_list);
            }
            page.add_text(&table.get_markup());
        }
    }

    fn add_paragraphs(&mut self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        let msg_func_unexpected = |variant_name: &str| format!("In gen_from_model::add_paragraph(), unexpected Paragraph variant = \"{}\"", variant_name);
        // let add_error_unexpected = |paragraph_variant: &str| self.add_error(&msg_func_unexpected(paragraph_variant));
        let mut generated_navigation_paragraphs_added = false;
        for paragraph in topic.paragraphs.iter() {
            // First see if it's necessary to add generated navigation paragraphs like subtopics
            // and subcategories.
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
                model::Paragraph::Code { text} => {
                    self.add_code(page, text)
                }
                model::Paragraph::GenStart => {},
                model::Paragraph::GenEnd => {},
                model::Paragraph::List { type_, header, items} => {
                    self.add_list(page, type_, header, items);
                },
                model::Paragraph::Placeholder => {
                    self.add_error(&msg_func_unexpected("Placeholder"));
                },
                model::Paragraph::QuoteEnd => {
                    page.add(wiki::DELIM_QUOTE_END);
                }
                model::Paragraph::QuoteStart => {
                    page.add_paragraph(wiki::DELIM_QUOTE_START);
                }
                model::Paragraph::SectionHeader { name, depth } => {
                    page.add_headline(name, *depth);
                }
                model::Paragraph::Table { has_header, rows} => {
                    self.add_table(page, *has_header, rows);
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
            Self::add_topic_list(self, page, &direct_topics, model::LIST_LABEL_CATEGORY_TOPICS);
            if indirect_topics.len() > direct_topics.len() {
                self.add_topic_list(page, &indirect_topics, model::LIST_LABEL_CATEGORY_TOPICS_ALL);
            }
        }
        // Self::add_topic_list(page, &topic.subtopics,model::LIST_LABEL_SUBTOPICS);
        self.add_subtopic_tree(page, topic);
        self.add_topic_list(page,&topic.combo_subtopics,model::LIST_LABEL_COMBINATIONS);
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
                let page_link = page_link(&topic_key.namespace, &topic_key.topic_name, None);
                page.add_list_item_unordered(1, &format!("{} ({})", &page_link, topic_count));
            }
            page.add_linefeed();
        }
    }
     */

    fn add_subcategory_tree(&self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        let node_rc = topic.category_tree_node.as_ref().unwrap();
        let node = b!(&node_rc);
        if node.height() > 2 {
            // let filter_func = |node: Ref<TopicTreeNode>| node.height() > 1;
            // let max_depth = node.max_depth_for_max_count_filtered(SUBCATEGORY_TREE_MAX_SIZE, &filter_func);
            let nodes = node.unroll_to_depth(None, None);
            //bg!(&topic.name, node.description_line(), max_depth, nodes.len());
            self.gen_partial_topic_tree(page, &nodes, node.depth(), true, Some(model::LIST_LABEL_SUBCATEGORIES));
        }
    }

    fn add_subtopic_tree(&self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        if let Some(node_rc) = &topic.subtopic_tree_node {
            let node = b!(&node_rc);
            if node.height() > 1 {
                let nodes = node.unroll_to_depth(None, None);
                //bg!(&topic.name, node.description_line(), max_depth, nodes.len());
                self.gen_partial_topic_tree(page, &nodes, node.depth(), false, Some(model::LIST_LABEL_SUBTOPICS));
            }
        }
    }

    pub fn gen_partial_topic_tree(&self, page: &mut wiki::WikiGenPage, nodes: &Vec<Rc<RefCell<model::TopicTreeNode>>>, start_depth: usize, is_category: bool, label: Option<&str>) {
        if !nodes.is_empty() {
            if let Some(label) = label {
                page.add_line(label);
            }
            for node_rc in nodes.iter() {
                let node = b!(node_rc);
                let use_this_node = if is_category { !node.is_leaf() } else { true };
                if use_this_node {
                    let depth = node.depth() - start_depth;
                    let link = self.page_link_simple(&node.item);
                    let topic_count_label = if is_category {
                        let topic_count = node.subtree_leaf_count();
                        format!(" ({})", util::format::format_count(topic_count))
                    } else {
                        "".to_string()
                    };
                    let line = format!("{}{}", link, topic_count_label);
                    page.add_list_item_unordered(depth + 1, &line);
                }
            }
        }
    }

    fn text_block_to_markup(&mut self, text_block: &model::TextBlock) -> String {
        let mut markup = "".to_string();
        for text_item in text_block.items.iter() {
            match text_item {
                model::TextItem::Text { text } => {
                    markup.push_str(text);
                },
                model::TextItem::Link { link } => {
                    markup.push_str(&self.link_to_markup(link));
                }
            }
        }
        markup
    }

    fn add_code(&mut self, page: &mut wiki::WikiGenPage, text: &str) {
        //bg!(&text);
        page.add_line(wiki::DELIM_CODE_START);
        page.add_line(text);
        page.add_paragraph(wiki::DELIM_CODE_END);
    }

    fn add_list(&mut self, page: &mut wiki::WikiGenPage, type_: &model::ListType, header: &model::TextBlock, items: &Vec<model::ListItem>) {
        match type_ {
            model::ListType::Subtopics | model::ListType::Combinations => {}, // These are generated elsewhere.
            _ => {
                page.add(&self.text_block_to_markup(header));
                page.add_linefeed();
                for item in items.iter() {
                    let markup = &self.text_block_to_markup(&item.block);
                    page.add_list_item_unordered(item.depth, markup);
                }
                page.add_linefeed();
            }
        }
    }

    fn add_table(&mut self, page: &mut wiki::WikiGenPage, has_header: bool, rows:&Vec<Vec<model::TextBlock>>) {
        for (index, cells) in rows.iter().enumerate() {
            let cells_as_markup = cells.iter()
                .map(|cell| self.text_block_to_markup(cell))
                .collect::<Vec<_>>();
            let is_header = has_header && index == 0;
            page.add_table_row(is_header,&cells_as_markup);
        }
        page.end_paragraph();
    }

    fn link_to_markup(&mut self, link: &model::Link) -> String {
        let msg_func_unexpected = |type_, variant: &str| format!("In gen_from_model::add_link(), unexpected {} variant = \"{}\"", type_, variant);
        let label = match &link.label {
            Some(label) => Some(label.as_str()),
            None => None,
        };
        match &link.type_ {
            model::LinkType::Topic { topic_key } => {
                let page_name = self.model.topic_name(&topic_key);
                let text = wiki::gen::page_link(&self.model.qualify_namespace(&topic_key.namespace), &page_name, label);
                text
            },
            model::LinkType::Section { section_key } => {
                let text = wiki::gen::section_link(&self.model.qualify_namespace(section_key.namespace()),section_key.topic_name(), &section_key.section_name, label);
                //bg!(&text);
                text
            },
            model::LinkType::External { url } => {
                let text = wiki::gen::external_link(&url, label);
                text
            },
            model::LinkType::Image { source, alignment: _, size: _, type_: _ } => {
                // For now ignore alignment, size, and type (what happens when you click on the image).
                // pub(crate) fn image_part(image_namespace: &str, image_file_name: &str, image_link_type: &WikiImageLinkType, image_size: &WikiImageSize) -> String {
                match source {
                    model::ImageSource::Internal { namespace, file_name } => {
                        let link_type = wiki::gen::WikiImageLinkType::Direct;
                        let size = wiki::gen::WikiImageSize::Original;
                        let text = wiki::gen::image_part(&namespace, &file_name, &link_type, &size);
                        text
                    }
                    model::ImageSource::External {..} => {
                        self.add_error(&msg_func_unexpected("ImageSource", "External"));
                        "".to_string()
                    }
                }
            },
            model::LinkType::InternalUnresolved { .. } => {
                self.add_error(&msg_func_unexpected("LinkType", "InternalUnresolved"));
                "".to_string()
            }
        }
    }

    pub fn add_inbound_links_section_optional(&self, page: &mut wiki::WikiGenPage, topic: &Topic) {
        let has_attribute_links = self.model.attributes.has_attribute_links(&page.topic_name);
        let has_inbound_links = !topic.inbound_topic_keys.is_empty();
        if has_attribute_links || has_inbound_links {
            page.add_headline("Inbound Links", 1);
            self.add_attribute_value_topics_list_optional(page);
            self.add_inbound_links_optional(page, topic);
        }
    }

    pub fn add_attribute_value_topics_list_optional(&self, page: &mut wiki::WikiGenPage) {
        if let Some(list) = self.model.attributes.values.get(&page.topic_name) {
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
    }

    pub fn add_inbound_links_optional(&self, page: &mut wiki::WikiGenPage, topic: &Topic) {
        if !topic.inbound_topic_keys.is_empty() {
            page.add_line("Inbound links:");
            for topic_key in topic.inbound_topic_keys.iter() {
                let link = self.page_link_simple(&topic_key);
                page.add_list_item_unordered(1, &link);
            }
            page.add_linefeed();
        }
    }

    fn add_error(&mut self, msg: &str) {
        self.errors.add(&self.current_topic_key.as_ref().unwrap(),msg);
    }

    pub fn page_link_simple(&self, topic_key: &model::TopicKey) -> String {
        //ebug_assert!(self.model.has_topic(topic_key), "Topic key not found: {}", topic_key.to_string());
        Self::page_link(topic_key)
    }

    pub fn section_link_simple(&self, topic_key: &model::TopicKey, section_name: &str) -> String {
        //ebug_assert!(self.model.has_topic(topic_key), "Topic key not found: {}", topic_key.to_string());
        Self::section_link(topic_key, section_name)
    }

    pub fn domain_link(&self, domain_name: &str, on_attribute_value_page: bool) -> String {
        if on_attribute_value_page {
            wiki::section_link_same_page(&domain_name, None)
        } else {
            wiki::section_link(&self.model.namespace_navigation(), PAGE_NAME_ATTR_VALUE, domain_name, Some(domain_name))
        }
    }

    pub fn page_link(topic_key: &model::TopicKey) -> String {
        let link = wiki::page_link(&topic_key.namespace, &topic_key.topic_name, None);
        link
    }

    pub fn section_link(topic_key: &model::TopicKey, section_name: &str) -> String {
        let link = wiki::section_link(&topic_key.namespace, &topic_key.topic_name, section_name, None);
        link
    }

    pub fn copy_image_files(path_from: &str, path_to: &str, print: bool) {
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

}
