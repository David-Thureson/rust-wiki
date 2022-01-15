use crate::*;
use super::*;

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

pub fn topic_ref_to_topic_key(topic_ref: &str) -> Result<model::TopicKey, String> {
    // Something like "tools:books:Zero to One".
    if !topic_ref.contains(DELIM_NAMESPACE) {
        return Err(format!("Namespace delimiter \"{}\" not found in topic reference \"{}\".", DELIM_NAMESPACE, topic_ref));
    }
    let (topic_name, namespace) = util::parse::rsplit_2(topic_ref, DELIM_NAMESPACE);
    Ok(model::TopicKey::new(namespace, topic_name))
}
