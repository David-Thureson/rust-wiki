use crate::model::*;
use crate::connectedtext::NAMESPACE_TOOLS;
use std::path::PathBuf;
use std::fs;
use std::cell::RefMut;
use crate::*;
use super::*;
use crate::model::report::WikiReport;
use util::parse::split_3_two_delimiters_rc;

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
// const CT_PIPE: &str = "|";
// const CT_CURRENT_TOPIC: &str = "(($CURRENTTOPIC))";

pub fn main() {
    build_wiki(Some(100));
}

struct BuildProcess {
    wiki: WikiRc,
    namespace_main: NamespaceRc,
    export_path: String,
    export_file_name: String,
    topic_limit: Option<usize>,
}

impl BuildProcess {
    pub fn new(wiki_name: &str, namespace_main: &str, export_path: &str, export_file_name: &str, topic_limit: Option<usize>) -> Self {
        let wiki = Wiki::new(wiki_name);
        let wiki_rc = r!(wiki);
        let namespace_rc = r!(Namespace::new(&wiki_rc, None, namespace_main));
        m!(&wiki_rc).add_namespace(&namespace_rc);
        Self {
            wiki: wiki_rc,
            namespace_main: namespace_rc,
            export_path: export_path.to_string(),
            export_file_name: export_file_name.to_string(),
            topic_limit,
        }
    }

    pub fn build(&mut self) {
        self.parse_from_text_file();
        WikiReport::new(&self.wiki).paragraphs().go();
    }

    fn parse_from_text_file(&mut self) {
        // Read the single topic file from disk and break it into topics, then break each topic
        // each topic into paragraphs. At this point we don't care about whether the paragraphs are
        // plain or mixed text, attribute tables, section headers, breadcrumbs, etc.
        self.read_text_file_as_topics();
        // Figure out the real nature of each paragraph.
        self.refine_paragraphs();
    }

