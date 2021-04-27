use std::fs::File;
use std::{fs, io};
use util::file_io;
use std::io::BufRead;

use crate::*;
use super::*;

pub fn main() {
    report_categories();
}

fn report_categories() {
    // Something like [[$CATEGORY:Books]].
    let mut groups = group::Grouper::new("Categories");
    for (_, lines) in get_topic_text() {
        for line in lines {
            if line.contains(TAG_CATEGORY) {
                let category = util::parse::between(&line, &TAG_CATEGORY, "]]").to_string();
                groups.record_entry(&category);
            }
        }
    }
    groups.print_by_count(0, None);
}

