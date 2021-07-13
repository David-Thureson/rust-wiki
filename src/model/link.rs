// https://www.dokuwiki.org/images

use super::*;

#[derive(Clone)]
pub struct Link {
    pub label: Option<String>,
    pub type_: LinkType,
}

#[derive(Clone)]
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

#[derive(Clone)]
pub enum ImageSource {
    Internal {
        namespace: String,
        file_name: String,
    },
    External {
        url: String,
    }
}

#[derive(Clone)]
pub enum ImageAlignment {
    Center,
    Left,
    Right,
}

#[derive(Clone)]
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

#[derive(Clone)]
pub enum ImageLinkType {
    // These are all based on Dokuwiki's terms and behavior: https://www.dokuwiki.org/images
    Detail, // Link to a detail page showing metadata for the image.
    Direct, // Link to the full-size image.
    LinkOnly, // Don't show the image, just the link.
    NoLink, // Clicking doesn't lead anywhere.
}

impl Link {
    fn new(label: Option<&str>, type_: LinkType) -> Self {
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
        let section_key = Topic::make_section_key(namespace_name, topic_name, section_name);
        let type_ = LinkType::Section {
            section_key,
        };
        Self::new(label, type_)
    }

    pub fn new_topic(label: Option<&str>, namespace_name: &str, topic_name: &str) -> Self {
        let topic_key = Topic::make_key(namespace_name, topic_name);
        let type_ = LinkType::Topic {
            topic_key,
        };
        Self::new(label, type_)
    }

}

impl ImageSource {
    pub fn new_internal(namespace: &str, file_name: &str) -> Self {
        Self::Internal {
            namespace: namespace.to_string(),
            file_name: file_name.to_string(),
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
