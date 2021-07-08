use crate::*;
use crate::model::*;

pub fn report_build_breakdown(wiki_rc: &WikiRc) {
    let wiki = b!(wiki_rc);
    let namespace_count = wiki.topics.len();
    let topic_count = wiki.topics.len();
    let category_count = wiki.categories.len();
    let attribute_count = wiki.attributes.len();
    println!()

}

