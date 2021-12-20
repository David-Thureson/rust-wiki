use std::fs;

use super::gen::*;

pub struct WikiGenPage {
    pub namespace: String,
    pub topic_name: String,
    pub content: String,
}

impl WikiGenPage {
    pub fn new(namespace: &str, topic_name: &str, headline: Option<&str>) -> Self {
        let mut page = Self {
            namespace: namespace.to_string(),
            topic_name: topic_name.to_string(),
            content: "".to_string()
        };
        let headline = headline.unwrap_or(topic_name);
        page.add_headline(headline, 1);
        page
    }

    pub fn add_headline(&mut self, text: &str, level: usize) {
        // Like "----- Categories -----" where a level 1 (top) headline has five hyphens.
        debug_assert!(level >= 1);
        debug_assert!(level <= 5);
        let equal_signs = "=".repeat(6 - level);
        self.content.push_str(&format!("{}{}{}\n\n", equal_signs, text, equal_signs));
    }

    pub fn add_category(&mut self, qualified_namespace: &str, category_name: &str) {
        // Like "Category: [[APIs]]".
        self.content.push_str(&format!("Category: {}\n\n", page_link(qualified_namespace, category_name, None)));
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

    pub fn add_text(&mut self, text: &str) {
        self.content.push_str(text);
    }

    /*
    pub fn add_list_item_unordered(&mut self, text: &str) {
        self.content.push_str(&format!("  * {}\n", text));
    }
    */

    pub fn add_paragraph(&mut self, text: &str) {
        self.content.push_str(&format!("{}\n\n", text));
    }

    pub fn add_list(&mut self, list: &WikiList) {
        self.content.push_str(&format!("{}\n\n", list.get_markup()));
    }

    pub fn write(self) {
        let mut namespace_path= namespace_to_path(&self.namespace);
        if !namespace_path.is_empty() {
            namespace_path = format!("/{}", namespace_path);
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
