use itertools::Itertools;
use std::fs;

use crate::*;
use std::hash::{Hasher, Hash};
use super::*;
use crate::model::TopicKey;

#[allow(dead_code)]
pub(crate) enum WikiImageSize {
    Small,
    Medium,
    Large,
    Original,
}

impl WikiImageSize {
    pub(crate) fn suffix(&self) -> Option<&str> {
        match self {
            WikiImageSize::Small => Some("200"),
            WikiImageSize::Medium => Some("400"),
            WikiImageSize::Large => Some("600"),
            WikiImageSize::Original => None,
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) enum WikiImageLinkType {
    Detail,
    Direct,
    NoLink,
    LinkOnly,
}

impl WikiImageSize {
    pub(crate) fn to_name(&self) -> String {
        match self {
            WikiImageSize::Large => "Large",
            WikiImageSize::Medium => "Medium",
            WikiImageSize::Small => "Small",
            WikiImageSize::Original => "Original",
        }.to_string()
    }
}

impl PartialEq for WikiImageSize {
    fn eq(&self, other: &Self) -> bool {
        self.to_name() == other.to_name()
    }
}

impl Eq for WikiImageSize {}

impl Hash for WikiImageSize {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_name().hash(state);
    }
}

impl WikiImageLinkType {
    pub(crate) fn suffix(&self) -> Option<&str> {
        match self {
            WikiImageLinkType::Detail => None,
            WikiImageLinkType::Direct => Some("direct"),
            WikiImageLinkType::NoLink => Some("nolink"),
            WikiImageLinkType::LinkOnly => Some("linkonly"),
        }
    }
}

pub(crate) struct WikiAttributeTable {
    pub(crate) rows: Vec<WikiAttributeRow>,
}

pub(crate) struct WikiAttributeRow {
    pub(crate) label: String,
    pub(crate) markup: String,
}

pub(crate) struct WikiList {
    pub(crate) label: Option<String>,
    pub(crate) items: Vec<String>,
}

impl WikiAttributeTable {
    pub(crate) fn new() -> Self {
        Self {
            rows: vec![],
        }
    }

    pub(crate) fn add_row(&mut self, label: &str, markup: &str) {
        self.rows.push(WikiAttributeRow::new(label, markup));
    }

    #[allow(dead_code)]
    pub(crate) fn add_markup(&self, page_text: &mut String) {
        page_text.push_str(&self.get_markup());
    }

    pub(crate) fn get_markup(&self) -> String {
        // The attributes table should look something like:
        //   ^ Color    | Blue |
        //   ^ Count    | 4    |
        self.rows.iter()
            .map(|row| format!("^ {} | {} |\n", row.label, row.markup))
            .join("")
    }
}

impl WikiAttributeRow {
    pub(crate) fn new(label: &str, markup: &str) -> Self {
        Self {
            label: label.to_string(),
            markup: markup.to_string(),
        }
    }
}

impl WikiList {
    pub(crate) fn new(label: Option<String>) -> Self {
        Self {
            label,
            items: vec![],
        }
    }

    pub(crate) fn add_item(&mut self, markup: &str) {
        self.items.push(format!("  * {}", markup));
    }

    #[allow(dead_code)]
    pub(crate) fn add_item_indent(&mut self, depth: usize, markup: &str) {
        self.items.push(format!("{}* {}", "  ".repeat(depth + 1), markup));
    }

    #[allow(dead_code)]
    pub(crate) fn add_to_page(&self, page_text: &mut String) {
        page_text.push_str(&self.get_markup());
    }

    pub(crate) fn get_markup(&self) -> String {
        let mut markup = "".to_string();
        if let Some(label) = &self.label {
            markup.push_str(&format!("{}:\n", label));
        }
        for item in self.items.iter() {
            markup.push_str(&format!("{}\n", item));
        }
        markup
    }

    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }

}

#[allow(dead_code)]
pub(crate) fn add_headline(page_text: &mut String, text: &str, level: usize) {
    // Like "----- Categories -----" where a level 1 (top) headline has five hyphens.
    debug_assert!(level >= 1);
    debug_assert!(level <= 5);
    let markers = DELIM_HEADER.repeat(6 - level);
    page_text.push_str(&format!("{}{}{}\n\n", markers, text, markers));
}

