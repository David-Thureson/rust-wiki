use crate::model::{Table, TextBlock, LinkRc, Model, TopicKey};
use crate::Itertools;
use std::collections::BTreeMap;
use crate::dokuwiki::gen_tools_wiki::PROJECT_NAME;

const GLOSSARY_TABLE_COLUMN_COUNT: usize = 3;

#[derive(Clone, Debug)]
pub(crate) struct Glossary {
    pub name: String,
    pub _tags: Vec<&'static str>,
    pub items: BTreeMap<String, GlossaryItem>,
    pub raw_list: Table,
}

#[derive(Clone, Debug)]
pub(crate) enum GlossaryItem {
    Term {
        term: String,
        acronym: Option<String>,
        link: Option<LinkRc>,
        definition: TextBlock,
        tags: Vec<&'static str>,
    },
    Acronym {
        acronym: String,
        term_name: String,
        link: Option<LinkRc>,
        is_abbreviation: bool,
    }
}

impl Glossary {
    pub(crate) fn new_with_raw_list(name: &str, tags: Vec<&'static str>, raw_list: Table) -> Self {
        Self {
            name: name.to_string(),
            _tags: tags,
            items: Default::default(),
            raw_list,
        }
    }

    pub(crate) fn build_from_raw_list(&mut self, model: &Model) -> Option<String> {
        assert!(self.items.is_empty());
        assert!(!self.raw_list.is_empty());
        let mut errors = vec![];
        self.raw_list.trim();
        let mut terms = BTreeMap::new();
        let mut acronyms = BTreeMap::new();
        for (row_index, row) in self.raw_list.get_rows().iter().enumerate() {
            if row.len() != GLOSSARY_TABLE_COLUMN_COUNT {
                errors.push(format!("Row {}: Expected {} columns, found {}.", row_index, GLOSSARY_TABLE_COLUMN_COUNT, row.len()));
                continue;
            }
            // At first treat everything like a term including acronyms.
            // At this point the first cell should have a text block with a single resolved
            // TextItem.

            let item_name = row[0].get_text_block().get_single_resolved_text();
            assert!(!item_name.is_empty());
            let (item_name, acronym) = util::parse::split_1_or_2_trim(item_name, "(");
            let acronym = acronym.map(|x| util::parse::before(x, ")").trim().to_string());

            let link_text_block = row[1].get_text_block();
            let item_count = link_text_block.get_resolved_items().len();
            assert!(item_count < 2);
            let link = if item_count == 1 {
                Some(link_text_block.get_single_link())
            } else {
                None
            };

            let definition = row[2].get_text_block().clone();

            let definition_text = definition.get_display_text();
            if definition_text.starts_with("Acronym ") || definition_text.starts_with("Abbreviation ") {
                if acronym.is_some() {
                    errors.push(format!("Row {}: Seems to be an acronym or abbreviation but it has an acronym: item_name = \"{}\"; acronym = \"{}\".", row_index, item_name, acronym.unwrap()));
                }
                let (term_name, is_abbreviation) = if definition_text.starts_with("Acronym") {
                    (util::parse::between_trim(&definition_text, "Acronym for", "."), false)
                } else {
                    (util::parse::between_trim(&definition_text, "Abbreviation of", "."), false)
                };
                let acronym = GlossaryItem::new_acronym(item_name.to_string(), term_name.to_string(), link, is_abbreviation);
                let key = item_name.to_lowercase();
                if terms.contains_key(&key) || acronyms.contains_key(&key) {
                    errors.push(format!("Row {}: Duplicate key = \"{}\".", row_index, key));
                }
                acronyms.insert(key, acronym);
            } else {
                let tags = vec![];
                let term = GlossaryItem::new_term(item_name.to_string(), acronym, link, definition, tags);
                let key = item_name.to_lowercase();
                if terms.contains_key(&key) || acronyms.contains_key(&key) {
                    errors.push(format!("Row {}: Duplicate key = \"{}\".", row_index, key));
                }
                terms.insert(key, term);
            }
        }

        // Find cases where a term has an acronym but we don't have the corresponding acronym
        // entry.
        for term in terms.values() {
            match term {
                GlossaryItem::Term { term, acronym, .. } => {
                    if let Some(acronym) = acronym {
                        let key = acronym.to_lowercase();
                        if !acronyms.contains_key(&key) {
                            println!("Glossary::build_from_raw_list(): Creating acronym \"{}\" for term \"{}\".", acronym, term);
                            acronyms.insert(key, GlossaryItem::new_acronym(acronym.to_string(), term.to_string(), None, false));
                        }
                    }
                },
                _ => {},
            }
        }

        // Find cases where we have an acronym but not the corresponding term entry.
        for acronym in acronyms.values() {
            match acronym {
                GlossaryItem::Acronym { acronym, term_name, .. } => {
                    if !ACRONYMS_NO_TERM_OK.contains(&&**acronym) {
                        let key = term_name.to_lowercase();
                        if !terms.contains_key(&key) {
                            // terms.insert(key, GlossaryItem::new_term(term_name.to_string(), Some(acronym.to_string()), link.clone(), TextBlock::new_resolved_text(""), vec![]));
                            errors.push(format!("Term \"{}\" not found for acronym \"{}\".", term_name, acronym));
                        }
                    }
                },
                _ => {},
            }
        }

        // Find cases where a term looks like it should be linked to a topic.
        let namespace = PROJECT_NAME.to_lowercase();
        for term in terms.values() {
            match term {
                GlossaryItem::Term { term, acronym, link, .. } => {
                    if link.is_none() {
                        // For now assume the only namespace is "tools".
                        let topic_key = TopicKey::new(&namespace, term);
                        if model.get_topics().contains_key(&topic_key) {
                            println!("Glossary::build_from_raw_list(): Term \"{}\" might need a link to {}.", term, topic_key);
                        } else {
                            if let Some(acronym) = acronym {
                                let topic_key = TopicKey::new(&namespace, &format!("{} ({})", term, acronym));
                                if model.get_topics().contains_key(&topic_key) {
                                    println!("Glossary::build_from_raw_list(): Term \"{}\" might need a link to {}.", term, topic_key);
                                }
                            }
                        }
                    }
                },
                _ => {},
            }
        }

        self.items.append(&mut terms);
        self.items.append(&mut acronyms);

        let error_string = if errors.is_empty() {
            None
        } else {
            Some(errors.iter()
                .map(|x| format!("\n{}", x))
                .join(""))
        };
        error_string
    }

