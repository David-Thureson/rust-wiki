// pub mod parse;
// pub use parse::*;

pub mod report;
// pub use report::*;

// pub mod to_dokuwiki;
// pub use to_dokuwiki::*;

pub mod to_model;

pub const PATH_CT_EXPORT: &str = r"T:\Private Wiki Export";
pub const PATH_CT_EXPORT_TOOLS: &str = r"T:\Private Wiki Export\Tools";
pub const PATH_CT_EXPORT_HOME: &str = r"T:\Private Wiki Export\Home";
pub const PATH_CT_EXPORT_FILE_BACKUP_FOLDER: &str = r"T:\ConnectedText\Project Backup";
pub const PATH_CT_EXPORT_IMAGES: &str = r"T:\Private Wiki Export\Images";

pub const FILE_NAME_EXPORT_TOOLS: &str = "Tools.txt";

pub const TAG_CATEGORY: &str = "$CATEGORY:";
pub const TAG_ALIGN_RIGHT: &str = "%%text-align=right%%";

pub const NAMESPACE_TOOLS: &str = "tools";
pub const NAMESPACE_HOME: &str = "home";
pub const _NAMESPACE_ATTRIBUTES: &str = "attr";

const _ATTR_NAME_CATEGORY: &str = "Category";

const CT_FORMAT_BOLD: &str = "**";
const CT_ATTRIBUTE_ASSIGN: &str = ":=";

// const TOPIC_LIMIT_TOOLS: Option<usize> = None;
const _TOPIC_LIMIT_TOOLS: Option<usize> = Some(100);
// const TOPIC_LIMIT_HOME: Option<usize> = None;
const _TOPIC_LIMIT_HOME: Option<usize> = Some(50);

const _IMAGE_SIZE: usize = 750;
const _IMAGE_SIZE_LARGE: usize = 1_500;