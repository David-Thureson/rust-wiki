use crate::*;
use crate::model::*;
use std::fs;
use super::*;
use crate::model::PUBLIC_ATTRIBUTES;
use std::collections::BTreeMap;
use crate::model::glossary::Glossary;

#[derive(Debug)]
pub(crate) struct BuildProcess {
    pub(crate) wiki_name: String,
    pub(crate) namespace_main: String,
    pub(crate) path_source: String,
    pub(crate) gen_path_pages: String,
    pub(crate) compare_only: bool,
    pub(crate) is_public: bool,
    pub(crate) topic_source_files: BTreeMap<String, TopicFile>,
    pub(crate) topic_dest_files: BTreeMap<String, TopicFile>,
    pub(crate) topic_files_to_delete: Vec<String>,
    pub(crate) topic_refs: TopicRefs,
    pub(crate) errors: TopicErrorList,
    pub(crate) topic_limit: Option<usize>,
    topic_parse_state: TopicParseState,
}

#[derive(Debug)]
pub(crate) struct TopicFile {
    pub(crate) namespace_name: String,
    pub(crate) file_name: String,
    pub(crate) topic_name: String,
    pub(crate) content: String,
}

#[derive(Debug)]
struct TopicParseState {
    is_past_attributes: bool,
    is_past_first_header: bool,
    is_in_code: bool,
    is_in_non_code_marker: bool,
    marker_exit_string: Option<String>,
    is_past_real_sections: bool,
    is_debug: bool,
}

impl BuildProcess {
    pub(crate) fn new(wiki_name: &str, namespace_main: &str, path_source: &str, compare_only: bool, is_public: bool, topic_limit: Option<usize>) -> Self {
        Self {
            wiki_name: wiki_name.to_string(),
            namespace_main: namespace_main.to_string(),
            path_source: path_source.to_string(),
            gen_path_pages: "".to_string(),
            compare_only,
            is_public,
            topic_source_files: Default::default(),
            topic_dest_files: Default::default(),
            topic_files_to_delete: vec![],
            topic_refs: Default::default(),
            errors: TopicErrorList::new(),
            topic_limit,
            topic_parse_state: TopicParseState::new(),
        }
    }

