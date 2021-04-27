use super::*;

pub fn main() {
    // report_categories();
    // get_categories();
    // report_categories_by_topic_count();
    report_category_tree_2();
}

/*
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
*/

