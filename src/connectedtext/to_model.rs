use crate::model::*;
use crate::connectedtext::NAMESPACE_TOOLS;
use std::path::PathBuf;
use std::fs;
use crate::*;
use super::*;

const TOPIC_BREAK: &str = "{{Topic}} ";
// const PARAGRAPH_BREAK: &str = "\r\n\r\n";
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

    }

    fn parse_from_text_file(&mut self) {
        // Read the single topic file from disk and break it into topics, then break each topic
        // each topic into paragraphs. At this point we don't care about whether the paragraphs are
        // plain or mixed text, attribute tables, section headers, breadcrumbs, etc.
        self.read_text_file_as_topics();
    }

    fn read_text_file_as_topics(&mut self) {
        let export_text = fs::read_to_string(PathBuf::from(&self.export_path).join(&self.export_file_name)).unwrap();
        for topic_text in export_text.split(TOPIC_BREAK)
                .filter(|topic_text| !topic_text.trim().is_empty())
                .take(self.topic_limit.unwrap_or(usize::max_value())) {
            let (topic_name, _topic_text) = util::parse::split_2(topic_text, LINE_BREAK);
            let topic = Topic::new(&self.wiki, &self.namespace_main, &topic_name);

            // Break the topic into paragraphs.


            m!(&self.wiki).add_topic(&r!(topic));

        }
    }

}

fn build_wiki(topic_limit: Option<usize>) {
    let mut bp = BuildProcess::new("Tools",NAMESPACE_TOOLS,PATH_CONNECTEDTEXT_EXPORT_TOOLS,FILE_NAME_EXPORT_TOOLS, topic_limit);
    bp.build();
}

