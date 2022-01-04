use super::*;
use std::collections::BTreeMap;

// pub type AttributeRc = Rc<RefCell<Attribute>>;
// pub type AttributeValueRc = Rc<RefCell<AttributeValue>>;

// This is the overall kind of topic like Author, Domain, or Language.
#[derive(Debug)]
pub struct AttributeType {
    pub name: String,
    pub value_type: AttributeValueType,
    pub values: BTreeMap<String, Vec<TopicKey>>,
}

// This is an instance of an attribute of some type in a single topic, possibly with multiple
// values.
#[derive(Debug)]
pub struct AttributeInstance {
    pub attribute_type_name: String,
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

impl AttributeType {
    pub fn new(name: &str, value_type: &AttributeValueType) -> Self {
        Self {
            name: name.to_string(),
            value_type: value_type.clone(),
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
                    Ok(value.to_string())
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
        // In most cases the display string, which will be used on wiki pages, is the same as the
        // string stored as the value. For dates, though, the value is something like "2022-01-03"
        // so that it sorts correctly, while the display string is something like "2022-Jan-03".
        match self.value_type {
            AttributeValueType::Date => util::date_time::naive_date_to_doc_format(&util::date_time::naive_date_from_sortable_format(value).unwrap()),
            _ => value.to_string(),
        }
    }

    pub fn get_topic_count(&self) -> usize {
        self.values.values().map(|topics| topics.len()).sum()
    }

    /*
    pub fn get_topics(&self) -> Vec<TopicRc> {
        let mut topics: Vec<TopicRc> = self.values.values()
            .map(|value| b!(value).get_topics())
            .flatten()
            .collect::<Vec<_>>();
        topics.sort();
        topics.dedup();
        topics
    }
     */
}

impl AttributeInstance {

    pub fn new(attribute_type_name: &str, values: Vec<String>) -> Self {
        Self {
            attribute_type_name: attribute_type_name.to_string(),
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

