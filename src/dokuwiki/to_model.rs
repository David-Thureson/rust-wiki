use crate::model::*;
use std::path::PathBuf;
use std::fs;
use super::*;
// use crate::model::report::WikiReport;
use util::parse::{split_3_two_delimiters_rc, split_trim, between};
use crate::Itertools;

const DW_PARAGRAPH_BREAK: &str = "\n\n";
const DW_LINE_BREAK: &str = "\n";
const CT_MARK_SECTION_HEADER: &str = "=";
const CT_PREFIX_IMAGE_FOLDER: &str = r"Images\";

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
        let mut wiki = Wiki::new(&self.wiki_name, &self.namespace_main);
        let namespace_main = self.namespace_main.clone();
        let namespace_book = wiki.namespace_book();
        wiki.add_namespace(&namespace_book);

        let topic_limit_per_namespace = self.topic_limit.map(|topic_limit| topic_limit / 2);
        self.parse_from_folder(&mut wiki, &namespace_main, topic_limit_per_namespace);
        self.parse_from_folder(&mut wiki, &namespace_book, topic_limit_per_namespace);
        // Figure out the real nature of each paragraph.

        /*
        self.refine_paragraphs(&mut wiki);

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
        wiki
    }

    fn parse_from_folder(&mut self, wiki: &mut Wiki, namespace_name: &str, topic_limit: Option<usize>) {
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
                assert!(topic_name.starts_with(DELIM_HEADLINE));
                assert!(topic_name.ends_with(DELIM_HEADLINE));
                topic_name = topic_name.replace(DELIM_HEADLINE, "").trim().to_string();
                dbg!(&topic_name);
                let mut topic = Topic::new(namespace_name, &topic_name);
                for paragraph in paragraphs.iter() {
                    topic.add_paragraph(Paragraph::new_unknown(paragraph));
                }
                wiki.add_topic(topic);
                topic_count += 1;
                if topic_limit.map_or(false, |topic_limit| topic_count >= topic_limit) {
                    break;
                }
            }
        }
    }

    /*
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
        // let debug = topic_text.contains("||Card Type||MC||");
        let mut new_text = "".to_string();
        let lines = topic_text.split("\n").collect::<Vec<_>>();
        let mut line_index = 0;
        while line_index < lines.len() {
            // if debug { dbg!(line_index, &lines[line_index]); }
            if lines[line_index].trim().starts_with(CT_TABLE_START) {
                // The line is "{|", so we're starting a table that may be a quote.
                let line_table_start = line_index;
                // let debug = lines[line_table_start + 1].contains("||Card Type||MC||");
                // if debug { //bg!(line_table_start); }
                // Find the end of this table.
                let mut line_table_end = line_index + 1;
                // If this is an attributes block it will start like this and we need to ignore it:
                // {|
                // ||Added||[[Added:=20180524]]||
                if !lines[line_table_start + 1].contains(CT_ATTRIBUTE_ASSIGN) && !lines[line_table_start + 1].trim().ends_with(CT_TABLE_DELIM) {
                    loop {
                        // if debug { //bg!(line_table_end, &lines[line_table_end]); }
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
                // if debug { //bg!(line_table_start, line_table_end); }
                if line_table_end - line_table_start > 1 {
                    // There's at least one row.
                    // The first row in the table should start with "||".
                    // let debug = lines[line_table_start + 1].contains("This short, but packed demonstration");
                    // if debug { //bg!(line_table_start, line_table_end, &lines[line_table_start], &lines[line_table_end]); }
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

    fn refine_paragraphs(&mut self, wiki: &mut Wiki) {
        for topic in wiki.topics.values_mut() {
            let context = format!("Refining paragraphs for \"{}\".", topic.name);
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
        // if topic.name.contains("Zero") { //bg!("source_paragraph", &source_paragraph.get_variant_name()); }
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
                if let Some(new_paragraph) = self.paragraph_as_category_rc(topic, &text, context)? {
                    topic.paragraphs[paragraph_index] = new_paragraph;
                    return Ok(());
                }
                if let Some(new_paragraph) = self.paragraph_as_section_header_rc(topic, &text, context)? {
                    topic.paragraphs[paragraph_index] = new_paragraph;
                    return Ok(());
                }
                if let Some(new_paragraph) = self.paragraph_as_bookmark_rc(topic, &text, context)? {
                    topic.paragraphs[paragraph_index] = new_paragraph;
                    return Ok(());
                }
                if let Some(new_paragraph) = self.paragraph_as_quote_start_or_end_rc(topic, &text, context)? {
                    topic.paragraphs[paragraph_index] = new_paragraph;
                    return Ok(());
                }
                if let Some(new_paragraph) = self.paragraph_as_table_rc(topic, &text, context)? {
                    topic.paragraphs[paragraph_index] = new_paragraph;
                    return Ok(());
                }
                if let Some(new_paragraph) = self.paragraph_as_list_rc(topic, &text, context)? {
                    topic.paragraphs[paragraph_index] = new_paragraph;
                    return Ok(());
                }
                if let Some(new_paragraph) = self.paragraph_as_text_rc(topic, &text, context)? {
                    topic.paragraphs[paragraph_index] = new_paragraph;
                    return Ok(());
                }
                let new_paragraph = self.paragraph_as_text_unresolved(&text);
                topic.paragraphs[paragraph_index] = new_paragraph;
                return Ok(());
            },
            _ => {},
        };
        // if topic.name.contains("Zero") { dbg!("new_paragraph", &new_paragraph.get_variant_name()); }
        // topic.paragraphs[paragraph_index] = new_paragraph;
        topic.paragraphs[paragraph_index] = source_paragraph;
        Ok(())
    }

    fn paragraph_as_category_rc(&mut self, topic: &mut Topic, text: &str, _context: &str) -> Result<Option<Paragraph>, String> {
        // If it's a category line it will look like this:
        //   [[$CATEGORY:Books]]
        Ok(util::parse::between_optional_trim(text, CT_CATEGORY_PREFIX, CT_BRACKETS_RIGHT)
            .map(|category_name| {
                topic.category = Some(category_name.to_string());
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
                topic.parents.push(TopicKey::new(NAMESPACE_TOOLS, &parent));
            }
            //bg!(&topic.name, &parents, &topic.parents);
            Ok(Some(Paragraph::Breadcrumbs))
        } else {
            Ok(None)
        }
    }

    fn paragraph_as_quote_start_or_end_rc(&mut self, _topic: &mut Topic, text: &str, _context: &str) -> Result<Option<Paragraph>, String> {
        if text.trim().eq(CT_TEMP_DELIM_QUOTE_START) {
            Ok(Some(Paragraph::QuoteStart))
        } else if text.trim().eq(CT_TEMP_DELIM_QUOTE_END) {
            Ok(Some(Paragraph::QuoteEnd))
        } else {
            Ok(None)
        }
    }

    fn paragraph_as_table_rc(&mut self, topic: &mut Topic, text: &str, context: &str) -> Result<Option<Paragraph>, String> {
        //if topic.name.contains("Zero") {
        //bg!(&topic.name, text);
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
            // if topic.name.contains("Zero") { //bg!(is_attributes); }

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
                    if name.eq("Subject") {
                        name = ATTRIBUTE_NAME_DOMAIN.to_string();
                    }
                    if name.eq("Date") {
                        name = ATTRIBUTE_NAME_ADDED.to_string();
                    }
                    let max_value_count = if name.eq("Added") { Some(1) } else { None };
                    let values = row.remove(0);
                    // let debug = name.eq("Date") && values.eq("[[Date:=20160824]], [[Date:=20160505]]");
                    assert!(row.is_empty());
                    let attribute = topic.temp_attributes.entry(name.clone())
                        .or_insert(vec![]);
                    let values = between(&values, CT_BRACKETS_LEFT, CT_BRACKETS_RIGHT);
                    let bracket_delim_with_space = format!("{}, {}", CT_BRACKETS_RIGHT, CT_BRACKETS_LEFT);
                    let bracket_delim_no_space = format!("{},{}", CT_BRACKETS_RIGHT, CT_BRACKETS_LEFT);
                    let assignments = values.replace(&bracket_delim_with_space, &bracket_delim_no_space);
                    // if debug { //bg!(values, &bracket_delim_with_space, &bracket_delim_no_space, &assignments); };
                    for assignment in assignments.split(&bracket_delim_no_space) {
                        if max_value_count.map_or(true, |max_value_count| max_value_count > attribute.len()) {
                            let value = util::parse::after(assignment, CT_ATTRIBUTE_ASSIGN).trim().to_string();
                            // if debug { //bg!(&value); }
                            if !value.contains("*") && !value.is_empty() {
                                if attribute.contains(&value) {
                                    return Err(format!("{} In attribute \"{}\", duplicated value \"{}\".", context, name, value));
                                }
                                attribute.push(value);
                            }
                        }
                    }
                }
                // if topic.name.contains("Zero") { //bg!(&topic.attributes); }
                Ok(Some(Paragraph::Attributes))
            } else {
                // Normal (non-attribute) table.
                let has_header = rows[0].iter().all(|cell| cell.contains(CT_FORMAT_BOLD));
                let mut rows_as_text_blocks = vec![];
                for (index, cells) in rows.iter().enumerate() {
                    let is_header = has_header && index == 0;
                    let mut cells_as_text_blocks = vec![];
                    for cell in cells.iter() {
                        // If this is the header row, remove the bold formatting.
                        let cell_cleaned = if is_header { cell.replace(CT_FORMAT_BOLD, "").to_string() } else { cell.to_string() };
                        let text_block= self.make_text_block_rc(&topic.name, &cell_cleaned, context)?;
                        cells_as_text_blocks.push(text_block);
                    }
                    rows_as_text_blocks.push(cells_as_text_blocks);
                }
                Ok(Some(Paragraph::Table { has_header, rows: rows_as_text_blocks }))
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

    fn paragraph_as_text_unresolved(&self, text: &str) -> Paragraph {
        Paragraph::new_text_unresolved(text)
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