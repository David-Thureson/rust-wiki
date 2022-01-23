use std::hash::{Hash, Hasher};

use crate::*;
use std::fs;

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

/*
pub(crate) fn add_headline(page_text: &mut String, text: &str, level: usize) {
    // Like "----- Categories -----" where a level 1 (top) headline has five hyphens.
    debug_assert!(level >= 1);
    debug_assert!(level <= 5);
    let equal_signs = "=".repeat(6 - level);
    page_text.push_str(&format!("{}{}{}\n\n", equal_signs, text, equal_signs));
}

pub(crate) fn add_image_internal_link(page_text: &mut String, page_namespace: &str, page_name: &str, image_namespace: &str, image_file_name: &str, image_size: &WikiImageSize) {
    let image_part = image_part(image_namespace, image_file_name, &WikiImageLinkType::NoLink, image_size);
    page_text.push_str(&format!("[[{}:{}|{}]]\n\n", page_namespace, legal_file_name(page_name), &image_part));
}

pub(crate) fn add_image_link_to_full_image(page_text: &mut String, image_namespace: &str, image_file_name: &str, image_size: &WikiImageSize) {
    let image_part = image_part(image_namespace, image_file_name, &WikiImageLinkType::Direct, image_size);
    page_text.push_str(&format!("{}\n\n", &image_part));
}

pub(crate) fn add_image_table_row(page_text: &mut String, image_namespace: &str, image_size: &WikiImageSize, end_table: bool, image_file_names: &[&str]) {
    let markup = format!("|{}", image_file_names.iter()
        .map(|file_name| format!(" {} |", image_part(image_namespace, file_name, &WikiImageLinkType::Direct, image_size)))
        .join(""));
    let suffix = if end_table { "\n" } else { "" };
    page_text.push_str(&format!("{}\n{}", markup, suffix));
}

pub(crate) fn add_image_row(page_text: &mut String, image_namespace: &str, image_size: &WikiImageSize, image_file_names: &[&str]) {
    let markup = image_file_names.iter()
        .map(|file_name| format!("{}\n", image_part(image_namespace, file_name, &WikiImageLinkType::Direct, image_size)))
        .join("");
    add_paragraph(page_text, &markup);
}

pub(crate) fn image_part(image_namespace: &str, image_file_name: &str, image_link_type: &WikiImageLinkType, image_size: &WikiImageSize) -> String {
    let link_type_string = image_link_type.suffix();
    let size_string = image_size.suffix();
    let suffix = match (link_type_string, size_string) {
        (Some(link_type_string), Some(size_string)) => format!("{}&{}", link_type_string, size_string),
        (Some(link_type_string), None) => format!("{}", link_type_string),
        (None, Some(size_string)) => format!("{}", size_string),
        (None, None) => "".to_string(),
    };
    format!("{{{{:{}:{}?{}|}}}}", image_namespace, legal_file_name(image_file_name), suffix)
}

pub(crate) fn add_page_link(page_text: &mut String, namespace: &str, page_name: &str, label: Option<&str>) {
    // Like "[[nav:categories|Categories]]".
    page_text.push_str(&format!("{}\n\n", page_link(namespace, page_name, label)));
}

pub(crate) fn add_section_link(page_text: &mut String, namespace: &str, page_name: &str, section_name: &str, label: Option<&str>) {
    // Like "[[nav:categories|Categories#Five]]".
    page_text.push_str(&format!("{}\n\n", section_link(namespace, page_name, section_name, label)));
}

pub(crate) fn add_section_link_same_page(page_text: &mut String, section_name: &str, label: Option<&str>) {
    // Like "[[#All|All Categories]]".
    page_text.push_str(&format!("{}\n\n", section_link_same_page(section_name, label)));
}

pub(crate) fn add_line(page_text: &mut String) {
    page_text.push_str("\n");
}

pub(crate) fn add_list_item_unordered(page_text: &mut String, text: &str) {
    page_text.push_str(&format!("  * {}\n", text));
}

pub(crate) fn add_paragraph(page_text: &mut String, text: &str) {
    page_text.push_str(&format!("{}\n\n", text));
}

pub(crate) fn namespace_prefix(namespace: &str) -> String {
    if namespace.is_empty() {
        "".to_string()
    } else {
        format!("{}:", namespace)
    }
}

pub(crate) fn page_link(namespace: &str, page_name: &str, label: Option<&str>) -> String {
    format!("[[{}{}{}]]", namespace_prefix(namespace), legal_file_name(page_name), label.map_or("".to_string(), |x| format!("|{}", x)))
}

pub(crate) fn section_link(namespace: &str, page_name: &str, section_name: &str, label: Option<&str>) -> String {
    // Like "[[nav:categories#All|All Categories]]".
    format!("[[{}{}#{}{}]]", namespace_prefix(namespace), legal_file_name(page_name), section_name, label.map_or("".to_string(), |x| format!("|{}", x)))
}

pub(crate) fn section_link_same_page(section_name: &str, label: Option<&str>) -> String {
    // Like "[[#All|All Categories]]".
    format!("[[#{}{}]]", section_name, label.map_or("".to_string(), |x| format!("|{}", x)))
}

pub(crate) fn external_link(url: &str, label: Option<&str>) -> String {
    // Like "[[https://github.com/|external link|GitHub]]".
    format!("[[{}{}]]", url, label.map_or("".to_string(), |x| format!("|{}", x)))
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
            if c.is_alphabetic() || c.is_digit(10) || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .join("");
    let page_name = format::remove_repeated(&page_name, "_");
    let page_name = page_name.replace("_.", ".");
    page_name
}

pub(crate) fn namespace_to_path(namespace: &str) -> String {
    namespace.replace(":", "/")
}

pub(crate) fn write_page(folder: &str, namespace: &str, name: &str, text: &str) {
    fs::write(format!("{}/{}/{}.txt", folder, namespace_to_path(namespace), legal_file_name(name)), text).unwrap();
}

pub(crate) fn copy_image_file(from_path: &str, from_file_name: &str, to_path: &str, to_namespace: &str, to_file_name: &str) {
    let from_full_file_name = format!("{}/{}", from_path, from_file_name);
    let to_full_file_name = format!("{}/{}/{}", to_path, namespace_to_path(to_namespace), legal_file_name(to_file_name));
    println!("{} => {}", from_full_file_name, to_full_file_name);
    fs::copy(from_full_file_name, to_full_file_name).unwrap();
}

pub(crate) fn bold(value: &str) -> String {
    format!("**{}**", value)
}

pub(crate) fn italic(value: &str) -> String {
    format!("//{}//", value)
}
*/