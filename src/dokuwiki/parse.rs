use crate::*;
use super::*;
use crate::model::TopicKey;

pub fn parse_link_optional(text: &str) -> Result<Option<model::Link>, String> {
    // Example topic link:
    //   [[tools:combinations|Combinations]]
    // Example section link:
    //   [[tools:combinations#Notes|Combinations: Notes]]
    // Example external link:
    //   [[https://github.com/|external link|GitHub]]
    // Example image link:
    //   {{tools:antlr_plugin_on_pycharm_added.png?direct}}
    let text = text.trim();
    //bg!(text);
    if text.starts_with(DELIM_LINK_START) {
        if !text.ends_with(DELIM_LINK_END) {
            return Err(format!("Text seems to be a link because it starts with \"{}\" but it does not end with \"{}\": \"{}\".", DELIM_LINK_START, DELIM_LINK_END, text));
        }
        let text = util::parse::between_trim(text, DELIM_LINK_START, DELIM_LINK_END);
        //bg!(text);
        let (dest, label) = util::parse::split_1_or_2(text, DELIM_LINK_LABEL);
        let link = if dest.trim().to_lowercase().starts_with(PREFIX_HTTPS) {
            model::Link::new_external(label, dest)
        } else {
            // Internal link.
            let (topic_ref, section_name) = util::parse::split_1_or_2(dest, DELIM_LINK_SECTION);
            let topic_key = topic_ref_to_topic_key(topic_ref)?;
            if let Some(section_name) = section_name {
                model::Link::new_section(label, &topic_key.namespace, &topic_key.topic_name, section_name)
            } else {
                model::Link::new_topic(label, &topic_key.namespace, &topic_key.topic_name)
            }
        };
        //bg!(&link);
        return Ok(Some(link));
    } else if text.starts_with(DELIM_IMAGE_START) {
        if !text.ends_with(DELIM_IMAGE_END) {
            return Err(format!("Text seems to be an image link because it starts with \"{}\" but it does not end with \"{}\": \"{}\".", DELIM_IMAGE_START, DELIM_IMAGE_END, text));
        }
        // An image link is something like:
        //   {{tools:antlr_plugin_on_pycharm_added.png?direct}}
        let text = util::parse::between_trim(text, DELIM_IMAGE_START, DELIM_IMAGE_END);
        let (file_ref, _options) = util::parse::split_1_or_2(text, DELIM_IMAGE_OPTIONS);
        let image_source = if file_ref.to_lowercase().starts_with(PREFIX_HTTPS) {
            model::ImageSource::new_external(file_ref)
        } else {
            let (file_name, namespace) = util::parse::rsplit_2(file_ref, DELIM_NAMESPACE);
            model::ImageSource::new_internal(namespace, file_name)
        };
        // For now assume the image link type, size, alignment, etc.
        let label = None;
        let link = model::Link::new_image(label, image_source, model::ImageAlignment::Left, model::ImageSize::Original, model::ImageLinkType::Direct);
        //bg!(&link);
        return Ok(Some(link));
    }
    Ok(None)
}

pub fn parse_header_optional(text: &str) -> Result<Option<(String, usize)>, String> {
    // A section header will look like:
    //   ===Section Name===
    // The level is between 0 and 5 where 0 is the main page title. The number of "=" is six
    // minus the level.
    let text = text.trim();
    if text.starts_with(DELIM_HEADER) {
        if !text.ends_with(DELIM_HEADER) {
            return Err(format!("Text seems to be a section header because it starts with \"{}\" but it does not end with \"{}\": \"{}\".", DELIM_HEADER, DELIM_HEADER, text));
        }
        if text.trim().contains(DELIM_LINEFEED) {
            return Err(format!("The text seems to be a section header but it has linefeeds: \"{}\".", text));
        }
        for depth in 0..=5 {
            let delim = DELIM_HEADER.repeat(6 - depth);
            if text.starts_with(&delim) {
                if !text.ends_with(&delim) {
                    return Err(format!("Text seems to be a section header because it starts with \"{}\" but it does not end with a matching-length \"{}\": \"{}\".", delim, delim, text));
                }
                let name = util::parse::between_trim(text, &delim, &delim);
                return Ok(Some((name.to_string(), depth)));
            }
        }
        return Err(format!("Text seems to be a section header but the header level couldn't be determined: \"{}\".", text));
    }
    Ok(None)
}

pub fn parse_bookmark_optional(text: &str) -> Result<Option<Vec<TopicKey>>, String> {
    // A bookmark paragraph showing the parent and grandparent topic will look like this with
    // the links worked out:
    //   **[[tools:android|Android]] => [[tools:android_development|Android Development]] => Android Sensors**
    // or like this in a new entry where only the topic names appear:
    //   **tools:Android => tools:Android Development => Android Sensors**
    // In the latter case they may or may not have the bold ("**") markup.
    // A bookmark paragraph for a combination topic with two parents will look like this:
    //   **[[tools:excel|Excel]] => Excel and MySQL <= [[tools:mysql|MySQL]]**
    // or:
    //   **tools:Excel => tools:Excel and MySQL <= MySQL**
    let text = text.trim().replace(DELIM_BOLD, "");
    //bg!(text);
    if text.contains(DELIM_BOOKMARK_RIGHT) {
        dbg!(&text);
        if text.contains(DELIM_BOOKMARK_LEFT) {
            // Presumably bookmarks for a combo topic.
            let left = util::parse::before(&text, DELIM_BOOKMARK_RIGHT).trim();
            let right = util::parse::after(&text, DELIM_BOOKMARK_LEFT).trim();
            let left = eval_bookmark_topic_ref(left)?;
            let right = eval_bookmark_topic_ref(right)?;
            return Ok(Some(vec![left, right]));
        } else {
            // Presumably a bookmark for a topic with a single parent topic. We only care about the
            // one just to the left of the current topic name, so the second-to-last topic
            // reference in the chain.
            let topic_refs = text.rsplit(DELIM_BOOKMARK_RIGHT).collect::<Vec<_>>();
            //bg!(&topic_refs);
            let topic_key = eval_bookmark_topic_ref(topic_refs[1])?;
            return Ok(Some(vec![topic_key]));
        }
    }
    Ok(None)
}

fn eval_bookmark_topic_ref(topic_ref: &str) -> Result<TopicKey, String> {
    let topic_ref = topic_ref.replace(DELIM_LINK_START, "").replace(DELIM_LINK_END, "");
    let (topic_ref, _label) = util::parse::split_1_or_2(&topic_ref, DELIM_LINK_LABEL);
    if topic_ref.contains(DELIM_NAMESPACE) {
        let (topic_name, namespace) = util::parse::rsplit_2(&topic_ref, DELIM_NAMESPACE);
        Ok(TopicKey::new(namespace, topic_name))
    } else {
        Err(format!("Expected a namespace in the bookmark topic reference \"{}\".", &topic_ref))
    }
}


pub fn topic_ref_to_topic_key(topic_ref: &str) -> Result<model::TopicKey, String> {
    // Something like "tools:books:Zero to One".
    if !topic_ref.contains(DELIM_NAMESPACE) {
        return Err(format!("Namespace delimiter \"{}\" not found in topic reference \"{}\".", DELIM_NAMESPACE, topic_ref));
    }
    let (topic_name, namespace) = util::parse::rsplit_2(topic_ref, DELIM_NAMESPACE);
    Ok(model::TopicKey::new(namespace, topic_name))
}
