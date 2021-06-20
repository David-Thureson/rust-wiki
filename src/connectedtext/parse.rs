use std::fs::File;
use std::{fs, io};
use util::file_io;
use std::io::BufRead;
use std::collections::BTreeMap;

use crate::*;
use std::time::Instant;

const PATH_CONNECTEDTEXT_EXPORT: &str = "T:\\Private Wiki Export";

pub const TAG_CATEGORY: &str = "$CATEGORY:";

pub fn get_topic_text(topic_limit: Option<usize>) -> BTreeMap<String, Vec<String>> {
    let start_time = Instant::now();
    let mut topics = BTreeMap::new();
    for path in fs::read_dir(PATH_CONNECTEDTEXT_EXPORT).unwrap() {
        if let Some(topic_limit) = topic_limit {
            if topics.len() == topic_limit {
                break;
            }
        }
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
    let limit_label = topic_limit.map_or("".to_string(), |x| fc(x));
    println!("get_topic_text({}): topics = {}, elapsed = {:?}", limit_label, fc(topics.len()), Instant::now() - start_time);
    topics
}

pub fn parse_line_as_category(line: &str) -> Option<String> {
    // If it's a category line it will look like this:
    //   [[$CATEGORY:Books]]
    parse::between_optional_trim(line, "[[$CATEGORY:", "]]").map(|x| x.to_string())
}

pub fn parse_line_as_attribute(line: &str) -> Result<Option<(String, Vec<String>)>, String> {
    // If it's a category line it will look like this if it has multiple values:
    //   ||Author||[[Author:=Kenneth W. Harl]], [[Author:=The Great Courses]]||
    if line.contains(":=") {
        if let Some(between_pipes) = parse::between_optional_trim(line, "||", "||") {
            let name = parse::before(&between_pipes, "||").trim().to_string();
            let name = fix_attribute_name(&name);
            let remaining = parse::after(&between_pipes, &format!("{}||", name));
            let remaining = parse::between(remaining, "[[", "]]");
            let remaining = remaining.replace("]], [[", "]],[[");
            let mut values = vec![];
            for value in remaining.split("]],[[") {
                // let value = parse::between(value,"[[", "]]").trim();
                //bg!(&line, &name, &remaining, &value);
                let value_split = value.split(":=").collect::<Vec<_>>();
                if value_split.len() != 2 {
                    return Err(format!("Problem parsing attribute value in \"{}\".", line));
                }
                let value = value.split(":=").collect::<Vec<_>>()[1].trim();
                values.push(value.to_string());
            }
            return Ok(Some((name.to_string(), values)));
        }
    }
    Ok(None)
}

fn fix_attribute_name(name: &str) -> &str {
    match name {
        "Subject" => "Domain",
        _ => name,
    }
}

#[derive(Clone, Debug)]
pub struct Topic {
    pub name: String,
    pub category_name: Option<String>,
    pub category_topic_names: Vec<String>,
    pub indirect_topic_count: usize,
}

