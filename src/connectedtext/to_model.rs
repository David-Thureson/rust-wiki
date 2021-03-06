use crate::*;
use crate::model::*;
use crate::connectedtext::NAMESPACE_TOOLS;
use std::path::PathBuf;
use std::fs;
use super::*;
// use crate::model::report::WikiReport;
use util::parse::{split_3_two_delimiters_rc, split_trim, between};
use crate::{Itertools, model};
#[allow(unused_imports)]
use crate::connectedtext::report::report_category_tree;
#[allow(unused_imports)]
use crate::model::report::report_attributes;

const CT_BRACKETS_LEFT: &str = "[[";
const CT_BRACKETS_RIGHT: &str = "]]";
// const CT_BRACKET_SINGLE_LEFT: &str = "[";
// const CT_BRACKET_SINGLE_RIGHT: &str = "]";
const CT_TOPIC_BREAK: &str = "{{Topic}} ";
// const CT_PARAGRAPH_BREAK: &str = "\r\n\r\n";
const CT_PARAGRAPH_BREAK: &str = "\n\n";
const CT_TEMP_PARAGRAPH_BREAK: &str = "{temp paragraph break}";
const CT_TABLE_START_SINGLE_LINE_BREAK: &str = "\n{|";
const CT_TABLE_START_DOUBLE_LINE_BREAK: &str = "\n\n{|";
//const CT_LINE_BREAK: &str = "\r\n";
const CT_LINE_BREAK: &str = "\n";
const CT_CATEGORY_PREFIX: &str = "[[$CATEGORY:";
const CT_BOOKMARK_DELIM_RIGHT: &str = "»";
const CT_BOOKMARK_DELIM_LEFT: &str = "«";
const CT_TABLE_START: &str = "{|";
const CT_TABLE_END: &str = "|}";
const CT_TABLE_DELIM: &str = "||";
const CT_PREFIX_IMAGE: &str = "$IMG:";
const CT_PREFIX_URL: &str = "$URL:";
const CT_PREFIX_FILE: &str = "$FILE:";
const CT_PREFIX_APP: &str = "$APP:";
const CT_PREFIX_CLOUD: &str = "$CLOUD";
const CT_PREFIX_ASK: &str = "$ASK:";
const CT_PREFIX_TREE: &str = "$TREE";
const CT_PIPE: &str = "|";
const CT_DELIM_SECTION_IN_LINK: &str = "#";
const CT_MARK_SECTION_HEADER: &str = "=";
const CT_PREFIX_IMAGE_FOLDER: &str = r"Images\";
// const CT_CURRENT_TOPIC: &str = "(($CURRENTTOPIC))";
const CT_IMAGE_SIZE_STD: &str = "(($IMG_SIZE))";
const CT_IMAGE_SIZE_LARGE: &str = "(($IMG_SIZE_LARGE))";
const CT_IMAGE_SIZE_100_PCT: &str = "100%";
const CT_DELIM_CODE_START: &str = "{{{";
const CT_DELIM_CODE_END: &str = "}}}";
pub(crate) const CT_TEMP_DELIM_QUOTE_START: &str = "{TEMP QUOTE START}";
const CT_TEMP_DELIM_QUOTE_END: &str = "{TEMP QUOTE END}";

// const CT_CODING_LANGUAGES: [&str; 6] = ["JavaScript", "Java", "Kotlin", "Python", "Scala", "Rust"];

struct BuildProcess {
    wiki_name: String,
    namespace_main: String,
    export_path: String,
    export_file_name: String,
    errors: TopicErrorList,
    topic_limit: Option<usize>,
}

impl BuildProcess {
    pub(crate) fn new(wiki_name: &str, namespace_main: &str, export_path: &str, export_file_name: &str, topic_limit: Option<usize>) -> Self {
        Self {
            wiki_name: wiki_name.to_string(),
            namespace_main: namespace_main.to_string(),
            export_path: export_path.to_string(),
            export_file_name: export_file_name.to_string(),
            errors: TopicErrorList::new(),
            topic_limit,
        }
    }