/*
#[allow(dead_code)]
pub(crate) fn add_image_internal_link(page_text: &mut String, page_namespace: &str, page_name: &str, image_namespace: &str, image_file_name: &str, image_size: &WikiImageSize) {
    TopicKey::assert_legal_namespace(page_namespace);
    let image_part = image_part(image_namespace, image_file_name, &WikiImageLinkType::NoLink, image_size);
    page_text.push_str(&format!("[[{}:{}|{}]]\n\n", page_namespace, legal_file_name(page_name), &image_part));
}

#[allow(dead_code)]
pub(crate) fn add_image_link_to_full_image(page_text: &mut String, image_namespace: &str, image_file_name: &str, image_size: &WikiImageSize) {
    TopicKey::assert_legal_namespace(image_namespace);
    let image_part = image_part(image_namespace, image_file_name, &WikiImageLinkType::Direct, image_size);
    page_text.push_str(&format!("{}\n\n", &image_part));
}

#[allow(dead_code)]
pub(crate) fn add_image_table_row(page_text: &mut String, image_namespace: &str, image_size: &WikiImageSize, end_table: bool, image_file_names: &[&str]) {
    TopicKey::assert_legal_namespace(image_namespace);
    let markup = format!("|{}", image_file_names.iter()
        .map(|file_name| format!(" {} |", image_part(image_namespace, file_name, &WikiImageLinkType::Direct, image_size)))
        .join(""));
    let suffix = if end_table { "\n" } else { "" };
    page_text.push_str(&format!("{}\n{}", markup, suffix));
}

#[allow(dead_code)]
pub(crate) fn add_image_row(page_text: &mut String, image_namespace: &str, image_size: &WikiImageSize, image_file_names: &[&str]) {
    TopicKey::assert_legal_namespace(image_namespace);
    let markup = image_file_names.iter()
        .map(|file_name| format!("{}\n", image_part(image_namespace, file_name, &WikiImageLinkType::Direct, image_size)))
        .join("");
    add_paragraph(page_text, &markup);
}
*/

pub(crate) fn image_ref_from_file_name(image_namespace: &str, image_file_name: &str) -> String {
    TopicKey::assert_legal_namespace(image_namespace);
    format!("{}:{}", image_namespace, legal_file_name(image_file_name))
}

pub(crate) fn image_part(image_ref: &str, image_link_type: &WikiImageLinkType, image_size: &WikiImageSize) -> String {
    // image_ref is either a URL or something like "tools:abc.txt".
    let link_type_string = image_link_type.suffix();
    let size_string = image_size.suffix();
    let suffix = match (link_type_string, size_string) {
        (Some(link_type_string), Some(size_string)) => format!("{}&{}", link_type_string, size_string),
        (Some(link_type_string), None) => format!("{}", link_type_string),
        (None, Some(size_string)) => format!("{}", size_string),
        (None, None) => "".to_string(),
    };
    format!("{{{{{}?{}}}}}", image_ref, suffix)
}

/*
pub(crate) fn image_part(image_namespace: &str, image_file_name: &str, image_link_type: &WikiImageLinkType, image_size: &WikiImageSize) -> String {
    TopicKey::assert_legal_namespace(image_namespace);
    let link_type_string = image_link_type.suffix();
    let size_string = image_size.suffix();
    let suffix = match (link_type_string, size_string) {
        (Some(link_type_string), Some(size_string)) => format!("{}&{}", link_type_string, size_string),
        (Some(link_type_string), None) => format!("{}", link_type_string),
        (None, Some(size_string)) => format!("{}", size_string),
        (None, None) => "".to_string(),
    };
    // format!("{{{{:{}:{}?{}|}}}}", image_namespace, legal_file_name(image_file_name), suffix)
    format!("{{{{{}:{}?{}}}}}", image_namespace, legal_file_name(image_file_name), suffix)
}
*/

#[allow(dead_code)]
pub(crate) fn add_page_link(page_text: &mut String, namespace: &str, page_name: &str, label: Option<&str>) {
    // Like "[[nav:categories|Categories]]".
    TopicKey::assert_legal_namespace(namespace);
    page_text.push_str(&format!("{}\n\n", page_link(namespace, page_name, label)));
}

#[allow(dead_code)]
pub(crate) fn add_section_link(page_text: &mut String, namespace: &str, page_name: &str, section_name: &str, label: Option<&str>) {
    // Like "[[nav:categories|Categories#Five]]".
    TopicKey::assert_legal_namespace(namespace);
    page_text.push_str(&format!("{}\n\n", section_link(namespace, page_name, section_name, label)));
}

