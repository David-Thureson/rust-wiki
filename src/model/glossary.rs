use crate::model::{Table, TopicKey, TextBlock};
use crate::Itertools;

const GLOSSARY_TABLE_COLUMN_COUNT: usize = 3;

pub(crate) struct Glossary {
    name: String,
    tags: Vec<&'static str>,
    terms: Vec<GlossaryItem>,
    raw_list: Table,
}

pub(crate) enum GlossaryItem {
    Term {
        name: String,
        acronym: Option<String>,
        topic: Option<TopicKey>,
        url: Option<String>,
        definition: Option<TextBlock>,
    },
    Acronym {
        acronym: String,
        topic_name: String,
    }
}

impl Glossary {
    pub(crate) fn new_with_raw_list(name: &str, tags: Vec<&'static str>, raw_list: Table) -> Self {
        Self {
            name: name.to_string(),
            tags,
            terms: vec![],
            raw_list,
        }
    }

    pub(crate) fn build_from_raw_list(&mut self) -> Option<String> {
        assert!(self.terms.is_empty());
        assert!(!self.raw_list.is_empty());
        let mut errors = vec![];
        for (row_index, row) in self.raw_list.get_rows().iter().enumerate() {
            if row.len() != GLOSSARY_TABLE_COLUMN_COUNT {
                errors.push(format!("Row {}: Expected {} columns, found {}.", row_index, GLOSSARY_TABLE_COLUMN_COUNT, row.len()));
                continue;
            }
            if row[0].get_text_block().starts_with_text("Acronym") {
                // This is an acronym.

            } else {
                // Not an acronym.
            }
        }





        let error_string = if errors.is_empty() {
            None
        } else {
            Some(errors.iter()
                .map(|x| format!("\n{}", x))
                .join(""))
        };
        error_string
    }

}

impl GlossaryItem {
    pub fn new_term(name: String, acronym: Option<String>, topic: Option<TopicKey>, url: Option<String>, definition: Option<TextBlock>) -> Self {
        Self::Term {
            name,
            acronym,
            topic,
            url,
            definition
        }
    }

    pub fn new_acronym(acronym: String, topic_name: String) -> Self {
        Self::Acronym {
            acronym,
            topic_name,
        }
    }

}