    pub(crate) fn build(&mut self) -> Model {
        let mut wiki = Model::new(&self.wiki_name, &self.namespace_main);
        wiki.add_namespace(&wiki.namespace_book());
        // wiki.add_namespace(NAMESPACE_CATEGORY);
        wiki.add_namespace(&wiki.namespace_navigation());
        wiki.add_namespace(&wiki.namespace_attribute());
        self.parse_from_text_file(&mut wiki);
        // wiki.catalog_links();
        self.check_links(&wiki);
        self.check_subtopic_relationships(&mut wiki);
        self.errors.print_and_list_missing_topics(Some("First pass"));

        // WikiReport::new().categories().paragraphs().attributes().lists().go(&wiki);
        // report_category_tree(&wiki);
        // wiki.catalog_possible_list_types().print_by_count(0, None);
        wiki.add_missing_category_topics();
        // wiki.move_topics_to_namespace_by_category("Navigation", &wiki.namespace_navigation());
        // wiki.move_topics_to_namespace_by_category("Nonfiction Books", &wiki.namespace_book());
        // wiki.catalog_links();
        self.errors.clear();
        self.check_links(&wiki);
        self.errors.print(Some("After moving navigation and book topics."));
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
        wiki
    }

    fn parse_from_text_file(&mut self, wiki: &mut Model) {
        // Read the single topic file from disk and break it into topics, then break each topic
        // into paragraphs. At this point we don't care about whether the paragraphs are plain or
        // mixed text, attribute tables, section headers, breadcrumbs, etc.
        self.read_text_file_as_topics(wiki);
        // Figure out the real nature of each paragraph.
        self.refine_paragraphs(wiki);
    }

    fn read_text_file_as_topics(&mut self, wiki: &mut Model) {
        let path_export_file = PathBuf::from(&self.export_path).join(&self.export_file_name);

        // Back up the export file.
        util::file::back_up_file_next_number_r(&path_export_file, PATH_CT_EXPORT_FILE_BACKUP_FOLDER, "Project Export", "txt", 4).unwrap();

        //bg!(util::file::path_name(&path_export_file));
        let export_text = fs::read_to_string(&path_export_file).unwrap();
        let export_text = export_text.replace("\u{feff}", "");
        // Sometimes there's a linefeed followed by a line with whitespace, followed by a line
        // break. So trim the end of every line and reassemble the block of text. This also turns
        // every line break into a standard \n. We can't trim the beginning of the lines since
        // things like list items depend on the number of spaces at the beginning.
        let export_text = export_text.lines()
            .map(|x| x.trim_end())
            //.filter(|x| x != &CT_TABLE_END)
            .join("\n");
        // Get rid of extra line breaks.
        let export_text = export_text.replace(&CT_LINE_BREAK.repeat(3), &CT_LINE_BREAK.repeat(2));
        for topic_text in export_text.split(CT_TOPIC_BREAK)
                .filter(|topic_text| !topic_text.trim().is_empty())
                .take(self.topic_limit.unwrap_or(usize::max_value())) {
            //bg!(&topic_text);
            let (topic_name, topic_text) = util::parse::split_2(topic_text, CT_LINE_BREAK);
            //bg!(topic_name);
            let mut topic_name = topic_name.to_string();
            if topic_name.starts_with("_") {
                topic_name = topic_name[1..].to_string();
            }
            //rintln!("{}", topic_name);

            let mut topic = Topic::new(&self.namespace_main, &topic_name);

            if topic_name.chars().next().unwrap().is_ascii_lowercase() {
                let topic_key = TopicKey::new(&self.namespace_main, &topic_name);
                self.errors.add(&topic_key,"Starts with a lowercase letter.");
            }

            let topic_text = &Self::preprocess_topic_text_for_tables_as_quotes(&topic_name, topic_text);

            // Pull out the code sections ("{{{" and "}}}") before breaking into paragraphs.
            let context = format!("read_text_file_as_topics: code splits for \"{}\".", topic_name);
            match util::parse::split_delimited_and_normal_rc(topic_text, CT_DELIM_CODE_START, CT_DELIM_CODE_END, &context) {
                Ok(code_splits) => {
                    for (is_code, entry_text) in code_splits.iter() {
                        if *is_code {
                            let entry_text = entry_text.trim_start_matches("\n").trim_end_matches("\n");
                            let language_marker = "".to_string();
                            // for language in CT_CODING_LANGUAGES.iter() {
                            //     if topic.get_name().starts_with(language) {
                            //         language_marker = format!(" {}", language);
                            //     }
                            //     break;
                            // }
                            let marker_start = format!("{}{}{}",
                                                       crate::dokuwiki::MARKER_CODE_START_PREFIX,
                                                       language_marker, crate::dokuwiki::MARKER_LINE_END);
                            // if !marker_start.eq("<code>") { //rintln!("read_text_file_as_topics(): topic = \"{}\", marker = \"{}\".", topic.get_name(), marker_start); }
                            topic.add_paragraph(Paragraph::new_marker(&marker_start));
                            let items = vec![TextItem::new_text(&entry_text)];
                            let text_block = TextBlock::new_resolved(items);
                            topic.add_paragraph(Paragraph::new_text(text_block));
                            topic.add_paragraph(Paragraph::new_marker(crate::dokuwiki::MARKER_CODE_END));
                        } else {
                            // Break the topic into paragraphs.
                            // First, though, find cases where there is a table start "{|" in the
                            // line right after some text, and make sure the table start ends up
                            // preceeded by two linefeeds.
                            let entry_text= entry_text.replace(CT_PARAGRAPH_BREAK, CT_TEMP_PARAGRAPH_BREAK);
                            let entry_text = entry_text.replace(CT_TABLE_START_SINGLE_LINE_BREAK, CT_TABLE_START_DOUBLE_LINE_BREAK);
                            let entry_text= entry_text.replace(CT_TEMP_PARAGRAPH_BREAK, CT_PARAGRAPH_BREAK);
                            for paragraph_text in entry_text.split(CT_PARAGRAPH_BREAK) {
                                if !paragraph_text.is_empty() && !paragraph_text.contains(CT_PREFIX_ASK) && !paragraph_text.contains(CT_PREFIX_TREE) {
                                    topic.add_paragraph(Paragraph::new_unknown(paragraph_text));
                                }
                            }
                        }
                    }
                },
                Err(msg) => {
                    let topic_key = TopicKey::new(&self.namespace_main, &topic_name);
                    self.errors.add(&topic_key, &msg);
                },
            }

            //rintln!("{}: {}", topic_name, topic.paragraphs.len());

            wiki.add_topic(topic);
        }
    }

