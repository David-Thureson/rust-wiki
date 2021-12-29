use crate::{model, Itertools};
use crate::dokuwiki as wiki;
use crate::model::{NAMESPACE_CATEGORY, TextBlock, TopicKey};

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

    pub fn gen(&mut self) {
        for topic in self.model.topics.values() {
            self.current_topic_key = Some(topic.get_key());
            let mut page = wiki::WikiGenPage::new(&self.model.qualify_namespace(&topic.namespace), &topic.name, None);
            self.add_category_optional(&mut page, &topic);
            self.add_breadcrumbs_optional(&mut page, &topic);
            self.add_paragraphs(&mut page, &topic);
            page.write();
        }
        self.errors.print(Some("GenFromModel::gen()"));
    }

    fn add_category_optional(&mut self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        if let Some(category) = topic.category.as_ref() {
            page.add_category(&self.model.qualify_namespace(NAMESPACE_CATEGORY),category);
        }
    }

    fn add_breadcrumbs_optional(&mut self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        match topic.parents.len() {
            0 => {},
            1 => {
                let mut topic_keys = vec![];
                let mut parent_topic_key = &topic.parents[0];
                loop {
                    topic_keys.push(parent_topic_key);
                    let parent_topic = self.model.topics.get(parent_topic_key).unwrap();
                    match parent_topic.parents.len() {
                        0 => {
                            break;
                        },
                        1 => {
                            parent_topic_key = &parent_topic.parents[0];
                        },
                        _ => {
                            panic!("Unexpected number of parent topics for topic \"{}\".", parent_topic.name);
                        }
                    }
                    topic_keys.reverse();
                    let breadcrumbs = topic_keys.iter()
                        .map(|topic_key| Self::page_link(self.model, topic_key))
                        .join(&format!(" {} ", wiki::DELIM_BOOKMARK_RIGHT));
                    let breadcrumbs = format!("{}{} {} {}{}", wiki::DELIM_BOLD, breadcrumbs, wiki::DELIM_BOOKMARK_RIGHT, topic.name, wiki::DELIM_BOLD);
                    page.add_paragraph(&breadcrumbs);
                }
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
        for paragraph in topic.paragraphs.iter() {
            match paragraph {
                model::Paragraph::Attributes => {},
                model::Paragraph::Breadcrumbs => {},
                model::Paragraph::Category => {}, // This was already added to the page.
                model::Paragraph::Code { .. } => {}
                model::Paragraph::GenStart => {},
                model::Paragraph::GenEnd => {},
                model::Paragraph::List { type_: _, header, items} => {
                    self.add_list(page, header, items);
                },
                model::Paragraph::Placeholder => {
                    self.add_error(&msg_func_unexpected("Placeholder"));
                },
                model::Paragraph::QuoteEnd => {
                    page.add_paragraph(wiki::DELIM_QUOTE_END);
                }
                model::Paragraph::QuoteStart => {
                    page.add_line(wiki::DELIM_QUOTE_START);
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

    fn add_list(&mut self, page: &mut wiki::WikiGenPage, header: &model::TextBlock, items: &Vec<model::ListItem>) {
        page.add(&self.text_block_to_markup(header));
        page.add_linefeed();
        for item in items.iter() {
            let markup = &self.text_block_to_markup(&item.block);
            page.add_list_item_unordered(item.depth,markup);
        }
    }

    fn add_table(&mut self, page: &mut wiki::WikiGenPage, has_header: bool, rows:&Vec<Vec<TextBlock>>) {
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
                let text = wiki::gen::page_link(&topic_key.0, &page_name, label);
                text
            },
            model::LinkType::Section { section_key } => {
                let text = wiki::gen::section_link(&section_key.0.0,&section_key.0.1,&section_key.1, label);
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

    pub fn page_link(model: &model::Wiki, topic_key: &TopicKey) -> String {
        let qual_namespace = model.qualify_namespace(&topic_key.0);
        let link = wiki::page_link(&qual_namespace, &model.topics.get(topic_key).unwrap().name, None);
        link
    }
}
