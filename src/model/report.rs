use super::*;

pub struct WikiReport {
    paragraphs: bool,
}

impl WikiReport {
    pub fn new() -> Self {
        Self {
            paragraphs: false,
        }
    }

    pub fn paragraphs(mut self) -> Self {
        self.paragraphs = true;
        self
    }

    pub fn go(&self, wiki: &Wiki) {
        let namespace_count = wiki.topics.len();
        let topic_count = wiki.topics.len();
        let category_count = wiki.categories.len();
        let attribute_count = wiki.attributes.len();
        println!("namespaces = {}, topics = {}, categories = {}, attributes = {}",
            namespace_count, topic_count, category_count, attribute_count);
        let child_depth = 1;
        if self.paragraphs {
            self.paragraph_breakdown(wiki, child_depth);
        }
    }

    fn paragraph_breakdown(&self, wiki: &Wiki, depth: usize) {
        let mut groups = util::group::Grouper::new("Paragraph Types");
        for topic in wiki.topics.values() {
            for paragraph in topic.paragraphs.iter() {
                groups.record_entry(&paragraph.get_variant_name().to_string());
            }
        }
        groups.print_by_count(depth, None);
    }
}