    fn preprocess_topic_text_for_tables_as_quotes(topic_name: &str, topic_text: &str) -> String {
        // A ConnectedText table might be used for what will be a quotation in the generic model.
        // It will look something like:
        //   {|
        //   ||This short, but packed demonstration will show you why tens of thousands of data analysts from more than 1,800 companies rely on Alteryx daily to prep, blend, and analyze data, to deliver deeper business insights in hours, not weeks. You’ll see how our drag-and-drop interface allows you to:
        //     * Blend data from a wide variety of sources, including internal, third-party, and cloud-based data
        //     * Analyze data with over 60 pre-built tools for predictive and spatial analytics, with no programming required
        //
        //    Second paragraph.
        //   |}
        // The key difference from a normal table or an attribute block is that there is at least
        // one row within the start and end delimiters that does not end with "||".
        TopicKey::assert_legal_topic_name(topic_name);
        //et debug = topic_text.contains("||Card Type||MC||");
        let mut new_text = "".to_string();
        let lines = topic_text.split("\n").collect::<Vec<_>>();
        let mut line_index = 0;
        while line_index < lines.len() {
            //f debug { dbg!(line_index, &lines[line_index]); }
            if lines[line_index].trim().starts_with(CT_TABLE_START) {
                // The line is "{|", so we're starting a table that may be a quote.
                let line_table_start = line_index;
                //et debug = lines[line_table_start + 1].contains("||Card Type||MC||");
                //f debug { //bg!(line_table_start); }
                // Find the end of this table.
                let mut line_table_end = line_index + 1;
                // If this is an attributes block it will start like this and we need to ignore it:
                // {|
                // ||Added||[[Added:=20180524]]||
                if !lines[line_table_start + 1].contains(CT_ATTRIBUTE_ASSIGN) && !lines[line_table_start + 1].trim().ends_with(CT_TABLE_DELIM) {
                    loop {
                        //f debug { //bg!(line_table_end, &lines[line_table_end]); }
                        //if line_table_end == lines.len() || lines[line_table_end].trim().is_empty() {
                        if line_table_end == lines.len() {
                            //anic!("No table end for topic \"{}\".", topic_name);
                            break;
                        }
                        //bg!(&line_table_end, lines[line_table_end]);
                        if lines[line_table_end].trim().starts_with(CT_TABLE_END) {
                            break;
                        }
                        line_table_end += 1;
                    }
                }
                let mut is_quotation = false;
                // We know the lines on which the table starts and ends.
                //f debug { //bg!(line_table_start, line_table_end); }
                if line_table_end - line_table_start > 1 {
                    // There's at least one row.
                    // The first row in the table should start with "||".
                    //et debug = lines[line_table_start + 1].contains("This short, but packed demonstration");
                    //f debug { //bg!(line_table_start, line_table_end, &lines[line_table_start], &lines[line_table_end]); }
                    if !lines[line_table_start + 1].starts_with(CT_TABLE_DELIM) {
                        panic!("No first row table delimiter for topic \"{}\" at line {}.", topic_name, line_table_start + 1);
                    }
                    // There's a special case where the table is a bookmark block like
                    //   {|
                    //   ||**[[Main Topic]] » (($CURRENTTOPIC))**
                    //   |}
                    // We'll need to leave this cases alone because they're handled later on.
                    if !lines[line_table_start + 1].contains(CT_BOOKMARK_DELIM_RIGHT) {
                        // The rows in a table used for a quotation don't end with "||".
                        for i in line_table_start + 1..line_table_end {
                            // if topic_name.contains("Zero") { //bg!(line_table_start, line_table_end, i, lines[i]); }
                            if !lines[i].trim().ends_with(CT_TABLE_DELIM) {
                                is_quotation = true;
                                break;
                            }
                        }
                    }
                }
                if is_quotation {
                    // Put a placeholder in the topic text that will later be replaced with a
                    // Paragraph::QuoteStart.
                    new_text.push_str(&format!("{}\n\n", CT_TEMP_DELIM_QUOTE_START));
                    // Copy the row lines into the new topic text, removing any "||" at the
                    // start of the line. These lines will then be interpreted as normal paragraphs
                    // later in the process.
                    for i in line_table_start+1..line_table_end {

                        new_text.push_str(&format!("{}\n", lines[i].replace(CT_TABLE_DELIM, "")));
                    }
                    // Put a placeholder in the topic text that will later be replaced with a
                    // Paragraph::QuoteEnd.
                    new_text.push_str(&format!("\n\n{}\n\n", CT_TEMP_DELIM_QUOTE_END));
                } else {
                    // This is a table but not a quotation, so append the lines to the new output
                    // topic text unchanged.
                    line_table_end = line_table_end.min(lines.len() - 1);
                    for i in line_table_start..=line_table_end {
                        new_text.push_str(&format!("{}\n", lines[i]));
                    }
                }
                line_index = line_table_end + 1;
            } else {
                // Nothing going on, so simply append this line to the new output topic text.
                new_text.push_str(&format!("{}\n", lines[line_index]));
                line_index += 1;
            }
        }
        new_text = new_text.trim_end_matches("\n").to_string();
        //bg!(&topic_name, &topic_text, &new_text);
        new_text
    }

