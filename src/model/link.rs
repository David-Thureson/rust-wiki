// https://www.dokuwiki.org/images

use super::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Link {
    label: Option<String>,
    type_: LinkType,
}

#[derive(Clone, Debug)]
pub enum LinkType {
    Topic {
        topic_key: TopicKey,
    },
    Section {
        section_key: SectionKey,
    },
    External {
        url: String,
    },
    Image {
        source: ImageSource,
        alignment: ImageAlignment,
        size: ImageSize,
        type_: ImageLinkType,
    },
    InternalUnresolved {
        dest: String,
    }
}

#[derive(Clone, Debug)]
pub enum ImageSource {
    Internal {
        namespace: String,
        file_name: String,
    },
    External {
        url: String,
    }
}

#[derive(Clone, Debug)]
pub enum ImageAlignment {
    Center,
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub enum ImageSize {
    DokuSmall,
    DokuMedium,
    DokuLarge,
    Height {
        height: usize,
    },
    Original,
    Width {
        width: usize,
    },
    WidthHeight {
        width: usize,
        height: usize,
    },
}

#[derive(Clone, Debug)]
pub enum ImageLinkType {
    // These are all based on Dokuwiki's terms and behavior: https://www.dokuwiki.org/images
    Detail, // Link to a detail page showing metadata for the image.
    Direct, // Link to the full-size image.
    LinkOnly, // Don't show the image, just the link.
    NoLink, // Clicking doesn't lead anywhere.
}

impl Link {
    pub fn new(label: Option<&str>, type_: LinkType) -> Self {
        Self {
            label: label.map(|label| label.to_string()),
            type_,
        }
    }

    pub fn new_image(label: Option<&str>, source: ImageSource, alignment: ImageAlignment, size: ImageSize, type_: ImageLinkType) -> Self {
        let type_ = LinkType::Image {
                source,
                alignment,
                size,
                type_,
            };
        Self::new(label, type_)
    }

    pub fn new_external(label: Option<&str>, url: &str) -> Self {
        let type_ = LinkType::External {
            url: url.to_string(),
        };
        Self::new(label, type_)
    }

    pub fn new_internal_unresolved(label: Option<&str>, dest: &str) -> Self {
        let type_ = LinkType::InternalUnresolved {
            dest: dest.to_string()
        };
        Self::new(label, type_)
    }

    pub fn new_section(label: Option<&str>, namespace_name: &str, topic_name: &str, section_name: &str) -> Self {
        TopicKey::assert_legal_namespace(namespace_name);
        TopicKey::assert_legal_topic_name(topic_name);
        let section_key = SectionKey::new(namespace_name, topic_name, section_name);
        let type_ = LinkType::Section {
            section_key,
        };
        Self::new(label, type_)
    }

    pub fn new_section_internal(label: Option<&str>, namespace_name: &str, topic_name: &str, section_name: &str) -> Self {
        TopicKey::assert_legal_namespace(namespace_name);
        TopicKey::assert_legal_topic_name(topic_name);
        let section_key = SectionKey::new(namespace_name, topic_name, section_name);
        let type_ = LinkType::Section {
            section_key,
        };
        Self::new(label, type_)
    }

    pub fn new_topic(label: Option<&str>, namespace_name: &str, topic_name: &str) -> Self {
        TopicKey::assert_legal_namespace(namespace_name);
        TopicKey::assert_legal_topic_name(topic_name);
        let topic_key = TopicKey::new(namespace_name, topic_name);
        let type_ = LinkType::Topic {
            topic_key,
        };
        Self::new(label, type_)
    }

    pub fn get_label(&self) -> &Option<&str> {
        &self.label.map(|label| label.as_str())
    }

    pub fn get_type(&self) -> &LinkType {
        &self.type_
    }