#[allow(dead_code)]
pub(crate) fn add_section_link_same_page(page_text: &mut String, section_name: &str, label: Option<&str>) {
    // Like "[[#All|All Categories]]".
    page_text.push_str(&format!("{}\n\n", section_link_same_page(section_name, label)));
}

#[allow(dead_code)]
pub(crate) fn add_line(page_text: &mut String) {
    page_text.push_str("\n");
}

#[allow(dead_code)]
pub(crate) fn add_list_item_unordered(page_text: &mut String, text: &str) {
    page_text.push_str(&format!("  * {}\n", text));
}

#[allow(dead_code)]
pub(crate) fn add_list_item_unordered_depth(page_text: &mut String, depth: usize, text: &str) {
    let indent = "  ".repeat(depth);
    page_text.push_str(&format!("  {}* {}\n", indent, text));
}

#[allow(dead_code)]
pub(crate) fn add_paragraph(page_text: &mut String, text: &str) {
    page_text.push_str(&format!("{}\n\n", text));
}

pub(crate) fn namespace_prefix(namespace: &str) -> String {
    TopicKey::assert_legal_namespace(namespace);
    if namespace.is_empty() {
        "".to_string()
    } else {
        format!("{}:", namespace)
    }
}

pub(crate) fn page_link(namespace: &str, page_name: &str, label: Option<&str>) -> String {
    TopicKey::assert_legal_namespace(namespace);
    // format!("[[{}{}{}]]", namespace_prefix(namespace), legal_file_name(page_name), label.map_or("".to_string(), |x| format!("|{}", x)))
    format!("[[{}{}{}{}]]", namespace_prefix(namespace), legal_file_name(page_name), DELIM_LINK_LABEL, label.unwrap_or(page_name))
}

pub(crate) fn page_link_from_string_label(namespace: &str, page_name: &str, label: &Option<String>) -> String {
    TopicKey::assert_legal_namespace(namespace);
    // format!("[[{}{}{}]]", namespace_prefix(namespace), legal_file_name(page_name), label.map_or("".to_string(), |x| format!("|{}", x)))
    format!("[[{}{}{}{}]]", namespace_prefix(namespace), legal_file_name(page_name), DELIM_LINK_LABEL, &label.as_ref().unwrap_or(&page_name.to_string()))
}

pub(crate) fn section_link(namespace: &str, page_name: &str, section_name: &str, label: Option<&str>) -> String {
    // Like "[[nav:categories#All|All Categories]]".
    // format!("[[{}{}#{}{}]]", namespace_prefix(namespace), legal_file_name(page_name), section_name, label.map_or("".to_string(), |x| format!("|{}", x)))
    // if label.map_or(false, |label| label.contains("#")) {
    //     label = None;
    //}
    TopicKey::assert_legal_namespace(namespace);
    // let label = label.unwrap_or(format!({}: {}))
    //format!("[[{}{}#{}|{}#{}]]", namespace_prefix(namespace), legal_file_name(page_name), section_name, label.unwrap_or(page_name), section_name)
    //format!("[[{}{}#{}]]", namespace_prefix(namespace), legal_file_name(page_name), section_name)
    let label = label.map_or(format!("{}: {}", page_name, section_name), |label| label.to_string());
    // format!("[[{}{}#{}|{}: {}]]", namespace_prefix(namespace), legal_file_name(page_name), section_name, page_name, section_name)
    format!("[[{}{}#{}{}{}]]", namespace_prefix(namespace), legal_file_name(page_name), section_name, DELIM_LINK_LABEL, label)
}

pub(crate) fn section_link_from_string_label(namespace: &str, page_name: &str, section_name: &str, label: &Option<String>) -> String {
    // Like "[[nav:categories#All|All Categories]]".
    // format!("[[{}{}#{}{}]]", namespace_prefix(namespace), legal_file_name(page_name), section_name, label.map_or("".to_string(), |x| format!("|{}", x)))
    // if label.map_or(false, |label| label.contains("#")) {
    //     label = None;
    //}
    TopicKey::assert_legal_namespace(namespace);
    // let label = label.unwrap_or(format!({}: {}))
    //format!("[[{}{}#{}|{}#{}]]", namespace_prefix(namespace), legal_file_name(page_name), section_name, label.unwrap_or(page_name), section_name)
    //format!("[[{}{}#{}]]", namespace_prefix(namespace), legal_file_name(page_name), section_name)
    let label = if let Some(label) = label {
        label.clone()
    } else {
        format!("{}: {}", page_name, section_name)
    };
    // format!("[[{}{}#{}|{}: {}]]", namespace_prefix(namespace), legal_file_name(page_name), section_name, page_name, section_name)
    format!("[[{}{}#{}{}{}]]", namespace_prefix(namespace), legal_file_name(page_name), section_name, DELIM_LINK_LABEL, label)
}

