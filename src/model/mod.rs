use crate::*;

pub(crate) mod attribute;
pub(crate) use attribute::*;

// pub(crate) mod breadcrumbs;
// pub(crate) use breadcrumbs::*;

pub(crate) mod category;
pub(crate) use category::*;

pub(crate) mod date;

pub(crate) mod domain;
pub(crate) use domain::*;

pub(crate) mod link;
pub(crate) use link::*;

pub(crate) mod list;
pub(crate) use list::*;

// pub(crate) mod namespace;
// pub(crate) use namespace::*;

pub(crate) mod paragraph;
pub(crate) use paragraph::*;

pub(crate) mod parse;

// pub(crate) mod section;
// pub(crate) use section::*;

pub(crate) mod report;

pub(crate) mod table;
pub(crate) use table::*;

pub(crate) mod textblock;
pub(crate) use textblock::*;

pub(crate) mod topic;
pub(crate) use topic::*;

pub(crate) mod topic_error_list;
pub(crate) use topic_error_list::*;

pub(crate) mod model;
pub(crate) use model::*;

pub(crate) type TopicTree = util::tree::Tree<TopicKey>;
pub(crate) type TopicTreeNode = util::tree::TreeNode<TopicKey>;

pub(crate) const NAMESPACE_ROOT: &str = "";
// pub(crate) const NAMESPACE_UNDECIDED: &str = "{undecided}";
const NAMESPACE_BOOK: &str = ":book";
const NAMESPACE_NAVIGATION: &str = ":nav";
const NAMESPACE_ATTRIBUTE: &str = ":attr";

// pub(crate) const ATTRIBUTE_VALUE_MISSING: &str = "{missing}";

pub(crate) const CATEGORY_RUST_PROJECTS: &str = "Rust Projects";

pub(crate) const ATTRIBUTE_NAME_ADDED: &str = "Added";
pub(crate) const ATTRIBUTE_NAME_DOMAIN: &str = "Domain";
pub(crate) const ATTRIBUTE_NAME_FOLDER: &str = "Folder";
pub(crate) const ATTRIBUTE_NAME_LANGUAGE: &str = "Language";
pub(crate) const ATTRIBUTE_NAME_PC_NAME: &str = "PC Name";
pub(crate) const ATTRIBUTE_NAME_STARTED: &str = "Started";
pub(crate) const ATTRIBUTE_NAME_UPDATED: &str = "Updated";

pub(crate) const LIST_LABEL_SUBCATEGORIES: &str = "Subcategories:";
pub(crate) const LIST_LABEL_CATEGORY_TOPICS: &str = "Topics:";
pub(crate) const LIST_LABEL_CATEGORY_TOPICS_ALL: &str = "All Topics:";
pub(crate) const LIST_LABEL_SUBTOPICS: &str = "Subtopics:";
pub(crate) const LIST_LABEL_COMBINATIONS: &str = "Combinations:";

pub(crate) const ATTRIBUTE_ORDER: [&str; 54] = ["School", "Title", "Series", "Course", "Author",
    "Narrator", "Translator", "Year", "Language", "Domain", "Paradigm", "Format", "Location",
    "Acquired", "Read", "Started", "Updated", "Completed", "Abandoned", "Repeat", "Platform",
    "IDE", "GitHub", "Book", "Context", "License Type", "Name", "Operating System", "PC Name",
    "Folder", "Priority", "Status", "Organization", "Founder", "Company", "Email", "Phone", "Twitter",
    "Skype", "Facebook", "Slack", "LinkedIn", "Meetup", "Operation Code", "Address", "Pricing",
    "Signed Up", "Downloaded", "Installed", "Used Online", "Used Locally", "Presented", "Created",
    "Added"];

pub(crate) const FOLDER_WIKI_GEN_BACKUP: &str = r"C:\Wiki Gen Backup";
pub(crate) const FOLDER_PREFIX_WIKI_GEN_BACKUP: &str = "Wiki Gen";
pub(crate) const FOLDER_WIKI_COMPARE_OLD: &str = r"C:\Wiki Gen Backup\Old";
pub(crate) const FOLDER_WIKI_COMPARE_NEW: &str = r"C:\Wiki Gen Backup\New";

pub(crate) const PREFIX_HTTP: &str = "http://";
pub(crate) const PREFIX_HTTPS: &str = "https://";
pub(crate) const PREFIX_SFTP: &str = "sftp://";

