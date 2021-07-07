
/*
use std::cmp::Ordering;

pub type TopicReferenceKey = (String, String);

#[derive(Clone, Eq, Ord, PartialEq)]
pub struct TopicReference {
    namespace: String,
    topic_name: String,
}

impl TopicReference {
    pub(crate) fn new(namespace: &str, topic_name: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            topic_name: topic_name.to_string()
        }
    }

    pub fn get_key(&self) -> TopicReferenceKey {
        (self.namespace.clone(), self.topic_name.clone())
    }

    pub fn get_full_name(&self) -> String {
        format!("{{{}: {}}}", self.namespace, self.topic_name)
    }
}

impl PartialOrd for TopicReference {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (&self.namespace, &self.topic_name).partial_cmp(&(&other.namespace, &other.topic_name))
    }
}
*/