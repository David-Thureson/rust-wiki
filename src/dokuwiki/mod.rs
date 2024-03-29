pub(crate) mod gen;
pub(crate) use gen::*;

pub(crate) mod gen_from_model;

pub(crate) mod gen_page;
pub(crate) use gen_page::*;

pub mod gen_tools_wiki;

pub(crate) mod parse;
pub(crate) use parse::*;

pub mod to_model;

//pub(crate) mod model;
//pub(crate) use model::*;

pub(crate) const PATH_PAGES: &str = "C:/Doku/DokuWikiStick/dokuwiki/data/pages";
pub(crate) const PATH_MEDIA: &str = "C:/Doku/DokuWikiStick/dokuwiki/data/media";
pub(crate) const PATH_TEMP_SOURCE: &str = "C:/Wiki Gen Backup/Temp_Source";

pub(crate) const FILE_MONITOR_PROJECT_NAME_DOKUWIKI: &str = "DokuWiki";
pub(crate) const FILE_MONITOR_SCAN_MINUTES: f32 = 1.0;

pub(crate) const RECENT_TOPICS_THRESHOLD: usize = 100;

pub(crate) const PAGE_NAME_SIDEBAR:         &str = "Sidebar";
pub(crate) const PAGE_NAME_MAIN:            &str = "Main";
pub(crate) const PAGE_NAME_START:           &str = "Start";
pub(crate) const PAGE_NAME_RECENT_TOPICS:   &str = "Recent Topics";
pub(crate) const PAGE_NAME_REPORTS:         &str = "Reports";
pub(crate) const PAGE_NAME_DOKUWIKI_MARKUP: &str = "DokuWiki Markup";
pub(crate) const PAGE_NAME_ALL_TOPICS:      &str = "All Topics";
pub(crate) const PAGE_NAME_CATEGORIES:      &str = "Categories";
pub(crate) const PAGE_NAME_SUBTOPICS:       &str = "Subtopics";
pub(crate) const PAGE_NAME_ATTR:            &str = "Attributes";
pub(crate) const PAGE_NAME_ATTR_VALUE:      &str = "Attribute Values";
pub(crate) const PAGE_NAME_ATTR_YEAR:       &str = "Years";
pub(crate) const PAGE_NAME_ATTR_DATE:       &str = "Dates";
pub(crate) const PAGE_NAME_TERMS:           &str = "Terms";
pub(crate) const PAGE_NAME_CLOUD_TERMS:     &str = "Cloud and Data Management Terms";
pub(crate) const PAGE_NAME_PROFISEE_TERMS:  &str = "Profisee Terms";
pub(crate) const PAGE_NAME_SOC_SVC_TERMS:   &str = "Social Services Terms";

pub(crate) const HEADLINE_LINKS: &str = "Generated Links";

pub(crate) const DELIM_NAMESPACE: &str = ":";
pub(crate) const DELIM_LINEFEED: &str = "\n";
pub(crate) const DELIM_PARAGRAPH: &str = "\n\n";
pub(crate) const DELIM_BREADCRUMB_RIGHT: &str = "=>";
pub(crate) const DELIM_BREADCRUMB_LEFT: &str = "<=";
pub(crate) const DELIM_BOLD: &str = "**";
#[allow(dead_code)]
pub(crate) const DELIM_ITALIC: &str = "//";
pub(crate) const DELIM_LINK_START: &str = "[[";
pub(crate) const DELIM_LINK_END: &str = "]]";
pub(crate) const DELIM_LINK_LABEL: &str = "|";
pub(crate) const DELIM_LINK_SECTION: &str = "#";
pub(crate) const DELIM_IMAGE_START: &str = "{{";
pub(crate) const DELIM_IMAGE_END: &str = "}}";
pub(crate) const DELIM_IMAGE_OPTIONS: &str = "?";
pub(crate) const DELIM_HEADER: &str = "=";
pub(crate) const DELIM_TABLE_CELL: &str = "|";
pub(crate) const DELIM_TABLE_CELL_BOLD: &str = "^";
pub(crate) const DELIM_LIST_ITEM_ORDERED: &str = "-";
pub(crate) const DELIM_LIST_ITEM_UNORDERED: &str = "*";
pub(crate) const DELIM_LIST_ITEM_DEPTH: &str = "  ";

pub(crate) const MARKER_LINE_START: &str = "<";
pub(crate) const MARKER_LINE_START_CLOSE: &str = "</";
pub(crate) const MARKER_LINE_END: &str = ">";
// pub(crate) const MARKER_QUOTE_START: &str = "<WRAP round box>";
pub(crate) const MARKER_QUOTE_START_PREFIX: &str = "<WRAP";
// pub(crate) const MARKER_QUOTE_END: &str = "</WRAP>";
pub(crate) const MARKER_CODE_START_PREFIX: &str = "<code";
pub(crate) const MARKER_CODE_END: &str = "</code>";
// Change the symbols if this causes a parsing issue.
pub(crate) const MARKER_REDACTION: &str = "~~~redacted~~~";
pub(crate) const MARKER_REDACTION_FINAL: &str = "%%*****%%";
#[allow(dead_code)]
pub(crate) const MARKER_PUBLIC_IN_TEXT_FILE: &str = "^ Visibility | Public |";
pub(crate) const MARKER_DELETE_THIS_FILE: &str = "{{{delete}}}";
pub(crate) const MARKER_TERMS_ADD_DEFINITION: &str = "(//add definition//)";

pub(crate) const TEMP_DELIM_IMG_START: &str = "[[{{";
pub(crate) const TEMP_DELIM_IMG_END: &str = "}}]]";
pub(crate) const TEMP_COMMA: &str = "~temp comma~";

pub(crate) const PREFIX_CATEGORY: &str = "Category: ";

/*
pub(crate) fn back_up_from_live() {
    let path_dest = path_backup();


    iuoeu
}

pub(crate) fn back_up_from_compare() {
}
*/