    fn refine_paragraphs(&mut self, wiki: &mut Model) {
        for topic in wiki.get_topics_mut().values_mut() {
            let context = format!("Refining paragraphs for \"{}\".", topic.get_name());
            let paragraph_count = topic.get_paragraphs().len();
            for paragraph_index in 0..paragraph_count {
                match self.refine_one_paragraph_rc(topic, paragraph_index, &context) {
                    Err(msg) => {
                        let topic_key = TopicKey::new(&self.namespace_main, &topic.get_name());
                        self.errors.add(&topic_key, &msg);
                    },
                    _ => (),
                }
            }
        }
    }

    fn refine_one_paragraph_rc(&mut self, topic: &mut Topic, paragraph_index: usize, context: &str) -> Result<(), String> {
        let source_paragraph= topic.replace_paragraph_with_placeholder(paragraph_index);
        // if topic.get_name().contains("Zero") { //bg!("source_paragraph", &source_paragraph.get_variant_name()); }
        /* let new_paragraph = match source_paragraph {
            Paragraph::Unknown { text } => {
                self.paragraph_as_category_rc(topic, &text, context)?
                    .or(self.paragraph_as_section_header_rc(topic, &text, context)?)
                    .or(self.paragraph_as_bookmark_rc(topic, &text, context)?)
                    .or(self.paragraph_as_quote_start_or_end_rc(topic, &text, context)?)
                    .or(self.paragraph_as_table_rc(topic, &text, context)?)
                    .or(self.paragraph_as_list_rc(topic, &text, context)?)
                    .or(self.paragraph_as_text_rc(topic, &text, context)?)
                    .unwrap_or(self.paragraph_as_text_unresolved(&text))
            },
            _ => source_paragraph,
        };
         */
        match source_paragraph {
            Paragraph::Unknown { text } => {
                let text = util::parse::trim_linefeeds(&text);
                if let Some(new_paragraph) = self.paragraph_as_category_rc(topic, &text, context)? {
                    topic.replace_paragraph(paragraph_index, new_paragraph);
                    return Ok(());
                }
                if let Some(new_paragraph) = self.paragraph_as_section_header_rc(topic, &text, context)? {
                    topic.replace_paragraph(paragraph_index, new_paragraph);
                    return Ok(());
                }
                if let Some(new_paragraph) = self.paragraph_as_bookmark_rc(topic, &text, context)? {
                    topic.replace_paragraph(paragraph_index, new_paragraph);
                    return Ok(());
                }
                if let Some(new_paragraph) = self.paragraph_as_quote_start_or_end_rc(topic, &text, context)? {
                    topic.replace_paragraph(paragraph_index, new_paragraph);
                    return Ok(());
                }
                if let Some(new_paragraph) = self.paragraph_as_table_rc(topic, &text, context)? {
                    topic.replace_paragraph(paragraph_index, new_paragraph);
                    return Ok(());
                }
                if let Some(new_paragraph) = self.paragraph_as_list_rc(topic, &text, context)? {
                    topic.replace_paragraph(paragraph_index, new_paragraph);
                    return Ok(());
                }
                if let Some(new_paragraph) = self.paragraph_as_text_rc(topic, &text, context)? {
                    topic.replace_paragraph(paragraph_index, new_paragraph);
                    return Ok(());
                }
                let new_paragraph = self.paragraph_as_text_unresolved(&text);
                topic.replace_paragraph(paragraph_index, new_paragraph);
                return Ok(());
            },
            _ => {},
        };
        // if topic.get_name().contains("Zero") { dbg!("new_paragraph", &new_paragraph.get_variant_name()); }
        // topic.paragraphs[paragraph_index] = new_paragraph;
        topic.replace_paragraph(paragraph_index, source_paragraph);
        Ok(())
    }

