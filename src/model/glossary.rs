use crate::model::{Table, TextBlock, LinkRc, Model, TopicKey, TableCell, HorizontalAlignment};
use crate::Itertools;
use std::collections::BTreeMap;
use crate::dokuwiki::gen_tools_wiki::PROJECT_NAME;

// TO DO: Look for mismatched links between terms and acronyms.

const ABBREVIATION: &str = "Abbreviation";
const ACRONYM: &str = "Acronym";
const PREFIX_ABBREVIATION: &str = "Abbreviation of";
const PREFIX_ACRONYM: &str = "Acronym for";

#[derive(Clone, Debug)]
pub(crate) struct Glossary {
    pub _name: String,
    pub topic_key: Option<TopicKey>,
    pub _tags: Vec<String>,
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
        tags: Vec<String>,
    },
    Acronym {
        acronym: String,
        term_name: String,
        link: Option<LinkRc>,
        tags: Vec<String>,
        is_abbreviation: bool,
    }
}

impl Glossary {
    pub(crate) fn new_with_raw_list(name: &str, topic_key: Option<TopicKey>, tags: Vec<String>, raw_list: Table) -> Self {
        Self {
            _name: name.to_string(),
            topic_key,
            _tags: tags,
            items: Default::default(),
            raw_list,
        }
    }

