use crate::*;
use super::*;

pub struct WikiReport {
    wiki: WikiRc,
    paragraphs: bool,
}

impl WikiReport {
    pub fn new(wiki_rc: &WikiRc) -> Self {
        Self {
            wiki: wiki_rc.clone(),
            paragraphs: false,
        }
    }

    pub fn paragraphs(mut self) -> Self {
        self.paragraphs = true;
        self
    }

    pub fn go(&self) {
        let wiki = b!(&self.wiki);
        let namespace_count = wiki.topics.len();
        let topic_count = wiki.topics.len();
        let category_count = wiki.categories.len();
        let attribute_count = wiki.attributes.len();
        println!("namespaces = {}, topics = {}, categories = {}, attributes = {}",
            namespace_count, topic_count, category_count, attribute_count);
        let child_depth = 1;
        if self.paragraphs {
            self.paragraph_breakdown(child_depth);
        }
    }

    fn paragraph_breakdown(&self, depth: usize) {
        let wiki = b!(&self.wiki);
        let mut groups = util::group::Grouper::new("Paragraph Types");
        wiki.get_paragraphs().iter().for_each(|paragraph_rc| {
            groups.record_entry(&b!(paragraph_rc).get_variant_name().to_string());
        });
        groups.print_by_count(depth, None);
    }
}

