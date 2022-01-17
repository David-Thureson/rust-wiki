use crate::model::*;
use std::fs;
use super::*;

pub fn main() {
    // let topic_limit = None;
    let topic_limit = Some(20);
    build_model(gen_tools_wiki::PROJECT_NAME, &gen_tools_wiki::PROJECT_NAME.to_lowercase(), topic_limit, gen_tools_wiki::get_attr_to_index());
}

struct BuildProcess {
    wiki_name: String,
    namespace_main: String,
    path_source: String,
    errors: TopicErrorList,
    topic_limit: Option<usize>,
}

impl BuildProcess {
    pub fn new(wiki_name: &str, namespace_main: &str, path_source: &str, topic_limit: Option<usize>) -> Self {
        Self {
            wiki_name: wiki_name.to_string(),
            namespace_main: namespace_main.to_string(),
            path_source: path_source.to_string(),
            errors: TopicErrorList::new(),
            topic_limit,
        }
    }

    pub fn build(&mut self) -> Wiki {
        let mut model = Wiki::new(&self.wiki_name, &self.namespace_main);
        let namespace_main = self.namespace_main.clone();
        let namespace_book = model.namespace_book();
        model.add_namespace(&namespace_book);

        let topic_limit_per_namespace = self.topic_limit.map(|topic_limit| topic_limit / 2);
        self.parse_from_folder(&mut model, &namespace_main, topic_limit_per_namespace);
        self.parse_from_folder(&mut model, &namespace_book, topic_limit_per_namespace);

        // Figure out the real nature of each paragraph.
        self.refine_paragraphs(&mut model);

        /*
        wiki.catalog_links();
        self.check_links(&wiki);
        self.check_subtopic_relationships(&mut wiki);
        self.errors.print(Some("First pass"));
        self.errors.list_missing_topics();

        // WikiReport::new().categories().paragraphs().attributes().lists().go(&wiki);
        // report_category_tree(&wiki);
        // wiki.catalog_possible_list_types().print_by_count(0, None);
        wiki.add_missing_category_topics();
        wiki.catalog_links();
        self.errors.clear();
        self.check_links(&wiki);
        self.errors.print(Some("After adding missing category topics."));
        // Call the make tree functions after the last call to wiki.catalog_links().
        wiki.make_category_tree();
        wiki.make_subtopic_tree();
        //bg!(&wiki.attributes);
        let attr_errors = wiki.catalog_attributes();
        attr_errors.print(Some("wiki.catalog_attributes()"));
        if attr_errors.is_empty() {
            //report_attributes(&wiki);
        }
        wiki.catalog_domains();

         */
        model
    }

    fn parse_from_folder(&mut self, model: &mut Wiki, namespace_name: &str, topic_limit: Option<usize>) {
        // Read each page's text file and read it as a topic, then break each topic into
        // paragraphs. At this point we don't care about whether the paragraphs are plain or mixed
        // text, attribute tables, section headers, breadcrumbs, etc.

        let mut topic_count = 0;
        let path_source = format!("{}/{}", self.path_source, gen::namespace_to_path(namespace_name));
        for dir_entry_result in fs::read_dir(path_source).unwrap() {
            let dir_entry = dir_entry_result.as_ref().unwrap();
            let file_name = util::file::dir_entry_to_file_name(dir_entry);
            if file_name.ends_with(".txt") {
                let content = fs::read_to_string(&dir_entry.path()).unwrap();
                let mut paragraphs = content.split(DELIM_PARAGRAPH).collect::<Vec<_>>();
                // The first paragraph should have the topic name as a page header, like:
                //   ======A Mind for Numbers======
                let mut topic_name = paragraphs.remove(0).to_string();
                assert!(topic_name.starts_with(DELIM_HEADER));
                assert!(topic_name.ends_with(DELIM_HEADER));
                topic_name = topic_name.replace(DELIM_HEADER, "").trim().to_string();
                dbg!(&topic_name);
                let mut topic = Topic::new(namespace_name, &topic_name);
                for paragraph in paragraphs.iter() {
                    topic.add_paragraph(Paragraph::new_unknown(paragraph));
                }
                model.add_topic(topic);
                topic_count += 1;
                if topic_limit.map_or(false, |topic_limit| topic_count >= topic_limit) {
                    break;
                }
            }
        }
    }