    pub fn catalog_links(model: &mut Model) {
        for topic in model.get_topics_mut().values_mut() {
            topic.clear_outbound_links();
            topic.clear_inbound_topic_keys();
            topic.clear_listed_topics();
            topic.clear_subtopics();
            topic.clear_combo_subtopics();
        }
        for topic in model.get_topics_mut().values_mut() {
            for paragraph in topic.get_paragraphs().iter() {
                match paragraph {
                    Paragraph::List { type_, header, items } => {
                        let (is_combos, is_subtopics) = match type_ {
                            ListType::Combinations => (true, false),
                            ListType::Subtopics => (false, true),
                            _ => (false, false),
                        };
                        topic.add_outbound_links(Self::catalog_links_text_block(header));
                        for list_item in items.iter() {
                            if list_item.get_depth() == 1 {
                                let mut links = Self::catalog_links_text_block(&list_item.get_text_block());
                                for link in links.iter() {
                                    match &link.type_ {
                                        LinkType::Topic { topic_key } => {
                                            topic.add_listed_topic_optional(topic_key);
                                            if is_combos {
                                                topic.add_combo_subtopic(topic_key.clone());
                                            } else if is_subtopics {
                                                topic.add_subtopic(topic_key.clone());
                                            }
                                            break;
                                        },
                                        _ => {},
                                    }
                                }
                                topic.add_outbound_links(links);
                            }
                        }
                    },
                    Paragraph::Text { text_block} => {
                        topic.add_outbound_links(Self::catalog_links_text_block(text_block));
                    },
                    _ => {},
                }
            }
        }

        // Set inbound links.
        let mut map = BTreeMap::new();
        for topic in model.get_topics().values() {
            let topic_key = topic.get_key();
            for link in topic.get_outbound_links().iter() {
                let outbound_topic_key = match &link.type_ {
                    LinkType::Topic { topic_key } => Some(topic_key.clone()),
                    LinkType::Section { section_key } => Some(section_key.get_topic_key().clone()),
                    _ => None,
                };
                if let Some(outbound_topic_key) = outbound_topic_key {
                    let entry = map.entry(outbound_topic_key.clone()).or_insert(vec![]);
                    if !entry.contains(&topic_key) {
                        entry.push(topic_key.clone());
                    }
                }
            }
        }
        for (topic_key, mut inbound_topic_keys) in map.drain_filter(|_k, _v| true) {
            if let Some(topic) = model.get_topic_mut(&topic_key) {
                topic.add_inbound_topic_keys(inbound_topic_keys);
            }
        }

        // Sort all of the vectors of TopicKeys.
        for topic in model.get_topics().values_mut() {
            topic.sort_topic_key_lists();
        }
    }

    fn catalog_links_text_block(text_block: &TextBlock) -> Vec<Link> {
        match text_block {
            TextBlock::Resolved { items } => {
                let mut links = vec![];
                for item in items.iter() {
                    match item {
                        TextItem::Link { link } => {
                            links.push(link.clone());
                        },
                        _ => {},
                    }
                }
                links
            },
            _ => panic!("Expected a resolved text block."),
        }
    }

}

impl ImageSource {
    pub fn new_internal(namespace: &str, file_name: &str) -> Self {
        TopicKey::assert_legal_namespace(namespace);
        Self::Internal {
            namespace: namespace.to_string(),
            file_name: file_name.to_string(),
        }
    }

    pub fn new_external(url: &str) -> Self {
        Self::External {
            url: url.to_string(),
        }
    }
}

impl ImageSize {
    pub fn get_name(&self) -> String {
        match self {
            ImageSize::DokuSmall => "Doku small (200)".to_string(),
            ImageSize::DokuMedium => "Doku medium (400)".to_string(),
            ImageSize::DokuLarge => "Doku large (600)".to_string(),
            ImageSize::Height { height } => format!("height = {}", height),
            ImageSize::Original => "original".to_string(),
            ImageSize::Width { width } => format!("width = {}", width),
            ImageSize::WidthHeight { width, height } => format!("width = {}; height = {}", width, height),
        }
    }
}
