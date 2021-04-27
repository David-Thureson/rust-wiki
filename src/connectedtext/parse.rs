use std::fs::File;
use std::{fs, io};
use util::file_io;
use std::io::BufRead;
use std::collections::BTreeMap;

use crate::*;
use std::cmp::Reverse;

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

pub fn get_categories() -> BTreeMap<String, Topic> {
    let mut topics = BTreeMap::new();
    for (topic_name, lines) in get_topic_text() {
        let mut category_name = None;
        for line in lines {
            if line.contains(TAG_CATEGORY) {
                category_name = Some(util::parse::between(&line, &TAG_CATEGORY, "]]").to_lowercase().to_string());
                break;
            }
        }
        let topic = Topic {
            name: topic_name.clone(),
            category_name,
            category_topic_names: vec![],
            indirect_topic_count: 0,
        };
        topics.insert(topic_name.clone(), topic);
    }
    //bg!(&topics);
    let mut temp_categories = topics.clone();
    for topic in topics.values() {
        if let Some(category_name) = &topic.category_name {
            let entry = temp_categories.entry(category_name.to_string()).or_insert(Topic {
                name: category_name.clone(),
                category_name: None,
                category_topic_names: vec![],
                indirect_topic_count: 0,
            });
            entry.category_topic_names.push(topic.name.clone());
            entry.indirect_topic_count += 1;
        }
    }
    // Keep only the topics that are categories.
    let mut categories = BTreeMap::new();
    temp_categories.values()
        .filter(|topic| !topic.category_topic_names.is_empty())
        .for_each(|topic| {
            let mut new_topic = topic.clone();
            new_topic.category_topic_names.sort();
            categories.insert(topic.name.clone(), new_topic);
        });
    //bg!(&categories);
    categories
}

pub fn report_category_tree() {
    let categories = get_categories();
    categories.values()
        .filter(|category| category.category_name.is_none())
        .for_each(|category| report_category_tree_one(&categories, 0, category));
}

fn report_category_tree_one(categories: &BTreeMap<String, Topic>, depth: usize, topic: &Topic) {
    let line = format!("{}: topics: {}", topic.name, topic.category_topic_names.len());
    format::println_indent_tab(depth, &line);
    categories.values()
        .filter(|category| category.category_name == Some(topic.name.clone()))
        .for_each(|category| report_category_tree_one(&categories, depth + 1, category));
}

pub fn report_categories_by_topic_count() {
    let mut categories = get_categories().values().map(|topic| topic.clone()).collect::<Vec<_>>();
    categories.sort_by_cached_key(|topic| Reverse(topic.category_topic_names.len()));
    categories.iter()
        .for_each(|topic| {
            let topic_category = match &topic.category_name {
                Some(category_name) => format!(" ({})", category_name),
                None => "".to_string()
            };
            println!("{}{}: {}", topic.name, topic_category, topic.category_topic_names.len());
        });
}

pub fn report_category_tree_2() {
    let mut parent_child_pairs = vec![];
    for (topic_name, lines) in get_topic_text() {
        for line in lines {
            if line.contains(TAG_CATEGORY) {
                let category_name = util::parse::between(&line, &TAG_CATEGORY, "]]").to_lowercase().to_string();
                parent_child_pairs.push((category_name, topic_name.clone()));
                break;
            }
        }
    }

    // parent_child_pairs.sort();
    // parent_child_pairs.iter().for_each(|(parent, child)| { println!("\"{}\"\t\"{}\"", parent, child); });

    let tree = util::tree::Tree::create(parent_child_pairs);
    tree.report_by_node_count();
}

#[derive(Clone, Debug)]
pub struct Topic {
    pub name: String,
    pub category_name: Option<String>,
    pub category_topic_names: Vec<String>,
    pub indirect_topic_count: usize,
}


