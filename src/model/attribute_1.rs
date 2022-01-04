// Date/time formatting escape sequences: https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html

use super::*;
use std::collections::BTreeMap;
use chrono::NaiveDate;
use std::fmt::{Display, Formatter};
use std::cmp::Ordering;

// pub type AttributeRc = Rc<RefCell<Attribute>>;
// pub type AttributeValueRc = Rc<RefCell<AttributeValue>>;

// This is the overall kind of topic like Author, Domain, or Language.
#[derive(Debug)]
pub struct AttributeType {
    pub name: String,
    pub type_name: String,
    pub values: BTreeMap<AttributeValue, Vec<TopicKey>>,
}

// This is an instance of an attribute of some type in a single topic, possibly with multiple
// values.
#[derive(Debug)]
pub struct AttributeInstance {
    pub attribute_type_name: String,
    pub values: Vec<AttributeValue>,
}

#[derive(Debug)]
pub enum AttributeValue {
    Boolean {
        value: bool,
    },
    Date {
        value: NaiveDate,
    },
    Number {
        value: usize,
    },
    String {
        value: String,
    },
    Unknown {
        value: String,
    },
    Year {
        value: usize,
    }
}

impl AttributeType {
    pub fn new(name: &str, type_name: &str) -> Self {
        Self {
            name: name.to_string(),
            type_name: type_name.to_string(),
            values: Default::default(),
        }
    }

    /*
    pub fn add_value_string(&mut self, value: &str) {
        let entry = self.values.entry(value.to_string()).or_insert(r!(AttributeValue::new(AttributeTypedValue::new_string(value))));
    }
    */

    pub fn value_count(&self) -> usize {
        self.values.len()
    }

    /*
    pub fn topic_count(&self) -> usize {
        self.values.values().map(|value| b!(value).topic_count()).sum()
    }

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

impl AttributeValue {

    pub fn get_key(&self) -> String {
        self.to_string().to_lowercase()
    }

    pub fn get_display_string(&self) -> String {
        // In most cases the display string, which will be used on wiki pages, is the same as the
        // to_string(). For dates, though, the regular to_string() is something like "2022-01-03"
        // so that it sorts correctly, while the display string is something like "2022-Jan-03".
        match self {
            AttributeValue::Date { value} => util::date_time::date_for_document(value),
            _ => self.to_string(),
        }
    }

}

impl Display for AttributeValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let s = match self {
            AttributeValue::Boolean { value} => bool_to_yes_no(*value),
            AttributeValue::Date { value} => util::date_time::format_naive_date_sortable(value),
            AttributeValue::Number { value} => util::format::format_count(*value),
            AttributeValue::String { value} => value.clone(),
            AttributeValue::Unknown { value} => value.clone(),
            AttributeValue::Year { value} => value.to_string(),
        };
        write!(f, "{}", s)
    }
}

impl PartialEq for AttributeValue {
    fn eq(&self, other: &Self) -> bool {
        self.get_key() == other.get_key()
    }
}

impl Eq for AttributeValue {
}

impl PartialOrd for AttributeValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_key().partial_cmp(&other.get_key())
    }
}

impl Ord for AttributeValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_key().cmp(&other.get_key())
    }
}