    pub(crate) fn build(&mut self, project: Option<file_monitor::model::Project>) -> Model {
        let mut model = Model::new(&self.wiki_name, &self.namespace_main, self.is_public);

        if let Some(project) = project {
            model.set_file_monitor_project(project);
        }

        let namespace_main = self.namespace_main.clone();
        let namespace_book = model.namespace_book();
        model.add_namespace(&namespace_book);

        // Fill self.topic_source_files with the raw content of the topics found in files, if
        // necessary excluding non-public topics.
        // let topic_limit_per_namespace = self.topic_limit.map(|topic_limit| topic_limit / 2);
        self.read_from_folder(&mut model, &namespace_main, self.topic_limit);
        // self.parse_from_folder(&mut model, &namespace_book, topic_limit_per_namespace);
        assert!(!self.topic_source_files.is_empty());

        if self.is_public {
            model.finalize_redacted_phrases();
            // model.print_redacted_phrases();

            // Remove any topic source files whose title contains a redacted phrase. This will
            // handle cases like a combination topic marked as Public in which one or both of the
            // parent topics are marked Private.
            let dbg_topic_count_before = self.topic_source_files.len();
            // self.topic_source_files.retain(|topic_file| !redaction::text_contains_phrase(&topic_file.topic_name, model.get_redacted_phrases()));
            let mut filtered_source_files = BTreeMap::new();
            for (key, topic_file) in self.topic_source_files.drain_filter(|_k, _v| true) {
                if redaction::text_contains_phrase(&topic_file.topic_name, model.get_redacted_phrases()) {
                    // Remove this topic.
                    self.topic_files_to_delete.push(key);
                } else {
                    // Keep this topic.
                    filtered_source_files.insert(key, topic_file);
                }
            }
            std::mem::swap(&mut self.topic_source_files, &mut filtered_source_files);
            let dbg_topic_count_after = self.topic_source_files.len();
            println!("BuildProcess::build(): started with {} topics, ended with {}.", dbg_topic_count_before, dbg_topic_count_after);
        }

        // Turn the raw file content into topics in the model.
        self.parse_topics(&mut model);
        let dbg_topic_source_file_count = self.topic_source_files.len();
        let dbg_topic_count = model.get_topics().len();
        let dbg_topic_ref_count = model.get_topic_refs().len();
        //bg!(dbg_topic_source_file_count, dbg_topic_count, dbg_topic_ref_count);
        assert_eq!(dbg_topic_source_file_count, dbg_topic_count, "dbg_topic_source_file_count = {}, dbg_topic_count = {}", dbg_topic_source_file_count, dbg_topic_count);
        assert_eq!(dbg_topic_source_file_count, dbg_topic_ref_count, "dbg_topic_source_file_count = {}, dbg_topic_ref_count = {}", dbg_topic_source_file_count, dbg_topic_ref_count);

        self.topic_refs = model.get_topic_refs().clone();

        // Figure out the real nature of each paragraph.
        self.refine_paragraphs(&mut model);

        model.build_glossaries();

        if !self.is_public {
            tools_wiki::project::add_project_info_to_model(&mut model);
            tools_wiki::project::update_projects_and_libraries(&mut model);
        }

        // if !model.is_public() {
        //     model.remove_non_public_parent_topic_refs();
        // }

        // if model.is_public() {
        //     model.remove_non_public_topics();
        // }

        // model.catalog_links();
        check_links(&model, &mut self.errors);
        //bg!(model.get_topics().keys());
        // It's not necessary to check whether parents link to subtopics, since those links will be
        // generated.
        // self.check_subtopic_relationships(&mut model);
        // if !model.is_public() {
            self.errors.print_and_list_missing_topics(Some("First pass"));
        // }

        // modelReport::new().categories().paragraphs().attributes().lists().go(&model);
        // report_category_tree(&model);
        // model.catalog_possible_list_types().print_by_count(0, None);

        if !model.is_public() {
            model.add_missing_category_topics();
        }
        // model.catalog_links();
        self.errors.clear();
        check_links(&model, &mut self.errors);
        self.errors.print(Some("After adding missing category topics."));
        // Call the make tree functions after the last call to model.catalog_links().
        model.make_category_tree();
        model.make_subtopic_tree();
        if !self.is_public {
            // One-time cleanup. Remove Edited attributes that have the same date as Added.
            // model.remove_edited_same_as_added();
            model.update_attributes_from_file_monitor();
        }

        // One-time fix.
        // remove_edited_attribute_from_private_topics(&mut model);

        if !self.is_public {
            model.add_visibility_attributes();
        }
        //bg!(&model.attributes);
        let attr_errors = model.catalog_attributes();
        attr_errors.print(Some("model.catalog_attributes()"));
        if attr_errors.is_empty() {
            //report_attributes(&model);
        }
        model.catalog_domains();
        model
    }

    fn read_from_folder(&mut self, model: &mut Model, namespace_name: &str, topic_limit: Option<usize>) {
        // Read each page's text file. If this is a public build and the topic is not public,
        // add that file name and topic name to the list of redacted phrases but otherwise don't
        // include the topic in the build.
        let mut errors = vec![];
        TopicKey::assert_legal_namespace(namespace_name);
        let mut topic_count = 0;
        let path_source_relative = format!("/{}", gen::namespace_to_path(namespace_name));
        let path_source = format!("{}{}", self.path_source, path_source_relative);
        for dir_entry_result in fs::read_dir(path_source).unwrap() {
            let dir_entry = dir_entry_result.as_ref().unwrap();
            let file_name = util::file::dir_entry_to_file_name(dir_entry);
            if file_name.ends_with(".txt") {
                //let path_name = util::file::path_name(dir_entry.path());
                let content_raw = fs::read_to_string(&dir_entry.path()).unwrap();

                // Get rid of any lines that have only whitespace, as this may confuse the parsing
                // code. Also get rid of any whitespace at the end of non-empty lines.
                let content = content_raw.split(DELIM_LINEFEED)
                    .map(|line| line.trim_end())
                    .join(DELIM_LINEFEED);

                // if content.contains("“") { dbg!(&file_name); panic!() }
                // let content = content.replace("‘", "'");
                // let content = content.replace("’", "'");
                // let content = content.replace("“", "\"");
                // let content = content.replace("”", "\"");

                if content.contains(MARKER_DELETE_THIS_FILE) {
                    errors.push(format!("{} should be deleted.", file_name));
                }
                let topic_name_line = util::parse::before(&content, DELIM_LINEFEED);
                assert!(topic_name_line.starts_with(DELIM_HEADER), "Topic name \"{}\" should start with \"{}\".", &topic_name_line, DELIM_HEADER);
                assert!(topic_name_line.ends_with(DELIM_HEADER), "Topic name \"{}\" should end with \"{}\".", &topic_name_line, DELIM_HEADER);
                let topic_name = topic_name_line.replace(DELIM_HEADER, "").trim().to_string();

                // If we're doing a public-only build we don't even want to read the file unless
                // it's explicitly tagged as a public topic.
                if model.is_public() && !content.contains(MARKER_PUBLIC_IN_TEXT_FILE) {
                    // This is a private topic.
                    model.add_redacted_phrase(topic_name.clone());
                    let file_name_no_extension = util::parse::before(&file_name, ".txt").to_string();
                    let topic_ref = format!("{}:{}", namespace_name, file_name_no_extension);
                    model.add_redacted_phrase(file_name_no_extension);
                    model.add_redacted_phrase(topic_ref);
                    let topic_file_key = make_topic_file_key(namespace_name, &file_name);
                    assert!(!self.topic_files_to_delete.contains(&topic_file_key));
                    self.topic_files_to_delete.push(topic_file_key);
                } else {
                    let topic_source_file = TopicFile::new(namespace_name, &file_name, &topic_name, content);
                    self.add_topic_source_file(topic_source_file);
                    topic_count += 1;
                    if topic_limit.map_or(false, |topic_limit| topic_count >= topic_limit) {
                        return;
                    }
                }
            }
        }
        if !errors.is_empty() {
            println!("\nErrors from read_from_folder():");
            for msg in errors.iter() {
                println!("\t{}", msg);
            }
            panic!()
        }
    }