    fn read_text_file_as_topics(&mut self) {
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

            let mut topic = Topic::new(&self.wiki, &self.namespace_main, &topic_name);

            // Break the topic into paragraphs.
            topic_text.split(CT_PARAGRAPH_BREAK).for_each(|paragraph_text| topic.add_paragraph(Paragraph::new_unknown(paragraph_text)));
            //rintln!("{}: {}", topic_name, topic.paragraphs.len());

            m!(&self.wiki).add_topic(&r!(topic));
        }
    }

    fn refine_paragraphs(&mut self) {
        let topics = m!(&self.wiki).topics.values().map(|x| x.clone()).collect::<Vec<_>>();
        for topic_rc in topics.iter() {
            let mut topic_ref = m!(&topic_rc);
            let context = format!("Refining paragraphs for {}", topic_ref.name);
            for paragraph_index in 0..topic_ref.paragraphs.len() {
                match self.refine_one_paragraph_rc(&mut topic_ref, paragraph_index, &context) {
                    Err(msg) => { topic_ref.add_error(&msg); }
                    _ => (),
                }
            }
        }
    }

    fn refine_one_paragraph_rc(&mut self, topic_ref: &mut RefMut<Topic>, paragraph_index: usize, context: &str) -> Result<(), String> {
        let source_paragraph_rc: ParagraphRc = std::mem::replace(&mut topic_ref.paragraphs[paragraph_index], r!(Paragraph::Breadcrumbs));
        let new_paragraph = match &*b!(&source_paragraph_rc) {
            Paragraph::Unknown { text } => {
                self.paragraph_as_category_rc(topic_ref, &text, context)?
                    .or(self.paragraph_as_bookmark_rc(topic_ref, &text, context)?)
                    .unwrap_or(self.paragraph_as_text_unresolved(&text))
            },
            _ => panic!("Expected Paragraph::Unknown.")
        };
        topic_ref.paragraphs[paragraph_index] = r!(new_paragraph);
        Ok(())
    }

    fn paragraph_as_category_rc(&mut self, topic_ref: &mut RefMut<Topic>, text: &str, _context: &str) -> Result<Option<Paragraph>, String> {
        // If it's a category line it will look like this:
        //   [[$CATEGORY:Books]]
        Ok(util::parse::between_optional_trim(text, CT_CATEGORY_PREFIX, CT_BRACKET_RIGHT)
            .map(|category_name| {
                let category_rc = m!(&self.wiki).get_or_create_category(self.wiki.clone(), category_name);
                topic_ref.category = Some(category_rc);
                Paragraph::Category
            }))
    }

    fn paragraph_as_bookmark_rc(&mut self, topic_ref: &mut RefMut<Topic>, text: &str, context: &str) -> Result<Option<Paragraph>, String> {
        // A bookmark paragraph showing the parent and grandparent topic will look like this:
        //   {|
        //   ||**[[Android]] » [[Android Development]] » (($CURRENTTOPIC))**
        //   |}
        // A bookmark paragraph for a combination topic with two parents will look like this:
        //   {|
        //   ||**[[AutoVoice]] » (($CURRENTTOPIC)) « [[Tasker]]**
        //   |}
        let context = &format!("{}: Seems to be a bookmark paragraph", context);
        if text.contains(CT_BOOKMARK_DELIM_RIGHT) {
            let lines = text.lines().collect::<Vec<_>>();
            if lines.len() != 3 {
                return Err(format!("{}: Expected 3 lines, found {}.", context, lines.len()));
            }
            if lines[0] != CT_TABLE_START || lines[2] != CT_TABLE_END {
                return Err(format!("{}: Table delimiters are not right.", context));
            }
            let line = lines[1];
            if line.contains(CT_BOOKMARK_DELIM_LEFT) {
                // This is a combination topic with two owners.
                let (left, _, right) = split_3_two_delimiters_rc(line, CT_BOOKMARK_DELIM_RIGHT, CT_BOOKMARK_DELIM_LEFT, context)?;
                let (left, right) = (remove_brackets_rc(left, context)?, remove_brackets_rc(right, context)?);
                let left_topic_rc = b!(&self.wiki).find_topic_rc(NAMESPACE_TOOLS, &left, context)?;
                let right_topic_rc = b!(&self.wiki).find_topic_rc(NAMESPACE_TOOLS, &right, context)?;
                topic_ref.parents.push(left_topic_rc);
                topic_ref.parents.push(right_topic_rc);
            } else {
                // The topic has one owner.
                let splits = line.split(CT_BOOKMARK_DELIM_RIGHT).collect::<Vec<_>>();
                // The second-to-last item should be the parent of the current topic.
                let parent_split_index = splits.len() - 2;
                let parent_topic_name = remove_brackets_rc(splits[parent_split_index], context)?;
                let parent_topic_rc = b!(&self.wiki).find_topic_rc(NAMESPACE_TOOLS, &parent_topic_name, context)?;
                topic_ref.parents.push(parent_topic_rc);
            }
            Ok(Some(Paragraph::Breadcrumbs))
        } else {
            Ok(None)
        }
    }

    fn paragraph_as_text_unresolved(&self, text: &str) -> Paragraph {
        Paragraph::new_text_unresolved(text)
    }
}

fn build_wiki(topic_limit: Option<usize>) {
    let mut bp = BuildProcess::new("Tools",NAMESPACE_TOOLS,PATH_CONNECTEDTEXT_EXPORT,FILE_NAME_EXPORT_TOOLS, topic_limit);
    bp.build();
}

pub fn remove_brackets_rc(text: &str, context: &str) -> Result<String, String> {
    let text = text.trim();
    if !text.starts_with(CT_BRACKET_LEFT) || !text.ends_with(CT_BRACKET_RIGHT) {
        Err(format!("{}: Malformed bracketed string \"{}\"", context, text))
    } else {
        Ok(util::parse::between_trim(text, CT_BRACKET_LEFT, CT_BRACKET_RIGHT).to_string())
    }
}
