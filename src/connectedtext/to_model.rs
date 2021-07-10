use crate::model::*;
use crate::connectedtext::NAMESPACE_TOOLS;
use std::path::PathBuf;
use std::fs;
use super::*;
use crate::model::report::WikiReport;
use util::parse::{split_3_two_delimiters_rc, split_trim, between};
use std::collections::BTreeMap;
use crate::Itertools;

const CT_BRACKET_LEFT: &str = "[[";
const CT_BRACKET_RIGHT: &str = "]]";
const CT_TOPIC_BREAK: &str = "{{Topic}} ";
// const CT_PARAGRAPH_BREAK: &str = "\r\n\r\n";
const CT_PARAGRAPH_BREAK: &str = "\n\n";
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
const CT_PIPE: &str = "|";
const CT_DELIM_SECTION: &str = "#";
const CT_PREFIX_IMAGE_FOLDER: &str = r"Images\";
// const CT_CURRENT_TOPIC: &str = "(($CURRENTTOPIC))";
const CT_IMAGE_SIZE_STD: &str = "(($IMG_SIZE))";
const CT_IMAGE_SIZE_LARGE: &str = "(($IMG_SIZE_LARGE))";
const CT_IMAGE_SIZE_100_PCT: &str = "100%";

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
        // Sometimes there's a linefeed followed by a line with whitespace, followed by a line
        // break. So trim the end of every line and reassemble the block of text. This also turns
        // every line break into a standard \n. We can't trim the beginning of the lines since
        // things like list items depend on the number of spaces at the beginning.
        let export_text = export_text.lines().map(|x| x.trim_end()).join("\n");
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
                    .or(self.paragraph_as_list_rc(topic, &text, context)?)
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

    fn paragraph_as_list_rc(&mut self, _topic: &mut Topic, text: &str, context: &str) -> Result<Option<Paragraph>, String> {
        // Example with two levels (the first level has one space before the asterisk):
        // Projects:
        //   * [[Android]]
        //    * [[Algomator]]
        //    * [[Sensor (coding project)]]
        //   * [[Windows]]
        //    * [[By the Numbers]]
        //    * [[Genealogy (coding project)]]
        let context = &format!("{} Seems to be a list paragraph.", context);
        let err_func = |msg: &str| Err(format!("{} paragraph_as_list_rc: {}: text = \"{}\".", context, msg, text));
        if !text.contains(" *") {
            return Ok(None);
        }
        let lines = text.lines().collect::<Vec<_>>();
        if lines.len() < 2 || !lines[1].trim().starts_with("*") {
            return Ok(None);
        }
        // At this point we're going to assume that it's a list and consider it an error if
        // that doesn't work out.
        if lines[0].trim().starts_with("*") {
            return err_func("List with no header.");
        }
        let type_ = ListType::from_header(lines[1].trim());
        // The header may be a simple label like "Subtopics:" but it could also be a longer piece
        // of text containing links and other markup.
        let header = self.make_text_block_rc(lines[1], context)?;
        let mut items = vec![];
        for line in lines.iter().skip(1) {
            // The depth of the list item is the number of spaces before the asterisk.
            match line.find("*") {
                Some(depth) => {
                    let item_text = line[depth + 1..].trim();
                    let item_text_block = self.make_text_block_rc(item_text, context)?;
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

    fn paragraph_as_text_unresolved(&self, text: &str) -> Paragraph {
        Paragraph::new_text_unresolved(text)
    }

    fn make_text_block_rc(&self, text: &str, context: &str) -> Result<TextBlock, String> {
        let text = text.trim();
        let mut text_block = TextBlock::new();
        let delimited_splits = util::parse::split_delimited_and_normal_rc(text, CT_BRACKET_LEFT, CT_BRACKET_RIGHT, context)?;
        for (item_is_delimited, item_text) in delimited_splits.iter() {
            if *item_is_delimited {
                // Assume it's an internal or external link, or an image link.
                text_block.items.push(TextItem::new_link(self.make_link_rc(item_text, context)?));
            } else {
                // Assume it's plain text.
                text_block.items.push(TextItem::new_text(item_text));
            }
        }
        Ok(text_block)
    }

    fn make_link_rc(&self, text: &str, context: &str) -> Result<Link, String> {
        let text = text.trim();
        // The brackets should have been removed by this point.
        if text.contains(CT_BRACKET_LEFT) || text.contains(CT_BRACKET_RIGHT) {
            return Err(format!("{} Brackets found in text for a link. They should have been removed.", context));
        }
        if text.starts_with(CT_PREFIX_IMAGE) {
            return self.make_image_link_rc(text, context);
        }
        if text.starts_with(CT_PREFIX_URL) {
            // External link.
            let (url, label) = util::parse::split_1_or_2_trim(&text, CT_PIPE);
            return Ok(Link::new_external(label, url));
        }
        // Assume it's an internal link, either to a topic or a section of a topic.
        let (dest, label) = util::parse::split_1_or_2_trim(&text, CT_PIPE);
        if dest.contains(CT_DELIM_SECTION) {
            // Link to a section of a topic.
            let (topic_name, section_name) = util::parse::split_2_trim(dest, CT_DELIM_SECTION);
            return Ok(Link::new_section(label,&self.namespace_main, topic_name, section_name));
        } else {
            // Link to a whole topic.
            return Ok(Link::new_topic(label, &self.namespace_main, dest));
        }
    }

    fn make_image_link_rc(&self, text: &str, context: &str) -> Result<Link, String> {
        // Something like this (with no brackets):
        //   $IMG:Images\libgdx project setup main.jpg|100%|NONE
        //   $IMG:Images\libgdx generated new project.png|(($IMG_SIZE))|NONE
        // The brackets should have been removed by this point.
        let err_func = |msg: &str| Err(format!("{} make_image_link_rc: {}: text = \"{}\".", context, msg, text));
        let text = text.trim();
        if text.contains(CT_BRACKET_LEFT) || text.contains(CT_BRACKET_RIGHT) {
            return err_func("Brackets found in text for an image link. They should have been removed.");
        }
        if !text.starts_with(CT_PREFIX_IMAGE) {
            return err_func(&format!("Presumed image link doesn't start with {}.", CT_PREFIX_IMAGE));
        }
        let text = text[CT_PREFIX_IMAGE.len()..].trim().to_string();
        let splits = util::parse::split_trim(&text, CT_PIPE);
        if splits.len() != 3 {
            return err_func("There are not three pipe-delimited segments.");
        }
        let source = splits[0].trim();
        let size = splits[1].trim();
        if !source.starts_with(CT_PREFIX_IMAGE_FOLDER) {
            return err_func(&format!("Image destination does not start with \"{}\".", CT_PREFIX_IMAGE_FOLDER));
        }
        let source = source[CT_PREFIX_IMAGE_FOLDER.len()..].to_string();
        let size = match size {
            CT_IMAGE_SIZE_STD | CT_IMAGE_SIZE_LARGE => ImageSize::DokuLarge,
            CT_IMAGE_SIZE_100_PCT => ImageSize::Original,
            _ => {
                return err_func("Unexpected image size.");
            }
        };
        let source = ImageSource::new_internal(&self.namespace_main, &source);
        let link = Link::new_image(None, source, ImageAlignment::Left, size, ImageLinkType::Direct);
        Ok(link)
    }

    fn print_errors(&self) {
        println!("\nErrors:");
        for topic_name in self.errors.keys() {
            println!("\n\t{}", topic_name);
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

fn remove_brackets_rc(text: &str, context: &str) -> Result<String, String> {
    let text = text.trim();
    if !text.starts_with(CT_BRACKET_LEFT) || !text.ends_with(CT_BRACKET_RIGHT) {
        Err(format!("{} Malformed bracketed string \"{}\"", context, text))
    } else {
        Ok(util::parse::between_trim(text, CT_BRACKET_LEFT, CT_BRACKET_RIGHT).to_string())
    }
}

