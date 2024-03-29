use crate::model::{Table, TextBlock, TextItem, LinkRc, Model, TopicKey, TableCell, HorizontalAlignment};
use crate::Itertools;
use std::collections::BTreeMap;
use crate::dokuwiki::gen_tools_wiki::PROJECT_NAME;
use crate::dokuwiki::MARKER_TERMS_ADD_DEFINITION;

// TO DO: Look for mismatched links between terms and acronyms.

const ABBREVIATION: &str = "Abbreviation";
const ACRONYM: &str = "Acronym";
const PREFIX_ABBREVIATION: &str = "Abbreviation of";
const PREFIX_ACRONYM: &str = "Acronym for";

#[derive(Clone, Debug)]
pub(crate) struct Glossary {
    pub topic_key: Option<TopicKey>,
    pub items: BTreeMap<String, GlossaryItem>,
    pub raw_list: Table,
}

#[derive(Clone, Debug)]
pub(crate) struct GlossaryItem {
    type_: GlossaryItemType,
    name: String,
    term: String,
    acronym: Option<String>,
    link: Option<LinkRc>,
    definition: TextBlock,
    alt_definition: Option<TextBlock>,
    tags: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) enum GlossaryItemType {
    Abbreviation,
    Acronym,
    Term,
}

impl Glossary {
    pub(crate) fn new_with_raw_list(topic_key: Option<TopicKey>, raw_list: Table) -> Self {
        Self {
            topic_key,
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
        let mut keys = vec![];
        for (row_index, row) in self.raw_list.get_rows().iter().enumerate() {

            assert!(row.len() == 4, "On the terms page, row {}: Term row does not have four cells: {:?}", row_index, row);

            // At first treat everything like a term including acronyms.
            // At this point the first cell should have a text block with a single resolved
            // TextItem.

            let term = row[0].get_text_block().get_single_resolved_text();
            //et debug = item_name.eq("Application Virtual Machine");
            assert!(!term.is_empty());
            let (mut term, mut acronym) = Self::split_term_acronym(term);

            let link_text_block = row[1].get_text_block();
            let item_count = link_text_block.get_resolved_items().len();
            assert!(item_count < 2);
            let link = if item_count == 1 {
                // Normally there should be a single TextItem::Link in the text block, but it may
                // have been redacted in which case it will be a TextItem::Text and we can ignore
                // it.
                match &link_text_block.get_resolved_items()[0] {
                    TextItem::Link { link } => Some(link.clone()),
                    _ => None,
                }
                //Some(link_text_block.get_single_link())
            } else {
                None
            };

            let definition = row[2].get_text_block().clone();
            //f debug { dbg!(&definition); panic!() }

            let definition_text = definition.get_display_text().trim().to_string();
            if !definition_text.is_empty() && !definition_text.ends_with(".") && !definition_text.ends_with("?") && !definition_text.ends_with(MARKER_TERMS_ADD_DEFINITION) {
                warnings.push(format!("Row {}: Definition does not end with a period, question mark, or \"{}\": term = \"{}\"", row_index, MARKER_TERMS_ADD_DEFINITION, term));
            }

            let tags = row[3].get_text_block().get_display_text();
            let mut tags = tags.split(",").map(|x| x.trim().to_lowercase().to_string()).collect::<Vec<_>>();
            tags.sort();

            let type_ = if definition_text.starts_with(ACRONYM) {
                GlossaryItemType::Acronym
            } else if definition_text.starts_with(ABBREVIATION) {
                GlossaryItemType::Abbreviation
            } else {
                GlossaryItemType::Term
            };
            let item_key = term.to_lowercase();
            let prefix = type_.get_prefix();
            let (name, list) = match type_ {
                GlossaryItemType::Term => {
                    let name = match &acronym {
                        Some(acronym) => format!("{} ({})", term, acronym),
                        None => term.to_string(),
                    };
                    (name, &mut terms)
                },
                GlossaryItemType::Acronym | GlossaryItemType::Abbreviation => {
                    let name = term.clone();
                    if let Some(acronym) = acronym {
                        warnings.push(format!("Row {}: Seems to be an acronym or abbreviation but it has an acronym: item_name = \"{}\"; acronym = \"{}\".", row_index, term, acronym));
                    }
                    acronym = Some(term.clone());
                    if definition_text.starts_with(prefix) {
                        term = util::parse::between_trim_first(&definition_text, prefix, ".").to_string();
                    } else {
                        warnings.push(format!("Row {}: Definition for \"{}\" does not start with \"{}\": \"{}\".", row_index, term, prefix, definition_text));
                    }
                    (name, &mut acronyms)
                },
            };
            let item = GlossaryItem::new(type_, name, term, acronym.clone(), link, definition, tags);
            Self::add_item(list, &mut keys, &mut warnings, item_key, item);
        }

        // Find cases where a term has an acronym but we don't have the corresponding acronym
        // entry.
        for item in terms.values() {
            if let Some(acronym) = &item.acronym {
                let key = acronym.to_lowercase();
                if !acronyms.contains_key(&key) {
                    warnings.push(format!("Glossary::build_from_raw_list(): Creating acronym \"{}\" for term \"{}\".", acronym, item.term));
                    let acronym_item = GlossaryItem::new(GlossaryItemType::Acronym, acronym.to_string(), item.term.clone(), Some(acronym.to_string()), None, TextBlock::new_resolved_text(""),vec![]);
                    Self::add_item(&mut acronyms, &mut keys, &mut warnings, key, acronym_item);
                }
            }
        }

        // Find cases where we have an acronym but not the corresponding term entry.
        for item in acronyms.values() {
            if !ACRONYMS_NO_TERM_OK.contains(&item.acronym.as_ref().unwrap().as_str()) {
                let key = item.term.to_lowercase();
                if !terms.contains_key(&key) {
                    warnings.push(format!("Term \"{}\" not found for acronym \"{}\".", item.term, item.acronym.as_ref().unwrap()));
                }
            }
        }

        // Find cases where a term looks like it should be linked to a topic.
        let namespace = PROJECT_NAME.to_lowercase();
        for item in terms.values() {
            if item.link.is_none() {
                // For now assume the only namespace is "tools".
                if let Some(topic_key) = TopicKey::new_if_legal(&namespace, &item.term) {
                    if model.get_topics().contains_key(&topic_key) {
                        warnings.push(format!("Glossary::build_from_raw_list(): Term \"{}\" might need a link to {}.", item.term, topic_key));
                    }
                }
                if let Some(acronym) = &item.acronym {
                    if let Some(topic_key) = TopicKey::new_if_legal(&namespace, &format!("{} ({})", item.term, acronym)) {
                        if model.get_topics().contains_key(&topic_key) {
                            warnings.push(format!("Glossary::build_from_raw_list(): Term \"{}\" might need a link to {}.", item.term, topic_key));
                        }
                    }
                }
            }
        }

        // Update acronyms to match the links, tags, and definitions of their corresponding terms.
        for item in acronyms.values_mut() {
            let key = item.term.to_lowercase();
            if let Some(term_item) = terms.get(&key) {
                item.link = term_item.link.clone();
                item.tags = term_item.tags.clone();

                let prefix = item.type_.get_prefix();
                let mut definition = term_item.definition.clone();
                let mut beginning = format!("{} {}.", prefix, term_item.term);
                if !term_item.definition.get_display_text().is_empty() {
                    beginning.push_str(" ");
                }
                definition.insert_item(0, TextItem::new_text(&beginning));
                item.definition = definition;

                // Make an alternative definition for when the acronym or abbreviation appears in
                // a table of only acronyms or abbreviations. In that case we don't need the
                // "Acronym for " or "Abbreviation of " prefix.
                let mut alt_definition = term_item.definition.clone();
                let mut beginning = format!("{}.", term_item.term);
                if !term_item.definition.get_display_text().is_empty() {
                    beginning.push_str(" ");
                }
                alt_definition.insert_item(0, TextItem::new_text(&beginning));
                item.alt_definition = Some(alt_definition);
            }
        }

        self.items.append(&mut terms);
        self.items.append(&mut acronyms);

        warnings
    }

    fn add_item(list: &mut BTreeMap<String, GlossaryItem>, keys: &mut Vec<String>, warnings: &mut Vec<String>, mut key: String, item: GlossaryItem) {
        if keys.contains(&key) {
            warnings.push(format!("Duplicate key = \"{}\".", key));
            key = format!("{}_", key);
        }
        list.insert(key, item);
    }

    pub(crate) fn make_table(&self, include_terms: bool, include_acronyms: bool, mut show_tags: bool, included_tags: &Option<Vec<&str>>, excluded_tags: &Option<Vec<&str>>, filter_is_public: bool) -> Table {
        // This is the final table used for generating the page, as opposed to the initial raw
        // table we got at the beginning of the process.
        // If both include_terms and include_acronyms are false, the table will be empty.
        assert!(include_terms || include_acronyms);
        // If included_tags or excluded_tags are provided, they can't be empty.
        if let Some(tags) = included_tags {
            assert!(!tags.is_empty());
        }
        if let Some(tags) = excluded_tags {
            assert!(!tags.is_empty());
        }

        if filter_is_public {
            show_tags = false;
        }

        let excluded_tags = if filter_is_public {
            match excluded_tags {
                Some(tags) => {
                    let mut tags = tags.clone();
                    tags.push("p");
                    Some(tags)
                },
                None => {
                    Some(vec!["p"])
                },
            }
        } else {
            excluded_tags.clone()
        };

        let mut table = Table::new(false);
        for item in self.items.values()
            .filter(|item|
                if item.type_.is_term() { include_terms } else { include_acronyms }
                    && item.is_included_by_tag(included_tags)
                    && !item.is_excluded_by_tag(&excluded_tags)) {
            let definition = if !item.type_.is_term() && !include_terms {
                // We're creating a table only of acronyms and abbreviations, so use a slightly
                // shorter definition that doesn't include the "Acronym for" or "Abbreviation of"
                // prefix.
                &item.alt_definition.as_ref().expect(&format!("Expected item to have an alt_definition: \"{}\". Maybe it doesn't start correctly with \"Acronym for\" or \"Abbreviation of\".", item.name))
            } else {
                &item.definition
            };
            let row = Self::make_table_row(&item.name, &item.link, definition, &item.tags, show_tags);
            table.add_row(row);
        }
        table
    }

    fn make_table_row(name: &str, link: &Option<LinkRc>, definition: &TextBlock, tags: &Vec<String>, show_tags: bool) -> Vec<TableCell> {
        let align = HorizontalAlignment::Left;
        let bold = false;
        let mut cells = vec![];
        cells.push(TableCell::new_resolved_text(name, bold, &align));
        cells.push(TableCell::new_link_rc_opt(link.clone(), bold, &align));
        cells.push(TableCell::new_text_block(definition.clone(), bold, &align));
        if show_tags {
            let tags = tags.iter().join(", ");
            cells.push(TableCell::new_resolved_text(&tags, bold, &align));
        }
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
                if let Some(link) = &item.link {
                    list.push(link.clone());
                }
                list.append(&mut item.definition.get_links());
            }
            list
        }
    }

    fn split_term_acronym(text: &str) -> (String, Option<String>) {
        let (term, acronym) = util::parse::split_1_or_2_trim(text, "(");
        let term = term.to_string();
        let acronym = acronym.map(|x| util::parse::before(x, ")").trim().to_string());
        (term, acronym)
    }
}

