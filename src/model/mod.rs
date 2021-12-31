pub mod attribute;
pub use attribute::*;

// pub mod breadcrumbs;
// pub use breadcrumbs::*;

pub mod category;
pub use category::*;

pub mod link;
pub use link::*;

pub mod list;
pub use list::*;

// pub mod namespace;
// pub use namespace::*;

pub mod paragraph;
pub use paragraph::*;

pub mod parse;
pub use parse::*;

// pub mod section;
// pub use section::*;

pub mod report;

pub mod textblock;
pub use textblock::*;

pub mod topic;
pub use topic::*;

pub mod topic_error_list;
pub use topic_error_list::*;

pub mod wiki;
pub use wiki::*;

pub type CategoryTree = util::tree::Tree<TopicKey>;
pub type CategoryTreeNode = util::tree::TreeNode<TopicKey>;

pub const NAMESPACE_ROOT: &str = "";
pub const NAMESPACE_UNDECIDED: &str = "{undecided}";
pub const NAMESPACE_BOOK: &str = ":book";
pub const NAMESPACE_NAVIGATION: &str = ":nav";

pub const ATTRIBUTE_VALUE_MISSING: &str = "{missing}";
