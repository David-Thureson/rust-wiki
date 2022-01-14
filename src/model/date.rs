use crate::model::{Wiki, ATTRIBUTE_NAME_ADDED, AttributeType, AttributeValueType, TopicKey, Topic};
use std::collections::BTreeMap;
use chrono::NaiveDate;

pub fn interpolate_added_date(model: &mut Wiki) {
    let attr_added_sequence = model.get_attribute_order(ATTRIBUTE_NAME_ADDED).unwrap();
    let mut changes = BTreeMap::new();

    // See if any dates can be worked out within a topic.
    for topic in model.topics.values().filter(|topic| !topic.attributes.contains_key(ATTRIBUTE_NAME_ADDED)) {
        let min_other_date = topic.attributes.values()
            .filter_map(|attr_instance| {
                let attr_type = model.attributes.attributes.get(&attr_instance.attribute_type_name).unwrap();
                if attr_type.value_type.eq(&AttributeValueType::Date) {
                    Some(attr_instance.values.iter().map(|value| AttributeType::value_to_date(value)).min().unwrap())
                } else {
                    None
                }
            }).min();
        if let Some(min_other_date) = min_other_date {
            println!("Based on other dates in topic: \"{}\": {}", topic.name, min_other_date);
            changes.insert(topic.get_key(), min_other_date);
        }
    }

    // If a given topic has no added date, set its date to the earliest date of any topics that
    // link to it. Do this in multiple rounds, since a topic may get an added date in one round
    // that is used to set the added date of another topic in a subsequent round.
    loop {
        let mut changed_count = 0;
        // We want only topics where there is no Added date and where it doesn't yet appear in the
        // changes list as having been given a new Added date.
        for topic in model.topics.values().filter(|topic| get_topic_added_date(topic, &changes).is_none()) {
            let other_dates = topic.inbound_topic_keys.iter()
                .filter(|inbound_topic_key| !inbound_topic_key.topic_name.eq_ignore_ascii_case("Main"))
                .filter_map(|inbound_topic_key| {
                    get_topic_added_date(&model.topics.get(inbound_topic_key).unwrap(), &changes)
                        .map(|date| Some((inbound_topic_key.clone(), date)))
                })
                .map(|x| x.unwrap())
                .collect::<Vec<_>>();
            //bg!(&topic.name, &other_dates);
            if !other_dates.is_empty() {
                /*
                if let Some(min_other_date) = min_other_date {
                    println!("Based on other dates in topic: \"{}\": {}", topic.name, min_other_date);
                    changes.insert(topic.get_key(), min_other_date);
                }
                */


                changed_count += 1;
            }
        }
        if changed_count == 0 {
            break;
        }
    }

    for (topic_key, date) in changes.iter() {
        let topic = model.topics.get_mut(topic_key).unwrap();
        topic.set_attribute_date(ATTRIBUTE_NAME_ADDED, attr_added_sequence, date);
    }

}

fn get_topic_added_date(topic: &Topic, changes: &BTreeMap<TopicKey, NaiveDate>) -> Option<NaiveDate> {
    match changes.get(&topic.get_key()) {
        Some(date) => Some(*date),
        None => {
            topic.attributes.get(ATTRIBUTE_NAME_ADDED)
                .map(|attr_instance| {
                    debug_assert!(attr_instance.values.len() == 1);
                    AttributeType::value_to_date(&attr_instance.values[0])
                })
        },
    }
}
