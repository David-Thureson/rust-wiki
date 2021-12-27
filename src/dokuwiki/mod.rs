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
pub const PAGE_NAME_TERMS: &str = "Terms";

/*
pub fn back_up_from_live() {
    let path_dest = path_backup();


    iuoeu
}

pub fn back_up_from_compare() {
}
*/