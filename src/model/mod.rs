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

pub(crate) mod glossary;

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

pub(crate) mod redaction;

pub(crate) mod report;

pub(crate) mod table;
pub(crate) use table::*;

pub(crate) mod textblock;
pub(crate) use textblock::*;

pub mod topic;
pub use topic::*;

pub(crate) mod topic_error_list;
pub(crate) use topic_error_list::*;

pub(crate) mod model;
pub(crate) use model::*;

pub(crate) type TopicTree = util::tree::Tree<TopicKey>;
pub(crate) type TopicTreeNode = util::tree::TreeNode<TopicKey>;

pub(crate) const PANIC_ON_MODEL_ERROR: bool = true;

pub(crate) const FILE_NAME_REDACT: &str = "C:/Projects/Rust/utility/wiki/redact.txt";

pub const NAMESPACE_ROOT: &str = "";
// pub(crate) const NAMESPACE_UNDECIDED: &str = "{undecided}";
pub const NAMESPACE_TOOLS: &str = ":tools";
pub const NAMESPACE_BOOK: &str = ":book";
pub const NAMESPACE_NAVIGATION: &str = ":nav";
// const NAMESPACE_ATTRIBUTE: &str = ":attr";

// pub(crate) const ATTRIBUTE_VALUE_MISSING: &str = "{missing}";

pub(crate) const CATEGORY_RUST_CRATES: &str = "Rust Crates";
pub(crate) const CATEGORY_RUST_PROJECTS: &str = "Rust Projects";

/*
pub(crate) const LIST_LABEL_SUBCATEGORIES: &str = "Subcategories:";
pub(crate) const LIST_LABEL_CATEGORY_TOPICS: &str = "Topics:";
pub(crate) const LIST_LABEL_CATEGORY_TOPICS_ALL: &str = "All Topics:";
pub(crate) const LIST_LABEL_SUBTOPICS: &str = "Subtopics:";
pub(crate) const LIST_LABEL_COMBINATIONS: &str = "Combinations:";
*/

pub(crate) const FOLDER_WIKI_GEN_BACKUP: &str = "C:/Wiki Gen Backup";
pub(crate) const FOLDER_PREFIX_WIKI_GEN_BACKUP: &str = "Wiki Gen";
pub(crate) const FOLDER_WIKI_COMPARE_OLD: &str = "C:/Wiki Gen Backup/Old";
pub(crate) const FOLDER_WIKI_COMPARE_NEW: &str = r"C:/Wiki Gen Backup/New";

pub(crate) const PREFIX_HTTP: &str = "http://";
pub(crate) const PREFIX_HTTPS: &str = "https://";
pub(crate) const PREFIX_SFTP: &str = "sftp://";

#[allow(dead_code)]
pub(crate) const ATTRIBUTE_VALUE_PRIVATE: &str = "Private";
#[allow(dead_code)]
pub(crate) const ATTRIBUTE_VALUE_PUBLIC: &str = "Public";
#[allow(dead_code)]
pub(crate) const ATTRIBUTE_VALUE_UNKNOWN: &str = "Unknown";

