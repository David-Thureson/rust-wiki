use std::rc::Rc;
use std::cell::RefCell;

use super::*;
use std::collections::BTreeMap;

pub type AttributeRc = Rc<RefCell<Attribute>>;
// pub type AttributeValueRc = Rc<RefCell<AttributeValue>>;

pub enum AttributeType {
    Boolean,
    Date,
    // Choice,
    Number,
    String,
    Unknown,
}

pub struct Attribute {
    pub wiki: WikiRc,
    pub type_: AttributeType,
    pub name: String,
    pub values: BTreeMap<String, String>,
}

/*
pub struct AttributeValueList {
    pub list: BTreeMap<String, Vec<String>>,
}
*/

/*
pub struct AttributeValue {
    pub attribute: AttributeRc,
    pub value: String,
    pub topics: BTreeMap<TopicKey, TopicRc>,
}
*/

/*
pub enum AttributeTypedValue {
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
    }
}
*/

impl Attribute {
    pub fn new(wiki: WikiRc, type_: AttributeType, name: &str) -> Self {
        Self {
            wiki,
            name: name.to_string(),
            type_,
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
    pub fn new(attribute: &AttributeRc, value: String) -> Self {
        Self {
            attribute: attribute.clone(),
            value,
            topics: Default::default()
        }
    }

    pub fn topic_count(&self) -> usize {
        self.topics.len()
    }

    pub fn get_topics(&self) -> Vec<TopicRc> {
        self.topics.values().map(|topic_rc| topic_rc.clone()).collect()
    }
}

impl AttributeTypedValue {
    pub fn new_string(value: &str) -> Self {
        Self::String {
            value: value.to_string(),
        }
    }
}
*/