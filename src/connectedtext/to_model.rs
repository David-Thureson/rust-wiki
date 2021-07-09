use crate::model::*;
use crate::connectedtext::NAMESPACE_TOOLS;
use std::path::PathBuf;
use std::fs;
use std::cell::RefMut;
use crate::*;
use super::*;
use crate::model::report::WikiReport;

const TOPIC_BREAK: &str = "{{Topic}} ";
const PARAGRAPH_BREAK: &str = "\r\n\r\n";
const LINE_BREAK: &str = "\r\n";

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
        for topic_text in export_text.split(TOPIC_BREAK)
            .filter(|topic_text| !topic_text.trim().is_empty())
            .take(self.topic_limit.unwrap_or(usize::max_value())) {
            //bg!(&topic_text);
            let (topic_name, topic_text) = util::parse::split_2(topic_text, LINE_BREAK);
            //rintln!("{}", topic_name);

            let mut topic = Topic::new(&self.wiki, &self.namespace_main, &topic_name);

            // Break the topic into paragraphs.
            topic_text.split(PARAGRAPH_BREAK).for_each(|paragraph_text| topic.add_paragraph(Paragraph::new_unknown(paragraph_text)));
            //rintln!("{}: {}", topic_name, topic.paragraphs.len());

            m!(&self.wiki).add_topic(&r!(topic));
        }
    }

    fn refine_paragraphs(&mut self) {
        // for topic_rc in m!(&self.wiki).topics.values_mut() {
        // for topic_rc in m!(&self.wiki).topics.values().clone() {
        let topics = m!(&self.wiki).topics.values().map(|x| x.clone()).collect::<Vec<_>>();
        for topic_rc in topics.iter() {
            let mut topic = m!(&topic_rc);
            /*
            let mut new_paragraphs = vec![];
            for paragraph in topic.paragraphs.iter_mut() {
                match paragraph {
                    Paragraph::Unknown { text } => {
                        new_paragraphs.push(
                            self.paragraph_as_category(&topic_rc, &text)
                                .unwrap_or(self.paragraph_as_text_unresolved(&topic_rc, &text)));
                    },
                    _ => panic!("Expected Paragraph::Unknown.")
                };
            }
            std::mem::replace(&mut topic.paragraphs, new_paragraphs);
            */
            /*
            for paragraph_rc in topic.paragraphs.iter() {
                let paragraph = b!(paragraph_rc);
                match paragraph {
                    Paragraph::Unknown { text } => {
                        (self.paragraph_as_category(&topic_rc, &text)
                            .unwrap_or(self.paragraph_as_text_unresolved(&topic_rc, &text)))
                    },
                    _ => panic!("Expected Paragraph::Unknown.")
                };
            }
            for paragraph_index in 0..topic.paragraphs.len() {
                topic.paragraphs[paragraph_index] = r!({
                    // let source_paragraph = b!(&topic.paragraphs[paragraph_index]);
                    // let source_paragraph = Ref::leak(b!(&topic.paragraphs[paragraph_index].clone()));
                    // match b!(&topic.paragraphs[paragraph_index]) {
                    match Ref::leak(b!(&topic.paragraphs[paragraph_index].clone())) {
                        Paragraph::Unknown { text } => {
                            (self.paragraph_as_category(&topic_rc, &text)
                                .unwrap_or(self.paragraph_as_text_unresolved(&topic_rc, &text)))
                        },
                        _ => panic!("Expected Paragraph::Unknown.")
                    }
                    });
            }
             */
            /*
            // This approach works, but it requires building an array of new paragraphs, then
            // them placing them in the topic at the end. This is fine for this code but it
            // wouldn't work for later cases when we might want to look through a collection of
            // Rc<RefCell<T>> and replace only a few of them.
            let mut new_paragraphs = vec![];
            for source_paragraph_rc in topic.paragraphs.iter() {
                let source_paragraph = b!(source_paragraph_rc);
                new_paragraphs.push(match &*source_paragraph {
                    Paragraph::Unknown { text } => {
                        r!(self.paragraph_as_category(&topic_rc, &text)
                            .unwrap_or(self.paragraph_as_text_unresolved(&topic_rc, &text)))
                    },
                    _ => panic!("Expected Paragraph::Unknown.")
                });
            }
            topic.paragraphs = new_paragraphs;
             */
            // In this approach we replace the paragraphs one at a time. It's not necessary here
            // because we're going to replace all of them anyway, but it will be needed later when
            // we want to look through a collection of Rc<RefCell<T>> and replace only a few of
            // them.
            /*
            for i in 0..topic.paragraphs.len() {
                let source_paragraph = b!(&topic.paragraphs[i]);
                let new_paragraph = match &*source_paragraph {
                    Paragraph::Unknown { text } => {
                        r!(self.paragraph_as_category(&mut topic, &text)
                            .unwrap_or(self.paragraph_as_text_unresolved(&topic_rc, &text)))
                    },
                    _ => panic!("Expected Paragraph::Unknown.")
                };
                drop(source_paragraph);
                topic.paragraphs[i] = new_paragraph;
            }
             */
            for i in 0..topic.paragraphs.len() {
                // let source_paragraph = (*b!(&topic.paragraphs[i])).clone();
                let source_paragraph_rc: ParagraphRc = std::mem::replace(&mut topic.paragraphs[i], r!(Paragraph::Breadcrumbs));
                let new_paragraph = match &*b!(&source_paragraph_rc) {
                    Paragraph::Unknown { text } => {
                        self.paragraph_as_category(&mut topic, &text)
                            .unwrap_or(self.paragraph_as_text_unresolved(&text))
                    },
                    _ => panic!("Expected Paragraph::Unknown.")
                };
                topic.paragraphs[i] = r!(new_paragraph);
            }
        }
    }

    // fn paragraph_as_category(&mut self, topic_rc: &TopicRc, text: &str) -> Option<Paragraph> {
    fn paragraph_as_category(&mut self, topic_ref: &mut RefMut<Topic>, text: &str) -> Option<Paragraph> {
        // If it's a category line it will look like this:
        //   [[$CATEGORY:Books]]
        util::parse::between_optional_trim(text, "[[$CATEGORY:", "]]")
            .map(|category_name| {
                let category_rc = m!(&self.wiki).get_or_create_category(self.wiki.clone(), category_name);
                topic_ref.category = Some(category_rc);
                Paragraph::Category
            })
    }

    fn paragraph_as_text_unresolved(&self, text: &str) -> Paragraph {
        Paragraph::new_text_unresolved(text)
    }
}

fn build_wiki(topic_limit: Option<usize>) {
    let mut bp = BuildProcess::new("Tools",NAMESPACE_TOOLS,PATH_CONNECTEDTEXT_EXPORT,FILE_NAME_EXPORT_TOOLS, topic_limit);
    bp.build();
}

