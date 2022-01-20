use std::fs;

use super::*;
use crate::Itertools;
use crate::model::TopicKey;

pub struct WikiGenPage {
    pub namespace: String,
    pub topic_name: String,
    pub content: String,
}

impl WikiGenPage {
    pub fn new(namespace: &str, topic_name: &str, headline: Option<&str>) -> Self {
        TopicKey::assert_legal_namespace(namespace);
        TopicKey::assert_legal_topic_name(topic_name);
        let mut page = Self {
            namespace: namespace.to_string(),
            topic_name: topic_name.to_string(),
            content: "".to_string()
        };
        let headline = headline.unwrap_or(topic_name);
        page.add_headline(headline, 0);
        page
    }

    pub fn add_headline(&mut self, text: &str, level: usize) {
        // Like "----- Categories -----" where a level 1 (top) headline has five hyphens.
        debug_assert!(level <= 5);
        let equal_signs = "=".repeat(6 - level);
        self.content.push_str(&format!("{}{}{}\n\n", equal_signs, text, equal_signs));
    }

    pub fn add_category(&mut self, qualified_namespace: &str, category_name: &str) {
        // Like "Category: [[APIs]]".
        TopicKey::assert_legal_namespace(qualified_namespace);
        self.content.push_str(&format!("{}{}\n\n", PREFIX_CATEGORY, page_link(qualified_namespace, category_name, None)));
    }

    /*
    pub fn add_image_internal_link(&mut self, page_namespace: &str, page_name: &str, image_namespace: &str, image_file_name: &str, image_size: &WikiImageSize) {
        let image_part = image_part(image_namespace, image_file_name, &WikiImageLinkType::NoLink, image_size);
        self.content.push_str(&format!("[[{}:{}|{}]]\n\n", page_namespace, legal_file_name(page_name), &image_part));
    }
    */

    /*
    pub fn add_image_link_to_full_image(&mut self, image_namespace: &str, image_file_name: &str, image_size: &WikiImageSize) {
        let image_part = image_part(image_namespace, image_file_name, &WikiImageLinkType::Direct, image_size);
        self.content.push_str(&format!("{}\n\n", &image_part));
    }

    pub fn add_image_table_row(&mut self, image_namespace: &str, image_size: &WikiImageSize, end_table: bool, image_file_names: &[&str]) {
        let markup = format!("|{}", image_file_names.iter()
            .map(|file_name| format!(" {} |", image_part(image_namespace, file_name, &WikiImageLinkType::Direct, image_size)))
            .join(""));
        let suffix = if end_table { "\n" } else { "" };
        self.content.push_str(&format!("{}\n{}", markup, suffix));
    }

    pub fn add_image_row(&mut self, image_namespace: &str, image_size: &WikiImageSize, image_file_names: &[&str]) {
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
     */
    pub fn add_page_link(&mut self, namespace: &str, page_name: &str, label: Option<&str>) {
        // Like "[[nav:categories|Categories]]".
        TopicKey::assert_legal_namespace(namespace);
        self.content.push_str(&format!("{}\n\n", page_link(namespace, page_name, label)));
    }

    pub fn add_section_link(&mut self, namespace: &str, page_name: &str, section_name: &str, label: Option<&str>) {
        // Like "[[nav:categories|Categories#Five]]".
        TopicKey::assert_legal_namespace(namespace);
        self.content.push_str(&format!("{}\n\n", section_link(namespace, page_name, section_name, label)));
    }

    pub fn add_section_link_same_page(&mut self, section_name: &str, label: Option<&str>) {
        // Like "[[#All|All Categories]]".
        self.content.push_str(&format!("{}\n\n", section_link_same_page(section_name, label)));
    }

    pub fn add_linefeed(&mut self) {
        self.content.push_str("\n");
    }

    pub fn end_paragraph(&mut self) {
        self.content.push_str("\n\n");
    }

    pub fn add(&mut self, text: &str) {
        self.content.push_str(text);
    }

    pub fn add_line(&mut self, text: &str) {
        self.content.push_str(&format!("{}\n", text));
    }

    pub fn add_line_with_break(&mut self, text: &str) {
        self.content.push_str(&format!("{}\\\\\n", text));
    }

    pub fn add_text(&mut self, text: &str) {
        self.content.push_str(text);
    }

    pub fn add_list_item_unordered(&mut self, depth: usize, text: &str) {
        //bg!(&depth, &text);
        self.content.push_str(&format!("{}* {}\n", "  ".repeat(depth), text));
        // self.content.push_str(&format!("  * {}\n", text));
    }

    pub fn add_paragraph(&mut self, text: &str) {
        self.content.push_str(&format!("{}\n\n", text));
    }

    pub fn add_list(&mut self, list: &WikiList) {
        self.content.push_str(&format!("{}\n\n", list.get_markup()));
    }

    pub fn add_table_row(&mut self, row_index: usize, has_header: bool, has_label_column: bool, cells: &Vec<String>) {
        // A table header row should look something like:
        //   ^ Color ^ Blue ^
        // A regular table row should look something like:
        //   | Color | Blue |
        let last_delimiter = if has_header && row_index == 0 { DELIM_TABLE_CELL_BOLD } else { DELIM_TABLE_CELL };
        let markup = format!("{}{}\n", cells.iter().enumerate()
            .map(|(cell_index, cell_text)| {
                let delimiter = if (has_header && row_index == 0) || (has_label_column || cell_index == 0) { DELIM_TABLE_CELL_BOLD } else { DELIM_TABLE_CELL };
                format!("{} {}", delimiter, cell_text)
            })
            .join(""),
            last_delimiter
        );
        self.content.push_str(&markup);
    }

    pub fn write(self) {
        let mut namespace_path= namespace_to_path(&self.namespace);
        //bg!(&namespace_path);
        if !namespace_path.is_empty() {
            namespace_path = format!("/{}", namespace_path);
            //bg!(&namespace_path);
        }
        let legal_file_name = legal_file_name(&self.topic_name);
        //bg!(PATH_PAGES, &namespace_path, &legal_file_name);
        let full_file_name = format!("{}{}/{}.txt", PATH_PAGES, namespace_path, legal_file_name);
        //bg!(&full_file_name);
        if full_file_name.contains("//") {
            panic!("File name has double slashes: \"{}\".", &full_file_name);
        }
        fs::write(full_file_name, self.content).unwrap();
    }
}
