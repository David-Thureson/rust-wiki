use super::*;

// pub(crate) type TextBlockRc = Rc<RefCell<TextBlock>>;

#[derive(Clone, Debug)]
pub(crate) enum TextBlock {
    Resolved {
        items: Vec<TextItem>,
    },
    Unresolved {
        text: String,
    },
}

#[derive(Clone, Debug)]
pub(crate) enum TextItem {
    Text {
        text: String,
    },
    Link {
        link: LinkRc,
    },
}

impl TextBlock {
    pub(crate) fn new_unresolved(text: &str) -> Self {
        assert!(!text.starts_with('\n'), "TextBlock text starts with linefeed: \"{}\".", text);
        assert!(!text.ends_with('\n'), "TextBlock text ends with linefeed: \"{}\".", text);
        Self::Unresolved {
            text: text.to_string(),
        }
    }

    pub(crate) fn new_resolved(items: Vec<TextItem>) -> Self {
        Self::Resolved {
            items,
        }
    }

    pub(crate) fn get_unresolved_text(&self) -> String {
        match self {
            Self::Resolved { .. } => panic!("Expected an unresolved text block."),
            Self::Unresolved { text } => text.clone(),
        }
    }

    pub(crate) fn get_resolved_items(&self) -> &Vec<TextItem> {
        match self {
            Self::Resolved { items } => items,
            Self::Unresolved { .. } => panic!("Expected a resolved text block."),
        }
    }

   pub(crate) fn get_links(&self) -> Vec<LinkRc> {
        let mut links = vec![];
        match self {
            TextBlock::Resolved { items } => {
                for text_item in items.iter() {
                    match text_item {
                        TextItem::Link { link } => {
                            links.push(link.clone());
                        },
                        TextItem::Text { .. } => {},
                    }
                }
            },
            TextBlock::Unresolved { .. } => {},
        }
        links
    }

    pub(crate) fn get_display_text(&self) -> String {
        match self {
            TextBlock::Resolved { items } => items.iter().map(|text_item| text_item.get_display_text()).join(""),
            TextBlock::Unresolved { text } => text.clone(),
        }
    }


        /*
        pub(crate) fn update_internal_links(&mut self, keys: &Vec<(TopicKey, TopicKey)>) {
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
        */
}

impl TextItem {
    pub(crate) fn new_text(text: &str) -> Self {
        assert!(!text.starts_with('\n'), "TextItem text starts with linefeed: \"{}\".", text);
        assert!(!text.ends_with('\n'), "TextItem text ends with linefeed: \"{}\".", text);
        TextItem::Text {
            text: text.to_string(),
        }
    }

    pub(crate) fn new_link(link: Link) -> Self {
        TextItem::Link {
            link: r!(link),
        }
    }

    pub(crate) fn get_display_text(&self) -> String {
        match self {
            TextItem::Link { link } => b!(link).get_display_text(),
            TextItem::Text { text } => text.to_string(),
        }
    }

    /*
    pub(crate) fn update_internal_link_optional(self, keys: &Vec<(TopicKey, TopicKey)>) -> Self {
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
    */
}