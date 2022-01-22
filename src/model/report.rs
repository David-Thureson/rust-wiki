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

    pub fn go(&self, wiki: &Model) {
        let namespace_count = wiki.get_namespaces().len();
        let topic_count = wiki.get_topics().len();
        let category_count = wiki.get_categories().len();
        let attribute_count = wiki.get_attribute_list().get_attributes().len();
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

    fn category_breakdown(&self, wiki: &Model, depth: usize) {
        let mut groups = util::group::Grouper::new("Categories");
        for topic in wiki.get_topics().values() {
            if let Some(category) = &topic.get_category() {
                groups.record_entry(&category);
            }
        }
        groups.print_by_count(depth, Some(5));
    }

    fn paragraph_breakdown(&self, wiki: &Model, depth: usize) {
        let mut groups = util::group::Grouper::new("Paragraph Types");
        for topic in wiki.get_topics().values() {
            for paragraph in topic.get_paragraphs().iter() {
                groups.record_entry(&paragraph.get_variant_name().to_string());
            }
        }
        groups.print_by_count(depth, None);
    }

    fn attribute_breakdown(&self, wiki: &Model, depth: usize) {
        let mut groups = util::group::Grouper::new("Attributes");
        for topic in wiki.get_topics().values() {
            for (name, values) in topic.get_temp_attributes().iter() {
                AttributeType::assert_legal_attribute_type_name(&name);
                groups.record_entry_with_count(name, values.len());
            }
        }
        groups.print_by_count(depth, Some(5));
    }

    fn list_breakdown(&self, wiki: &Model, depth: usize) {
        let mut groups = util::group::Grouper::new("List Types");
        for topic in wiki.get_topics().values() {
            for paragraph in topic.get_paragraphs().iter() {
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

pub fn report_attributes(wiki: &Model) {
    for attribute_type in wiki.get_attributes().values() {
        println!("\n{}: {} ({})", attribute_type.get_name(), attribute_type.get_value_type().get_variant_name(), attribute_type.get_topic_count());
        for (value, topics) in attribute_type.get_values().iter() {
            let display_value = attribute_type.get_value_display_string(value);
            let topic_list = topics.iter().map(|topic_key| format!("\"{}\"", topic_key.get_topic_name())).join(", ");
            println!("\t{} ({}) in {}", display_value, topics.len(), topic_list);
        }
    }
    // Code generation:
    // Print the names of the string attribute types as a list of quoted strings.
    // println!("{}", wiki.attributes.iter().filter(|(_name, type_)| type_.value_type == AttributeValueType::String).map(|(name, _type_)| format!("\"{}\"", name)).join(", "));
    // Print the names of the date attribute types as a list of quoted strings.
    println!("{}", wiki.get_attributes().iter().filter(|(_name, type_)| AttributeValueType::Date.eq(type_.get_value_type())).map(|(name, _type_)| format!("\"{}\"", name)).join(", "));
    // Print the names of the year attribute types as a list of quoted strings.
    // println!("{}", wiki.attributes.iter().filter(|(_name, type_)| type_.value_type == AttributeValueType::Year).map(|(name, _type_)| format!("\"{}\"", name)).join(", "));
}

pub fn report_attributes_with_multiple_values(wiki: &Model) {
    // For round-trip parsing/generating of the wiki, we need conventions for handling multiple
    // attributes separated by commas, since commas can also appear inside a value. First simply do
    // a survey of attributes that appear to have multiple values within a given topic but might
    // really be a case where a value such as a book title has commas that we misinterpreted.
    println!("\nReport: Attributes with Multiple Values:\n");
    for attr_type in wiki.get_attributes().values()
            .filter(|attr_type| attr_type.get_value_type().eq(&AttributeValueType::String)) {
        let mult_value_count = wiki.get_topics().values()
            .map(|topic| {
                let topic_value_count = topic.get_attributes().get(&attr_type.get_name()).map_or(0, |attr_instance| attr_instance.get_values().len());
                if topic_value_count > 0 { 1 } else { 0 }
            })
            .sum::<usize>();
        if mult_value_count > 0 {
            println!("{}: {} ({}): topics with multiple values = {}",
                     attr_type.get_name(), attr_type.get_value_type().get_variant_name(),
                     attr_type.get_topic_count(), mult_value_count);
        }
    }
    println!();
}

// pub fn report_attribute_values_with_commas(wiki: &Wiki) {
    // Based on the results of report_attributes_with_multiple_values(), show the attribute
    // values that have one or more commas. This is not a prablem with the initial parse of the
    // ConnectedText wikis because each value is contained in markup like this:
    //   [[Title:=Soldier, Sailor, Frogman, Spy, Airman, Gangster, Kill or Die: How the Allies Won on D-Day]]
    // However, when we start doing round trips with Dokuwiki, we'll need a way to
/*
Address: String (1): topics with multiple values = 1
Book: String (1): topics with multiple values = 1
Company: String (5): topics with multiple values = 5
Course: String (1): topics with multiple values = 1
Organization: String (16): topics with multiple values = 16
School: String (18): topics with multiple values = 18
Series: String (70): topics with multiple values = 70
Title: String (295): topics with multiple values = 295
*/
//}