    /*
    pub(crate) fn get_all_text_blocks_cloned(&self) -> Vec<TextBlock> {
        if self.terms.is_empty() {
            self.raw_list.get_all_text_blocks_cloned()
        } else {
            let mut list = vec![];
            for item in self.terms.iter() {
                match item {
                    GlossaryItem::Acronym { .. } => {},
                    GlossaryItem::Term { term: _, acronym: _, link: _, definition, .. } => {
                        list.push(definition.clone());
                    }
                }
            }
            list
        }
    }
    */

    /*
    pub(crate) fn get_links(&self) -> Vec<LinkRc> {
        if self.terms.is_empty() {
            self.raw_list.get_links()
        } else {
            let mut list = vec![];
            for item in self.terms.iter() {
                match item {
                    GlossaryItem::Acronym { .. } => {},
                    GlossaryItem::Term { term: _, acronym: _, link, .. } => {
                        if let Some(link) = link {
                            list.push(link.clone());
                        }
                    }
                }
            }
            list
        }
    }
    */
}

impl GlossaryItem {
    pub fn new_term(term: String, acronym: Option<String>, link: Option<LinkRc>, definition: TextBlock, tags: Vec<&'static str>) -> Self {
        Self::Term {
            term,
            acronym,
            link,
            definition,
            tags,
        }
    }

    pub fn new_acronym(acronym: String, term_name: String, link: Option<LinkRc>, is_abbreviation: bool) -> Self {
        Self::Acronym {
            acronym,
            term_name,
            link,
            is_abbreviation,
        }
    }

}

const ACRONYMS_NO_TERM_OK: [&str; 3] = ["CACHE", "CLI", "FTE"];
