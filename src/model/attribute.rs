use super::*;
use std::collections::BTreeMap;
use chrono::NaiveDate;

// pub type AttributeRc = Rc<RefCell<Attribute>>;
// pub type AttributeValueRc = Rc<RefCell<AttributeValue>>;

pub struct AttributeList {
    attribute_types: BTreeMap<String, AttributeType>,
    attribute_orders: BTreeMap<String, usize>,
    attributes_to_index: Vec<String>,
    attribute_values: BTreeMap<String, Vec<(TopicKey, String)>>,
}

// This is the overall kind of topic like Author, Domain, or Language.
#[derive(Debug)]
pub struct AttributeType {
    name: String,
    value_type: AttributeValueType,
    sequence: usize,
    values: BTreeMap<String, Vec<TopicKey>>,
}

// This is an instance of an attribute of some type in a single topic, possibly with multiple
// values.
#[derive(Debug)]
pub struct AttributeInstance {
    attribute_type_name: String,
    sequence: usize,
    values: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum AttributeValueType {
    Boolean,
    Date,
    Number,
    String,
    Unknown,
    Year,
}

impl AttributeList {
    pub fn new() -> Self {
        Self {
            attribute_types: Default::default(),
            attribute_orders: Default::default(),
            attributes_to_index: vec![],
            attribute_values: Default::default(),
        }
    }

    pub fn get_attribute_types(&self) -> &BTreeMap<String, AttributeType> {
        &self.attribute_types
    }

    pub fn get_attribute_types_mut(&mut self) -> &mut BTreeMap<String, AttributeType> {
        &mut self.attribute_types
    }

    pub fn sort_attribute_topic_lists(&mut self) {
        // In the values map, each entry is a list of pairs of topic keys and attribute type names.
        // Sort each of these lists by topic name first, then attribute type name.
        for list in self.attribute_values.values_mut() {
            list.sort_by(|a, b| a.0.get_topic_name().to_lowercase().cmp(&b.0.get_topic_name().to_lowercase()).then(a.1.cmp(&b.1)));
        }
    }

    pub fn get_attribute_type(&self, name: &str) -> Option<&AttributeType> {
        self.attribute_types.get(name)
    }

    pub fn clear_attribute_types_and_values(&mut self) {
        self.attribute_types.clear();
        self.attribute_values.clear();
    }

    pub fn clear_attribute_orders(&mut self) {
        self.attribute_orders.clear();
    }

    pub fn add_attribute_order(&mut self, type_name: String, sequence: usize) {
        self.attribute_orders.insert(type_name, sequence);
    }

    pub fn get_attribute_orders(&self) -> &BTreeMap<String, usize> {
        &self.attribute_orders
    }

    pub fn set_attributes_to_index(&mut self, attr: Vec<String>) {
        debug_assert!(self.attributes_to_index.is_empty());
        self.attributes_to_index = attr;
    }

    pub fn is_attribute_indexed(&self, name: &str) -> bool {
        self.attributes_to_index.contains(&name.to_string())
    }

    pub fn get_attribute_values(&self) -> &BTreeMap<String, Vec<(TopicKey, String)>> {
        &self.attribute_values
    }

    pub fn has_attribute_links(&self, value: &str) -> bool {
        self.attribute_values.get(value).map_or(false, |list| !list.is_empty())
    }

    pub fn add_attribute_value(&mut self, value: String, topic_key: TopicKey, attribute_type_name: String) {
        let entry = self.attribute_values.entry(value).or_insert(vec![]);
        entry.push((topic_key, attribute_type_name));
    }

    pub fn get_topics_with_attribute_value(&self, value: &str) -> Vec<(TopicKey, String)> {
        let mut topic_keys = vec![];
        if let Some(found_topic_keys) = self.attribute_values.get(value) {
            topic_keys.append(&mut found_topic_keys.clone());
        }
        topic_keys
    }

