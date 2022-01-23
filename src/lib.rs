#![macro_use]
#![feature(cell_leak)]
#![feature(btree_drain_filter)]

pub(crate) mod connectedtext;
pub mod dokuwiki;
pub(crate) mod model;

pub(crate) use util::*;
// pub(crate) use std::rc::Rc;
// pub(crate) use std::cell::RefCell;
pub(crate) use itertools::Itertools;
// pub(crate) use util::{format, group, parse};
// pub(crate) use util::format::{fc, ff};
// pub(crate) use util::{b, b2, m, rse, r};

// These paths won't work for something like DokuWiki. They're simply folders in which to generate
// wiki pages and copy image files for comparisons to test the generating and file copying code.
// They're also used to make backups of the live wiki.
// pub(crate) const PATH_WORKING: &str = "C:/Wiki Working";
// pub(crate) const FOLDER_GEN: &str = "Gen";
// pub(crate) const FOLDER_BACKUP: &str = "Backup";
// pub(crate) const FOLDER_PAGES: &str = "Pages";
// pub(crate) const FOLDER_MEDIA: &str = "Media";

const _FILE_NUMBER_DIGITS: usize = 4;

/*
fn path_gen_root() -> String {
    format!("{}/{}", PATH_WORKING, FOLDER_GEN)
}

fn path_backup_root() -> String {
    format!("{}/{}", PATH_WORKING, FOLDER_BACKUP)
}
*/
/*
pub(crate) fn path_gen() -> String {
    path_gen_or_backup(FOLDER_GEN)
}

pub(crate) fn path_backup() -> String {
    path_gen_or_backup(FOLDER_BACKUP)
}

fn path_gen_or_backup(type_: &str) -> String {
    let path_base = format!("{}/{}", PATH_WORKING, type_);
    let date_string = date_for_file_name_now();
    util::file::path_folder_next_number_r(&path_base, type_, FILE_NUMBER_DIGITS).unwrap()
}
*/

/*
pub(crate) fn r<T>(value: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(value))
}
*/

/*
pub(crate) fn b<T>(rc: &'static Rc<RefCell<T>>) -> Ref<'static, T> {
    RefCell::borrow(rc)
}
*/

/*
// Shorthand for something like Rc::new(RefCell::new(a)).
#[macro_export]
#[macro_use]
macro_rules! r {
    ($a:expr)=>{
    // ($a:ident)=>{
        {
            Rc::new(RefCell::new($a))
        }
    }
}

// Shorthand for something like RefCell::borrow(a).
#[macro_export]
#[macro_use]
macro_rules! b {
    ($a:expr)=>{
        {
            RefCell::borrow($a)
        }
    }
}

// Shorthand for something like RefCell::borrow_mut(a).
#[macro_export]
#[macro_use]
macro_rules! m {
    ($a:expr)=>{
        {
            RefCell::borrow_mut($a)
        }
    }
}
*/
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
