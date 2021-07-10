use crate::model::*;
use crate::connectedtext::NAMESPACE_TOOLS;
use std::path::PathBuf;
use std::fs;
use super::*;
use crate::model::report::WikiReport;
use util::parse::{split_3_two_delimiters_rc, split_trim, between};
use std::collections::BTreeMap;

const CT_BRACKET_LEFT: &str = "[[";
const CT_BRACKET_RIGHT: &str = "]]";
const CT_TOPIC_BREAK: &str = "{{Topic}} ";
const CT_PARAGRAPH_BREAK: &str = "\r\n\r\n";
const CT_LINE_BREAK: &str = "\r\n";
const CT_CATEGORY_PREFIX: &str = "[[$CATEGORY:";
const CT_BOOKMARK_DELIM_RIGHT: &str = "»";
const CT_BOOKMARK_DELIM_LEFT: &str = "«";
const CT_TABLE_START: &str = "{|";
const CT_TABLE_END: &str = "|}";
const CT_TABLE_DELIM: &str = "||";
// const CT_PIPE: &str = "|";
// const CT_CURRENT_TOPIC: &str = "(($CURRENTTOPIC))";

pub fn main() {
    build_wiki(Some(100));
}

struct BuildProcess {
    wiki_name: String,
    namespace_main: String,
    export_path: String,
    export_file_name: String,
    errors: BTreeMap<String, Vec<String>>,
    topic_limit: Option<usize>,
}

impl BuildProcess {
    pub fn new(wiki_name: &str, namespace_main: &str, export_path: &str, export_file_name: &str, topic_limit: Option<usize>) -> Self {
        Self {
            wiki_name: wiki_name.to_string(),
            namespace_main: namespace_main.to_string(),
            export_path: export_path.to_string(),
            export_file_name: export_file_name.to_string(),
            errors: Default::default(),
            topic_limit,
        }
    }

    pub fn build(&mut self) {
        let mut wiki = Wiki::new(&self.wiki_name);
        wiki.add_namespace(NAMESPACE_TOOLS);
        self.parse_from_text_file(&mut wiki);
        self.print_errors();
        WikiReport::new().categories().paragraphs().attributes().go(&wiki);
    }

    fn parse_from_text_file(&mut self, wiki: &mut Wiki) {
        // Read the single topic file from disk and break it into topics, then break each topic
        // each topic into paragraphs. At this point we don't care about whether the paragraphs are
        // plain or mixed text, attribute tables, section headers, breadcrumbs, etc.
        self.read_text_file_as_topics(wiki);
        // Figure out the real nature of each paragraph.
        self.refine_paragraphs(wiki);
    }

    fn read_text_file_as_topics(&mut self, wiki: &mut Wiki) {
        let path_export_file = PathBuf::from(&self.export_path).join(&self.export_file_name);
        //bg!(util::file::path_name(&path_export_file));
        let export_text = fs::read_to_string(&path_export_file).unwrap();
        let export_text = export_text.replace("\u{feff}", "");
        for topic_text in export_text.split(CT_TOPIC_BREAK)
            .filter(|topic_text| !topic_text.trim().is_empty())
            .take(self.topic_limit.unwrap_or(usize::max_value())) {
            //bg!(&topic_text);
            let (topic_name, topic_text) = util::parse::split_2(topic_text, CT_LINE_BREAK);
            //rintln!("{}", topic_name);

            let mut topic = Topic::new(&self.namespace_main, &topic_name);

            // Break the topic into paragraphs.
            for paragraph_text in topic_text.split(CT_PARAGRAPH_BREAK) {
                if !paragraph_text.is_empty() {
                    topic.add_paragraph(Paragraph::new_unknown(paragraph_text));
                }
            }
            //rintln!("{}: {}", topic_name, topic.paragraphs.len());

            wiki.add_topic(topic);
        }
    }

    fn refine_paragraphs(&mut self, wiki: &mut Wiki) {
        for topic in wiki.topics.values_mut() {
            let context = format!("Refining paragraphs for \"{}\".", topic.name);
            let paragraph_count = topic.paragraphs.len();
            for paragraph_index in 0..paragraph_count {
                match self.refine_one_paragraph_rc(topic, paragraph_index, &context) {
                    Err(msg) => {
                        let entry = self.errors.entry(topic.name.clone()).or_insert(vec![]);
                        entry.push(msg);
                    },
                    _ => (),
                }
            }
        }
    }