    pub fn set_attribute_types_and_values(&mut self, attribute_types: BTreeMap<String, AttributeType>, attribute_values: BTreeMap<String, Vec<(TopicKey, String)>>) {
        assert!(self.attribute_types.is_empty());
        assert!(self.attribute_values.is_empty());
        self.attribute_types = attribute_types;
        self.attribute_values = attribute_values;
    }
}

impl AttributeType {
    pub fn new(name: &str, value_type: &AttributeValueType, sequence: usize) -> Self {
        Self::assert_legal_attribute_type_name(name);
        Self {
            name: name.to_string(),
            value_type: value_type.clone(),
            sequence,
            values: Default::default(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_value_type(&self) -> &AttributeValueType {
        &self.value_type
    }

    pub fn get_values(&self) -> &BTreeMap<String, Vec<TopicKey>> {
        &self.values
    }

    pub fn get_sequence(&self) -> usize {
        self.sequence
    }

    pub fn assert_legal_attribute_type_name(name: &str) {
        if name != name.trim() {
            panic!("Attribute type name \"{}\" is not trimmed.", name);
        }
        name.chars().enumerate().for_each(|(i, c)| {
            //bg!(i, c, c.is_ascii_uppercase(), c.is_ascii_lowercase());
            if c != ' ' && ((i == 0 && !c.is_ascii_uppercase()) || (i != 0 && !c.is_ascii_alphabetic())) {
                panic!("Attribute type name \"{}\" contains invalid characters.", name);
            }
        });
    }

    pub fn is_legal_attribute_value(value: &str) -> bool {
        if value != value.trim() {
            return false;
        }
        for c in value.chars() {
            if c == '[' || c == ']' {
                return false;
            }
        }
        true
    }

    pub fn assert_legal_attribute_value(value: &str) {
        if value != value.trim() {
            panic!("Attribute value \"{}\" is not trimmed.", value);
        }
        value.chars().for_each(|c| {
            // if !c.is_ascii_alphanumeric() && c != ' ' && c != '.' && c != '!' && c != ',' && c != '\'' && c != '@' && c != '+' && c != '/' && c != ';' && c != '&' && c != ':' && c != '%' && c != '-' && c != '(' && c != ')' {
            if c == '[' || c == ']' {
                panic!("Attribute value \"{}\" contains invalid characters.", value);
            }
        });
    }

    pub fn add_value_for_topic(&mut self, value: &str, topic_key: &TopicKey) -> Result<String, String> {
        // If this attribute type does not have the value, add it. Then either way add a reference
        // to the topic, showing that this topic has this value for this attribute type.
        let canonical_value = Self::value_to_canonical_form(&self.value_type, value)?;
        let entry = self.values.entry(canonical_value.clone())
                .or_insert(vec![]);
        if !entry.contains(topic_key) {
            entry.push(topic_key.clone());
        }
        Ok(canonical_value)
    }

    pub fn add_date_value(&mut self, value: &NaiveDate, topic_key: &TopicKey) -> Result<String, String> {
        self.add_value_for_topic(&Self::date_to_canonical_value(value), topic_key)
    }

    pub fn get_canonical_value(&self, value: &str) -> Result<String, String> {
        Self::value_to_canonical_form(&self.value_type, value)
    }

    pub fn value_to_canonical_form(value_type: &AttributeValueType, value: &str) -> Result<String, String> {
        match value_type {
            AttributeValueType::Boolean => {
                let value = util::bool::string_to_bool(value)?;
                Ok(util::bool::bool_to_yes_no(value))
            }
            AttributeValueType::Date => {
                let value= util::date_time::naive_date_from_multiple_formats(value)?;
                Ok(util::date_time::naive_date_to_sortable_format(&value))
            }
            AttributeValueType::Number => {
                let value = util::number::usize_from_string(value)?;
                Ok(value.to_string())
            }
            AttributeValueType::String | AttributeValueType::Unknown => {
                Ok(value.trim().to_string())
            }
            AttributeValueType::Year => {
                let value = util::number::usize_from_string(value)?;
                if value > 2_100 {
                    Err(format!("Expected a year attribute value, but the value is \"{}\".", value))
                } else {
                    Ok(util::format::format_zeros(value, 4))
                }
            }
        }
    }

    pub fn value_to_presumed_type(attribute_type_name: &str, value: &str) -> AttributeValueType {
        if attribute_type_name.to_lowercase().contains("year") && Self::value_to_canonical_form(&AttributeValueType::Year, value).is_ok() {
            return AttributeValueType::Year;
        }
        for value_type in [AttributeValueType::Boolean, AttributeValueType::Date, AttributeValueType::Number].iter() {
            if Self::value_to_canonical_form(value_type, value).is_ok() {
                return value_type.clone();
            }
        }
        AttributeValueType::String
    }

    pub fn value_count(&self) -> usize {
        self.values.len()
    }

    pub fn get_value_display_string(&self, value: &str) -> String {
        Self::value_to_display_string(&self.value_type, value)
    }

    pub fn value_to_display_string(value_type: &AttributeValueType, value: &str) -> String {
        // In most cases the display string, which will be used on wiki pages, is the same as the
        // string stored as the value. For dates, though, the value is something like "2022-01-03"
        // so that it sorts correctly, while the display string is something like "2022-Jan-03".
        match value_type {
            AttributeValueType::Date => Self::date_to_display_string(&Self::value_to_date(value)),
            AttributeValueType::Year => value.parse::<usize>().unwrap().to_string(),
            _ => value.to_string(),
        }
    }

    pub fn date_to_display_string(value: &NaiveDate) -> String {
        // util::date_time::naive_date_to_mon_format(&value)
        util::date_time::naive_date_to_doc_format(&value)
    }

    pub fn date_to_canonical_value(value: &NaiveDate) -> String {
        util::date_time::naive_date_to_sortable_format(value)
    }

    pub fn value_to_date(value: &str) -> NaiveDate {
        util::date_time::naive_date_from_sortable_format(value).unwrap()
    }

    pub fn get_topic_count(&self) -> usize {
        self.values.values().map(|topics| topics.len()).sum()
    }

    pub fn list_attribute_types(model: &Model) {
        let mut list = vec![];
        println!("\nAttribute types:");
        for attribute_type in model.get_attribute_types().values() {
            let value_type = attribute_type.value_type.get_variant_name();
            list.push(attribute_type.name.clone());
            let value_count = attribute_type.values.len();
            let usage_count = attribute_type.values.values()
                .map(|a| a.len())
                .sum::<usize>();
            println!("{} ({}): value count = {}; usage count = {}", attribute_type.name, value_type.to_lowercase(),
                     util::format::format_count(value_count), util::format::format_count(usage_count));
        }
        println!("\n{}", list.iter().map(|x| format!("\"{}\"", x)).join(", \n"));
    }

    pub fn get_distinct_attr_values(model: &Model, value_type: &AttributeValueType) -> Vec<String> {
        let mut values = vec![];
        for attribute_type in model.get_attribute_types().values()
            .filter(|attribute_type| attribute_type.value_type.eq(value_type)) {
            for value in attribute_type.values.keys() {
                values.push(value.clone());
            }
        }
        values.sort();
        values.dedup();
        values
    }

    pub fn get_topics_for_attr_value(model: &Model, value_type: &AttributeValueType, match_value: &str, included_attr_names: Option<Vec<&str>>) -> Vec<TopicKey> {
        let mut topic_keys = vec![];
        for attribute_type in model.get_attribute_types().values()
            .filter(|attribute_type| attribute_type.value_type.eq(value_type))
            .filter(|attribute_type| included_attr_names.as_ref().map_or(true, |included| included.contains(&&*attribute_type.name))) {
            for (_found_value, found_topic_keys) in attribute_type.values.iter()
                .filter(|(found_value, _found_topic_keys)| found_value.as_str() == match_value) {
                for found_topic_key in found_topic_keys.iter() {
                    topic_keys.push(found_topic_key.clone());
                }
            }
        }
        topic_keys.sort();
        topic_keys.dedup();
        TopicKey::sort_topic_keys_by_name(&mut topic_keys);
        topic_keys
    }

    // Create a list of pairs of the attribute type name and the topic key.
    pub fn get_typed_topics_for_attr_value(model: &Model, value_type: &AttributeValueType, match_value: &str, included_attr_names: Option<Vec<&str>>) -> Vec<(String, TopicKey)> {
        let mut list = vec![];
        for attribute_type in model.get_attribute_types().values()
            .filter(|attribute_type| attribute_type.value_type.eq(value_type))
            .filter(|attribute_type| included_attr_names.as_ref().map_or(true, |included| included.contains(&&*attribute_type.name))) {
            for (_found_value, found_topic_keys) in attribute_type.values.iter()
                .filter(|(found_value, _found_topic_keys)| found_value.as_str() == match_value) {
                for found_topic_key in found_topic_keys.iter() {
                    list.push((attribute_type.name.clone(), found_topic_key.clone()));
                }
            }
        }
        list.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
        list
    }

    pub fn fill_attribute_orders(model: &mut Model) {
        model.clear_attribute_orders();
        for (i, type_name) in ATTRIBUTE_ORDER.iter().enumerate() {
            model.add_attribute_order(type_name.to_string(), i);
        }
    }
}

impl AttributeInstance {

    pub fn new(attribute_type_name: &str, sequence: usize, values: Vec<String>) -> Self {
        Self {
            attribute_type_name: attribute_type_name.to_string(),
            sequence,
            values,
        }
    }

    pub fn get_attribute_type_name(&self) -> &str {
        &self.attribute_type_name
    }

    pub fn get_sequence(&self) -> usize {
        self.sequence
    }

    pub fn get_values(&self) -> &Vec<String> {
        &self.values
    }

}

impl AttributeValueType {

    pub fn get_variant_name(&self) -> &str {
        match self {
            Self::Boolean => "Boolean",
            Self::Date => "Date",
            Self::Number => "Number",
            Self::String => "String",
            Self::Unknown => "Unknown",
            Self::Year => "Year",
        }
    }
}

impl PartialEq for AttributeValueType {
    fn eq(&self, other: &Self) -> bool {
        self.get_variant_name() == other.get_variant_name()
    }
}

impl Eq for AttributeValueType {}

/*
impl AttributeValueList {
    pub fn new() -> Self {
        Self {
            list: Default::default(),
        }
    }
}
*/

/*
impl AttributeValue {
    pub fn topic_count(&self) -> usize {
        self.topics.len()
    }

    pub fn get_topics(&self) -> Vec<TopicRc> {
        self.topics.values().map(|topic_rc| topic_rc.clone()).collect()
    }
}
*/