    fn parse_topics(&mut self, model: &mut Model) {
        // Turn the raw file content into topics in the model. This includes redaction (for a
        // public build) and breaking each file's content into paragraphs. At this point we don't
        // care about whether the paragraphs are plain or mixed text, attribute tables, section
        // headers, breadcrumbs, etc.

        for topic_source_file in self.topic_source_files.values() {
            let content = topic_source_file.content.clone();

            // Double linefeeds are fine since they count as paragraph breaks, but any
            // linefeeds after that should be removed.
            let mut content = util::format::remove_repeated_n(&content, "\n", 2);
            assert_no_extra_lines(&topic_source_file.file_name, &content);

            if model.is_public() {
                if let Some(mut new_content) = redaction::redact_text(&content, model.get_redacted_phrases()) {
                    //rintln!("BuildProcess::parse_topics(): redactions in \"{}\".", topic_source_file.topic_name);
                    std::mem::swap(&mut content, &mut new_content);
                } else {
                    //rintln!("BuildProcess::parse_topics(): \t\t\t***** NO REDACTIONS IN ***** \"{}\".", topic_source_file.topic_name);
                }
            }

            let mut paragraphs = content.split(DELIM_PARAGRAPH).collect::<Vec<_>>();
            // The first paragraph should have the topic name as a page header, like:
            //   ======A Mind for Numbers======
            // We already parsed this first line in read_from_folder() and we have the topic name,
            // so we don't need this paragraph.
            let first_paragraph = paragraphs.remove(0).to_string();
            // The first paragraph should be a single line.
            assert!(!first_paragraph.contains(DELIM_LINEFEED));

            let mut topic = Topic::new(&topic_source_file.namespace_name, &topic_source_file.topic_name);
            let mut line_index = 2;
            for paragraph in paragraphs.iter() {
                let lines_this_paragraph = paragraph.matches(DELIM_LINEFEED).count() + 2;
                topic.add_paragraph(Paragraph::new_unknown(line_index,paragraph));
                line_index += lines_this_paragraph;
            }
            model.add_topic(topic);
        }
    }

