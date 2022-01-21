use crate::*;
use super::*;
use crate::model::{TopicKey, HorizontalAlignment};

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
                model::Link::new_section(label, topic_key.get_namespace(), topic_key.get_topic_name(), section_name)
            } else {
                model::Link::new_topic(label, topic_key.get_namespace(), topic_key.get_topic_name())
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
    // This doesn't look like a link.
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
    // This doesn't look like a section header.
    Ok(None)
}

pub fn parse_breadcrumb_optional(text: &str) -> Result<Option<Vec<TopicKey>>, String> {
    // A breadcrumb paragraph showing the parent and grandparent topic will look like this with
    // the links worked out:
    //   **[[tools:android|Android]] => [[tools:android_development|Android Development]] => Android Sensors**
    // or like this in a new entry where only the topic names appear:
    //   **tools:Android => tools:Android Development => Android Sensors**
    // In the latter case they may or may not have the bold ("**") markup.
    // A breadcrumb paragraph for a combination topic with two parents will look like this:
    //   **[[tools:excel|Excel]] => Excel and MySQL <= [[tools:mysql|MySQL]]**
    // or:
    //   **tools:Excel => tools:Excel and MySQL <= MySQL**
    let text = text.trim().replace(DELIM_BOLD, "");
    //bg!(text);
    if !text.contains(DELIM_LINEFEED) && text.contains(DELIM_BREADCRUMB_RIGHT) {
        //bg!(&text);
        if text.contains(DELIM_BREADCRUMB_LEFT) {
            // Presumably breadcrumbs for a combo topic.
            let left = util::parse::before(&text, DELIM_BREADCRUMB_RIGHT).trim();
            let right = util::parse::after(&text, DELIM_BREADCRUMB_LEFT).trim();
            let left = eval_breadcrumb_topic_ref(left)?;
            let right = eval_breadcrumb_topic_ref(right)?;
            return Ok(Some(vec![left, right]));
        } else {
            // Presumably a breadcrumb for a topic with a single parent topic. We only care about the
            // one just to the left of the current topic name, so the second-to-last topic
            // reference in the chain.
            let topic_refs = text.rsplit(DELIM_BREADCRUMB_RIGHT).collect::<Vec<_>>();
            //bg!(&topic_refs);
            let topic_key = eval_breadcrumb_topic_ref(topic_refs[1])?;
            return Ok(Some(vec![topic_key]));
        }
    }
    // This doesn't look like a breadcrumb.
    Ok(None)
}

pub fn parse_marker_optional(text: &str) -> Result<Option<(String, String)>, String> {
    // A marker will be a one-line paragraph with something like "<WRAP round box>", "</WRAP>",
    // "<code>", "</code>", "<code Rust>", "<file>", "<html>", or "<php>".
    let text = text.trim();
    // For now we handle only "<code...>" (with or without a language) and "<WRAP...>".
    if text.starts_with(MARKER_QUOTE_START_PREFIX) || text.starts_with(MARKER_CODE_START_PREFIX) {
    // if text.starts_with(MARKER_LINE_START) {
        // We can assume this is a marker.
        if !text.ends_with(MARKER_LINE_END) {
            return Err(format!("Text seems to be a marker because it starts with \"{}\" but it does not end with \"{}\": \"{}\".", MARKER_LINE_START, MARKER_LINE_END, text));
        }
        if text.trim().contains(DELIM_LINEFEED) {
            return Err(format!("The text seems to be a marker but it has linefeeds: \"{}\".", text));
        }
        // Switch the starting "<" to "</".
        let exit_text = text.replace(MARKER_LINE_START, MARKER_LINE_START_CLOSE);
        let marker_exit_string = if exit_text.contains(" ") {
            format!("{}{}", util::parse::before(&exit_text, " "), MARKER_LINE_END)
        } else {
            exit_text
        };
        return Ok(Some((text.to_string(), marker_exit_string)));
    }
    // This doesn't look like a section header.
    Ok(None)
}

