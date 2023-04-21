use wiki::*;
#[allow(unused_imports)]
use wiki::model::{make_topic_ref, NAMESPACE_TOOLS};

pub(crate) fn main() {

    let compare_only = false;
    let filter_is_public = false;
    let filter_main_topic_ref = None;

    // let compare_only = false;
    // let filter_is_public = false;
    // let filter_main_topic_ref = Some(make_topic_ref(NAMESPACE_TOOLS, "tempo_project"));

    // connectedtext::to_dokuwiki::main();
    // connectedtext::to_model::main();

    // dokuwiki::gen_tools_wiki::main();
    // dokuwiki::to_model::main();
    // dokuwiki::gen_tools_wiki::gen_from_connectedtext_round_trip();

    util::date_time::print_elapsed(true, "round trip", "", ||
        dokuwiki::gen_tools_wiki::dokuwiki_round_trip(compare_only, filter_is_public, filter_main_topic_ref.clone())
    );
    
    // tools_wiki::project::update_coding_project_info(compare_only);
}
