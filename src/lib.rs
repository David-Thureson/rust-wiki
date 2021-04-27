pub mod connectedtext;
pub mod dokuwiki;
pub mod model;

pub use util::{format, group, parse};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