    fn paragraph_as_category_rc(&mut self, topic: &mut Topic, text: &str, _context: &str) -> Result<Option<Paragraph>, String> {
        // If it's a category line it will look like this:
        //   [[$CATEGORY:Books]]
        Ok(util::parse::between_optional_trim(text, CT_CATEGORY_PREFIX, CT_BRACKETS_RIGHT)
            .map(|category_name| {
                topic.set_category(category_name);
                Paragraph::Category
            }))
    }

    fn paragraph_as_section_header_rc(&mut self, _topic: &mut Topic, text: &str, context: &str) -> Result<Option<Paragraph>, String> {
        // A section header will look like:
        //   =Title=
        // with the number of equal signs indicating the depth.
        let context = &format!("{} Seems to be a section header paragraph.", context);
        let err_func = |msg: &str| Err(format!("{} paragraph_as_section_header_rc: {}: text = \"{}\".", context, msg, text));
        let text = text.trim();
        if text.starts_with(CT_MARK_SECTION_HEADER) {
            let lines = text.lines().collect::<Vec<_>>();
            if lines.len() > 1 {
                return err_func(&format!("Expected a single line, found {}.", lines.len()));
            }
            let marker_char = CT_MARK_SECTION_HEADER.chars().nth(0).unwrap();
            let mut depth = 0;
            for char in text.chars() {
                if char == marker_char {
                    depth += 1;
                } else {
                    break;
                }
            }
            let section_marker = CT_MARK_SECTION_HEADER.repeat(depth);
            debug_assert!(text.starts_with(&section_marker));
            if !text.ends_with(&section_marker) {
                return err_func("Matching marker on right side not found.");
            }
            let section_name = util::parse::between(text, &section_marker, &section_marker).trim();
            if section_name.is_empty() {
                return err_func("Empty section header.");
            }
            if section_name.ends_with(CT_MARK_SECTION_HEADER) {
                return err_func("Too many markers on right side.");
            }
            Ok(Some(Paragraph::new_section_header(section_name, depth)))
        } else {
            Ok(None)
        }
    }

