use crate::*;

pub mod attribute;
pub use attribute::*;

// pub mod breadcrumbs;
// pub use breadcrumbs::*;

pub mod category;
pub use category::*;

pub mod domain;
pub use domain::*;

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

pub type TopicTree = util::tree::Tree<TopicKey>;
pub type TopicTreeNode = util::tree::TreeNode<TopicKey>;

pub const NAMESPACE_ROOT: &str = "";
pub const NAMESPACE_UNDECIDED: &str = "{undecided}";
const NAMESPACE_BOOK: &str = ":book";
const NAMESPACE_NAVIGATION: &str = ":nav";
const NAMESPACE_ATTRIBUTE: &str = ":attr";

pub const ATTRIBUTE_VALUE_MISSING: &str = "{missing}";

pub const ATTRIBUTE_NAME_ADDED: &str = "Added";
pub const ATTRIBUTE_NAME_DOMAIN: &str = "Domain";

pub const LIST_LABEL_SUBCATEGORIES: &str = "Subcategories:";
pub const LIST_LABEL_CATEGORY_TOPICS: &str = "Topics:";
pub const LIST_LABEL_CATEGORY_TOPICS_ALL: &str = "All Topics:";
pub const LIST_LABEL_SUBTOPICS: &str = "Subtopics:";
pub const LIST_LABEL_COMBINATIONS: &str = "Combinations:";

pub const ATTRIBUTE_ORDER: [&str; 52] = ["School", "Title", "Series", "Course", "Author", "Narrator", "Translator", "Year", "Language", "Domain", "Paradigm", "Format", "Location", "Acquired", "Read", "Started", "Completed", "Abandoned", "Repeat", "Platform", "IDE", "GitHub", "Book", "Context", "License Type", "Name", "Operating System", "PC Name", "Priority", "Status", "Organization", "Founder", "Company", "Email", "Phone", "Twitter", "Skype", "Facebook", "Slack", "LinkedIn", "Meetup", "Operation Code", "Address", "Pricing", "Signed Up", "Downloaded", "Installed", "Used Online", "Used Locally", "Presented", "Created", "Added"];
