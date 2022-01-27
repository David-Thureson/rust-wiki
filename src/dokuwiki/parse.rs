use crate::*;
use super::*;
use crate::model::{TopicKey, HorizontalAlignment, Model, TopicRefs};

pub(crate) fn parse_link_optional(topic_refs: &TopicRefs, text: &str) -> Result<Option<model::Link>, String> {
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
        let link = if model::Link::is_external_ref(dest) {
            model::Link::new_external(label, dest)
        } else {
            // Internal link.
            let (topic_ref, section_name) = util::parse::split_1_or_2(dest, DELIM_LINK_SECTION);
            let topic_key = topic_ref_to_topic_key(topic_refs, topic_ref)?;
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
        let image_source = if model::Link::is_external_ref(file_ref) {
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

pub(crate) fn parse_header_optional(text: &str) -> Result<Option<(String, usize)>, String> {
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

pub(crate) fn parse_breadcrumb_optional(text: &str) -> Result<Option<Vec<TopicKey>>, String> {
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

pub(crate) fn parse_marker_optional(text: &str) -> Result<Option<(String, String)>, String> {
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

pub(crate) fn parse_table_optional(text: &str) -> Result<Option<model::Table>, String> {
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

pub(crate) fn topic_ref_to_topic_key(topic_refs: &TopicRefs, topic_ref: &str) -> Result<model::TopicKey, String> {
    // Something like "tools:books:Zero to One".
    if !topic_ref.contains(DELIM_NAMESPACE) {
        return Err(format!("Namespace delimiter \"{}\" not found in topic reference \"{}\".", DELIM_NAMESPACE, topic_ref));
    }
    let (topic_name, namespace) = util::parse::rsplit_2(topic_ref, DELIM_NAMESPACE);
    // Ok(model::TopicKey::new(namespace, topic_name))
    Model::get_corrected_topic_key(topic_refs, namespace, topic_name)
}

pub(crate) fn text_or_topic_link_label(text: &str) -> Result<String, String> {
    // If this is a link, take the label if there is one, otherwise if it's a section link take the
    // section name, or if it's a topic link take the topic name.
    // If it's not a link, simply return the text.
    // This is used for the round trip with DokuWiki, in the attribute blocks. It reduces a given
    // attribute value to its string form without any links.
    // Example topic link:
    //   [[tools:combinations|Combinations]]
    // Example section link:
    //   [[tools:combinations#Notes|Combinations: Notes]]
    // Example external link:
    //   [[https://github.com/|external link|GitHub]]
    let text = text.trim();
    //bg!(text);
    let label = if !text.starts_with(DELIM_LINK_START) {
        // Not a link.
        text.to_string()
    } else {
        if !text.ends_with(DELIM_LINK_END) {
            return Err(format!("Text seems to be a link because it starts with \"{}\" but it does not end with \"{}\": \"{}\".", DELIM_LINK_START, DELIM_LINK_END, text));
        }
        let text = util::parse::between_trim(text, DELIM_LINK_START, DELIM_LINK_END);
        //bg!(text);
        let (dest, label) = util::parse::split_1_or_2(text, DELIM_LINK_LABEL);
        if let Some(label) = label {
            label.to_string()
        } else {
            let (topic_ref, _section_name) = util::parse::split_1_or_2(dest, DELIM_LINK_SECTION);
            let (_namespace, topic_name) = util::parse::rsplit_2(topic_ref, DELIM_NAMESPACE);
            topic_name.to_string()
        }
    };
    let label = util::parse::unquote(&label);
    Ok(label.to_string())
}

pub(crate) fn parse_list_optional(text: &str) -> Result<Option<model::List>, String> {
    let mut lines = text.split(DELIM_LINEFEED).collect::<Vec<_>>();
    let first_line = lines.remove(0);
    //util::parse::print_chars(first_line);
    let first_line_as_list_item = parse_list_item_optional(first_line)?;
    if lines.len() == 1 {
        // The text is a single line, so if that line is a list item, we call the text a list with
        // no label and only one line.
        if let Some(list_item) = first_line_as_list_item {
            let mut list = model::List::new(model::ListType::General, None);
            list.add_item(list_item);
            return Ok(Some(list));
        } else {
            // There's only one line and it's not a list item, so this text does not represent a
            // list.
            return Ok(None);
        }
    }
    // We have at least two lines. The first line may or may not be a list item. We need to see if
    // the rest of the lines are all list items.
    let mut rest_of_lines_as_list_items = lines.iter()
        .filter_map(|line| {
            let list_item = parse_list_item_optional(line).ok()?;
            list_item
        })
        .collect::<Vec<_>>();
    if rest_of_lines_as_list_items.is_empty() {
        // This is the most likely case, since most paragraphs in the wiki are not lists.
        if first_line_as_list_item.is_some() {
            return Err("The first line is a list item, but the other lines are not.".to_string());
        } else {
            // None of the lines are list items, so this text is not a list.
            return Ok(None);
        }
    } else if rest_of_lines_as_list_items.len() == lines.len() {
        // All of the lines starting from the second one are list items. So whether the first line
        // is a list item or a header, the text is a list.
        let mut list = if let Some(list_item) = first_line_as_list_item {
            // The first line is a list item. Make a list with no header and add this first list
            // item. Since we don't have a header to tell us the list type, go with General.
            let mut list = model::List::new(model::ListType::General, None);
            list.add_item(list_item);
            list
        } else {
            // The first line is not a list item so it must be the header.
            let type_ = model::ListType::from_header(first_line);
            model::List::new(type_, Some(model::TextBlock::new_unresolved(first_line)))
        };
        // We've dealt with the first line, whether it was a header or the first list item. Now add
        // the remaining lines/list items.
        for list_item in rest_of_lines_as_list_items.drain(..) {
            list.add_item(list_item);
        };
        return Ok(Some(list));
    } else {
        return Err("Some of the lines are list items and some are not.".to_string());
    }
}

pub(crate) fn parse_list_item_optional(line: &str) -> Result<Option<model::ListItem>, String> {
    assert!(!line.contains(DELIM_LINEFEED));
    let (is_list, is_ordered) = if line.trim().starts_with(DELIM_LIST_ITEM_ORDERED) {
        (true, true)
    } else if line.trim().starts_with(DELIM_LIST_ITEM_UNORDERED) {
        (true, false)
    } else {
        (false, false)
    };
    if !is_list {
        return Ok(None);
    }

    let text = if is_list {
        util::parse::after(line, DELIM_LIST_ITEM_ORDERED).trim()
    } else {
        util::parse::after(line, DELIM_LIST_ITEM_UNORDERED).trim()
    };
    let text_block = model::TextBlock::new_unresolved(text);

    let leading_space_count = util::parse::count_leading_spaces(line);
    if leading_space_count & 2 != 0 {
        return Err(format!("Expected an even number of spaces for a list item: \"{}\".", line));
    }
    if leading_space_count < 2 {
        return Err(format!("Expected at least two spaces for a list item: \"{}\".", line));
    }
    let depth = leading_space_count / 2;
    assert!(depth > 0);

    let list_item = model::ListItem::new(depth, is_ordered, text_block);

    Ok(Some(list_item))
}