impl GlossaryItem {
    pub fn new(type_: GlossaryItemType, name: String, term: String, acronym: Option<String>, link: Option<LinkRc>, definition: TextBlock, tags: Vec<String>) -> Self {
        Self {
            type_,
            name,
            term,
            acronym,
            link,
            definition,
            alt_definition: None,
            tags,
        }
    }

    fn is_included_by_tag(&self, tags: &Option<Vec<&str>>) -> bool {
        match tags {
            Some(tags) => self.is_matched_by_tag(&tags),
            None => true,
        }
    }

    fn is_excluded_by_tag(&self, tags: &Option<Vec<&str>>) -> bool {
        match tags {
            Some(tags) => self.is_matched_by_tag(&tags),
            None => false,
        }
    }

    fn is_matched_by_tag(&self, match_tags: &Vec<&str>) -> bool {
        for tag in self.tags.iter() {
            if match_tags.contains(&&**tag) {
                return true;
            }
        }
        return false;
    }

}

impl GlossaryItemType {
    pub fn get_prefix(&self) -> &str {
        match self {
            GlossaryItemType::Abbreviation => PREFIX_ABBREVIATION,
            GlossaryItemType::Acronym => PREFIX_ACRONYM,
            GlossaryItemType::Term => "",
        }
    }

    pub fn is_term(&self) -> bool {
        match self {
            GlossaryItemType::Term => true,
            _ => false,
        }
    }
}

const ACRONYMS_NO_TERM_OK: [&str; 3] = ["CACHE", "CLI", "FTE"];

