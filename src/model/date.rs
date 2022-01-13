
/*
use crate::model::{Wiki, ATTRIBUTE_NAME_ADDED, AttributeType, AttributeValueType};

pub fn interpolate_added_date(_model: &mut Wiki) {
    // First round: See if any dates can be worked out within a topic.
    /*
    for topic in model.topics.values_mut() {
        if !topic.attributes.contains_key(ATTRIBUTE_NAME_ADDED) {
            // The topic has no "Added" attribute.
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
                println!("\"{}\": {}", topic.name, min_other_date);
            }
        }
    }
    */
}

 */