    fn paragraph_as_bookmark_rc(&mut self, topic: &mut Topic, text: &str, context: &str) -> Result<Option<Paragraph>, String> {
        // A bookmark paragraph showing the parent and grandparent topic will look like this:
        //   {|
        //   ||**[[Android]] » [[Android Development]] » (($CURRENTTOPIC))**
        //   |}
        // A bookmark paragraph for a combination topic with two parents will look like this:
        //   {|
        //   ||**[[AutoVoice]] » (($CURRENTTOPIC)) « [[Tasker]]**
        //   |}
        let context = &format!("{} Seems to be a bookmark paragraph.", context);
        let err_func = |msg: &str| Err(format!("{} paragraph_as_bookmark_rc: {}: text = \"{}\".", context, msg, text));
        if text.contains(CT_BOOKMARK_DELIM_RIGHT) {
            let lines = text.lines().collect::<Vec<_>>();
            if lines.len() < 2 || lines.len() > 3 {
                return err_func(&format!("Expected 2 or 3 lines, found {}.", lines.len()));
            }
            if lines[0] != CT_TABLE_START {
                return err_func("Table start delimiter is not right.");
            }
            if lines.len() > 2 && lines[2] != CT_TABLE_END {
                return err_func("Table end delimiter is not right.");
            }
            // Get rid of the table row delimiter at the front and bold (**) markup.
            let line = lines[1].replace(CT_TABLE_DELIM, "").replace("**", "");
            let parents = if line.contains(CT_BOOKMARK_DELIM_LEFT) {
                // This is a combination topic with two owners.
                let (left, _, right) = split_3_two_delimiters_rc(&line, CT_BOOKMARK_DELIM_RIGHT, CT_BOOKMARK_DELIM_LEFT, context)?;
                vec![left, right]
            } else {
                // The topic has one parent.
                let splits = line.split(CT_BOOKMARK_DELIM_RIGHT).collect::<Vec<_>>();
                // The second-to-last item should be the parent of the current topic.
                let parent_split_index = splits.len() - 2;
                vec![splits[parent_split_index]]
            };
            for parent in parents.iter() {
                let parent = remove_brackets_rc(parent, context)?;
                let link_rc = r!(Link::new_topic(None, NAMESPACE_TOOLS, &parent));
                topic.add_parent(link_rc);
            }
            //bg!(topic.get_name(), &parents, &topic.parents);
            Ok(Some(Paragraph::Breadcrumbs))
        } else {
            Ok(None)
        }
    }

    fn paragraph_as_quote_start_or_end_rc(&mut self, _topic: &mut Topic, text: &str, _context: &str) -> Result<Option<Paragraph>, String> {
        if text.trim().eq(CT_TEMP_DELIM_QUOTE_START) {
            Ok(Some(Paragraph::new_marker(crate::dokuwiki::MARKER_QUOTE_START)))
        } else if text.trim().eq(CT_TEMP_DELIM_QUOTE_END) {
            Ok(Some(Paragraph::new_marker(crate::dokuwiki::MARKER_QUOTE_END)))
        } else {
            Ok(None)
        }
    }