    fn refine_one_paragraph_rc(&mut self, topic: &mut Topic, paragraph_index: usize, context: &str) -> Result<(), String> {
        let source_paragraph= std::mem::replace(&mut topic.paragraphs[paragraph_index], Paragraph::Placeholder);
        let new_paragraph = match source_paragraph {
            Paragraph::Unknown { text } => {
                self.paragraph_as_category_rc(topic, &text, context)?
                    .or(self.paragraph_as_bookmark_rc(topic, &text, context)?)
                    .or(self.paragraph_as_attributes_rc(topic, &text, context)?)
                    .unwrap_or(self.paragraph_as_text_unresolved(&text))
            },
            _ => panic!("Expected Paragraph::Unknown.")
        };
        topic.paragraphs[paragraph_index] = new_paragraph;
        Ok(())
    }

    fn paragraph_as_category_rc(&mut self, topic: &mut Topic, text: &str, _context: &str) -> Result<Option<Paragraph>, String> {
        // If it's a category line it will look like this:
        //   [[$CATEGORY:Books]]
        Ok(util::parse::between_optional_trim(text, CT_CATEGORY_PREFIX, CT_BRACKET_RIGHT)
            .map(|category_name| {
                topic.category = Some(category_name.to_string());
                Paragraph::Category
            }))
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
        if text.contains(CT_BOOKMARK_DELIM_RIGHT) {
            let lines = text.lines().collect::<Vec<_>>();
            if lines.len() != 3 {
                return Err(format!("{} Expected 3 lines, found {}.", context, lines.len()));
            }
            if lines[0] != CT_TABLE_START || lines[2] != CT_TABLE_END {
                return Err(format!("{} Table delimiters are not right.", context));
            }
            // Get rid of the table row delimiter at the front and bold (**) markup.
            let line = lines[1].replace(CT_TABLE_DELIM, "").replace("**", "");
            let parents = if line.contains(CT_BOOKMARK_DELIM_LEFT) {
                // This is a combination topic with two owners.
                let (left, _, right) = split_3_two_delimiters_rc(&line, CT_BOOKMARK_DELIM_RIGHT, CT_BOOKMARK_DELIM_LEFT, context)?;
                vec![left, right]
            } else {
                // The topic has one owner.
                let splits = line.split(CT_BOOKMARK_DELIM_RIGHT).collect::<Vec<_>>();
                // The second-to-last item should be the parent of the current topic.
                let parent_split_index = splits.len() - 2;
                vec![splits[parent_split_index]]
            };
            for parent in parents.iter() {
                let parent = remove_brackets_rc(parent, context)?;
                topic.parents.push(Topic::make_key(NAMESPACE_TOOLS, &parent));
            }
            Ok(Some(Paragraph::Breadcrumbs))
        } else {
            Ok(None)
        }
    }

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
                let values = between(&values, CT_BRACKET_LEFT, CT_BRACKET_RIGHT);
                let values = values.replace("]], [[", "]],[[");
                for value in values.split("]],[[") {
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

    fn paragraph_as_text_unresolved(&self, text: &str) -> Paragraph {
        Paragraph::new_text_unresolved(text)
    }

    fn print_errors(&self) {
        println!("\nErrors:");
        for topic_name in self.errors.keys() {
            println!("\t{}", topic_name);
            for msg in self.errors[topic_name].iter() {
                println!("\t\t{}", msg);
            }
        }
        println!();
    }
}

fn build_wiki(topic_limit: Option<usize>) {
    let mut bp = BuildProcess::new("Tools",NAMESPACE_TOOLS,PATH_CONNECTEDTEXT_EXPORT,FILE_NAME_EXPORT_TOOLS, topic_limit);
    bp.build();
}

pub fn remove_brackets_rc(text: &str, context: &str) -> Result<String, String> {
    let text = text.trim();
    if !text.starts_with(CT_BRACKET_LEFT) || !text.ends_with(CT_BRACKET_RIGHT) {
        Err(format!("{} Malformed bracketed string \"{}\"", context, text))
    } else {
        Ok(util::parse::between_trim(text, CT_BRACKET_LEFT, CT_BRACKET_RIGHT).to_string())
    }
}
