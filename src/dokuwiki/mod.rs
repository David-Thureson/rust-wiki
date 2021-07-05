pub mod gen;
pub use gen::*;

pub mod gen_page;
pub use gen_page::*;

pub mod parse;
pub use parse::*;
use crate::path_backup;

//pub mod model;
//pub use model::*;

pub const PAGE_NAME_SIDEBAR: &str = "Sidebar";
pub const PAGE_NAME_START:   &str = "Start";

pub fn back_up_from_live() {
    let path_dest = path_backup();


    iuoeu
}

pub fn back_up_from_compare() {
}
