use super::*;
use std::collections::BTreeMap;
use chrono::NaiveDate;

// pub type AttributeRc = Rc<RefCell<Attribute>>;
// pub type AttributeValueRc = Rc<RefCell<AttributeValue>>;

pub struct AttributeList {
    pub attributes: BTreeMap<String, AttributeType>,
    pub attribute_orders: BTreeMap<String, usize>,
    pub attributes_to_index: Vec<String>,
    pub values: BTreeMap<String, Vec<(TopicKey, String)>>,
}

// This is the overall kind of topic like Author, Domain, or Language.
#[derive(Debug)]
pub struct AttributeType {
    pub name: String,
    pub value_type: AttributeValueType,
    pub sequence: usize,
    pub values: BTreeMap<String, Vec<TopicKey>>,
}

// This is an instance of an attribute of some type in a single topic, possibly with multiple
// values.
#[derive(Debug)]
pub struct AttributeInstance {
    pub attribute_type_name: String,
    pub sequence: usize,
    pub values: Vec<String>,
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
            attributes: Default::default(),
            attribute_orders: Default::default(),
            attributes_to_index: vec![],
            values: Default::default(),
        }
    }

    pub fn has_attribute_links(&self, value: &str) -> bool {
        self.values.get(value).map_or(false, |list| !list.is_empty())
    }
}

impl AttributeType {
    pub fn new(name: &str, value_type: &AttributeValueType, sequence: usize) -> Self {
        Self {
            name: name.to_string(),
            value_type: value_type.clone(),
            sequence,
            values: Default::default(),
        }
    }

    pub fn add_value(&mut self, value: &str, topic_key: &TopicKey) -> Result<String, String> {
        let canonical_value = Self::value_to_canonical_form(&self.value_type, value)?;
        let entry = self.values.entry(canonical_value.clone())
                .or_insert(vec![]);
        if !entry.contains(topic_key) {
            entry.push(topic_key.clone());
        }
        Ok(canonical_value)
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
                let value= util::date_time::naive_date_from_sortable_format(value)
                    .unwrap_or(util::date_time::naive_date_from_compact_format(value)?);
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
        util::date_time::naive_date_to_mon_format(&value)
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

    pub fn catalog_attributes(model: &mut Wiki) -> TopicErrorList {
        let mut errors = TopicErrorList::new();
        Self::fill_attribute_orders(model);
        model.attributes.attributes.clear();
        model.attributes.values.clear();
        for topic in model.topics.values_mut() {
            topic.attributes.clear();
            for (temp_attr_name, temp_attr_values) in topic.temp_attributes.iter()
                    .filter(|(_name, values)| !values.is_empty()) {
                let attribute_type = model.attributes.attributes.entry(temp_attr_name.clone())
                    .or_insert({
                        let value_type = AttributeType::value_to_presumed_type(temp_attr_name,&*temp_attr_values[0]);
                        let sequence = model.attributes.attribute_orders.get(temp_attr_name).map_or_else(
                            || {
                                errors.add(&topic.get_key(), &format!("No sequence found for attribute type \"{}\".", temp_attr_name));
                                ATTRIBUTE_ORDER.len()
                            },
                            |sequence| { *sequence }
                            );
                        AttributeType::new(temp_attr_name, &value_type, sequence)
                    });
                let mut values_for_topic = vec![];
                for temp_value in temp_attr_values.iter() {
                    match attribute_type.add_value(temp_value,&topic.get_key()) {
                        Ok(canonical_value) => {
                            let values_entry = model.attributes.values.entry(canonical_value.clone()).or_insert(vec![]);
                            // Don't add a topic item if the topic has itself as an attribute.
                            if topic.name.ne(&canonical_value) {
                                values_entry.push((topic.get_key(), attribute_type.name.clone()));
                            }
                            values_for_topic.push(canonical_value)
                        },
                        Err(msg) => { errors.add(&topic.get_key(), &msg)}
                    };
                }
                topic.attributes.insert(temp_attr_name.clone(),AttributeInstance::new(temp_attr_name, attribute_type.sequence,values_for_topic));
            }
        }
        // In the values map, each entry is a list of pairs of topic keys and attribute type names.
        // Sort each of these lists by topic name first, then attribute type name.
        for list in model.attributes.values.values_mut() {
            list.sort_by(|a, b| a.0.topic_name.cmp(&b.0.topic_name).then(a.1.cmp(&b.1)));
        }
        // Self::list_attribute_types(model);
        errors
    }

    pub fn list_attribute_types(model: &Wiki) {
        let mut list = vec![];
        println!("\nAttribute types:");
        for attribute_type in model.attributes.attributes.values() {
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

    pub fn get_distinct_attr_values(model: &Wiki, value_type: &AttributeValueType) -> Vec<String> {
        let mut values = vec![];
        for attribute_type in model.attributes.attributes.values()
            .filter(|attribute_type| attribute_type.value_type.eq(value_type)) {
            for value in attribute_type.values.keys() {
                values.push(value.clone());
            }
        }
        values.sort();
        values.dedup();
        values
    }

    pub fn get_topics_for_attr_value(model: &Wiki, value_type: &AttributeValueType, match_value: &str, included_attr_names: Option<Vec<&str>>) -> Vec<TopicKey> {
        let mut topic_keys = vec![];
        for attribute_type in model.attributes.attributes.values()
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
    pub fn get_typed_topics_for_attr_value(model: &Wiki, value_type: &AttributeValueType, match_value: &str, included_attr_names: Option<Vec<&str>>) -> Vec<(String, TopicKey)> {
        let mut list = vec![];
        for attribute_type in model.attributes.attributes.values()
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

    pub fn fill_attribute_orders(model: &mut Wiki) {
        model.attributes.attribute_orders.clear();
        for (i, type_name) in ATTRIBUTE_ORDER.iter().enumerate() {
            model.attributes.attribute_orders.insert(type_name.to_string(), i);
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