    pub(crate) fn build_from_raw_list(&mut self, model: &Model) -> Vec<String> {
        assert!(self.items.is_empty());
        assert!(!self.raw_list.is_empty());
        let mut warnings = vec![];
        let mut terms = BTreeMap::new();
        let mut acronyms = BTreeMap::new();
        for (row_index, row) in self.raw_list.get_rows().iter().enumerate() {
            // At first treat everything like a term including acronyms.
            // At this point the first cell should have a text block with a single resolved
            // TextItem.

            let item_name = row[0].get_text_block().get_single_resolved_text();
            //et debug = item_name.eq("Application Virtual Machine");
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
            //f debug { dbg!(&definition); panic!() }

            let definition_text = definition.get_display_text().trim().to_string();

            let tags = row[3].get_text_block().get_display_text();
            let mut tags = tags.split(",").map(|x| x.trim().to_lowercase().to_string()).collect::<Vec<_>>();
            tags.sort();

            if definition_text.starts_with(ACRONYM) || definition_text.starts_with(ABBREVIATION) {
                if acronym.is_some() {
                    warnings.push(format!("Row {}: Seems to be an acronym or abbreviation but it has an acronym: item_name = \"{}\"; acronym = \"{}\".", row_index, item_name, acronym.unwrap()));
                }
                let (term_name, is_abbreviation) = if definition_text.starts_with(ACRONYM) {
                    if definition_text.starts_with(PREFIX_ACRONYM) {
                        (util::parse::between_trim(&definition_text, PREFIX_ACRONYM, ".").to_string(), false)
                    } else {
                        warnings.push(format!("Row {}: Definition starts with \"{}\" but not \"{}\".", row_index, ACRONYM, PREFIX_ACRONYM));
                        (definition_text.to_string(), false)
                    }
                } else {
                    if definition_text.starts_with(PREFIX_ABBREVIATION) {
                        (util::parse::between_trim(&definition_text, PREFIX_ABBREVIATION, ".").to_string(), true)
                    } else {
                        warnings.push(format!("Row {}: Definition starts with \"{}\" but not \"{}\".", row_index, ABBREVIATION, PREFIX_ABBREVIATION));
                        (definition_text.to_string(), true)
                    }
                };
                let acronym = GlossaryItem::new_acronym(item_name.to_string(), term_name.to_string(), link, tags, is_abbreviation);
                let mut key = item_name.to_lowercase();
                if terms.contains_key(&key) || acronyms.contains_key(&key) {
                    warnings.push(format!("Row {}: Duplicate key = \"{}\".", row_index, key));
                    key = format!("{}_", key);
                }
                acronyms.insert(key, acronym);
            } else {
                let term = GlossaryItem::new_term(item_name.to_string(), acronym, link, definition, tags);
                let mut key = item_name.to_lowercase();
                if terms.contains_key(&key) || acronyms.contains_key(&key) {
                    warnings.push(format!("Row {}: Duplicate key = \"{}\".", row_index, key));
                    key = format!("{}_", key);
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
                            warnings.push(format!("Glossary::build_from_raw_list(): Creating acronym \"{}\" for term \"{}\".", acronym, term));
                            acronyms.insert(key, GlossaryItem::new_acronym(acronym.to_string(), term.to_string(), None, vec![], false));
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
                            warnings.push(format!("Term \"{}\" not found for acronym \"{}\".", term_name, acronym));
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
                            warnings.push(format!("Glossary::build_from_raw_list(): Term \"{}\" might need a link to {}.", term, topic_key));
                        } else {
                            if let Some(acronym) = acronym {
                                let topic_key = TopicKey::new(&namespace, &format!("{} ({})", term, acronym));
                                if model.get_topics().contains_key(&topic_key) {
                                    warnings.push(format!("Glossary::build_from_raw_list(): Term \"{}\" might need a link to {}.", term, topic_key));
                                }
                            }
                        }
                    }
                },
                _ => {},
            }
        }

        // Update acronyms to match the links and tags of their corresponding terms.
        for acronym in acronyms.values_mut() {
            match acronym {
                GlossaryItem::Acronym { acronym: _, term_name, link, tags, .. } => {
                    let key = term_name.to_lowercase();
                    if let Some(term) = terms.get(&key) {
                        match term {
                            GlossaryItem::Term { term: _, acronym: _, link: term_link, definition: _, tags: term_tags } => {
                                *link = term_link.clone();
                                *tags = term_tags.clone();
                            },
                            GlossaryItem::Acronym { acronym, .. } => {
                                warnings.push(format!("Acronym \"{}\" found in terms list.", acronym));
                            }
                        }
                    }
                },
                GlossaryItem::Term { term, .. } => {
                    warnings.push(format!("Term \"{}\" found in acronyms list.", term));
                }
            }
        }

        self.items.append(&mut terms);
        self.items.append(&mut acronyms);

        warnings
    }

    pub(crate) fn make_table(&self) -> Table {
        // This is the final table used for generating the page, as opposed to the initial raw
        // table we got at the beginning of the process.
        let mut table = Table::new(false);
        for item in self.items.values() {
            let row = match item {
                GlossaryItem::Acronym { acronym, term_name, link, tags, is_abbreviation } => {
                    let prefix = if *is_abbreviation { PREFIX_ABBREVIATION } else { PREFIX_ACRONYM };
                    let definition = format!("{} {}.", prefix, term_name);
                    let definition = TextBlock::new_resolved_text(&definition);
                    Self::make_table_row(acronym, link, &definition, tags)
                },
                GlossaryItem::Term { term, acronym, link, definition, tags } => {
                    let name = match acronym {
                        Some(acronym) => format!("{} ({})", term, acronym),
                        None => term.to_string(),
                    };
                    Self::make_table_row(&name, link, &definition, tags)
                },
            };
            table.add_row(row);
        }
        table
    }

    fn make_table_row(name: &str, link: &Option<LinkRc>, definition: &TextBlock, tags: &Vec<String>) -> Vec<TableCell> {
        let align = HorizontalAlignment::Left;
        let bold = false;
        let mut cells = vec![];
        cells.push(TableCell::new_resolved_text(name, bold, &align));
        cells.push(TableCell::new_link_rc_opt(link.clone(), bold, &align));
        cells.push(TableCell::new_text_block(definition.clone(), bold, &align));
        let tags = tags.iter().join(", ");
        cells.push(TableCell::new_resolved_text(&tags, bold, &align));
        cells
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

    pub(crate) fn get_links(&self) -> Vec<LinkRc> {
        if self.items.is_empty() {
            self.raw_list.get_links()
        } else {
            let mut list = vec![];
            for item in self.items.values() {
                match item {
                    GlossaryItem::Acronym { acronym: _, term_name: _, link, .. } => {
                        if let Some(link) = link {
                            list.push(link.clone());
                        }
                    },
                    GlossaryItem::Term { term: _, acronym: _, link, definition, .. } => {
                        if let Some(link) = link {
                            list.push(link.clone());
                        }
                        list.append(&mut definition.get_links());
                    }
                }
            }
            list
        }
    }
}

impl GlossaryItem {
    pub fn new_term(term: String, acronym: Option<String>, link: Option<LinkRc>, definition: TextBlock, tags: Vec<String>) -> Self {
        Self::Term {
            term,
            acronym,
            link,
            definition,
            tags,
        }
    }

    pub fn new_acronym(acronym: String, term_name: String, link: Option<LinkRc>, tags: Vec<String>, is_abbreviation: bool) -> Self {
        Self::Acronym {
            acronym,
            term_name,
            link,
            tags,
            is_abbreviation,
        }
    }

}

const ACRONYMS_NO_TERM_OK: [&str; 3] = ["CACHE", "CLI", "FTE"];
