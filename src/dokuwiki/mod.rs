pub mod gen;
pub use gen::*;

pub mod gen_from_model;

pub mod gen_page;
pub use gen_page::*;

pub mod gen_tools_wiki;

pub mod parse;
pub use parse::*;

pub mod to_model;
pub use to_model::*;

//pub mod model;
//pub use model::*;

pub const PAGE_NAME_SIDEBAR: &str = "Sidebar";
pub const PAGE_NAME_MAIN: &str = "Main";
pub const PAGE_NAME_START:   &str = "Start";
pub const PAGE_NAME_RECENT_TOPICS: &str = "Recent Topics";
pub const PAGE_NAME_ALL_TOPICS: &str = "All Topics";
pub const PAGE_NAME_CATEGORIES: &str = "Categories";
pub const PAGE_NAME_SUBTOPICS: &str = "Subtopics";
pub const PAGE_NAME_ATTR: &str = "Attributes";
pub const PAGE_NAME_ATTR_VALUE: &str = "Attribute Values";
pub const PAGE_NAME_ATTR_YEAR: &str = "Years";
pub const PAGE_NAME_ATTR_DATE: &str = "Dates";
pub const PAGE_NAME_TERMS: &str = "Terms";

pub const DELIM_NAMESPACE: &str = ":";
pub const DELIM_LINEFEED: &str = "\n";
pub const DELIM_PARAGRAPH: &str = "\n\n";
pub const DELIM_BOOKMARK_RIGHT: &str = "=>";
pub const DELIM_BOOKMARK_LEFT: &str = "<=";
pub const DELIM_BOLD: &str = "**";
pub const DELIM_ITALIC: &str = "//";
pub const DELIM_LINK_START: &str = "[[";
pub const DELIM_LINK_END: &str = "]]";
pub const DELIM_LINK_LABEL: &str = "|";
pub const DELIM_LINK_SECTION: &str = "#";
pub const DELIM_IMAGE_START: &str = "{{";
pub const DELIM_IMAGE_END: &str = "}}";
pub const DELIM_IMAGE_OPTIONS: &str = "?";
pub const DELIM_SECTION_IN_LINK: &str = "#";
pub const DELIM_HEADER: &str = "=";
pub const DELIM_TABLE_CELL: &str = "|";
pub const DELIM_TABLE_CELL_BOLD: &str = "^";

pub const MARKER_LINE_START: &str = "<";
pub const MARKER_LINE_END: &str = ">";
pub const MARKER_QUOTE_START: &str = "<WRAP round box>";
pub const MARKER_QUOTE_END: &str = "</WRAP>";
pub const MARKER_CODE_START_PREFIX: &str = "<code";
pub const MARKER_CODE_END: &str = "</code>";

pub const TEMP_DELIM_IMG_START: &str = "[[{{";
pub const TEMP_DELIM_IMG_END: &str = "}}]]";

pub const PREFIX_CATEGORY: &str = "Category: ";
pub const PREFIX_HTTPS: &str = "https://";

/*
pub fn back_up_from_live() {
    let path_dest = path_backup();


    iuoeu
}

pub fn back_up_from_compare() {
}
*/