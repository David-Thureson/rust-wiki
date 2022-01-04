use std::collections::BTreeMap;
use crate::model::TopicKey;
use crate::Itertools;

pub struct TopicErrorList {
    errors: BTreeMap<TopicKey, Vec<String>>,
}

impl TopicErrorList {
    pub(crate) fn new() -> Self {
        Self {
            errors: Default::default(),
        }
    }

    pub(crate) fn add(&mut self, topic_key: &TopicKey, message: &str) {
        let entry = self.errors.entry(topic_key.clone()).or_insert(vec![]);
        let message = message.to_string();
        if !entry.contains(&message) {
            entry.push(message);
        }
    }

    pub(crate) fn append(&mut self, other: &mut TopicErrorList) {
        self.errors.append(&mut other.errors);
    }

    pub(crate) fn clear(&mut self) {
        self.errors.clear();
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub(crate) fn print(&self, context: Option<&str>) {
        let context = context.map_or("".to_string(), |context| format!(" for context = \"{}\"", context));
        if self.errors.is_empty() {
            println!("No errors{}:", context);
        } else {
            println!("\nErrors{}:", context);
            for topic_key in self.errors.keys() {
                println!("\n\t{}", topic_key);
                for msg in self.errors[topic_key].iter() {
                    println!("\t\t{}", msg);
                }
            }
            println!();
        }
    }

    pub fn list_missing_topics(&self) {
        let before = "Topic link [";
        let after = "] not found.";
        let mut map = BTreeMap::new();
        for error_topic_key in self.errors.keys() {
            for msg in self.errors[error_topic_key].iter() {
                // Looking for something like "Topic link [mysql connector/j] not found."
                if msg.starts_with(before) && msg.ends_with(after) {
                    let topic_name = util::parse::between_trim(msg, before, after);
                    let entry = map.entry(topic_name).or_insert(vec![]);
                    entry.push(error_topic_key.topic_name.clone());
                }
            }
        }
        for (ref_topic_name, error_topic_names) in map.iter() {
            println!("{}\t[{}]", ref_topic_name, error_topic_names.iter().join(", "));
        }
    }
}