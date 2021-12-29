use super::*;

// pub type TextBlockRc = Rc<RefCell<TextBlock>>;

#[derive(Clone, Debug)]
pub struct TextBlock {
    pub items: Vec<TextItem>,
}

#[derive(Clone, Debug)]
pub enum TextItem {
    Text {
        text: String,
    },
    Link {
        link: Link,
    },
}

impl TextBlock {
    pub fn new() -> Self {
        Self {
            items: vec![],
        }
    }

    pub fn update_internal_links(&mut self, keys: &Vec<(TopicKey, TopicKey)>) {
        for text_item in self.items.iter_mut() {
            //bg!(&text_item);
            *text_item = text_item.clone().update_internal_link_optional(keys);
            //bg!(&text_item);
        }
        //bg!(&self.items);
    }
}

impl TextItem {
    pub fn new_text(text: &str) -> Self {
        TextItem::Text {
            text: text.to_string(),
        }
    }

    pub fn new_link(link: Link) -> Self {
        TextItem::Link {
            link,
        }
    }

    pub fn update_internal_link_optional(self, keys: &Vec<(TopicKey, TopicKey)>) -> Self {
        match self {
            TextItem::Link { ref link} => {
                match &link.type_ {
                    LinkType::Topic { topic_key } => {
                        for (topic_key_old, topic_key_new) in keys.iter() {
                            if topic_key.eq(&topic_key_old) {
                                // let text_item_new = Self::new_link(Link::new_topic(link.label.map(|label| label.as_str()), &topic_key_new.0, &topic_key_new.1));
                                let type_ = LinkType::Topic {
                                    topic_key: topic_key_new.clone(),
                                };
                                // let link_new = Link::new(link.label.map(|label| label.as_str()), type_);
                                let link_new = Link {
                                    label: link.label.clone(),
                                    type_,
                                };
                                let text_item_new = Self::new_link(link_new);
                                //bg!(&self, &text_item_new);
                                return text_item_new;
                            }
                        }
                        self
                    },
                    //LinkType::Section { section_key } => {
                    //    self
                    //},
                    _ => self
                }
            },
            _ => self
        }
    }
}