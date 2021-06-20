use itertools::Itertools;
use std::fs;

use super::gen::*;

pub struct WikiGenPage {
    pub namespace: String,
    pub topic_name: String,
    pub content: String,
}

pub struct WikiAttributeTable {
    pub rows: Vec<WikiAttributeRow>,
}

pub struct WikiAttributeRow {
    pub label: String,
    pub markup: String,
}

pub struct WikiList {
    pub items: Vec<String>,
}

impl WikiGenPage {
    pub fn new(namespace: &str, topic_name: &str) -> Self {
        let mut page = Self {
            namespace: namespace.to_string(),
            topic_name: topic_name.to_string(),
            content: "".to_string()
        };
        page.add_headline(topic_name, 1);
        page
    }
    
    pub fn add_headline(&mut self, text: &str, level: usize) {
        // Like "----- Categories -----" where a level 1 (top) headline has five hyphens.
        debug_assert!(level >= 1);
        debug_assert!(level <= 5);
        let equal_signs = "=".repeat(6 - level);
        self.content.push_str(&format!("{}{}{}\n\n", equal_signs, text, equal_signs));
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
        self.content.push_str(&format!("{}\n\n", page_link(namespace, page_name, label)));
    }

    pub fn add_section_link(&mut self, namespace: &str, page_name: &str, section_name: &str, label: Option<&str>) {
        // Like "[[nav:categories|Categories#Five]]".
        self.content.push_str(&format!("{}\n\n", section_link(namespace, page_name, section_name, label)));
    }

    pub fn add_section_link_same_page(&mut self, section_name: &str, label: Option<&str>) {
        // Like "[[#All|All Categories]]".
        self.content.push_str(&format!("{}\n\n", section_link_same_page(section_name, label)));
    }

    pub fn add_line(&mut self) {
        self.content.push_str("\n");
    }

    /*
    pub fn add_list_item_unordered(&mut self, text: &str) {
        self.content.push_str(&format!("  * {}\n", text));
    }
    */

    pub fn add_paragraph(&mut self, text: &str) {
        self.content.push_str(&format!("{}\n\n", text));
    }

    pub fn write(self) {
        let full_file_name = format!("{}/{}/{}.txt", PATH_PAGES, namespace_to_path(&self.namespace), legal_file_name(&self.topic_name));
        fs::write(full_file_name, self.content).unwrap();
    }

}

impl WikiAttributeTable {
    pub fn new() -> Self {
        Self {
            rows: vec![],
        }
    }

    pub fn add_row(&mut self, label: &str, markup: &str) {
        self.rows.push(WikiAttributeRow::new(label, markup));
    }

    pub fn get_markup(&self) -> String {
        // The attributes table should look something like:
        //   ^ Color    | Blue |
        //   ^ Tapes    | 4    |
        self.rows.iter()
            .map(|row| format!("^ {} | {} |\n", row.label, row.markup))
            .join("")
    }
}

impl WikiAttributeRow {
    pub fn new(label: &str, markup: &str) -> Self {
        Self {
            label: label.to_string(),
            markup: markup.to_string(),
        }
    }
}

impl WikiList {
    pub fn new() -> Self {
        Self {
            items: vec![],
        }
    }

    pub fn add_item(&mut self, markup: &str) {
        self.items.push(format!("  * {}", markup));
    }

    pub fn add_item_indent(&mut self, depth: usize, markup: &str) {
        self.items.push(format!("{}* {}", "  ".repeat(depth + 1), markup));
    }

    pub fn get_markup(&self, label: Option<&str>) -> String {
        let mut markup = "".to_string();
        if let Some(label) = label {
            markup.push_str(&format!("{}:\n", label));
        }
        for item in self.items.iter() {
            markup.push_str(&format!("{}\n", item));
        }
        markup
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

}


pub fn copy_image_file(from_path: &str, from_file_name: &str, to_path: &str, to_namespace: &str, to_file_name: &str) {
    let from_full_file_name = format!("{}/{}", from_path, from_file_name);
    let to_full_file_name = format!("{}/{}/{}", to_path, namespace_to_path(to_namespace), legal_file_name(to_file_name));
    println!("{} => {}", from_full_file_name, to_full_file_name);
    std::fs::copy(from_full_file_name, to_full_file_name).unwrap();
}

pub fn bold(value: &str) -> String {
    format!("**{}**", value)
}

pub fn italic(value: &str) -> String {
    format!("//{}//", value)
}
