use crate::model::*;


#[allow(dead_code)]
pub(crate) fn report_category_tree(wiki: &Model) {
    let mut parent_child_pairs = vec![];
    for topic in wiki.get_topics().values() {
        if let Some(category_name) = &topic.get_category() {
            parent_child_pairs.push((category_name.clone(), topic.get_name().to_string()));
        }
    }
    let tree = util::tree::Tree::create(parent_child_pairs, true);
    tree.report_by_node_count();
}

