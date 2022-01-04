use super::*;

pub struct WikiReport {
    categories: bool,
    paragraphs: bool,
    attributes: bool,
    lists: bool,
}

impl WikiReport {
    pub fn new() -> Self {
        Self {
            categories: false,
            paragraphs: false,
            attributes: false,
            lists: false,
        }
    }

    pub fn categories(mut self) -> Self {
        self.categories = true;
        self
    }

    pub fn paragraphs(mut self) -> Self {
        self.paragraphs = true;
        self
    }

    pub fn attributes(mut self) -> Self {
        self.attributes = true;
        self
    }

    pub fn lists(mut self) -> Self {
        self.lists = true;
        self
    }

    pub fn go(&self, wiki: &Wiki) {
        let namespace_count = wiki.namespaces.len();
        let topic_count = wiki.topics.len();
        let category_count = wiki.categories.len();
        let attribute_count = wiki.attributes.len();
        println!("namespaces = {}, topics = {}, categories = {}, attributes = {}",
            namespace_count, topic_count, category_count, attribute_count);
        let child_depth = 1;
        if self.categories {
            self.category_breakdown(wiki, child_depth);
        }
        if self.paragraphs {
            self.paragraph_breakdown(wiki, child_depth);
        }
        if self.attributes {
            self.attribute_breakdown(wiki, child_depth);
        }
        if self.lists {
            self.list_breakdown(wiki, child_depth);
        }
    }

    fn category_breakdown(&self, wiki: &Wiki, depth: usize) {
        let mut groups = util::group::Grouper::new("Categories");
        for topic in wiki.topics.values() {
            if let Some(category) = &topic.category {
                groups.record_entry(&category);
            }
        }
        groups.print_by_count(depth, Some(5));
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

    fn attribute_breakdown(&self, wiki: &Wiki, depth: usize) {
        let mut groups = util::group::Grouper::new("Attributes");
        for topic in wiki.topics.values() {
            for (name, values) in topic.temp_attributes.iter() {
                groups.record_entry_with_count(name, values.len());
            }
        }
        groups.print_by_count(depth, Some(5));
    }

    fn list_breakdown(&self, wiki: &Wiki, depth: usize) {
        let mut groups = util::group::Grouper::new("List Types");
        for topic in wiki.topics.values() {
            for paragraph in topic.paragraphs.iter() {
                match paragraph {
                    Paragraph::List { type_, .. } => {
                        groups.record_entry(&type_.get_variant_name().to_string())
                    },
                    _ => {},
                }
            }
        }
        groups.print_by_count(depth, Some(5));
    }

}

pub fn report_attributes(wiki: &Wiki) {
    for attribute_type in wiki.attributes.values() {
        println!("\n{}: {} ({})", attribute_type.name, attribute_type.value_type.get_variant_name(), attribute_type.get_topic_count());
        for (value, topics) in attribute_type.values.iter() {
            let display_value = attribute_type.get_value_display_string(value);
            let topic_list = topics.iter().map(|topic_key| format!("\"{}\"", topic_key.topic_name)).join(", ");
            println!("\t{} ({}) in {}", display_value, topics.len(), topic_list);
        }
    }
    // Code generation:
    // Print the names of the string attribute types as a list of quoted strings.
    // println!("{}", wiki.attributes.iter().filter(|(_name, type_)| type_.value_type == AttributeValueType::String).map(|(name, _type_)| format!("\"{}\"", name)).join(", "));
    // Print the names of the date attribute types as a list of quoted strings.
    println!("{}", wiki.attributes.iter().filter(|(_name, type_)| type_.value_type == AttributeValueType::Date).map(|(name, _type_)| format!("\"{}\"", name)).join(", "));
    // Print the names of the year attribute types as a list of quoted strings.
    // println!("{}", wiki.attributes.iter().filter(|(_name, type_)| type_.value_type == AttributeValueType::Year).map(|(name, _type_)| format!("\"{}\"", name)).join(", "));
}

