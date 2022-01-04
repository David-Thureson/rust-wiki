pub mod gen;
pub use gen::*;

pub mod gen_from_model;

pub mod gen_page;
pub use gen_page::*;

pub mod gen_tools_wiki;

pub mod parse;
pub use parse::*;

//pub mod model;
//pub use model::*;

pub const PAGE_NAME_SIDEBAR: &str = "Sidebar";
pub const PAGE_NAME_MAIN: &str = "Main";
pub const PAGE_NAME_START:   &str = "Start";
pub const PAGE_NAME_RECENT_TOPICS: &str = "Recent Topics";
pub const PAGE_NAME_ALL_TOPICS: &str = "All Topics";
pub const PAGE_NAME_CATEGORIES: &str = "Categories";
pub const PAGE_NAME_SUBTOPICS: &str = "Subtopics";
pub const PAGE_NAME_ATTR_YEAR: &str = "Years";
pub const PAGE_NAME_ATTR_DATE: &str = "Dates";
pub const PAGE_NAME_TERMS: &str = "Terms";

pub const DELIM_BOOKMARK_RIGHT: &str = "=>";
pub const DELIM_BOOKMARK_LEFT: &str = "<=";
pub const DELIM_BOLD: &str = "**";
pub const DELIM_ITALIC: &str = "//";
pub const DELIM_QUOTE_START: &str = "<WRAP round box>";
pub const DELIM_QUOTE_END: &str = "</WRAP>";
pub const DELIM_CODE_START: &str = "<code>";
pub const DELIM_CODE_END: &str = "</code>";

/*
pub fn back_up_from_live() {
    let path_dest = path_backup();


    iuoeu
}

pub fn back_up_from_compare() {
}
*/