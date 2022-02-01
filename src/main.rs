use wiki::*;

pub(crate) fn main() {
    // connectedtext::to_dokuwiki::main();
    // connectedtext::to_model::main();

    // dokuwiki::gen_tools_wiki::main();
    // dokuwiki::to_model::main();
    // dokuwiki::gen_tools_wiki::gen_from_connectedtext_and_round_trip();

    // dokuwiki::gen_tools_wiki::dokuwiki_round_trip();
    tools_wiki::project::update_coding_project_info(true);
}