pub(crate) const ATTRIBUTE_NAME_ABANDONED: &str = "Abandoned";
pub(crate) const ATTRIBUTE_NAME_ACQUIRED: &str = "Acquired";
pub(crate) const ATTRIBUTE_NAME_ADDED: &str = "Added";
pub(crate) const ATTRIBUTE_NAME_ADDRESS: &str = "Address";
pub(crate) const ATTRIBUTE_NAME_AUTHOR: &str = "Author";
pub(crate) const ATTRIBUTE_NAME_BOOK: &str = "Book";
pub(crate) const ATTRIBUTE_NAME_CELL_PHONE: &str = "Cell Phone";
pub(crate) const ATTRIBUTE_NAME_COMPANY: &str = "Company";
pub(crate) const ATTRIBUTE_NAME_COMPLETED: &str = "Completed";
pub(crate) const ATTRIBUTE_NAME_CONTEXT: &str = "Context";
pub(crate) const ATTRIBUTE_NAME_COURSE: &str = "Course";
pub(crate) const ATTRIBUTE_NAME_CREATED: &str = "Created";
pub(crate) const ATTRIBUTE_NAME_DEPARTMENT: &str = "Department";
pub(crate) const ATTRIBUTE_NAME_DOMAIN: &str = "Domain";
pub(crate) const ATTRIBUTE_NAME_DOWNLOADED: &str = "Downloaded";
pub(crate) const ATTRIBUTE_NAME_EDITED: &str = "Edited";
pub(crate) const ATTRIBUTE_NAME_EMAIL: &str = "Email";
pub(crate) const ATTRIBUTE_NAME_FACEBOOK: &str = "Facebook";
pub(crate) const ATTRIBUTE_NAME_FOLDER: &str = "Folder";
pub(crate) const ATTRIBUTE_NAME_FORMAT: &str = "Format";
pub(crate) const ATTRIBUTE_NAME_FOUNDER: &str = "Founder";
pub(crate) const ATTRIBUTE_NAME_GITHUB: &str = "GitHub";
pub(crate) const ATTRIBUTE_NAME_IDE: &str = "IDE";
pub(crate) const ATTRIBUTE_NAME_INSTALLED: &str = "Installed";
pub(crate) const ATTRIBUTE_NAME_LANGUAGE: &str = "Language";
pub(crate) const ATTRIBUTE_NAME_LICENSE_TYPE: &str = "License Type";
pub(crate) const ATTRIBUTE_NAME_LINKEDIN: &str = "LinkedIn";
pub(crate) const ATTRIBUTE_NAME_LOCATION: &str = "Location";
pub(crate) const ATTRIBUTE_NAME_MEETUP: &str = "Meetup";
pub(crate) const ATTRIBUTE_NAME_NAME: &str = "Name";
pub(crate) const ATTRIBUTE_NAME_NARRATOR: &str = "Narrator";
pub(crate) const ATTRIBUTE_NAME_OPERATING_SYSTEM: &str = "Operating System";
pub(crate) const ATTRIBUTE_NAME_OPERATION_CODE: &str = "Operation Code";
pub(crate) const ATTRIBUTE_NAME_ORGANIZATION: &str = "Organization";
pub(crate) const ATTRIBUTE_NAME_PARADIGM: &str = "Paradigm";
pub(crate) const ATTRIBUTE_NAME_PC_NAME: &str = "PC Name";
pub(crate) const ATTRIBUTE_NAME_PHONE: &str = "Phone";
pub(crate) const ATTRIBUTE_NAME_PLATFORM: &str = "Platform";
pub(crate) const ATTRIBUTE_NAME_PRESENTED: &str = "Presented";
pub(crate) const ATTRIBUTE_NAME_PRICING: &str = "Pricing";
pub(crate) const ATTRIBUTE_NAME_PRIORITY: &str = "Priority";
pub(crate) const ATTRIBUTE_NAME_READ: &str = "Read";
pub(crate) const ATTRIBUTE_NAME_REPEAT: &str = "Repeat";
pub(crate) const ATTRIBUTE_NAME_SCHOOL: &str = "School";
pub(crate) const ATTRIBUTE_NAME_SERIES: &str = "Series";
pub(crate) const ATTRIBUTE_NAME_SIGNED_UP: &str = "Signed Up";
pub(crate) const ATTRIBUTE_NAME_SKYPE: &str = "Skype";
pub(crate) const ATTRIBUTE_NAME_SLACK: &str = "Slack";
pub(crate) const ATTRIBUTE_NAME_STARTED: &str = "Started";
pub(crate) const ATTRIBUTE_NAME_STATUS: &str = "Status";
pub(crate) const ATTRIBUTE_NAME_TIME_ZONE: &str = "Time Zone";
pub(crate) const ATTRIBUTE_NAME_TITLE: &str = "Title";
pub(crate) const ATTRIBUTE_NAME_TRANSLATOR: &str = "Translator";
pub(crate) const ATTRIBUTE_NAME_TWITTER: &str = "Twitter";
pub(crate) const ATTRIBUTE_NAME_UPDATED: &str = "Updated";
pub(crate) const ATTRIBUTE_NAME_USED_LOCALLY: &str = "Used Locally";
pub(crate) const ATTRIBUTE_NAME_USED_ONLINE: &str = "Used Online";
pub(crate) const ATTRIBUTE_NAME_VISIBILITY: &str = "Visibility";
pub(crate) const ATTRIBUTE_NAME_WORK_PHONE: &str = "Work Phone";
pub(crate) const ATTRIBUTE_NAME_YEAR: &str = "Year";