    fn refine_paragraphs(&mut self, model: &mut Wiki) {
        for topic in model.topics.values_mut() {
            let context = format!("Refining paragraphs for \"{}\".", topic.name);
            println!("\n==================================================================\n\n{}\n", context);
            let paragraph_count = topic.paragraphs.len();
            for paragraph_index in 0..paragraph_count {
                match self.refine_one_paragraph_rc(topic, paragraph_index, &context) {
                    Err(msg) => {
                        let topic_key = TopicKey::new(&self.namespace_main, &topic.name);
                        self.errors.add(&topic_key, &msg);
                    },
                    _ => (),
                }
            }
        }
    }

    fn refine_one_paragraph_rc(&mut self, topic: &mut Topic, paragraph_index: usize, context: &str) -> Result<(), String> {
        let source_paragraph= std::mem::replace(&mut topic.paragraphs[paragraph_index], Paragraph::Placeholder);
        match source_paragraph {
            Paragraph::Unknown { text } => {
                if !(self.paragraph_as_category_rc(topic, &text, context)?
                    || self.paragraph_as_section_header_rc(topic, &text, paragraph_index, context)?
                    || self.paragraph_as_bookmark_rc(topic, &text, context)?
                    // || self.paragraph_as_quote_start_or_end_rc(topic, &text, context)? {
                    || self.paragraph_as_table_rc(topic, &text, paragraph_index, context)?
                    // || self.paragraph_as_list_rc(topic, &text, context)? {
                    // || self.paragraph_as_text_rc(topic, &text, context)? {
                ) {
                    let new_paragraph = Paragraph::new_text_unresolved(&text);
                    topic.paragraphs[paragraph_index] = new_paragraph;
                }
                return Ok(());
            },
            _ => {},
        };
        // if topic.name.contains("Zero") { dbg!("new_paragraph", &new_paragraph.get_variant_name()); }
        // topic.paragraphs[paragraph_index] = new_paragraph;
        topic.paragraphs[paragraph_index] = source_paragraph;
        Ok(())
    }

    fn paragraph_as_category_rc(&mut self, topic: &mut Topic, text: &str, context: &str) -> Result<bool, String> {
        // If it's a category line it will look like this if it already has a link:
        //   Category: [[tools:nonfiction_books|Nonfiction Books]]
        // or like this if it does not yet have a link (which will be added during the re-gen
        // process we're in):
        //   Category: Nonfiction Books
        let context = &format!("{} Seems to be a category paragraph.", context);
        let err_func = |msg: &str| Err(format!("{} paragraph_as_category_rc: {}: text = \"{}\".", context, msg, text));
        if text.trim().starts_with(PREFIX_CATEGORY) {
            if text.trim().contains(DELIM_LINEFEED) {
                return err_func("The text seems to be a category format but it has linefeeds.");
            } else {
                let category_part = util::parse::after(text, PREFIX_CATEGORY).trim().to_string();
                match parse_link_optional(&category_part) {
                    Ok(Some(link)) => {
                        match link.label {
                            Some(label) => {
                                let category_name = label;
                                //rintln!("\"{}\" in \"{}\"", topic.name, category_name);
                                topic.category = Some(category_name);
                                return Ok(true);
                            },
                            None => {
                                return err_func("Expected the link to have a label which is the category name.");
                            },
                        }
                    },
                    Ok(None) => {
                        let category_name = category_part;
                        println!("\"{}\" in \"{}\"", topic.name, category_name);
                        topic.category = Some(category_name);
                        return Ok(true);
                    },
                    Err(msg) => {
                        return err_func(&msg);
                    }
                }
            }
        } else {
            Ok(false)
        }
    }

