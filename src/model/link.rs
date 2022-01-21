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
        for topic in model.topics.values_mut() {
            topic.outbound_links.clear();
            topic.inbound_topic_keys.clear();
            topic.listed_topics.clear();
            topic.subtopics.clear();
            topic.combo_subtopics.clear();
        }
        for topic in model.topics.values_mut() {
            for paragraph in topic.paragraphs.iter() {
                match paragraph {
                    Paragraph::List { type_, header, items } => {
                        let (is_combos, is_subtopics) = match type_ {
                            ListType::Combinations => (true, false),
                            ListType::Subtopics => (false, true),
                            _ => (false, false),
                        };
                        topic.outbound_links.append(&mut Self::catalog_links_text_block(header));
                        for list_item in items.iter() {
                            if list_item.depth == 1 {
                                let mut links = Self::catalog_links_text_block(&list_item.block);
                                for link in links.iter() {
                                    match &link.type_ {
                                        LinkType::Topic { topic_key } => {
                                            if !topic.listed_topics.contains(&topic_key) {
                                                topic.listed_topics.push(topic_key.clone());
                                            }
                                            if is_combos {
                                                topic.combo_subtopics.push(topic_key.clone());
                                            } else if is_subtopics {
                                                topic.subtopics.push(topic_key.clone());
                                            }
                                            break;
                                        },
                                        _ => {},
                                    }
                                }
                                topic.outbound_links.append(&mut links);
                            }
                        }
                    },
                    Paragraph::Text { text_block} => {
                        topic.outbound_links.append(&mut Self::catalog_links_text_block(text_block));
                    },
                    _ => {},
                }
            }
        }

        // Set inbound links.
        let mut map = BTreeMap::new();
        for topic in model.topics.values() {
            let topic_key = topic.get_key();
            for link in topic.outbound_links.iter() {
                let outbound_topic_key = match &link.type_ {
                    LinkType::Topic { topic_key } => Some(topic_key.clone()),
                    LinkType::Section { section_key } => Some(section_key.topic_key.clone()),
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
            if let Some(topic) = model.topics.get_mut(&topic_key) {
                topic.inbound_topic_keys.append(&mut inbound_topic_keys);
            }
        }

        // Sort all of the vectors of TopicKeys.
        for topic in model.topics.values_mut() {
            // topic.outbound_links.sort();
            TopicKey::sort_topic_keys_by_name(&mut topic.inbound_topic_keys);
            TopicKey::sort_topic_keys_by_name(&mut topic.subtopics);
            TopicKey::sort_topic_keys_by_name(&mut topic.combo_subtopics);
            TopicKey::sort_topic_keys_by_name(&mut topic.listed_topics);
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

    pub fn check_links(model: &Model) -> TopicErrorList {
        //bg!(model.topics.keys());
        let mut errors = TopicErrorList::new();
        for topic in model.topics.values() {
            let this_topic_key = topic.get_key();
            for link in topic.outbound_links.iter() {
                match &link.type_ {
                    LinkType::Topic { topic_key } => {
                        model.check_topic_link(&mut errors, "outbound_links", &this_topic_key, topic_key);
                    },
                    LinkType::Section { section_key } => {
                        //bg!(&section_key);
                        if !model.has_section(section_key) {
                            errors.add(&topic.get_key(), &format!("wiki::check_links(): Section link {} not found.", section_key));
                        }
                    },
                    _ => {},
                }
            }
            topic.parents.iter().for_each(|ref_topic_key| { model.check_topic_link(&mut errors, "parents", &this_topic_key, ref_topic_key); } );
            topic.inbound_topic_keys.iter().for_each(|ref_topic_key| { model.check_topic_link(&mut errors, "inbound_topic_keys", &this_topic_key, ref_topic_key); } );
            topic.subtopics.iter().for_each(|ref_topic_key| { model.check_topic_link(&mut errors, "subtopics", &this_topic_key, ref_topic_key); } );
            topic.combo_subtopics.iter().for_each(|ref_topic_key| { model.check_topic_link(&mut errors, "combo_subtopics", &this_topic_key, ref_topic_key); } );
            topic.listed_topics.iter().for_each(|ref_topic_key| { model.check_topic_link(&mut errors, "listed_topics", &this_topic_key, ref_topic_key); } );
        }
        errors
    }

    pub fn update_internal_links(model: &mut Model, keys: &Vec<(TopicKey, TopicKey)>) {
        //bg!(&keys);
        // For each entry in keys, the first TopicKey is the old value and the second is the new
        // value.
        for topic in model.topics.values_mut() {
            for paragraph in topic.paragraphs.iter_mut() {
                match paragraph {
                    Paragraph::List { type_: _, header, items} => {
                        header.update_internal_links(keys);
                        for item in items.iter_mut() {
                            item.block.update_internal_links(keys);
                        }
                    },
                    Paragraph::Table { table} => {
                        for row in table.rows.iter_mut() {
                            for cell in row.iter_mut() {
                                cell.text_block.update_internal_links(keys);
                            }
                        }
                    },
                    Paragraph::Text { text_block} => {
                        text_block.update_internal_links(keys);
                    },
                    _ => {},
                }
            }
            if !topic.parents.is_empty() {
                let old_parents = topic.parents.clone();
                topic.parents.clear();
                for parent_topic_key in old_parents.iter() {
                    let mut new_parent_topic_key = parent_topic_key.clone();
                    for (topic_key_old, topic_key_new) in keys.iter() {
                        if parent_topic_key.eq(&topic_key_old) {
                            new_parent_topic_key = topic_key_new.clone();
                            break;
                        }
                    }
                    topic.parents.push(new_parent_topic_key);
                }
            }
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