    fn paragraph_as_table_rc(&mut self, topic: &mut Topic, text: &str, context: &str) -> Result<Option<Paragraph>, String> {
        //if topic.get_name().contains("Zero") {
            //bg!(topic.get_name(), text);
        //}
        // A paragraph with a list of attributes will look something like this:
        //   {|
        //   ||Domain||[[Domain:=Serverless]], [[Domain:=Function as a Service / FaaS]]||
        //   ||Added||[[Added:=20201204]]||
        // A regular table will look like that but without things like [[Domain:=Serverless]].
        // The Terms page has a large example of a regular table.
        let context = &format!("{} Seems to be a table paragraph.", context);
        let lines = text.lines().collect::<Vec<_>>();
        if lines[0].trim().starts_with(CT_TABLE_START) {
            // && lines.len() > 1
            // && lines[1].starts_with(CT_TABLE_DELIM) {
            // && split_trim(lines[1], CT_TABLE_DELIM).len() == 4 {
            // This is a table in ConnectedText. But it will be interpreted either as:
            //  - A set of attributes.
            //  - A regular table.
            // If instead it was a quotation, it should have been detected earlier and should not
            // reach this code.
            if lines.len() < 2 {
                return Err(format!("{} Seems to be the start of a table but there are no rows.", context));
            }
            let is_attributes = lines[1].contains(CT_ATTRIBUTE_ASSIGN);
            // if topic.get_name().contains("Zero") { //bg!(is_attributes); }

            let mut rows = vec![];
            for line_index in 1..lines.len() {
                let line = lines[line_index].trim();
                if line == CT_TABLE_END {
                    break;
                }
                if !line.starts_with(CT_TABLE_DELIM) {
                    return Err(format!("{} Seems to be a table but the line doesn't start with the delimiter at line {}: \"{}\"", context, line_index, line));
                }
                let line = between(line, CT_TABLE_DELIM, CT_TABLE_DELIM);
                let split = split_trim(line, CT_TABLE_DELIM);
                if is_attributes && split.len() != 2 {
                    return Err(format!("{} Wrong number of table cells for an attribute row in \"{}\".", context, line));
                }
                rows.push(split);
            }
            if is_attributes {
                for row in rows.iter_mut() {
                    let mut name = row.remove(0);
                    AttributeType::assert_legal_attribute_type_name(&name);
                    if name.eq("Subject") {
                        name = ATTRIBUTE_NAME_DOMAIN.to_string();
                    }
                    if name.eq("Date") {
                        name = ATTRIBUTE_NAME_ADDED.to_string();
                    }
                    let max_value_count = if name.eq(ATTRIBUTE_NAME_ADDED) { Some(1) } else { None };
                    let values = row.remove(0);
                    //et debug = name.eq("Date") && values.eq("[[Date:=20160824]], [[Date:=20160505]]");
                    assert!(row.is_empty());
                    let attribute = topic.add_or_find_temp_attribute(&name);
                    let values = between(&values, CT_BRACKETS_LEFT, CT_BRACKETS_RIGHT);
                    let bracket_delim_with_space = format!("{}, {}", CT_BRACKETS_RIGHT, CT_BRACKETS_LEFT);
                    let bracket_delim_no_space = format!("{},{}", CT_BRACKETS_RIGHT, CT_BRACKETS_LEFT);
                    let assignments = values.replace(&bracket_delim_with_space, &bracket_delim_no_space);
                    //f debug { //bg!(values, &bracket_delim_with_space, &bracket_delim_no_space, &assignments); };
                    for assignment in assignments.split(&bracket_delim_no_space) {
                        if max_value_count.map_or(true, |max_value_count| max_value_count > attribute.len()) {
                            let value = util::parse::after(assignment, CT_ATTRIBUTE_ASSIGN).trim().to_string();
                            //f debug { //bg!(&value); }
                            if !value.contains("*") && !value.is_empty() {
                                if !AttributeType::is_legal_attribute_value(&value) {
                                    return Err(format!("{} In attribute \"{}\", value \"{}\" is invalid.", context, name, value));
                                }
                                if attribute.contains(&value) {
                                    return Err(format!("{} In attribute \"{}\", duplicated value \"{}\".", context, name, value));
                                }
                                attribute.push(value);
                            }
                        }
                    }
                }
                // if topic.get_name().contains("Zero") { //bg!(&topic.attributes); }
                Ok(Some(Paragraph::Attributes))
            } else {
                // Normal (non-attribute) table.
                let has_header = rows[0].iter().all(|cell| cell.contains(CT_FORMAT_BOLD));
                let mut table = model::Table::new(has_header);
                for (index, cells) in rows.iter().enumerate() {
                    let is_bold = has_header && index == 0;
                    let mut table_cells = vec![];
                    for cell in cells.iter() {
                        // Remove bold formatting.
                        let cell = cell.replace(CT_FORMAT_BOLD, "");
                        let horizontal = if cell.contains(TAG_ALIGN_RIGHT) { HorizontalAlignment::Right } else { HorizontalAlignment::Left };
                        let cell = cell.replace(TAG_ALIGN_RIGHT, "").trim().to_string();
                        let text_block= self.make_text_block_rc(topic.get_name(), &cell, context)?;
                        table_cells.push(TableCell::new_text_block(text_block, is_bold, &horizontal));
                    }
                    table.add_row(table_cells);
                }
                Ok(Some(Paragraph::new_table(table)))
            }
        } else {
            Ok(None)
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
    */

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
        let type_ = List::header_to_type(lines[0].trim());
        // The header may be a simple label like "Subtopics:" but it could also be a longer piece
        // of text containing links and other markup.
        let header = self.make_text_block_rc(topic.get_name(), lines[0], context)?;
        let mut list = List::new(&type_, Some(header));
        for line in lines.iter().skip(1) {
            // The depth of the list item is the number of spaces before the asterisk.
            match line.find("*") {
                Some(depth) => {
                    let item_text = line[depth + 1..].trim();
                    let item_text_block = self.make_text_block_rc(topic.get_name(), item_text, context)?;
                    let is_ordered = false;
                    let list_item = ListItem::new(depth, is_ordered, item_text_block);
                    list.add_item(list_item);
                },
                None => {
                    return err_func("List item with no \"*\".");
                }
            }
        }
        let paragraph = Paragraph::new_list(list);
        Ok(Some(paragraph))
    }

    fn paragraph_as_text_rc(&self, topic: &mut Topic, text: &str, context: &str) -> Result<Option<Paragraph>, String> {
        let context = &format!("{} Seems to be a text paragraph.", context);
        let text_block = self.make_text_block_rc(topic.get_name(), text, context)?;
        Ok(Some(Paragraph::new_text(text_block)))
    }

    fn paragraph_as_text_unresolved(&self, text: &str) -> Paragraph {
        Paragraph::new_text_unresolved(text)
    }

    fn check_links(&mut self, wiki: &Model) {
        let mut link_errors = wiki.check_links();
        self.errors.append(&mut link_errors);
    }

    fn check_subtopic_relationships(&mut self, wiki: &mut Model) {
        let mut subtopic_errors = wiki.check_subtopic_relationships();
        self.errors.append(&mut subtopic_errors);
    }

    fn make_text_block_rc(&self, topic_name: &str, text: &str, context: &str) -> Result<TextBlock, String> {
        //bg!(topic_name, text);
        // let err_func = |msg: &str| Err(format!("{} make_text_block_rc: {}: text = \"{}\".", context, msg, text));
        TopicKey::assert_legal_topic_name(topic_name);
        let text = text.trim();
        let mut items = vec![];
        let delimited_splits = util::parse::split_delimited_and_normal_rc(text, CT_BRACKETS_LEFT, CT_BRACKETS_RIGHT, context)?;
        for (item_is_delimited, item_text) in delimited_splits.iter() {
            if *item_is_delimited {
                // Assume it's an internal or external link, or an image link.
                if let Some(link) = self.make_link_rc(topic_name, item_text, context)? {
                    items.push(TextItem::new_link(link));
                }
            } else {
                // Assume it's plain text.
                // if item_text.starts_with('\n') {
                //     return err_func("Item text starts with linefeed");
                //}
                //if item_text.ends_with('\n') {
                //    return err_func("Item text ends with linefeed");
                //}
                items.push(TextItem::new_text(item_text));
            }
        }
        let text_block = TextBlock::new_resolved(items);
        Ok(text_block)
    }

    fn make_link_rc(&self, topic_name: &str, text: &str, context: &str) -> Result<Option<Link>, String> {
        TopicKey::assert_legal_topic_name(topic_name);
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
        if text.starts_with(CT_PREFIX_FILE) {
            // File link.
            let (file_ref, label) = util::parse::split_1_or_2_trim(&text, CT_PIPE);
            let file_ref = util::parse::after(file_ref, CT_PREFIX_FILE);
            return Ok(Some(Link::new_file(label, file_ref)));
        }
        if text.starts_with(CT_PREFIX_APP) {
            // Treat as a file link.
            let (file_ref, label) = util::parse::split_1_or_2_trim(&text, CT_PIPE);
            let file_ref = util::parse::after(file_ref, CT_PREFIX_APP);
            return Ok(Some(Link::new_file(label, file_ref)));
        }
        if text.starts_with(CT_PREFIX_CLOUD) {
            // Ignore this as it has no meaning outside of ConnectedText.
            return Ok(None);
        }
        if text.starts_with("$") {
            panic!("Unexpected ConnectedText link type (starts with \"$\"): text = {}", text)
            // println!("{}", text);
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

pub(crate) fn build_model(name: &str, namespace_main: &str, topic_limit: Option<usize>) -> Model {
    let mut bp = BuildProcess::new(name, namespace_main,PATH_CT_EXPORT,FILE_NAME_EXPORT_TOOLS, topic_limit);
    let model = bp.build();
    model
}

fn remove_brackets_rc(text: &str, context: &str) -> Result<String, String> {
    let text = text.trim();
    if !text.starts_with(CT_BRACKETS_LEFT) || !text.ends_with(CT_BRACKETS_RIGHT) {
        Err(format!("{} Malformed bracketed string \"{}\"", context, text))
    } else {
        Ok(util::parse::between_trim(text, CT_BRACKETS_LEFT, CT_BRACKETS_RIGHT).to_string())
    }
}