    fn refine_paragraphs(&mut self, model: &mut Model) {
        let mut glossaries = model.take_glossaries();
        for topic in model.get_topics_mut().values_mut() {
            let context = format!("Refining paragraphs for \"{}\".", topic.get_name());
            self.topic_parse_state = TopicParseState::new();
            // self.topic_parse_state.is_debug = topic.get_name().eq("DokuWiki Markup");
            //rintln!("\n==================================================================\n\n{}\n", context);
            let paragraph_count = topic.get_paragraph_count();
            for paragraph_index in 0..paragraph_count {
                match self.refine_one_paragraph_rc(topic, paragraph_index, &mut glossaries, &context) {
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
            //bg!(topic.get_name());
            self.topic_parse_state.check_end_of_topic(topic);
        }
        model.set_glossaries(glossaries);
    }

    fn refine_one_paragraph_rc(&mut self, topic: &mut Topic, paragraph_index: usize, glossaries: &mut GlossaryMap, context: &str) -> Result<(), String> {
        let source_paragraph = topic.replace_paragraph_with_placeholder(paragraph_index);
        // Check whether we've finished with the more or less hand-written part of the page and are
        // now in the fully generated sections like "Inbound Links". We don't want to parse these
        // generated sections and include them in the model because they'll be created
        // automatically. So if that's the case, leave the placeholder paragraph in place.
        if !self.topic_parse_state.is_past_real_sections {
            match source_paragraph {
                Paragraph::Unknown { line_index, text } => {
                    let text = util::parse::trim_linefeeds(&text);
                    if self.topic_parse_state.is_debug { println!("\nNEW PARAGRAPH{}\n|{}|\n", "=".repeat(80), &text); }
                    // For each of the calls in the next statement, the called function returns:
                    //   - Ok(true) to indicate that the paragraph was found to be of this type and
                    //     we can exit without calling any more paragraph_as_... functions.
                    //   - Ok(false) to indicate that the paragraph is not of this type so we
                    //     should keep going with further paragraph_as_... functions.
                    //   - Err(msg) if there was some problem parsing the paragraph text and we
                    //     need to quit trying to work on this paragraph and exit the current
                    //     function.
                    if !(self.paragraph_as_category_rc(topic, &text, context)?
                        || self.paragraph_as_section_header_rc(topic, &text, paragraph_index, context)?
                        || self.paragraph_as_breadcrumb_rc(topic, &text, context)?
                        || self.paragraph_as_marker_start_or_end_rc(topic, &text, paragraph_index, context)?
                        || self.paragraph_as_table_rc(topic, &text, paragraph_index, glossaries, context)?
                        || self.paragraph_as_list_rc(topic, &text, paragraph_index, context)?
                        || self.paragraph_as_text_rc(topic, line_index, &text, paragraph_index, context)?
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
                if self.is_public && text.contains(MARKER_REDACTION) {
                    // This appears to be a category reference to a private topic, or at least
                    // part of the referenced topic name is a redacted phrase, so leave this topic
                    // without a category. Returning Ok(true) means this paragraph has been
                    // handled and we don't want to keep trying to figure out what it is.
                    //rintln!("{}: Ignoring category reference with redaction: \"{}\".", context, text);
                    return Ok(true);
                }
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
                        //rintln!("\"{}\" in \"{}\"", topic.get_name(), category_name);
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
        match parse_breadcrumb_optional(text, context, self.is_public) {
            Ok(Some(parent_topic_keys)) => {
                // If the vector of topic keys is empty, that means we found the redaction marker,
                // so one or more of the parent references can't be used. In that case, leave the
                // topic without any parents. In this case we still want to return Ok(true) so that
                // we stop trying to parse this paragraph.
                let parent_links = parent_topic_keys.iter()
                    .map(|topic_key| r!(Link::new_topic_from_key(None, topic_key)))
                    .collect::<Vec<_>>();
                if !parent_links.is_empty() {
                    topic.set_parents(parent_links);
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
        //et debug = topic.get_name().eq("QuickBooks");
        //f debug { //rintln!("\n==================================================================\n"); }
        //f debug { //bg!(self.in_code, self.in_non_code_marker, self.marker_exit_string.as_ref(), text); }
        if self.topic_parse_state.is_debug { dbg!(self.topic_parse_state.is_in_code, self.topic_parse_state.is_in_non_code_marker); }
        if self.topic_parse_state.is_in_code || self.topic_parse_state.is_in_non_code_marker {
            if text.trim().eq(*&self.topic_parse_state.marker_exit_string.as_ref().unwrap()) {
                if self.topic_parse_state.is_debug { println!("\nparagraph_as_marker_start_or_end_rc(): Found marker exit string.\n"); }
                topic.replace_paragraph(paragraph_index, Paragraph::new_marker(&text));
                self.topic_parse_state.is_in_code = false;
                self.topic_parse_state.is_in_non_code_marker = false;
                self.topic_parse_state.marker_exit_string = None;
                //f debug { //bg!(self.in_code, self.in_non_code_marker, self.marker_exit_string.as_ref()); }
                return Ok(true);
            } else {
                // We're in a marker, but this paragraph is not the end marker, so don't process
                // this paragraph in the current function.
                return Ok(false);
            }
        }
        let context = &format!("{} Seems to be a marker start or end paragraph.", context);
        let err_func = |msg: &str| Err(format!("{} paragraph_as_marker_start_or_end_rc: {}: text = \"{}\".", context, msg, text));
        let text = text.trim();
        match parse_marker_optional(text) {
            Ok(Some((text, marker_exit_string))) => {
                if self.topic_parse_state.is_debug { println!("\nparagraph_as_marker_start_or_end_rc(): Found marker \"{}\", exit string = \"{}\".\n", text, marker_exit_string); }
                topic.replace_paragraph(paragraph_index, Paragraph::new_marker(&text));
                if marker_exit_string.eq(MARKER_CODE_END) {
                    self.topic_parse_state.is_in_code = true;
                } else {
                    self.topic_parse_state.is_in_non_code_marker = true;
                }
                self.topic_parse_state.marker_exit_string = Some(marker_exit_string);
                //f debug { //bg!(self.in_code, self.in_non_code_marker, self.marker_exit_string.as_ref()); }
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

    fn paragraph_as_table_rc(&mut self, topic: &mut Topic, text: &str, paragraph_index: usize, glossaries: &mut GlossaryMap, context: &str) -> Result<bool, String> {
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
        //et debug = topic.get_name().eq("Terms");
        //et debug = false;
        //f debug { dbg!(&text); }

        if self.topic_parse_state.is_in_code || self.topic_parse_state.is_in_non_code_marker {
            return Ok(false);
        }
        let context = &format!("{} Seems to be a table paragraph.", context);
        let err_func = |msg: &str| Err(format!("{} paragraph_as_table_rc: {}: text = \"{}\".", context, msg, text));
        match parse_table_optional(text) {
            Ok(Some(temp_table)) => {
                // if text.contains("tools:nav:dates|Added") { dbg!(text, &temp_table, temp_table.has_header, temp_table.get_column_count(), self.topic_parse_state.is_past_attributes, self.topic_parse_state.is_past_first_header); }
                //bg!(&table);
                //f debug { dbg!(&temp_table); }
                if !self.topic_parse_state.is_past_attributes && !self.topic_parse_state.is_past_first_header && !temp_table.has_header() && temp_table.get_column_count() == 2 {
                    // For now assume this is a table of attributes.
                    //f debug { println!("This is a table of attributes."); }
                    for row in temp_table.get_rows().iter() {
                        let text = row[0].get_text_block().get_unresolved_text();
                        let attr_type_name = text_or_topic_link_label(&text)?;
                        //bg!(&attr_type_name);
                        // If this is a public build, ignore attributes where the type name or text
                        // has been at least partially redacted. Also ignore certain attributes
                        // that don't go into a public build, like Visibility and contact
                        // information.
                        let mut use_this_attribute = true;
                        if self.is_public {
                            let is_attr_public = PUBLIC_ATTRIBUTES.contains(&&*attr_type_name);
                            //bg!(&attr_type_name, is_attr_public);
                            if !is_attr_public {
                                // if attr_type_name.ne("Visibility") {
                                //     println!("{}: Ignoring \"{}\" attribute because it's not in the public list.", context, attr_type_name);
                                // }
                                use_this_attribute = false;
                            }
                            if attr_type_name.contains(MARKER_REDACTION) {
                                println!("{}: Ignoring \"{}\" attribute because the type name contains a redaction.", context, attr_type_name);
                                use_this_attribute = false;
                            }
                        }
                        if use_this_attribute {
                            AttributeType::assert_legal_attribute_type_name(&attr_type_name);
                            let mut attr_values = vec![];
                            // let cell_items = row[1].text.split(",").collect::<Vec<_>>();
                            assert!(row.len() > 1, "row.len() = {}, context = {}", row.len(), context);
                            let text = row[1].get_text_block().get_unresolved_text();

                            if self.is_public && text.contains(MARKER_REDACTION) {
                                //rintln!("{}: Ignoring \"{}\" attribute because the values contain a redaction: \"{}\".", context, attr_type_name, text);
                            } else {
                                // We want to split the attribute values using commas, but commas might be
                                // part of a quoted string or inside a link, and in those cases we want to
                                // avoid them during the split.
                                // Replace any commas inside a quoted string with a placeholder.
                                let text = util::parse::replace_within_delimiters_rc(&text, "\"", "\"", ",", TEMP_COMMA, true, context).unwrap();
                                // Replace any commas inside a link with a placeholder.
                                let text = util::parse::replace_within_delimiters_rc(&text, DELIM_LINK_START, DELIM_LINK_END, ",", TEMP_COMMA, true, context).unwrap();
                                // Split the attribute values using the remaining commas, if any.
                                let cell_items = util::parse::split_trim(&text, ",");
                                // Put the commas back inside the quoted strings and links.
                                let cell_items = cell_items.iter().map(|item| item.replace(TEMP_COMMA, ",")).collect::<Vec<_>>();
                                // if topic.get_name().starts_with("Bayesian") { //bg!(topic.get_name(), &cell_items); }
                                // let cell_items = util::parse::split_outside_of_delimiters_rc(&text, ",", "\"", "\"", context).unwrap();

                                for cell_item in cell_items.iter() {
                                    let value = text_or_topic_link_label(cell_item)?.trim().to_string();
                                    assert!(!value.is_empty(), "In context \"{}\", attribute value is empty for \"{}\".", context, attr_type_name);
                                    AttributeType::assert_legal_attribute_value(&value);
                                    attr_values.push(value);
                                }
                                //bg!(&attr_values);
                                topic.add_temp_attribute_values(attr_type_name, attr_values);
                            }
                        }
                    }
                    self.topic_parse_state.is_past_attributes = true;
                } else {
                    // Assume this is a normal (non-attribute) table.
                    //f debug { println!("This is a regular table."); }
                    let mut table = Table::new(temp_table.assume_has_header());
                    for temp_row in temp_table.get_rows().iter() {
                        let mut cells = vec![];
                        for temp_cell in temp_row.iter() {
                            let text = temp_cell.get_text_block().get_unresolved_text().trim().to_string();
                            //bg!(topic.get_name(), &text);
                            let text_block = self.make_text_block_rc(&text, context)?;
                            cells.push(TableCell::new_text_block(text_block, temp_cell.is_bold(), &temp_cell.get_horizontal()));
                        }
                        table.add_row(cells);
                    }
                    if text.contains("tools:nav:dates|Added") { dbg!(&table); panic!() }
                    //f debug { dbg!(&table); }
                    let paragraph = if topic.get_name().eq(PAGE_NAME_TERMS) {
                        let glossary_name = PAGE_NAME_TERMS;
                        let glossary = Glossary::new_with_raw_list(Some(topic.get_topic_key()), table);
                        glossaries.insert(glossary_name.to_string(), glossary);
                        Paragraph::new_glossary(glossary_name)
                    } else {
                        Paragraph::new_table(table)
                    };
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
                // If the list is one of the generated types like "Subcategories" or "All topics"
                // we don't want to parse it and add it to the model. It will be generated later
                // automatically.
                //et debug = topic.get_name().eq("Software Projects");
                //et debug = text.contains("Tool Categories");
                //f debug { //bg!(&list); }
                if !list.is_generated() {
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
                        if self.is_public && resolved_text_block.is_redaction() {
                            //rintln!("{}: paragraph_as_list_rc(): ignoring fully redacted list item.", context);
                        } else {
                            let resolved_list_item = ListItem::new(list_item.get_depth(), list_item.is_ordered(), resolved_text_block);
                            resolved_list.add_item(resolved_list_item);
                        }
                    }
                    //f debug { //bg!(&resolved_list); }
                    // let paragraph = if resolved_list.is_empty() {
                        // There are no list items, most likely because all of them were redacted
                        // for the public build, so don't show the list at all.
                        // Paragraph::Placeholder
                    //} else {
                    let paragraph = Paragraph::new_list(resolved_list);
                    //};
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

    fn paragraph_as_text_rc(&self, topic: &mut Topic, line_index: usize, text: &str, paragraph_index: usize, context: &str) -> Result<bool, String> {
        let context = &format!("{} Seems to be a text paragraph starting at line {}.", context, line_index);
        // if topic.get_name().eq("Profisee Installs") { //rintln!("paragraph_as_text_rc: line {}: {}", line_index + 1, text); }
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
        // An image link will look like this:
        //   {{tools:antlr_plugin.png?direct}}
        // To make it easier to split the text into link and non-link parts, first change text like
        // the above to:
        //   [[{{tools:antlr_plugin.png?direct}}]]
        // This way all we have to do is split based on what is inside or outside of pairs of "[["
        // and "]]".
        //et debug = text.contains("[[tools:excel|Excel]] Data Science");
        //et debug = text.contains("Another term for a process virtual machine such");
        //et debug = text.contains("[[tools:excel|Excel]]");
        //f debug { dbg!(&text, context); }
        let text = text.replace(DELIM_IMAGE_START, TEMP_DELIM_IMG_START)
            .replace(DELIM_IMAGE_END, TEMP_DELIM_IMG_END);
        //f debug { dbg!(&text); }
        let delimited_splits = util::parse::split_delimited_and_normal_rc(&text, DELIM_LINK_START, DELIM_LINK_END, false, context)?;
        //f debug { dbg!(&delimited_splits); }
        // if text.contains("format|Num-Format") { dbg!(&delimited_splits); panic!(); }
        let mut items = vec![];
        for (item_is_delimited, item_text) in delimited_splits.iter() {
            if *item_is_delimited {
                //bg!(&item_text);
                // Assume it's an internal or external link, or an image link.
                if self.is_public && item_text.contains(MARKER_REDACTION) {
                    //rintln!("{}: make_text_block_rc(): removing a link with a redaction: \"{}\".", context, item_text);
                    // Replace the whole link with a simple text item consisting only of the
                    // redaction marker.
                    items.push(TextItem::new_redaction());
                } else {
                    let link_text = if item_text.starts_with(DELIM_IMAGE_START) {
                        item_text.clone()
                    } else {
                        // Put the brackets back on since the parsing function will expect them.
                        format!("{}{}{}", DELIM_LINK_START, item_text, DELIM_LINK_END)
                    };
                    //bg!(&link_text);
                    let link = self.make_link_rc(&link_text, context)?;
                    items.push(TextItem::new_link(link));
                }
            } else {
                // Assume it's plain text.
                items.push(TextItem::new_text_in_context(item_text, context));
            }
        }
        let text_block = TextBlock::new_resolved(items);
        //f debug { dbg!(&text_block, text_block.get_display_text()); panic!(); }
        Ok(text_block)
    }

    fn make_link_rc(&self, text: &str, context: &str) -> Result<Link, String> {
        // let text = text.trim();
        let err_func = |msg: &str| Err(format!("{} make_link_rc: {}: text = \"{}\".", context, msg, text));
        //bg!(context, text);
        match parse_link_optional(&self.topic_refs, &text)? {
            Some(link) => Ok(link),
            None => err_func("parse_link_optional didn't think it was a link."),
        }
    }

    pub(crate) fn add_topic_source_file(&mut self, topic_file: TopicFile) {
        let key = topic_file.get_key();
        assert!(!self.topic_source_files.contains_key(&key));
        self.topic_source_files.insert(key, topic_file);
    }

    #[allow(dead_code)]
    pub(crate) fn add_topic_dest_file(&mut self, topic_file: TopicFile) {
        let key = topic_file.get_key();
        assert!(!self.topic_dest_files.contains_key(&key));
        self.topic_dest_files.insert(key, topic_file);
    }

    pub(crate) fn write_main_topic_files(&self) {
        // let msg_prefix = "BuildProcess::write_main_topic_files(): ";
        assert!(!self.topic_dest_files.is_empty());

        for namespace in self.get_main_namespaces().iter() {
            let path_source_namespace = format!("{}/{}", self.path_source, namespace_to_path(namespace));
            let path_dest_namespace = format!("{}/{}", self.gen_path_pages, namespace_to_path(namespace));
            let path_temp_source = format!("{}/{}", PATH_TEMP_SOURCE, namespace_to_path(namespace));

            //rintln!("{}About to remove all files from the temp source folder [{}].",
            //         msg_prefix, path_temp_source);
            util::file::remove_files_r(&path_temp_source).unwrap();

            if path_source_namespace.eq(&path_dest_namespace) {
                // The source and destination paths are the same. This is the usual case when
                // compare_only is false. Move all of the files out of the source folder and into
                // the temp source folder.
                //rintln!("{}The source and destination are the same.\nAbout to move all files from [{}] to [{}].",
                //         msg_prefix, path_source_namespace, path_temp_source);
                // panic!();
                util::file::move_files_r(&path_source_namespace, &path_temp_source).unwrap();
            } else {
                // The source and destination paths are different. Typically this means
                // compare_only is true and the source folder is the main live Wiki folder, while
                // the destination is under [C:/Wiki Gen Backup]. So we want the destination folder
                // to start out empty (except for possible subfolders) and we want a temporary copy
                // of the source files.
                //rintln!("{}The source and destination are different.\nAbout to copy all files from [{}] to [{}].\nAbout to remove all files from [{}].",
                //         msg_prefix, path_source_namespace, path_temp_source, path_dest_namespace);
                // panic!();
                util::file::copy_folder_files_r(&path_source_namespace, &path_temp_source).unwrap();
                util::file::remove_files_r(&path_dest_namespace).unwrap();
            }
            // The destination folder should be empty, so it will only end up with those files that
            // are currently in self.topic_dest_files. If a given file has changed, simply write it
            // to the destination. If it hasn't changed, move it from the temporary source folder.
            // This way we won't change the timestamp on unchanged files, which would mess up the
            // file-monitor process.
            for (key, topic_file_dest) in self.topic_dest_files.iter()
                    .filter(|(_key, topic_file)| topic_file.namespace_name.eq(namespace)) {
                let path_one_dest = format!("{}/{}.txt", path_dest_namespace, topic_file_dest.file_name);
                let mut is_changed = true;
                //bg!(&topic_file_dest);
                //self.print_source_file_keys();
                //bg!(&key);
                if let Some(topic_file_source) = self.topic_source_files.get(key) {
                    //bg!(&topic_file_source);
                    // This file/topic existed before the round trip. See if it has changed.
                    is_changed = topic_file_source.content.ne(&topic_file_dest.content);
                    //bg!(is_changed);
                }
                //panic!();
                if is_changed {
                    //rintln!("{}The topic \"{}\" is new or has been changed during the round trip. About to write [{}].",
                    //         msg_prefix, topic_file_dest.topic_name, path_one_dest);
                    util::file::write_file_r(&path_one_dest, &topic_file_dest.content).unwrap();
                } else {
                    let path_one_source = format!("{}/{}.txt", path_temp_source, topic_file_dest.file_name);
                    //rintln!("{}The topic \"{}\" has not been changed during the round trip. About to copy [{}] to [{}].",
                    //         msg_prefix, topic_file_dest.topic_name, path_one_source, path_one_dest);
                    assert_ne!(path_one_source, path_one_dest);
                    util::file::copy_file_r(path_one_source, path_one_dest).unwrap();
                }
            }
        }
    }

    fn get_main_namespaces(&self) -> Vec<String> {
        let mut namespaces = self.topic_dest_files.values()
            .map(|topic_file| topic_file.namespace_name.to_string())
            .collect::<Vec<_>>();
        namespaces.sort();
        namespaces.dedup();
        namespaces
    }

    #[allow(dead_code)]
    fn print_source_file_keys(&self) {
        println!("\nBuildProcess::print_source_file_keys()");
        for key in self.topic_source_files.keys() {
            println!("\t\"{}\"", key);
        }
        println!();
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

impl TopicFile {
    pub(crate) fn new(namespace_name: &str, file_name: &str, topic_name: &str, content: String) -> Self {
        Self {
            namespace_name: namespace_name.to_string(),
            file_name: file_name.to_string(),
            topic_name: topic_name.to_string(),
            content,
        }
    }

    fn get_key(&self) -> String {
        make_topic_file_key(&self.namespace_name, &self.file_name)
    }
}

pub fn make_topic_file_key(namespace_name: &str, file_name: &str) -> String {
    let file_name_before_extension = util::parse::before(file_name, ".txt");
    format!("{}:{}", namespace_name, file_name_before_extension)
}

pub(crate) fn build_model(name: &str, namespace_main: &str, compare_only: bool, is_public: bool, topic_limit: Option<usize>, project: Option<file_monitor::model::Project>) -> (Model, BuildProcess) {
    let mut bp = BuildProcess::new(name, namespace_main,PATH_PAGES, compare_only, is_public, topic_limit);
    let model = bp.build(project);
    (model, bp)
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
            is_debug: false,
        }
    }

    fn check_end_of_topic(&self, topic: &Topic) {
        assert!(!self.is_in_code, "is_in_code in {}. Possibly needing blank lines before and after <WRAP>, <code>, a section header, etc.", topic.get_topic_key());
        assert!(!self.is_in_non_code_marker, "is_in_non_code_marker in {}. Possibly needing blank lines before and after <WRAP>, <code>, a section header, etc.", topic.get_topic_key());
        assert!(self.marker_exit_string.is_none(), "marker_exit_string = \"{}\" in {}", self.marker_exit_string.as_ref().unwrap(), topic.get_topic_key());
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

/*
fn remove_edited_attribute_from_private_topics(model: &mut Model) {
    // This is a one-time fix. A few hundred topics were manually set to Visibility = Private,
    // so they have a recent Edited attribute and they show up on the Recent Topics page. Get rid
    // of those Edited attributes.
    for topic in model.get_topics_mut().values_mut()
        .filter(|topic| !topic.is_public()) {
        topic.remove_temp_attribute(ATTRIBUTE_NAME_EDITED);
    }
}
*/
