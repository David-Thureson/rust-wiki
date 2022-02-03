use crate::*;
use crate::model::*;
use std::fs;
use super::*;

pub fn main() {
    let topic_limit = None;
    // let topic_limit = Some(20);
    build_model(gen_tools_wiki::PROJECT_NAME, &gen_tools_wiki::PROJECT_NAME.to_lowercase(), topic_limit);
}

struct BuildProcess {
    wiki_name: String,
    namespace_main: String,
    path_source: String,
    topic_refs: TopicRefs,
    errors: TopicErrorList,
    topic_limit: Option<usize>,
    topic_parse_state: TopicParseState,
}

struct TopicParseState {
    is_past_attributes: bool,
    is_past_first_header: bool,
    is_in_code: bool,
    is_in_non_code_marker: bool,
    marker_exit_string: Option<String>,
    is_past_real_sections: bool,
}

impl BuildProcess {
    pub(crate) fn new(wiki_name: &str, namespace_main: &str, path_source: &str, topic_limit: Option<usize>) -> Self {
        Self {
            wiki_name: wiki_name.to_string(),
            namespace_main: namespace_main.to_string(),
            path_source: path_source.to_string(),
            topic_refs: Default::default(),
            errors: TopicErrorList::new(),
            topic_limit,
            topic_parse_state: TopicParseState::new(),
        }
    }

    pub(crate) fn build(&mut self) -> Model {
        let mut model = Model::new(&self.wiki_name, &self.namespace_main);
        let namespace_main = self.namespace_main.clone();
        let namespace_book = model.namespace_book();
        model.add_namespace(&namespace_book);

        // let topic_limit_per_namespace = self.topic_limit.map(|topic_limit| topic_limit / 2);
        self.parse_from_folder(&mut model, &namespace_main, self.topic_limit);
        // self.parse_from_folder(&mut model, &namespace_book, topic_limit_per_namespace);
        assert!(!model.get_topics().is_empty());

        self.topic_refs = model.get_topic_refs().clone();

        // Figure out the real nature of each paragraph.
        self.refine_paragraphs(&mut model);

        // model.catalog_links();
        check_links(&model, &mut self.errors);
        //bg!(model.get_topics().keys());
        // It's not necessary to check whether parents link to subtopics, since those links will be
        // generated.
        // self.check_subtopic_relationships(&mut model);
        self.errors.print(Some("First pass"));
        self.errors.list_missing_topics();

        // modelReport::new().categories().paragraphs().attributes().lists().go(&model);
        // report_category_tree(&model);
        // model.catalog_possible_list_types().print_by_count(0, None);

        model.add_missing_category_topics();
        // model.catalog_links();
        self.errors.clear();
        check_links(&model, &mut self.errors);
        self.errors.print(Some("After adding missing category topics."));
        // Call the make tree functions after the last call to model.catalog_links().
        model.make_category_tree();
        model.make_subtopic_tree();
        //bg!(&model.attributes);
        let attr_errors = model.catalog_attributes();
        attr_errors.print(Some("model.catalog_attributes()"));
        if attr_errors.is_empty() {
            //report_attributes(&model);
        }
        model.catalog_domains();
        model
    }

