use std::collections::BTreeMap;
use crate::model::{TopicKey, Model, FILE_NAME_REDACT, Paragraph, TextBlock, TextItem};
use crate::dokuwiki::MARKER_REDACTION;
use std::ops::Range;
use crate::*;

#[derive(Debug)]
pub(crate) struct RedactionRecord {
    pub preview_only: bool,
    pub phrases: Vec<String>,
    pub private_topic_keys: Vec<TopicKey>,
    pub redactions: BTreeMap<TopicKey, Vec<Redaction>>,
}

#[derive(Debug)]
pub(crate) struct Redaction {
    pub phrases: Vec<String>,
    pub old: String,
    pub new: String,
}

impl RedactionRecord {
    pub fn redact(model: &mut Model, preview_only: bool) -> Self {
        let mut record = Self {
            preview_only,
            phrases: vec![],
            private_topic_keys: vec![],
            redactions: Default::default(),
        };
        record.redact_internal(model);
        record
    }

    fn redact_internal(&mut self, model: &mut Model) {
        self.make_phrase_and_topic_lists(model);
        for topic in model.get_topics_mut().values_mut()
                .filter(|topic| topic.is_public()) {
            let topic_key = topic.get_topic_key();
            let mut paragraph_replacements = BTreeMap::new();
            for (paragraph_index, paragraph) in topic.get_paragraphs().iter().enumerate() {
                match paragraph {
                    Paragraph::List { list } => {
                        let mut new_list = list.clone();
                        let mut is_changed = false;
                        if let Some(header) = list.get_header() {
                            if let Some(new_text_block) = self.redact_one_text_block(&topic_key, header) {
                                new_list.set_header(Some(new_text_block));
                                is_changed = true;
                            }
                        }
                        for (item_index, item) in list.get_items().iter().enumerate() {
                            if let Some(new_text_block) = self.redact_one_text_block(&topic_key, item.get_text_block()) {
                                new_list.set_item_text_block(item_index, new_text_block);
                                is_changed = true;
                            }
                        }
                        if is_changed {
                            let new_paragraph = Paragraph::new_list(new_list);
                            paragraph_replacements.insert(paragraph_index, new_paragraph);
                        }
                    },
                    Paragraph::Table { table } => {
                        let mut new_table = table.clone();
                        let mut is_changed = false;
                        for (row_index, row) in table.get_rows().iter().enumerate() {
                            for (col_index, cell) in row.iter().enumerate() {
                                if let Some(new_text_block) = self.redact_one_text_block(&topic_key, cell.get_text_block()) {
                                    new_table.set_cell_text_block(row_index, col_index, new_text_block);
                                    is_changed = true;
                                }
                            }
                        }
                        if is_changed {
                            let new_paragraph = Paragraph::new_table(new_table);
                            paragraph_replacements.insert(paragraph_index, new_paragraph);
                        }
                    },
                    Paragraph::Text { text_block } => {
                        if let Some(new_text_block) = self.redact_one_text_block(&topic_key, text_block) {
                            paragraph_replacements.insert(paragraph_index, Paragraph::new_text(new_text_block));
                        }
                    },
                    Paragraph::Attributes | Paragraph::Breadcrumbs | Paragraph::Category | Paragraph::GenStart
                    | Paragraph::GenEnd | Paragraph::Marker { .. } | Paragraph::Placeholder | Paragraph::SectionHeader { .. }
                    | Paragraph::TextUnresolved { .. } | Paragraph::Unknown { .. } => {},
                };
            }
            if !self.preview_only {
                for (index, new_paragraph) in paragraph_replacements.iter() {
                    topic.replace_paragraph(*index, new_paragraph.clone());
                }
            }
        }
        //bg!(&self.redactions); panic!();
        // self.print_phrase_map_counts(); panic!();
        self.print_phrase_map(); panic!();
        // self.print_potential_whitelist(); panic!();
    }