    fn paragraph_as_section_header_rc(&mut self, topic: &mut Topic, text: &str, paragraph_index: usize, context: &str) -> Result<bool, String> {
        // A section header will look like:
        //   ===Section Name===
        // The level is between 0 and 5 where 0 is the main page title. The number of "=" is six
        // minus the level.
        let context = &format!("{} Seems to be a section header paragraph.", context);
        let err_func = |msg: &str| Err(format!("{} paragraph_as_section_header_rc: {}: text = \"{}\".", context, msg, text));
        match parse_header_optional(text) {
            Ok(Some((name, depth))) => {
                //bg!(&name, depth);
                topic.paragraphs[paragraph_index] = Paragraph::new_section_header(&name, depth);
                Ok(true)
            },
            Ok(None) => {
                Ok(false)
            },
            Err(msg) => {
                return err_func(&msg);
            }
        }
    }

    fn paragraph_as_bookmark_rc(&mut self, topic: &mut Topic, text: &str, context: &str) -> Result<bool, String> {
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
        let context = &format!("{} Seems to be a bookmark paragraph.", context);
        let err_func = |msg: &str| Err(format!("{} paragraph_as_bookmark_rc: {}: text = \"{}\".", context, msg, text));
        match parse_bookmark_optional(text) {
            Ok(Some(parent_topic_keys)) => {
                //bg!(&parent_topic_keys);
                topic.parents = parent_topic_keys;
                Ok(true)
            },
            Ok(None) => {
                Ok(false)
            },
            Err(msg) => {
                return err_func(&msg);
            }
        }
    }
    /*
    fn paragraph_as_quote_start_or_end_rc(&mut self, _topic: &mut Topic, text: &str, _context: &str) -> Result<Option<Paragraph>, String> {
        if text.trim().eq(CT_TEMP_DELIM_QUOTE_START) {
            Ok(Some(Paragraph::QuoteStart))
        } else if text.trim().eq(CT_TEMP_DELIM_QUOTE_END) {
            Ok(Some(Paragraph::QuoteEnd))
        } else {
            Ok(None)
        }
    }
    */

    fn paragraph_as_table_rc(&mut self, topic: &mut Topic, text: &str, _paragraph_index: usize, context: &str) -> Result<bool, String> {
        // A paragraph with a list of attributes will look something like this:
        //   ^ [[tools:nav:attributes#Platform|Platform]] | [[tools:nav:attribute_values#Android|Android]] |
        //   ^ [[tools:nav:dates|Added]] | [[tools:nav:dates#Jul 24, 2018|Jul 24, 2018]] |
        // A given attribute type may have multiple values separated by commas, in which case the
        // values cell might look like:
        //   ^ [[tools:nav:attribute_values#Android|Android]], [[tools:nav:attribute_values#Windows|Windows]]
        // If the attributes were added by hand, and have not gone through the cycle of parsing
        // generating that we're in right now, the table might look like:
        //   ^ Platform | Android, Windows |
        //   ^ Added | Jul 24, 2018 |
        // and the date might be "2018-Jul-24", "2018-07-24", or some other supported format.
        // A regular table will look similar. The Terms page has a large example of a regular
        // table.
        let context = &format!("{} Seems to be a table paragraph.", context);
        let err_func = |msg: &str| Err(format!("{} paragraph_as_table_rc: {}: text = \"{}\".", context, msg, text));
        match parse_table_optional(text) {
            Ok(Some(text_table)) => {
                //bg!(&text_table);
                if !text_table.has_header() && text_table.has_label_column() {
                    // For now assume this is a table of attributes.
                    for row in text_table.rows.iter() {
                        let attr_type_name = text_or_topic_link_label(&row[0].text)?;
                        dbg!(&attr_type_name);
                        let mut attr_values = vec![];
                        // let cell_items = row[1].text.split(",").collect::<Vec<_>>();
                        let cell_items = util::parse::split_outside_of_delimiters_rc(&row[1].text, ",", "\"", "\"", context).unwrap();
                        for cell_item in cell_items.iter() {
                            attr_values.push(text_or_topic_link_label(cell_item)?);
                        }
                        dbg!(&attr_values);
                        topic.temp_attributes.insert(attr_type_name, attr_values);
                    }
                } else {
                    // Assume this is a normal (non-attribute) table.



                    // topic.paragraphs[paragraph_index] = Paragraph::new_section_header(&name, depth);
                }
                Ok(true)
            },
            Ok(None) => {
                Ok(false)
            },
            Err(msg) => {
                return err_func(&msg);
            }
        }
    }

