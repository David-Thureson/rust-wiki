use crate::*;
use crate::{model, Itertools};
use crate::dokuwiki as wiki;
use crate::dokuwiki::page_link;
use std::rc::Rc;
use std::cell::RefCell;

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
        let mut page = wiki::WikiGenPage::new(&self.model.qualify_namespace(model::NAMESPACE_NAVIGATION), wiki::PAGE_NAME_CATEGORIES,None);
        // model.category_tree().print_counts_to_depth();
        // model.category_tree().print_with_items(None);
        // for node_rc in model.category_tree().top_nodes.iter() {
        //     gen_category_subtree(&mut page, 1, b!(node_rc));
        //}
        // model.category_tree().print_with_items(None);
        let nodes = self.model.category_tree().unroll_to_depth(None);
        //bg!(nodes.len());
        Self::gen_partial_topic_tree(&mut page, &nodes, 0, true, None);
        page.write();
    }

    pub fn gen_subtopics_page(&self) {
        let mut page = wiki::WikiGenPage::new(&self.model.qualify_namespace(model::NAMESPACE_NAVIGATION), wiki::PAGE_NAME_SUBTOPICS,None);
        let nodes = self.model.subtopic_tree().unroll_to_depth(None);
        Self::gen_partial_topic_tree(&mut page, &nodes, 0, false, None);
        page.write();
    }

    pub fn gen(&mut self) {
        for topic in self.model.topics.values() {
            self.current_topic_key = Some(topic.get_key());
            //bg!(&self.current_topic_key);
            let mut page = wiki::WikiGenPage::new(&self.model.qualify_namespace(&topic.namespace), &topic.name, None);
            self.add_breadcrumbs_optional(&mut page, &topic);
            self.add_category_optional(&mut page, &topic);
            self.add_paragraphs(&mut page, &topic);
            page.write();
        }
        self.errors.print(Some("GenFromModel::gen()"));
    }

    fn add_category_optional(&mut self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        if let Some(category) = topic.category.as_ref() {
            page.add_category(&self.model.qualify_namespace(&self.model.main_namespace),category);
        }
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
                    .map(|topic_key| Self::page_link(self.model, topic_key))
                    .join(&format!(" {} ", wiki::DELIM_BOOKMARK_RIGHT));
                let breadcrumbs = format!("{}{} {} {}{}", wiki::DELIM_BOLD, breadcrumbs, wiki::DELIM_BOOKMARK_RIGHT, topic.name, wiki::DELIM_BOLD);
                page.add_paragraph(&breadcrumbs);
            },
            2 => {
                // Combination topic.
                let link_a = Self::page_link(self.model, &topic.parents[0]);
                let link_b = Self::page_link(self.model, &topic.parents[1]);
                let breadcrumbs = format!("{}{} {} {} {} {}{}", wiki::DELIM_BOLD, link_a, wiki::DELIM_BOOKMARK_RIGHT, topic.name, wiki::DELIM_BOOKMARK_LEFT, link_b, wiki::DELIM_BOLD);
                page.add_paragraph(&breadcrumbs);
            },
            _ => {
                panic!("Unexpected number of parent topics for topic \"{}\".", topic.name);
            },
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
                model::Paragraph::Attributes => {},
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
            Self::add_subcategory_tree(page, topic);
            let direct_topics = topic.direct_topics_in_category();
            let indirect_topics = topic.indirect_topics_in_category();
            Self::add_topic_list(page, &direct_topics, model::LIST_LABEL_CATEGORY_TOPICS);
            if indirect_topics.len() > direct_topics.len() {
                Self::add_topic_list(page, &indirect_topics, model::LIST_LABEL_CATEGORY_TOPICS_ALL);
            }
        }
        Self::add_topic_list(page, &topic.subtopics,model::LIST_LABEL_SUBTOPICS);
        Self::add_topic_list(page, &topic.combo_subtopics,model::LIST_LABEL_COMBINATIONS);
    }

    fn add_topic_list(page: &mut wiki::WikiGenPage, topic_keys: &Vec<model::TopicKey>, label: &str) {
        if !topic_keys.is_empty() {
            page.add_line(label);
            for topic_key in topic_keys.iter() {
                page.add_list_item_unordered(1, &page_link(&topic_key.namespace, &topic_key.topic_name, None));
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

    fn add_subcategory_tree(page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        let node_rc = topic.category_tree_node.as_ref().unwrap();
        let node = b!(&node_rc);
        if node.height() > 2 {
            // let filter_func = |node: Ref<TopicTreeNode>| node.height() > 1;
            // let max_depth = node.max_depth_for_max_count_filtered(SUBCATEGORY_TREE_MAX_SIZE, &filter_func);
            let nodes = node.unroll_to_depth(None, None);
            //bg!(&topic.name, node.description_line(), max_depth, nodes.len());
            Self::gen_partial_topic_tree(page, &nodes, node.depth(), true, Some(model::LIST_LABEL_SUBCATEGORIES));
        }
    }

    pub fn gen_partial_topic_tree(page: &mut wiki::WikiGenPage, nodes: &Vec<Rc<RefCell<model::TopicTreeNode>>>, start_depth: usize, is_category: bool, label: Option<&str>) {
        if !nodes.is_empty() {
            if let Some(label) = label {
                page.add_line(label);
            }
            for node_rc in nodes.iter() {
                let node = b!(node_rc);
                let use_this_node = if is_category { !node.is_leaf() } else { true };
                if use_this_node {
                    let depth = node.depth() - start_depth;
                    let link = wiki::page_link(&node.item.namespace, &node.item.topic_name, None);
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
                let text = wiki::gen::page_link(&topic_key.namespace, &page_name, label);
                text
            },
            model::LinkType::Section { section_key } => {
                let text = wiki::gen::section_link(section_key.namespace(),section_key.topic_name(), &section_key.section_name, label);
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
                        let text = wiki::gen::image_part(&namespace, &file_name, &wiki::gen::WikiImageLinkType::Direct, &wiki::gen::WikiImageSize::Large);
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

    fn add_error(&mut self, msg: &str) {
        self.errors.add(&self.current_topic_key.as_ref().unwrap(),msg);
    }

    pub fn page_link(model: &model::Wiki, topic_key: &model::TopicKey) -> String {
        let qual_namespace = model.qualify_namespace(&topic_key.namespace);
        let link = wiki::page_link(&qual_namespace, &model.topics.get(topic_key).unwrap().name, None);
        link
    }
}