pub(crate) fn section_link_same_page(section_name: &str, label: Option<&str>) -> String {
    // Like "[[#All|All Categories]]".
    // format!("[[#{}{}]]", section_name, label.map_or("".to_string(), |x| format!("|{}", x)))
    let label = label.unwrap_or(section_name);
    // format!("[[#{}|: {}]]", section_name, section_name)
    format!("[[#{}{}{}]]", section_name, DELIM_LINK_LABEL, label)
}

#[allow(dead_code)]
pub(crate) fn external_link(url: &str, label: Option<&str>) -> String {
    // Like "[[https://github.com/|external link|GitHub]]".
    format!("[[{}{}]]", url, label.map_or("".to_string(), |x| format!("{}{}", DELIM_LINK_LABEL, x)))
}

pub(crate) fn external_link_from_string_label(url: &str, label: &Option<String>) -> String {
    // Like "[[https://github.com/|external link|GitHub]]".
    let label = if let Some(label) = label {
        format!("{}{}", DELIM_LINK_LABEL, label)
    } else {
        "".to_string()
    };
    format!("[[{}{}]]", url, label)
}

pub(crate) fn file_ref(file_ref: &str, label: &Option<String>) -> String {
    let label = if let Some(label) = label {
        format!("{}: ", label)
    } else {
        "".to_string()
    };
    format!("{}[{}]", label, file_ref)
}

pub(crate) fn legal_file_name(name: &str) -> String {
    // https://www.dokuwiki.org/pagename. From that page_text:
    //   page_text names in DokuWiki are converted to lowercase automatically. Allowed characters are
    //   letters, digits and, within names, the "special characters" ., - and _. All other special
    //   characters (i.e. other than letters and digits – whitespace, plus, slash, percent, etc.)
    //   are converted to underscores. Colons (:) are used to identify namespaces.
    let page_name = name;
    let page_name = page_name.replace("(", "");
    let page_name = page_name.replace(")", "");
    let page_name = page_name.trim().to_lowercase()
        .chars()
        .map(|c| {
            // if c.is_alphabetic() || c.is_digit(10) || c == '.' || c == '-' || c == '_' {
            if c.is_alphabetic() || c.is_digit(10) || c == '.' || c == '_' {
                c
            } else if c == '+' {
                // This means "C++" will be turned into "cpp" rather than simply "c" which would
                // overwrite a topic titled "C".
                'p'
            } else {
                '_'
            }
        })
        .join("");
    let page_name = format::remove_repeated(&page_name, "_");
    let page_name = page_name.trim_end_matches("_").to_string();
    // if name.contains("Disrupt") { //bg!(name, &page_name); }
    page_name
}

pub(crate) fn internal_link_name(name: &str) -> String {
    let name = name.replace(".", "_");
    let name = legal_file_name(&name);
    name
}

pub(crate) fn namespace_to_path(namespace: &str) -> String {
    TopicKey::assert_legal_namespace(namespace);
    namespace.replace(":", "/")
}

#[allow(dead_code)]
pub(crate) fn write_page(folder: &str, namespace: &str, name: &str, text: &str) {
    TopicKey::assert_legal_namespace(namespace);
    fs::write(format!("{}/{}/{}.txt", folder, namespace_to_path(namespace), legal_file_name(name)), text).unwrap();
}

/*
pub(crate) fn copy_image_file(from_path: &str, from_file_name: &str, to_path: &str, to_namespace: &str, to_file_name: &str) {
    TopicKey::assert_legal_namespace(to_namespace);
    let from_full_file_name = format!("{}/{}", from_path, from_file_name);
    let to_full_file_name = format!("{}/{}/{}", to_path, namespace_to_path(to_namespace), legal_file_name(to_file_name));
    println!("{} => {}", from_full_file_name, to_full_file_name);
    std::fs::copy(from_full_file_name, to_full_file_name).unwrap();
}
*/

#[allow(dead_code)]
pub(crate) fn bold(value: &str) -> String {
    format!("{}{}{}", DELIM_BOLD, value, DELIM_BOLD)
}

#[allow(dead_code)]
pub(crate) fn italic(value: &str) -> String {
    format!("{}{}{}", DELIM_ITALIC, value, DELIM_ITALIC)
}

