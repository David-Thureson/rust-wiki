pub mod connectedtext;
pub mod dokuwiki;
pub mod model;

pub use itertools::Itertools;
pub use util::{format, group, parse};
pub use util::format::{fc, ff};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
