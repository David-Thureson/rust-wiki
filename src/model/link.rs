use super::*;

pub enum Link {
    Topic {
        topic: TopicRc,
        label: Option<String>,
    },
    Section {
        section: SectionRc,
        label: Option<String>,
    },
    External {
        url: String,
        label: Option<String>,
    },
    Image {
        file_name: String,
        //size:
    },
    ExternalImage {
        url: String,
        //size:
    },
}