pub(crate) const ATTRIBUTE_ORDER: [&str; 60] = [
    ATTRIBUTE_NAME_SCHOOL,
    ATTRIBUTE_NAME_TITLE,
    ATTRIBUTE_NAME_SERIES,
    ATTRIBUTE_NAME_COURSE,
    ATTRIBUTE_NAME_AUTHOR,
    ATTRIBUTE_NAME_NARRATOR,
    ATTRIBUTE_NAME_TRANSLATOR,
    ATTRIBUTE_NAME_YEAR,
    ATTRIBUTE_NAME_LANGUAGE,
    ATTRIBUTE_NAME_DOMAIN,
    ATTRIBUTE_NAME_PARADIGM,
    ATTRIBUTE_NAME_FORMAT,
    ATTRIBUTE_NAME_LOCATION,
    ATTRIBUTE_NAME_ACQUIRED,
    ATTRIBUTE_NAME_READ,
    ATTRIBUTE_NAME_STARTED,
    ATTRIBUTE_NAME_UPDATED,
    ATTRIBUTE_NAME_COMPLETED,
    ATTRIBUTE_NAME_ABANDONED,
    ATTRIBUTE_NAME_REPEAT,
    ATTRIBUTE_NAME_PLATFORM,
    ATTRIBUTE_NAME_IDE,
    ATTRIBUTE_NAME_GITHUB,
    ATTRIBUTE_NAME_BOOK,
    ATTRIBUTE_NAME_CONTEXT,
    ATTRIBUTE_NAME_LICENSE_TYPE,
    ATTRIBUTE_NAME_NAME,
    ATTRIBUTE_NAME_OPERATING_SYSTEM,
    ATTRIBUTE_NAME_PC_NAME,
    ATTRIBUTE_NAME_FOLDER,
    ATTRIBUTE_NAME_PRIORITY,
    ATTRIBUTE_NAME_STATUS,
    ATTRIBUTE_NAME_DEPARTMENT,
    ATTRIBUTE_NAME_ORGANIZATION,
    ATTRIBUTE_NAME_FOUNDER,
    ATTRIBUTE_NAME_COMPANY,
    ATTRIBUTE_NAME_EMAIL,
    ATTRIBUTE_NAME_PHONE,
    ATTRIBUTE_NAME_WORK_PHONE,
    ATTRIBUTE_NAME_CELL_PHONE,
    ATTRIBUTE_NAME_TIME_ZONE,
    ATTRIBUTE_NAME_TWITTER,
    ATTRIBUTE_NAME_SKYPE,
    ATTRIBUTE_NAME_FACEBOOK,
    ATTRIBUTE_NAME_SLACK,
    ATTRIBUTE_NAME_LINKEDIN,
    ATTRIBUTE_NAME_MEETUP,
    ATTRIBUTE_NAME_OPERATION_CODE,
    ATTRIBUTE_NAME_ADDRESS,
    ATTRIBUTE_NAME_PRICING,
    ATTRIBUTE_NAME_SIGNED_UP,
    ATTRIBUTE_NAME_DOWNLOADED,
    ATTRIBUTE_NAME_INSTALLED,
    ATTRIBUTE_NAME_USED_ONLINE,
    ATTRIBUTE_NAME_USED_LOCALLY,
    ATTRIBUTE_NAME_PRESENTED,
    ATTRIBUTE_NAME_CREATED,
    ATTRIBUTE_NAME_ADDED,
    ATTRIBUTE_NAME_EDITED,
    ATTRIBUTE_NAME_VISIBILITY,
];