    fn redact_one_text_block(&mut self, topic_key: &TopicKey, text_block: &TextBlock) -> Option<TextBlock> {
        let topic_key_string = topic_key.get_display_text();
        match text_block {
            TextBlock::Resolved { items } => {
                let mut replacement_items = BTreeMap::new();
                for (index, item) in items.iter().enumerate() {
                    match item {
                        TextItem::Link { link } => {
                            let link_topic_key = b!(link).get_topic_key();
                            if let Some(link_topic_key) = link_topic_key {
                                if self.private_topic_keys.contains(&link_topic_key) {
                                    replacement_items.insert(index, Self::redacted_text_item());
                                    let entry = self.redactions.entry(topic_key.clone()).or_insert(vec![]);
                                    //entry.push(Redaction::new(vec![topic_key_string.clone()], topic_key_string.clone(), MARKER_REDACTION.to_string()));
                                }
                            }
                        },
                        TextItem::Text { text } => {
                            let mut working_text = text.clone();
                            let mut phrases = vec![];
                            loop {
                                match self.find_match(&working_text) {
                                    Some((start_index, end_index, phrase)) => {
                                        //let before = if start_index == 0 { "" } else { &working_text[0..start_index] };
                                        working_text = format!("{}{}{}", &working_text[0..start_index], MARKER_REDACTION, &working_text[end_index..working_text.len()]);
                                        phrases.push(phrase);
                                    },
                                    None => {
                                        break;
                                    }
                                }
                            }
                            if working_text.ne(text) {
                                replacement_items.insert(index, TextItem::new_text(&working_text));
                                let entry = self.redactions.entry(topic_key.clone()).or_insert(vec![]);
                                entry.push(Redaction::new(phrases, text.clone(), working_text));
                            }
                        },
                    }
                }
                if replacement_items.is_empty() {
                    return None
                } else {
                    let mut new_text_block = text_block.clone();
                    for (index, item) in replacement_items {
                        new_text_block.replace_item(index, item);
                    }
                    return Some(new_text_block)
                }
            },
            TextBlock::Unresolved { .. } => {
                panic!("Found an unresolved text block in {}: {:?}.", topic_key, text_block)
            },
        }
    }

    fn find_match(&self, text: &str) -> Option<(usize, usize, String)> {
        let text_lower = text.to_lowercase();
        for phrase in self.phrases.iter() {
            if let Some(pos) = text_lower.find(phrase) {
                return Some((pos, pos + phrase.len(), phrase.clone()))
            }
        }
        None
    }

    fn redacted_text_item() -> TextItem {
        TextItem::new_text(MARKER_REDACTION)
    }

    fn make_phrase_and_topic_lists(&mut self, model: &Model) {
        self.phrases = util::file::read_file_as_lines_r(FILE_NAME_REDACT).unwrap();
        // for topic_key in model.get_topics().keys() {
        //     println!("{}: public = {}", topic_key.get_display_text(), model.topic_is_public(topic_key));
        // }
        // panic!();
        for topic_key in model.get_topics().keys()
                .filter(|topic_key| !model.topic_is_public(topic_key)) {
            self.private_topic_keys.push(topic_key.clone());
            self.phrases.push(topic_key.get_topic_name().to_string());
        }
        //bg!(&self.private_topic_keys); panic!();
        // Lowercase and get rid of any blank phrases and whitespace.
        self.phrases = self.phrases.iter()
            .map(|phrase| phrase.trim().to_lowercase())
            .filter(|phrase| !phrase.is_empty() && !PHRASE_WHITELIST.contains(&&**phrase))
            .collect();
        self.phrases.sort();
        self.phrases.dedup();
        //redact_phrases.sort_by_key(|x| (Reverse(x.len()), x));
        self.phrases.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
        //bg!(&self.phrases); panic!();
    }

    pub(crate) fn get_phrase_map(&self) -> BTreeMap<String, Vec<(TopicKey, String, String)>> {
        let mut map = BTreeMap::new();
        for (topic_key, redactions) in self.redactions.iter() {
            for redaction in redactions.iter() {
                for phrase in redaction.phrases.iter() {
                    let entry = map.entry(phrase.clone()).or_insert(vec![]);
                    entry.push((topic_key.clone(), redaction.old.clone(), redaction.new.clone()));
                }
            }
        }
        map
    }

    pub(crate) fn print_phrase_map(&self) {
        for (phrase, items) in self.get_phrase_map().iter() {
            println!("\"{}\"", phrase);
            for (topic_key, old, new) in items.iter() {
                println!("\t{}", topic_key.get_display_text());
                println!("\t\told: \"{}\"", old);
                println!("\t\tnew: \"{}\"", new);
            }
        }
    }

    pub(crate) fn print_phrase_map_counts(&self) {
        for (phrase, items) in self.get_phrase_map().iter() {
            println!("{} - \"{}\"", util::format::format_count(items.len()), phrase);
        }
    }

    pub(crate) fn print_potential_whitelist(&self) {
        let line = self.get_phrase_map().keys()
            .map(|phrase| format!("\"{}\"", phrase))
            .join(", ");
        println!("{}", line);
    }
}

impl Redaction {
    fn new(phrases: Vec<String>, old: String, new: String) -> Self {
        Self {
            phrases,
            old,
            new,
        }
    }
}

const PHRASE_WHITELIST: [&str; 32] = ["behavioral economics", "bluehost", "bold", "domains", "grit", "health", "machines", "main", "meetings",
    "meetup", "music", "nlp", "oracle vm virtualbox", "organizations", "pcs", "philips hue", "pmwiki", "podcasts", "practices",
    "precalculus", "privacy", "queue", "rework", "sbt", "security project", "simplify", "skype", "to do", "twitter", "virtualbox",
    "winit", "wordpress"];
