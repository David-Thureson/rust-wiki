// https://www.dokuwiki.org/images

use super::*;

pub struct Link {
    label: Option<String>,
    type_: LinkType,
}

pub enum LinkType {
    Topic {
        topic: TopicRc,
    },
    Section {
        section: SectionRc,
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

pub enum ImageSource {
    Internal {
        namespace: NamespaceRc,
        file_name: String,
    },
    External {
        url: String,
    }
}

pub enum ImageAlignment {
    Center,
    Left,
    Right,
}

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

pub enum ImageLinkType {
    Detail,
    Direct,
    LinkOnly,
    NoLink,
}

impl Link {
    pub fn new_internal_unresolved(label: Option<&str>, dest: &str) -> Self {
        let type_ = LinkType::InternalUnresolved {
            dest: dest.to_string()
        };
        Self {
            label: label.map(|label| label.to_string()),
            type_,
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
