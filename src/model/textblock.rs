use super::*;

// pub type TextBlockRc = Rc<RefCell<TextBlock>>;

pub struct TextBlock {
    pub items: Vec<TextItem>,
}

#[derive(Clone)]
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
            match text_item {
                Link { label, type_} => {
                    match link {
                        
                    }

                },
                _ => {},
            }
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
}