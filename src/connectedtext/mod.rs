// pub(crate) mod parse;
// pub(crate) use parse::*;

pub(crate) mod report;
// pub(crate) use report::*;

// pub(crate) mod to_dokuwiki;
// pub(crate) use to_dokuwiki::*;

pub(crate) mod to_model;

pub(crate) const PATH_CT_EXPORT: &str = r"T:\Private Wiki Export";
// pub(crate) const PATH_CT_EXPORT_TOOLS: &str = r"T:\Private Wiki Export\Tools";
// pub(crate) const PATH_CT_EXPORT_HOME: &str = r"T:\Private Wiki Export\Home";
pub(crate) const PATH_CT_EXPORT_FILE_BACKUP_FOLDER: &str = r"T:\ConnectedText\Project Backup";
pub(crate) const PATH_CT_EXPORT_IMAGES: &str = r"T:\Private Wiki Export\Images";

pub(crate) const FILE_NAME_EXPORT_TOOLS: &str = "Tools.txt";

// pub(crate) const TAG_CATEGORY: &str = "$CATEGORY:";
pub(crate) const TAG_ALIGN_RIGHT: &str = "%%text-align=right%%";

pub(crate) const NAMESPACE_TOOLS: &str = "tools";
// pub(crate) const NAMESPACE_HOME: &str = "home";
pub(crate) const _NAMESPACE_ATTRIBUTES: &str = "attr";

const _ATTR_NAME_CATEGORY: &str = "Category";

const CT_FORMAT_BOLD: &str = "**";
const CT_ATTRIBUTE_ASSIGN: &str = ":=";

// const TOPIC_LIMIT_TOOLS: Option<usize> = None;
const _TOPIC_LIMIT_TOOLS: Option<usize> = Some(100);
// const TOPIC_LIMIT_HOME: Option<usize> = None;
const _TOPIC_LIMIT_HOME: Option<usize> = Some(50);

const _IMAGE_SIZE: usize = 750;
const _IMAGE_SIZE_LARGE: usize = 1_500;