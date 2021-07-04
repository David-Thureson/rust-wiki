#![macro_use]

pub mod connectedtext;
pub mod dokuwiki;
pub mod model;

pub use itertools::Itertools;
pub use util::{format, group, parse};
pub use util::format::{fc, ff};
use std::rc::Rc;
use std::cell::{RefCell, Ref};

/*
pub fn r<T>(value: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(value))
}
*/

/*
pub fn b<T>(rc: &'static Rc<RefCell<T>>) -> Ref<'static, T> {
    RefCell::borrow(rc)
}
*/

// Shorthand for something like Rc::new(RefCell::new(a)).
#[macro_export]
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
macro_rules! b {
    ($a:expr)=>{
        {
            RefCell::borrow($a)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