    fn parse_from_folder(&mut self, model: &mut Model, namespace_name: &str, topic_limit: Option<usize>) {
        // Read each page's text file and read it as a topic, then break each topic into
        // paragraphs. At this point we don't care about whether the paragraphs are plain or mixed
        // text, attribute tables, section headers, breadcrumbs, etc.
        TopicKey::assert_legal_namespace(namespace_name);
        let mut topic_count = 0;
        let path_source = format!("{}/{}", self.path_source, gen::namespace_to_path(namespace_name));
        for dir_entry_result in fs::read_dir(path_source).unwrap() {
            let dir_entry = dir_entry_result.as_ref().unwrap();
            let file_name = util::file::dir_entry_to_file_name(dir_entry);
            if file_name.ends_with(".txt") {
                let content = fs::read_to_string(&dir_entry.path()).unwrap();
                assert_no_extra_lines(&file_name, &content);
                let mut paragraphs = content.split(DELIM_PARAGRAPH).collect::<Vec<_>>();
                // The first paragraph should have the topic name as a page header, like:
                //   ======A Mind for Numbers======
                let mut topic_name = paragraphs.remove(0).to_string();
                assert!(topic_name.starts_with(DELIM_HEADER));
                assert!(topic_name.ends_with(DELIM_HEADER));
                topic_name = topic_name.replace(DELIM_HEADER, "").trim().to_string();
                //bg!(&topic_name);
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

    fn refine_paragraphs(&mut self, model: &mut Model) {
        for topic in model.get_topics_mut().values_mut() {
            let context = format!("Refining paragraphs for \"{}\".", topic.get_name());
            self.topic_parse_state = TopicParseState::new();
            //rintln!("\n==================================================================\n\n{}\n", context);
            let paragraph_count = topic.get_paragraph_count();
            for paragraph_index in 0..paragraph_count {
                match self.refine_one_paragraph_rc(topic, paragraph_index, &context) {
                    Err(msg) => {
                        let topic_key = TopicKey::new(&self.namespace_main, topic.get_name());
                        self.errors.add(&topic_key, &msg);
                    },
                    _ => (),
                }
                // if self.topic_parse_state.is_past_real_sections {
                    // We've finished with the more or less hand-written part of the page and are
                    // now in the fully generated sections like "Inbound Links". We don't want to
                    // parse these generated sections and include them in the model because they'll
                    // be created automatically.
                    //break;
                // }
            }
            topic.assert_all_text_blocks_resolved();
            self.topic_parse_state.check_end_of_topic();
        }
    }

    fn refine_one_paragraph_rc(&mut self, topic: &mut Topic, paragraph_index: usize, context: &str) -> Result<(), String> {
        let source_paragraph = topic.replace_paragraph_with_placeholder(paragraph_index);
        // Check whether we've finished with the more or less hand-written part of the page and are
        // now in the fully generated sections like "Inbound Links". We don't want to parse these
        // generated sections and include them in the model because they'll be created
        // automatically. So if that's the case, leave the placeholder paragraph in place.
        if !self.topic_parse_state.is_past_real_sections {
            match source_paragraph {
                Paragraph::Unknown { text } => {
                    let text = util::parse::trim_linefeeds(&text);
                    if !(self.paragraph_as_category_rc(topic, &text, context)?
                        || self.paragraph_as_section_header_rc(topic, &text, paragraph_index, context)?
                        || self.paragraph_as_breadcrumb_rc(topic, &text, context)?
                        || self.paragraph_as_marker_start_or_end_rc(topic, &text, paragraph_index, context)?
                        || self.paragraph_as_table_rc(topic, &text, paragraph_index, context)?
                        || self.paragraph_as_list_rc(topic, &text, paragraph_index, context)?
                        || self.paragraph_as_text_rc(topic, &text, paragraph_index, context)?
                    ) {
                        panic!("Unable to resolve paragraph.")
                        // let new_paragraph = Paragraph::new_text_unresolved(&text);
                        // topic.replace_paragraph(paragraph_index, new_paragraph);
                    }
                    return Ok(());
                },
                _ => {},
            };
            // if topic.get_name().contains("Zero") { dbg!("new_paragraph", &new_paragraph.get_variant_name()); }
            // topic.paragraphs[paragraph_index] = new_paragraph;
            topic.replace_paragraph(paragraph_index, source_paragraph);
        }
        Ok(())
    }

    fn paragraph_as_category_rc(&mut self, topic: &mut Topic, text: &str, context: &str) -> Result<bool, String> {
        // If it's a category line it will look like this if it already has a link:
        //   Category: [[tools:nonfiction_books|Nonfiction Books]]
        // or like this if it does not yet have a link (which will be added during the re-gen
        // process we're in):
        //   Category: Nonfiction Books
        if self.topic_parse_state.is_in_code || self.topic_parse_state.is_in_non_code_marker {
            return Ok(false);
        }
        let context = &format!("{} Seems to be a category paragraph.", context);
        let err_func = |msg: &str| Err(format!("{} paragraph_as_category_rc: {}: text = \"{}\".", context, msg, text));
        if text.trim().starts_with(PREFIX_CATEGORY) {
            if text.trim().contains(DELIM_LINEFEED) {
                return err_func("The text seems to be a category format but it has linefeeds.");
            } else {
                let category_part = util::parse::after(text, PREFIX_CATEGORY).trim().to_string();
                match parse_link_optional(&self.topic_refs,&category_part) {
                    Ok(Some(link)) => {
                        match link.get_label() {
                            Some(label) => {
                                let category_name = label;
                                //rintln!("\"{}\" in \"{}\"", topic.get_name(), category_name);
                                topic.set_category(&category_name);
                                return Ok(true);
                            },
                            None => {
                                return err_func("Expected the link to have a label which is the category name.");
                            },
                        }
                    },
                    Ok(None) => {
                        let category_name = category_part;
                        println!("\"{}\" in \"{}\"", topic.get_name(), category_name);
                        topic.set_category(&category_name);
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
        // if text.starts_with("=LEFT(G6") { //bg!(text, self.in_code, self.in_non_code_marker, &self.marker_exit_string); }
        if self.topic_parse_state.is_in_code || self.topic_parse_state.is_in_non_code_marker {
            return Ok(false);
        }
        let context = &format!("{} Seems to be a section header paragraph.", context);
        let err_func = |msg: &str| Err(format!("{} paragraph_as_section_header_rc: {}: text = \"{}\".", context, msg, text));
        match parse_header_optional(text) {
            Ok(Some((name, depth))) => {
                //bg!(&name, depth);
                if depth > 0 {
                    self.topic_parse_state.is_past_first_header = true;
                }
                if name.eq(HEADLINE_LINKS) {
                    self.topic_parse_state.is_past_real_sections = true;
                } else {
                    topic.replace_paragraph(paragraph_index, Paragraph::new_section_header(&name, depth));
                    //if topic.get_name().contains("A2") { dbg!(text, &name, depth); panic!() }
                    //bg!(topic.get_name(), text, &name, depth);
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

    fn paragraph_as_breadcrumb_rc(&mut self, topic: &mut Topic, text: &str, context: &str) -> Result<bool, String> {
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
        if self.topic_parse_state.is_in_code || self.topic_parse_state.is_in_non_code_marker {
            return Ok(false);
        }
        let context = &format!("{} Seems to be a breadcrumb paragraph.", context);
        let err_func = |msg: &str| Err(format!("{} paragraph_as_breadcrumb_rc: {}: text = \"{}\".", context, msg, text));
        match parse_breadcrumb_optional(text) {
            Ok(Some(parent_topic_keys)) => {
                //bg!(&parent_topic_keys);
                let parent_links = parent_topic_keys.iter()
                    .map(|topic_key| r!(Link::new_topic(None, topic_key.get_namespace(), topic_key.get_topic_name())))
                    .collect::<Vec<_>>();
                topic.set_parents(parent_links);
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

    fn paragraph_as_marker_start_or_end_rc(&mut self, topic: &mut Topic, text: &str, paragraph_index: usize, context: &str) -> Result<bool, String> {
        // A quote might be multiple paragraphs. It will have markers preceeding and following
        // these paragraphs which themselves are paragraphs consisting of:
        //   <WRAP round box>
        //   </WRAP>
        // Likewise the code markers are separate paragraphs with:
        //   <code>;
        //   </code>;
        // Or it might specify the language, like "<code rust>". Other markers are "<html>" and
        // "<php>".
        // let debug = topic.get_name().eq("QuickBooks");
        // if debug { //rintln!("\n==================================================================\n"); }
        // if debug { //bg!(self.in_code, self.in_non_code_marker, self.marker_exit_string.as_ref(), text); }
        if self.topic_parse_state.is_in_code || self.topic_parse_state.is_in_non_code_marker {
            if text.trim().eq(*&self.topic_parse_state.marker_exit_string.as_ref().unwrap()) {
                topic.replace_paragraph(paragraph_index, Paragraph::new_marker(&text));
                self.topic_parse_state.is_in_code = false;
                self.topic_parse_state.is_in_non_code_marker = false;
                self.topic_parse_state.marker_exit_string = None;
                // if debug { //bg!(self.in_code, self.in_non_code_marker, self.marker_exit_string.as_ref()); }
                return Ok(true);
            }
        }
        let context = &format!("{} Seems to be a marker start or end paragraph.", context);
        let err_func = |msg: &str| Err(format!("{} paragraph_as_marker_start_or_end_rc: {}: text = \"{}\".", context, msg, text));
        let text = text.trim();
        match parse_marker_optional(text) {
            Ok(Some((text, marker_exit_string))) => {
                topic.replace_paragraph(paragraph_index, Paragraph::new_marker(&text));
                if marker_exit_string.eq(MARKER_CODE_END) {
                    self.topic_parse_state.is_in_code = true;
                } else {
                    self.topic_parse_state.is_in_non_code_marker = true;
                }
                self.topic_parse_state.marker_exit_string = Some(marker_exit_string);
                // if debug { //bg!(self.in_code, self.in_non_code_marker, self.marker_exit_string.as_ref()); }
                Ok(true)
            },
            Ok(None) => {
                // Not a marker paragraph, but also not an error.
                Ok(false)
            },
            Err(msg) => {
                return err_func(&msg);
            }
        }
    }

    fn paragraph_as_table_rc(&mut self, topic: &mut Topic, text: &str, paragraph_index: usize, context: &str) -> Result<bool, String> {
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
        if self.topic_parse_state.is_in_code || self.topic_parse_state.is_in_non_code_marker {
            return Ok(false);
        }
        let context = &format!("{} Seems to be a table paragraph.", context);
        let err_func = |msg: &str| Err(format!("{} paragraph_as_table_rc: {}: text = \"{}\".", context, msg, text));
        match parse_table_optional(text) {
            Ok(Some(temp_table)) => {
                // if text.contains("tools:nav:dates|Added") { dbg!(text, &temp_table, temp_table.has_header, temp_table.get_column_count(), self.topic_parse_state.is_past_attributes, self.topic_parse_state.is_past_first_header); }
                //bg!(&table);
                if !self.topic_parse_state.is_past_attributes && !self.topic_parse_state.is_past_first_header && !temp_table.has_header() && temp_table.get_column_count() == 2 {
                    // For now assume this is a table of attributes.
                    for row in temp_table.get_rows().iter() {
                        let text = row[0].get_text_block().get_unresolved_text();
                        let attr_type_name = text_or_topic_link_label(&text)?;
                        //bg!(&attr_type_name);
                        AttributeType::assert_legal_attribute_type_name(&attr_type_name);
                        let mut attr_values = vec![];
                        // let cell_items = row[1].text.split(",").collect::<Vec<_>>();
                        let text = row[1].get_text_block().get_unresolved_text();

                        // We want to split the attribute values using commas, but commas might be
                        // part of a quoted string or inside a link, and in those cases we want to
                        // avoid them during the split.
                        // Replace any commas inside a quoted string with a placeholder.
                        let text = util::parse::replace_within_delimiters_rc(&text,"\"", "\"", ",", TEMP_COMMA, context).unwrap();
                        // Replace any commas inside a link with a placeholder.
                        let text = util::parse::replace_within_delimiters_rc(&text,DELIM_LINK_START, DELIM_LINK_END, ",", TEMP_COMMA, context).unwrap();
                        // Split the attribute values using the remaining commas, if any.
                        let cell_items = util::parse::split_trim(&text, ",");
                        // Put the commas back inside the quoted strings and links.
                        let cell_items = cell_items.iter().map(|item| item.replace(TEMP_COMMA, ",")).collect::<Vec<_>>();
                        // if topic.get_name().starts_with("Bayesian") { //bg!(topic.get_name(), &cell_items); }
                        // let cell_items = util::parse::split_outside_of_delimiters_rc(&text, ",", "\"", "\"", context).unwrap();

                        for cell_item in cell_items.iter() {
                            let value = text_or_topic_link_label(cell_item)?;
                            AttributeType::assert_legal_attribute_value(&value);
                            attr_values.push(value);
                        }
                        //bg!(&attr_values);
                        topic.add_temp_attribute_values(attr_type_name, attr_values);
                    }
                    self.topic_parse_state.is_past_attributes = true;
                } else {
                    // Assume this is a normal (non-attribute) table.
                    let mut table = Table::new(temp_table.assume_has_header());
                    for temp_row in temp_table.get_rows().iter() {
                        let mut cells = vec![];
                        for temp_cell in temp_row.iter() {
                            let text = temp_cell.get_text_block().get_unresolved_text();
                            //bg!(topic.get_name(), &text);
                            let text_block = self.make_text_block_rc(&text, context)?;
                            cells.push(TableCell::new_text_block(text_block, temp_cell.is_bold(), &temp_cell.get_horizontal()));
                        }
                        table.add_row(cells);
                    }
                    if text.contains("tools:nav:dates|Added") { dbg!(&table); panic!() }
                    let paragraph = Paragraph::new_table(table);
                    //bg!(&paragraph);
                    topic.replace_paragraph(paragraph_index, paragraph);
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

    fn paragraph_as_list_rc(&mut self, topic: &mut Topic, text: &str, paragraph_index: usize, context: &str) -> Result<bool, String> {
        // Example with two levels (the first level has one space before the asterisk):
        // Projects:
        //   * [[Android]]
        //    * [[Algomator]]
        //    * [[Sensor (coding project)]]
        //   * [[Windows]]
        //    * [[By the Numbers]]
        //    * [[Genealogy (coding project)]]
        if self.topic_parse_state.is_in_code {
            return Ok(false);
        }
        let context = &format!("{} Seems to be a list paragraph.", context);
        let err_func = |msg: &str| Err(format!("{} paragraph_as_list_rc: {}: text = \"{}\".", context, msg, text));
        match parse_list_optional(text) {
            Ok(Some(list)) => {
                // If the list is one of the generated types like "Subcategories" or "All Topics"
                // we don't want to parse it and add it to the model. It will be generated later
                // automatically.
                if !list.get_type().is_generated() {
                    // Resolve links and such within the the header, if any.
                    let resolved_header = if let Some(unresolved_header) = list.get_header() {
                        let resolved_header = self.make_text_block_rc(&unresolved_header.get_unresolved_text(), context)?;
                        Some(resolved_header)
                    } else {
                        None
                    };
                    let mut resolved_list = List::new(list.get_type().clone(), resolved_header);
                    for list_item in list.get_items() {
                        let resolved_text_block = self.make_text_block_rc(&list_item.get_text_block().get_unresolved_text(), context)?;
                        let resolved_list_item = ListItem::new(list_item.get_depth(), list_item.is_ordered(), resolved_text_block);
                        resolved_list.add_item(resolved_list_item);
                    }
                    let paragraph = Paragraph::new_list(resolved_list);
                    topic.replace_paragraph(paragraph_index, paragraph);
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

    fn paragraph_as_text_rc(&self, topic: &mut Topic, text: &str, paragraph_index: usize, context: &str) -> Result<bool, String> {
        let context = &format!("{} Seems to be a text paragraph.", context);
        let text_block = if self.topic_parse_state.is_in_code {
            // We're in code, so take the text exactly as it is rather than looking for things like
            // links.
            let text_item = TextItem::new_text(text);
            TextBlock::new_resolved(vec![text_item])
        } else {
            self.make_text_block_rc(text, context)?
        };
        let paragraph = Paragraph::new_text(text_block);
        topic.replace_paragraph(paragraph_index, paragraph);
        Ok(true)
    }

    fn make_text_block_rc(&self, text: &str, context: &str) -> Result<TextBlock, String> {
        //bg!(context);
        // if text.contains("tools:nav") { dbg!(context, text); panic!() };
        let text = text.trim();
        // An image link will look like this:
        //   {{tools:antlr_plugin.png?direct}}
        // To make it easier to split the text into link and non-link parts, first change text like
        // the above to:
        //   [[{{tools:antlr_plugin.png?direct}}]]
        // This way all we have to do is split based on what is inside or outside of pairs of "[["
        // and "]]".
        let text = text.replace(DELIM_IMAGE_START, TEMP_DELIM_IMG_START)
            .replace(DELIM_IMAGE_END, TEMP_DELIM_IMG_END);
        let delimited_splits = util::parse::split_delimited_and_normal_rc(&text, DELIM_LINK_START, DELIM_LINK_END, context)?;
        // if text.contains("format|Num-Format") { dbg!(&delimited_splits); panic!(); }
        let mut items = vec![];
        for (item_is_delimited, item_text) in delimited_splits.iter() {
            if *item_is_delimited {
                //bg!(&item_text);
                // Assume it's an internal or external link, or an image link.
                let link_text = if item_text.starts_with(DELIM_IMAGE_START) {
                    item_text.clone()
                } else {
                    // Put the brackets back on since the parsing function will expect them.
                    format!("{}{}{}", DELIM_LINK_START, item_text, DELIM_LINK_END)
                };
                //bg!(&link_text);
                let link = self.make_link_rc(&link_text, context)?;
                items.push(TextItem::new_link(link));
            } else {
                // Assume it's plain text.
                items.push(TextItem::new_text(item_text));
            }
        }
        let text_block = TextBlock::new_resolved(items);
        Ok(text_block)
    }

    fn make_link_rc(&self, text: &str, context: &str) -> Result<Link, String> {
        let text = text.trim();
        let err_func = |msg: &str| Err(format!("{} make_link_rc: {}: text = \"{}\".", context, msg, text));
        //bg!(context, text);
        match parse_link_optional(&self.topic_refs, &text)? {
            Some(link) => Ok(link),
            None => err_func("parse_link_optional didn't think it was a link."),
        }
    }

    /*
    fn clear_errors(&mut self) {
        self.errors.clear();
    }
    */

    /*
    fn add_error(&mut self, topic_name: &str, msg: &str) {
        let topic_key = TopicKey::new(&self.namespace_main, topic_name);
        let entry = self.errors.entry(topic_key).or_insert(vec![]);
        entry.push(msg.to_string());
    }

     */
}

pub(crate) fn build_model(name: &str, namespace_main: &str, topic_limit: Option<usize>) -> Model {
    let mut bp = BuildProcess::new(name, namespace_main,PATH_PAGES, topic_limit);
    let model = bp.build();
    model
}

impl TopicParseState {
    fn new() -> Self {
        Self {
            is_past_attributes: false,
            is_past_first_header: false,
            is_in_code: false,
            is_in_non_code_marker: false,
            marker_exit_string: None,
            is_past_real_sections: false,
        }
    }

    fn check_end_of_topic(&self) {
        assert!(!self.is_in_code);
        assert!(!self.is_in_non_code_marker);
        assert!(self.marker_exit_string.is_none());
    }
}

fn assert_no_extra_lines(file_name: &str, content: &str) {
    let three_linefeeds = DELIM_LINEFEED.repeat(3);
    assert!(!content.contains(&three_linefeeds), "Page content has extra blank lines: {}", file_name);
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

pub(crate) fn complete_model(model: &mut Model) {
    model.catalog_links();

    let mut errors = TopicErrorList::new();
    check_links(model, &mut errors);
    errors.list_missing_topics();

    model.add_missing_category_topics();
    // model.catalog_links();
    errors.clear();
    check_links(&model, &mut errors);
    errors.print(Some("After adding missing category topics and checking links."));

    // Call the make tree functions after the last call to model.catalog_links().
    model.make_category_tree();
    model.make_subtopic_tree();
    //bg!(&model.attributes);
    let attr_errors = model.update_attributes();
    attr_errors.print(Some("model.catalog_attributes()"));
    if attr_errors.is_empty() {
        //report_attributes(&model);
    }
    model.catalog_domains();
}

fn check_links(model: &Model, errors: &mut TopicErrorList) {
    errors.append(&mut model.check_links());
}



