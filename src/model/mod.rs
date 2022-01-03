use crate::*;

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
use std::rc::Rc;
use std::cell::RefCell;

pub type TopicTree = util::tree::Tree<TopicKey>;
pub type TopicTreeNode = util::tree::TreeNode<TopicKey>;

pub const NAMESPACE_ROOT: &str = "";
pub const NAMESPACE_UNDECIDED: &str = "{undecided}";
pub const NAMESPACE_BOOK: &str = ":book";
pub const NAMESPACE_NAVIGATION: &str = ":nav";

pub const ATTRIBUTE_VALUE_MISSING: &str = "{missing}";

pub const LIST_LABEL_SUBCATEGORIES: &str = "Subcategories:";
pub const LIST_LABEL_CATEGORY_TOPICS: &str = "Topics:";
pub const LIST_LABEL_CATEGORY_TOPICS_ALL: &str = "All Topics:";
pub const LIST_LABEL_SUBTOPICS: &str = "Subtopics:";
pub const LIST_LABEL_COMBINATIONS: &str = "Combinations:";

pub fn sort_topic_tree(tree: &mut TopicTree) {
    tree.sort_recursive(&|node: &Rc<RefCell<TopicTreeNode>>| b!(node).item.topic_name.clone());
}
