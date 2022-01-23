// use crate::model::{ATTRIBUTE_NAME_ADDED, AttributeType, TopicKey, Topic};

/*
pub(crate) fn interpolate_added_date(model: &mut Model) {
    let attr_added_sequence = model.get_attribute_orders().get(ATTRIBUTE_NAME_ADDED).unwrap();
    let mut changes = BTreeMap::new();

    // See if any dates can be worked out within a topic.
    for topic in model.get_topics().values().filter(|topic| !topic.get_attributes().contains_key(ATTRIBUTE_NAME_ADDED)) {
        let min_other_date = topic.get_attributes().values()
            .filter_map(|attr_instance| {
                let attr_type = model.get_attribute_type(attr_instance.get_attribute_type_name()).unwrap();
                if attr_type.get_value_type().eq(&AttributeValueType::Date) {
                    Some(attr_instance.get_values().iter().map(|value| AttributeType::value_to_date(value)).min().unwrap())
                } else {
                    None
                }
            }).min();
        if let Some(min_other_date) = min_other_date {
            println!("Based on other dates in topic: \"{}\": {}", topic.get_name(), min_other_date);
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
        for topic in model.get_topics().values().filter(|topic| get_topic_added_date(topic, &changes).is_none()) {
            let other_dates = topic.get_inbound_topic_keys().iter()
                .filter(|inbound_topic_key| !inbound_topic_key.get_topic_name().eq_ignore_ascii_case("Main"))
                .filter_map(|inbound_topic_key| {
                    get_topic_added_date(&model.get_topics().get(inbound_topic_key).unwrap(), &changes)
                        .map(|date| Some((inbound_topic_key.clone(), date)))
                })
                .map(|x| x.unwrap())
                .collect::<Vec<_>>();
            //bg!(topic.get_name(), &other_dates);
            if !other_dates.is_empty() {
                /*
                if let Some(min_other_date) = min_other_date {
                    println!("Based on other dates in topic: \"{}\": {}", topic.get_name(), min_other_date);
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
        let topic = model.get_topics_mut().get_mut(topic_key).unwrap();
        topic.set_attribute_date(ATTRIBUTE_NAME_ADDED, *attr_added_sequence, date);
    }
}
*/

/*
fn get_topic_added_date(topic: &Topic, changes: &BTreeMap<TopicKey, NaiveDate>) -> Option<NaiveDate> {
    match changes.get(&topic.get_key()) {
        Some(date) => Some(*date),
        None => {
            topic.get_attributes().get(ATTRIBUTE_NAME_ADDED)
                .map(|attr_instance| {
                    debug_assert!(attr_instance.get_values().len() == 1);
                    AttributeType::value_to_date(&attr_instance.get_values()[0])
                })
        },
    }
}
*/