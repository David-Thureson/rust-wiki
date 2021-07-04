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
    Width {
        width: usize,
    },
    Height {
        height: usize,
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