pub(crate) const PUBLIC_ATTRIBUTES: [&str; 38] = [
    ATTRIBUTE_NAME_SCHOOL,
    ATTRIBUTE_NAME_TITLE,
    ATTRIBUTE_NAME_SERIES,
    ATTRIBUTE_NAME_COURSE,
    ATTRIBUTE_NAME_AUTHOR,
    ATTRIBUTE_NAME_NARRATOR,
    ATTRIBUTE_NAME_TRANSLATOR,
    ATTRIBUTE_NAME_YEAR,
    ATTRIBUTE_NAME_LANGUAGE,
    ATTRIBUTE_NAME_DOMAIN,
    ATTRIBUTE_NAME_PARADIGM,
    ATTRIBUTE_NAME_FORMAT,
    ATTRIBUTE_NAME_LOCATION,
    ATTRIBUTE_NAME_UPDATED,
    ATTRIBUTE_NAME_PLATFORM,
    ATTRIBUTE_NAME_IDE,
    ATTRIBUTE_NAME_BOOK,
    ATTRIBUTE_NAME_CONTEXT,
    ATTRIBUTE_NAME_LICENSE_TYPE,
    ATTRIBUTE_NAME_NAME,
    ATTRIBUTE_NAME_OPERATING_SYSTEM,
    ATTRIBUTE_NAME_PRIORITY,
    ATTRIBUTE_NAME_STATUS,
    ATTRIBUTE_NAME_DEPARTMENT,
    ATTRIBUTE_NAME_ORGANIZATION,
    ATTRIBUTE_NAME_FOUNDER,
    ATTRIBUTE_NAME_COMPANY,
    ATTRIBUTE_NAME_TIME_ZONE,
    ATTRIBUTE_NAME_PRICING,
    ATTRIBUTE_NAME_SIGNED_UP,
    ATTRIBUTE_NAME_DOWNLOADED,
    ATTRIBUTE_NAME_INSTALLED,
    ATTRIBUTE_NAME_USED_ONLINE,
    ATTRIBUTE_NAME_USED_LOCALLY,
    ATTRIBUTE_NAME_PRESENTED,
    ATTRIBUTE_NAME_CREATED,
    ATTRIBUTE_NAME_ADDED,
    ATTRIBUTE_NAME_EDITED,
];

pub(crate) const INDEXED_ATTRIBUTES: [&str; 24] = [
    ATTRIBUTE_NAME_AUTHOR,
    ATTRIBUTE_NAME_BOOK,
    ATTRIBUTE_NAME_COMPANY,
    ATTRIBUTE_NAME_CONTEXT,
    ATTRIBUTE_NAME_COURSE,
    ATTRIBUTE_NAME_DOMAIN,
    ATTRIBUTE_NAME_DEPARTMENT,
    ATTRIBUTE_NAME_FORMAT,
    ATTRIBUTE_NAME_FOUNDER,
    ATTRIBUTE_NAME_IDE,
    ATTRIBUTE_NAME_LANGUAGE,
    ATTRIBUTE_NAME_LICENSE_TYPE,
    ATTRIBUTE_NAME_LINKEDIN,
    ATTRIBUTE_NAME_NARRATOR,
    ATTRIBUTE_NAME_OPERATING_SYSTEM,
    ATTRIBUTE_NAME_ORGANIZATION,
    ATTRIBUTE_NAME_PARADIGM,
    ATTRIBUTE_NAME_PC_NAME,
    ATTRIBUTE_NAME_PLATFORM,
    ATTRIBUTE_NAME_SCHOOL,
    ATTRIBUTE_NAME_SERIES,
    ATTRIBUTE_NAME_STATUS,
    ATTRIBUTE_NAME_TIME_ZONE,
    ATTRIBUTE_NAME_TRANSLATOR,
];

pub fn date_now_to_doc_format() -> String {
    util::date_time::naive_date_to_doc_format(&util::date_time::naive_date_now())
}

/*
pub fn gen_attr_type_name_constants() {
    let mut constant_defs = vec![];
    let mut order_list = vec![];
    for type_name in ATTRIBUTE_ORDER.iter() {
        let constant_name = format!("ATTRIBUTE_NAME_{}", util::format::screaming_snake_case(type_name));
        constant_defs.push(format!("pub(crate) const {}: &str = \"{}\";", constant_name, type_name));
        order_list.push(constant_name);
    }
    constant_defs.sort();
    constant_defs.iter().for_each(|constant| println!("{}", constant));
    println!("\npub(crate) const ATTRIBUTE_ORDER: [&str; {}] = [", order_list.len());
    order_list.iter().for_each(|constant_name| println!("\t{},", constant_name));
    println!("];");
}

pub fn gen_attr_to_index() {
    let attr_type_names = vec!["Author", "Book", "Company", "Context", "Course", "Domain", "Domains", "Format", "Founder", "IDE", "Language", "License Type", "LinkedIn", "Narrator", "Operating System", "Organization", "PC Name", "Paradigm", "Platform", "School", "Series", "Status", "Translator"];
    let mut list = vec![];
    for type_name in attr_type_names.iter() {
        let constant_name = format!("ATTRIBUTE_NAME_{}", util::format::screaming_snake_case(type_name));
        list.push(constant_name);
    }
    list.sort();
    list.iter().for_each(|constant| println!("\t{},", constant));
}
*/
