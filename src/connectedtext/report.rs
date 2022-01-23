use crate::model::*;

pub fn report_category_tree(wiki: &Model) {
    let mut parent_child_pairs = vec![];
    for topic in wiki.get_topics().values() {
        if let Some(category_name) = &topic.get_category() {
            parent_child_pairs.push((category_name.clone(), topic.get_name().to_string()));
        }
    }
    /*
    for (topic_name, lines) in get_topic_text(None) {
        for line in lines {
            if line.contains(TAG_CATEGORY) {
                let category_name = util::parse::between(&line, &TAG_CATEGORY, "]]").to_lowercase().to_string();
                parent_child_pairs.push((category_name, topic_name.clone()));
                break;
            }
        }
    }
     */

    // parent_child_pairs.sort();
    // parent_child_pairs.iter().for_each(|(parent, child)| { println!("\"{}\"\t\"{}\"", parent, child); });

    let tree = util::tree::Tree::create(parent_child_pairs, true);
    tree.report_by_node_count();
}

