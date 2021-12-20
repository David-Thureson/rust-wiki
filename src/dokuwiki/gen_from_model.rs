use crate::model;
use crate::dokuwiki as wiki;
use crate::model::NAMESPACE_CATEGORY;

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
            let mut page = wiki::WikiGenPage::new(&topic.namespace, &topic.name, None);
            self.add_category_optional(&mut page, &topic);
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

    fn add_paragraphs(&mut self, page: &mut wiki::WikiGenPage, topic: &model::Topic) {
        let msg_func_unexpected = |variant_name: &str| format!("In gen_from_model::add_paragraph(), unexpected Paragraph variant = \"{}\"", variant_name);
        // let add_error_unexpected = |paragraph_variant: &str| self.add_error(&msg_func_unexpected(paragraph_variant));
        for paragraph in topic.paragraphs.iter() {
            match paragraph {
                model::Paragraph::Attributes => {},
                model::Paragraph::Breadcrumbs => {},
                model::Paragraph::Category => {}, // This was already added to the page.
                model::Paragraph::GenStart => {},
                model::Paragraph::GenEnd => {},
                model::Paragraph::List { .. } => {},
                model::Paragraph::Placeholder => {
                    self.add_error(&msg_func_unexpected("Placeholder"));
                },
                model::Paragraph::Quote { .. } => {}
                model::Paragraph::SectionHeader { name, depth } => {
                    page.add_headline(name, *depth);
                }
                model::Paragraph::Text { text_block } => {
                    self.add_text_block(page, text_block);
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

    fn add_text_block(&mut self, page: &mut wiki::WikiGenPage, text_block: &model::TextBlock) {
        for text_item in text_block.items.iter() {
            match text_item {
                model::TextItem::Text { text } => {
                    page.add_text(text);
                },
                model::TextItem::Link { link } => {
                    self.add_link(page, link);
                }
            }
        }
    }

    fn add_link(&mut self, page: &mut wiki::WikiGenPage, link: &model::Link) {
        let msg_func_unexpected = |type_, variant: &str| format!("In gen_from_model::add_link(), unexpected {} variant = \"{}\"", type_, variant);
        let label = match &link.label {
            Some(label) => Some(label.as_str()),
            None => None,
        };
        match &link.type_ {
            model::LinkType::Topic { topic_key } => {
                let page_name = self.model.topic_name(&topic_key);
                let text = wiki::gen::page_link(&topic_key.0, &page_name, label);
                page.add_text(&text);
            },
            model::LinkType::Section { section_key } => {
                let text = wiki::gen::section_link(&section_key.0.0,&section_key.0.1,&section_key.1, label);
                page.add_text(&text);
            },
            model::LinkType::External { url } => {
                let text = wiki::gen::external_link(&url, label);
                page.add_text(&text);
            },
            model::LinkType::Image { source, alignment: _, size: _, type_: _ } => {
                // For now ignore alignment, size, and type (what happens when you click on the image).
                // pub(crate) fn image_part(image_namespace: &str, image_file_name: &str, image_link_type: &WikiImageLinkType, image_size: &WikiImageSize) -> String {
                match source {
                    model::ImageSource::Internal { namespace, file_name } => {
                        let text = wiki::gen::image_part(&namespace, &file_name, &wiki::gen::WikiImageLinkType::Direct, &wiki::gen::WikiImageSize::Large);
                        page.add_text(&text);
                    }
                    model::ImageSource::External {..} => {
                        self.add_error(&msg_func_unexpected("ImageSource", "External"));
                    }
                }
            },
            model::LinkType::InternalUnresolved { .. } => {
                self.add_error(&msg_func_unexpected("LinkType", "InternalUnresolved"));
            }
        }
    }

    fn add_error(&mut self, msg: &str) {
        self.errors.add(&self.current_topic_key.as_ref().unwrap(),msg);
    }
}
