use wiki::*;

pub(crate) fn main() {

    let compare_only = false;

    // connectedtext::to_dokuwiki::main();
    // connectedtext::to_model::main();

    // dokuwiki::gen_tools_wiki::main();
    // dokuwiki::to_model::main();
    // dokuwiki::gen_tools_wiki::gen_from_connectedtext_round_trip();

    dokuwiki::gen_tools_wiki::dokuwiki_round_trip(compare_only);

    // tools_wiki::project::update_coding_project_info(compare_only);
}
