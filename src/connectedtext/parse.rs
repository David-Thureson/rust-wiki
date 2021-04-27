use std::fs::File;
use std::{fs, io};
use util::file_io;
use std::io::BufRead;
use std::collections::BTreeMap;

use crate::*;

const PATH_CONNECTEDTEXT_EXPORT: &str = "T:\\Private Wiki Export";

pub const TAG_CATEGORY: &str = "$CATEGORY:";

pub fn get_topic_text() -> BTreeMap<String, Vec<String>> {
    let mut topics = BTreeMap::new();
    for path in fs::read_dir(PATH_CONNECTEDTEXT_EXPORT).unwrap() {
        let dir_entry = path.as_ref().unwrap();
        let file_name = file_io::dir_entry_to_file_name(dir_entry);
        if file_name.to_lowercase().ends_with(".txt") {
            let topic_name = parse::before_ci(&file_name, ".txt");
            assert!(!topics.contains_key(&topic_name.to_string()));
            let mut lines = vec![];
            let file = File::open(format!("{}/{}", PATH_CONNECTEDTEXT_EXPORT, file_name)).unwrap();
            for raw_line_result in io::BufReader::new(file).lines() {
                //bg!(&raw_line_result);
                let line = raw_line_result.unwrap();
                lines.push(line);
            }
            topics.insert(topic_name.to_string(), lines);
        }
    }
    topics
}

