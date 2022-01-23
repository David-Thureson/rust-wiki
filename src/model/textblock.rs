use super::*;

// pub type TextBlockRc = Rc<RefCell<TextBlock>>;

#[derive(Clone, Debug)]
pub enum TextBlock {
    Resolved {
        items: Vec<TextItem>,
    },
    Unresolved {
        text: String,
    },
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
    pub fn new_unresolved(text: &str) -> Self {
        Self::Unresolved {
            text: text.to_string(),
        }
    }

    pub fn new_resolved(items: Vec<TextItem>) -> Self {
        Self::Resolved {
            items,
        }
    }

    pub fn get_unresolved_text(&self) -> String {
        match self {
            Self::Resolved { .. } => panic!("Expected an unresolved text block."),
            Self::Unresolved { text } => text.clone(),
        }
    }

    pub fn get_resolved_items(&self) -> &Vec<TextItem> {
        match self {
            Self::Resolved { items } => items,
            Self::Unresolved { .. } => panic!("Expected a resolved text block."),
        }
    }

    pub fn update_internal_links(&mut self, keys: &Vec<(TopicKey, TopicKey)>) {
        match self {
            Self::Resolved { items } => {
                for text_item in items.iter_mut() {
                    //bg!(&text_item);
                    *text_item = text_item.clone().update_internal_link_optional(keys);
                    //bg!(&text_item);
                }
                //bg!(&items);
            }
            Self::Unresolved { .. } => {
                panic!("This shouldn't be called for unresolved text blocks.")
            },
        }
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
                match &link.get_type() {
                    LinkType::Topic { topic_key } => {
                        for (topic_key_old, topic_key_new) in keys.iter() {
                            if topic_key.eq(&topic_key_old) {
                                // let text_item_new = Self::new_link(Link::new_topic(link.label.map(|label| label.as_str()), &topic_key_new.0, &topic_key_new.1));
                                let link_new = Link::new_topic_string_label(link.get_label().clone(), topic_key_new.get_namespace(), topic_key_new.get_topic_name());
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