pub fn parse_table_optional(text: &str) -> Result<Option<model::Table>, String> {
    // A table with the first column bolded might look like this:
    //   ^ Platform | Android, Windows |
    //   ^ Added | Jul 24, 2018 |
    let context = "parse_table_optional()";
    // let debug = text.contains("tools:nav:attributes#Language|Language");
    let text = text.trim();
    // if debug { dbg!(text, text.starts_with(DELIM_TABLE_CELL), text.starts_with(DELIM_TABLE_CELL_BOLD)); }
    if text.starts_with(DELIM_TABLE_CELL) || text.starts_with(DELIM_TABLE_CELL_BOLD) {
        // This looks like a table.
        let lines = text.split(DELIM_LINEFEED).collect::<Vec<_>>();
        // Every line should start with one of the cell delimiters.
        if !lines.iter().all(|line| line.starts_with(DELIM_TABLE_CELL) || line.starts_with(DELIM_TABLE_CELL_BOLD)) {
            return Err(format!("This looks like a table, but not every line starts with the expected \"{}\" or \"{}\".", DELIM_TABLE_CELL, DELIM_TABLE_CELL_BOLD));
        }
        let mut table = model::Table::new(false);
        let delim_bold_temp = format!("{}{}", DELIM_TABLE_CELL, DELIM_TABLE_CELL_BOLD);
        for line in lines.iter() {
            // For ease in splitting the row into cells, change the "^" delimiters to "|^".
            let line = line.replace(DELIM_TABLE_CELL_BOLD, &delim_bold_temp);
            // Non-bolded cells start with "|". This pipe also appears inside links as the
            // between the link destination and the label, so the row might look like this (with
            // the bold first cell already turned into "|^" above).
            // |^ [[tools:nav:attributes#Narrator|Narrator]] | [[tools:nav:attribute_values#Mark Steinberg|Mark Steinberg]] |
            let mut splits = util::parse::split_outside_of_delimiters_rc(&line, DELIM_TABLE_CELL, DELIM_LINK_START, DELIM_LINK_END, context)?;
            // We don't want the first and last splits because these are simply empty strings
            // outside the first and last "|" characters (or just a "^"), not actual table cells.
            assert_eq!("", splits[0]);
            splits.remove(0);
            // if !splits[splits.len() - 1].is_empty() {
            //     dbg!(&splits);
            //     panic!()
            // }
            // assert_eq!("", splits[splits.len() - 1]);
            splits.remove(splits.len() - 1);
            //bg!(&line, &splits);
            let mut row = vec![];
            for split in splits.iter() {
                let is_bold = split.trim().starts_with(DELIM_TABLE_CELL_BOLD);
                let cell_text = split.replace(DELIM_TABLE_CELL_BOLD, "");
                let horizontal = if cell_text.starts_with("  ") && cell_text.ends_with("  ") {
                    HorizontalAlignment::Center
                } else if cell_text.starts_with("  ") {
                    HorizontalAlignment::Right
                } else {
                    HorizontalAlignment::Left
                };
                let cell_text = cell_text.trim();
                row.push(model::TableCell::new_unresolved_text(cell_text, is_bold, &horizontal));
            }
            table.add_row(row);
        }
        if table.assume_has_header() {
            table.set_has_header(true);
        }
        return Ok(Some(table));
    }
    // This doesn't look like a table.
    Ok(None)
}

fn eval_breadcrumb_topic_ref(topic_ref: &str) -> Result<TopicKey, String> {
    let topic_ref = topic_ref.replace(DELIM_LINK_START, "").replace(DELIM_LINK_END, "");
    let (topic_ref, label) = util::parse::split_1_or_2(&topic_ref, DELIM_LINK_LABEL);
    if topic_ref.contains(DELIM_NAMESPACE) {
        let (topic_name_ref, namespace) = util::parse::rsplit_2(&topic_ref, DELIM_NAMESPACE);
        let topic_name = label.unwrap_or(topic_name_ref);
        Ok(TopicKey::new(namespace.trim(), topic_name.trim()))
    } else {
        Err(format!("Expected a namespace in the breadcrumb topic reference \"{}\".", &topic_ref))
    }
}

/*
fn eval_breadcrumb_topic_ref(topic_ref: &str) -> Result<TopicKey, String> {
    let topic_ref = topic_ref.replace(DELIM_LINK_START, "").replace(DELIM_LINK_END, "");
    let (topic_ref, _label) = util::parse::split_1_or_2(&topic_ref, DELIM_LINK_LABEL);
    if topic_ref.contains(DELIM_NAMESPACE) {
        let (topic_name, namespace) = util::parse::rsplit_2(&topic_ref, DELIM_NAMESPACE);
        Ok(TopicKey::new(namespace, topic_name))
    } else {
        Err(format!("Expected a namespace in the breadcrumb topic reference \"{}\".", &topic_ref))
    }
}
*/

pub fn topic_ref_to_topic_key(topic_ref: &str) -> Result<model::TopicKey, String> {
    // Something like "tools:books:Zero to One".
    if !topic_ref.contains(DELIM_NAMESPACE) {
        return Err(format!("Namespace delimiter \"{}\" not found in topic reference \"{}\".", DELIM_NAMESPACE, topic_ref));
    }
    let (topic_name, namespace) = util::parse::rsplit_2(topic_ref, DELIM_NAMESPACE);
    Ok(model::TopicKey::new(namespace, topic_name))
}

pub fn text_or_topic_link_label(text: &str) -> Result<String, String> {
    // If this is a link, take the label if there is one, otherwise if it's a section link take the
    // section name, or if it's a topic link take the topic name.
    // If it's not a link, simply return the text.
    // This is used for the round trip with DokuWiki, in the attribute blocks. It reduces a given
    // attribute value to its string form without any links.
    let label = match parse_link_optional(text)? {
        Some(link) => {
            let label = link.get_label();
            label.unwrap_or(match link.get_type() {
                model::LinkType::Topic { topic_key} => topic_key.get_topic_name(),
                model::LinkType::Section { section_key} => section_key.get_section_name(),
                _ => {
                    dbg!(&text, &link);
                    unimplemented!()
                },
            })
        },
        None => text,
    };
    let label = util::parse::unquote(&label);
    Ok(label.to_string())
}