    /*
    fn paragraph_as_attributes_rc(&mut self, topic: &mut Topic, text: &str, context: &str) -> Result<Option<Paragraph>, String> {
        // A paragraph with a list of attributes will look something like this:
        //   {|
        //   ||Domain||[[Domain:=Serverless]], [[Domain:=Function as a Service / FaaS]]||
        //   ||Added||[[Added:=20201204]]||
        let context = &format!("{} Seems to be an attributes paragraph.", context);
        let lines = text.lines().collect::<Vec<_>>();
        if lines[0].trim() == CT_TABLE_START
            && lines.len() > 1
            && lines[1].starts_with(CT_TABLE_DELIM)
            && split_trim(lines[1], CT_TABLE_DELIM).len() == 4 {
            // We're going to guess that this is a table of attributes.
            for line_index in 1..lines.len() {
                let line = lines[line_index].trim();
                if line == CT_TABLE_END {
                    break;
                }
                let line = between(line, CT_TABLE_DELIM, CT_TABLE_DELIM);
                let split = split_trim(line, CT_TABLE_DELIM);
                if split.len() != 2 {
                    return Err(format!("{} Wrong number of table cells in \"{}\".", context, line));
                }
                let (name, values) = (split[0].to_string(), split[1].to_string());
                let attribute = topic.attributes.entry(name.clone())
                    .or_insert(vec![]);
                let values = between(&values, CT_BRACKETS_LEFT, CT_BRACKETS_RIGHT);
                let bracket_delim_with_space = format!("{}, {}", CT_BRACKETS_LEFT, CT_BRACKETS_RIGHT);
                let bracket_delim_no_space = format!("{},{}", CT_BRACKETS_LEFT, CT_BRACKETS_RIGHT);
                let values = values.replace(&bracket_delim_with_space, &bracket_delim_no_space);
                for value in values.split(&bracket_delim_no_space) {
                    let mut value= value.trim().to_string();
                    if value.contains("*") {
                        value = "".to_string();
                    }
                    if attribute.contains(&value) {
                        return Err(format!("{} In attribute \"{}\", duplicated value \"{}\".", context, name, value));
                    }
                    attribute.push(value);
                }
            }
            Ok(Some(Paragraph::Attributes))
        } else {
            Ok(None)
        }
    }

    fn paragraph_as_list_rc(&mut self, topic: &mut Topic, text: &str, context: &str) -> Result<Option<Paragraph>, String> {
        // if text.contains("Determine if number of messages") {
        //     dbg!(&text);
        // }
        // Example with two levels (the first level has one space before the asterisk):
        // Projects:
        //   * [[Android]]
        //    * [[Algomator]]
        //    * [[Sensor (coding project)]]
        //   * [[Windows]]
        //    * [[By the Numbers]]
        //    * [[Genealogy (coding project)]]
        let context = &format!("{} Seems to be a list paragraph.", context);
        if !text.contains(" *") {
            return Ok(None);
        }
        let lines = text.lines().collect::<Vec<_>>();
        if lines.len() < 2 || !lines[1].trim().starts_with("*") {
            return Ok(None);
        }
        let err_func = |msg: &str| Err(format!("{} paragraph_as_list_rc: {}: partial = \n\t\t\t{}\n\t\t\t{}.", context, msg, lines[0], lines[1]));
        // At this point we're going to assume that it's a list and consider it an error if
        // that doesn't work out.
        if lines[0].trim().starts_with("*") {
            return err_func("List with no header.");
        }
        let type_ = ListType::from_header(lines[0].trim());
        // The header may be a simple label like "Subtopics:" but it could also be a longer piece
        // of text containing links and other markup.
        let header = self.make_text_block_rc(&topic.name, lines[0], context)?;
        let mut items = vec![];
        for line in lines.iter().skip(1) {
            // The depth of the list item is the number of spaces before the asterisk.
            match line.find("*") {
                Some(depth) => {
                    let item_text = line[depth + 1..].trim();
                    let item_text_block = self.make_text_block_rc(&topic.name, item_text, context)?;
                    items.push(ListItem::new(depth, item_text_block));
                },
                None => {
                    return err_func("List item with no \"*\".");
                }
            }
        }
        let paragraph = Paragraph::List {
            type_,
            header,
            items,
        };
        Ok(Some(paragraph))
    }

    fn paragraph_as_text_rc(&self, topic: &mut Topic, text: &str, context: &str) -> Result<Option<Paragraph>, String> {
        let context = &format!("{} Seems to be a text paragraph.", context);
        let text_block = self.make_text_block_rc(&topic.name, text, context)?;
        Ok(Some(Paragraph::new_text(text_block)))
    }

    fn check_links(&mut self, wiki: &Wiki) {
        let mut link_errors = wiki.check_links();
        self.errors.append(&mut link_errors);
    }

    fn check_subtopic_relationships(&mut self, wiki: &mut Wiki) {
        let mut subtopic_errors = wiki.check_subtopic_relationships();
        self.errors.append(&mut subtopic_errors);
    }

    fn make_text_block_rc(&self, topic_name: &str, text: &str, context: &str) -> Result<TextBlock, String> {
        let text = text.trim();
        let mut text_block = TextBlock::new();
        let delimited_splits = util::parse::split_delimited_and_normal_rc(text, CT_BRACKETS_LEFT, CT_BRACKETS_RIGHT, context)?;
        for (item_is_delimited, item_text) in delimited_splits.iter() {
            if *item_is_delimited {
                // Assume it's an internal or external link, or an image link.
                if let Some(link) = self.make_link_rc(topic_name, item_text, context)? {
                    text_block.items.push(TextItem::new_link(link));
                }
            } else {
                // Assume it's plain text.
                text_block.items.push(TextItem::new_text(item_text));
            }
        }
        Ok(text_block)
    }

    fn make_link_rc(&self, topic_name: &str, text: &str, context: &str) -> Result<Option<Link>, String> {
        let text = text.trim();
        let err_func = |msg: &str| Err(format!("{} make_link_rc: {}: text = \"{}\".", context, msg, text));
        // The brackets should have been removed by this point.
        if text.starts_with("$ASK:") {
            return Ok(None);
        }
        if text.contains(CT_BRACKETS_LEFT) || text.contains(CT_BRACKETS_RIGHT) {
            return err_func("Brackets found in text for a link. They should have been removed.");
        }
        if text.starts_with(CT_PREFIX_IMAGE) {
            return Ok(Some(self.make_image_link_rc(text, context)?));
        }
        if text.starts_with(CT_PREFIX_URL) {
            // External link.
            let (url, label) = util::parse::split_1_or_2_trim(&text, CT_PIPE);
            let url = util::parse::after(url, CT_PREFIX_URL);
            return Ok(Some(Link::new_external(label, url)));
        }
        // For now skip anything else starting with a "$" like $FILE.
        if text.starts_with("$") {
            return Ok(None);
        }
        // Assume it's an internal link, either to a topic or a section of a topic.
        let (mut dest, label) = util::parse::split_1_or_2_trim(&text, CT_PIPE);
        if dest.contains(CT_DELIM_SECTION_IN_LINK) {
            // Link to a section of a topic.
            let (mut link_topic_name, link_section_name) = util::parse::split_2_trim(dest, CT_DELIM_SECTION_IN_LINK);
            if link_topic_name.trim().is_empty() {
                // This is a link to a section in the same topic.
                link_topic_name = topic_name;
            }
            if link_topic_name.starts_with("_") {
                link_topic_name = &link_topic_name[1..];
            }
            return Ok(Some(Link::new_section(label,&self.namespace_main, link_topic_name, link_section_name)));
        } else {
            // Link to a whole topic.
            if dest.starts_with("_") {
                dest = &dest[1..];
            }
            return Ok(Some(Link::new_topic(label, &self.namespace_main, dest)));
        }
    }

    fn make_image_link_rc(&self, text: &str, context: &str) -> Result<Link, String> {
        // Something like this (with no brackets):
        //   $IMG:Images\libgdx project setup main.jpg|100%|NONE
        //   $IMG:Images\libgdx generated new project.png|(($IMG_SIZE))|NONE
        // The brackets should have been removed by this point.
        let err_func = |msg: &str| Err(format!("{} make_image_link_rc: {}: text = \"{}\".", context, msg, text));
        let text = text.trim();
        if text.contains(CT_BRACKETS_LEFT) || text.contains(CT_BRACKETS_RIGHT) {
            return err_func("Brackets found in text for an image link. They should have been removed.");
        }
        if !text.starts_with(CT_PREFIX_IMAGE) {
            return err_func(&format!("Presumed image link doesn't start with {}.", CT_PREFIX_IMAGE));
        }
        let text = text[CT_PREFIX_IMAGE.len()..].trim().to_string();
        let splits = util::parse::split_trim(&text, CT_PIPE);
        // if splits.len() != 3 {
        //     return err_func("There are not three pipe-delimited segments.");
        // }
        let source = splits[0].trim();
        let size = splits[1].trim();
        if !source.starts_with(CT_PREFIX_IMAGE_FOLDER) {
            return err_func(&format!("Image destination does not start with \"{}\".", CT_PREFIX_IMAGE_FOLDER));
        }
        let source = source[CT_PREFIX_IMAGE_FOLDER.len()..].to_string();
        let size = match size {
            CT_IMAGE_SIZE_STD | CT_IMAGE_SIZE_LARGE => ImageSize::DokuLarge,
            CT_IMAGE_SIZE_100_PCT => ImageSize::Original,
            _ => ImageSize::DokuLarge,
        };
        let source = ImageSource::new_internal(&self.namespace_main, &source);
        let link = Link::new_image(None, source, ImageAlignment::Left, size, ImageLinkType::Direct);
        Ok(link)
    }
    */
    /*
    fn clear_errors(&mut self) {
        self.errors.clear();
    }

    fn add_error(&mut self, topic_name: &str, msg: &str) {
        let topic_key = TopicKey::new(&self.namespace_main, topic_name);
        let entry = self.errors.entry(topic_key).or_insert(vec![]);
        entry.push(msg.to_string());
    }
    */
}

pub fn build_model(name: &str, namespace_main: &str, topic_limit: Option<usize>, attributes_to_index: Vec<&str>) -> Wiki {
    let mut bp = BuildProcess::new(name, namespace_main,gen::PATH_PAGES, topic_limit);
    let mut model = bp.build();
    model.attributes.attributes_to_index = attributes_to_index.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    model
}

/*
fn remove_brackets_rc(text: &str, context: &str) -> Result<String, String> {
    let text = text.trim();
    if !text.starts_with(CT_BRACKETS_LEFT) || !text.ends_with(CT_BRACKETS_RIGHT) {
        Err(format!("{} Malformed bracketed string \"{}\"", context, text))
    } else {
        Ok(util::parse::between_trim(text, CT_BRACKETS_LEFT, CT_BRACKETS_RIGHT).to_string())
    }
}
*/