use crate::dokuwiki as wiki;
use crate::connectedtext::NAMESPACE_TOOLS;
use crate::model;
use crate::connectedtext::to_model::build_wiki;

pub fn main() {
    gen_from_connectedtext(true, None);
}

fn gen_from_connectedtext(_copy_image_files_to_local_wiki: bool, topic_limit: Option<usize>) {
    println!("\nGenerating wiki from ConnectedText...");
    let wiki = build_wiki(topic_limit);
    // if copy_image_files_to_local_wiki {
    //     copy_image_files(db, NaiveDate::from_ymd(1900, 3, 20), true);
    // }
    gen_sidebar_page();
    gen_start_page();
    // gen_recent_topics_page();
    // gen_categories_page();
    // gen_terms_page();
    crate::dokuwiki::gen::gen_from_model(&wiki);
    println!("\nDone generating wiki.");
}

fn gen_sidebar_page() {
    let mut page = wiki::WikiGenPage::new(model::NAMESPACE_ROOT, wiki::PAGE_NAME_SIDEBAR, None);
    add_main_page_links(&mut page, false, true);
    page.write();
}

fn gen_start_page() {
    let mut page = wiki::WikiGenPage::new(model::NAMESPACE_ROOT, wiki::PAGE_NAME_START, Some("Tools"));
    page.add_headline("Main Pages",2);
    add_main_page_links(&mut page, true, false);
    page.write();
}

fn add_main_page_links(page: &mut wiki::WikiGenPage, use_list: bool, include_start_page: bool) {
    let mut links = vec![];
    if include_start_page {
        links.push(wiki::page_link(model::NAMESPACE_ROOT, wiki::PAGE_NAME_START, Some("Start")));
    };
    links.append(&mut vec![
        wiki::page_link(NAMESPACE_TOOLS, "Main", None),
        wiki::page_link(NAMESPACE_TOOLS, "Recent Topics", None),
        wiki::page_link(NAMESPACE_TOOLS, "Categories", None),
        wiki::page_link(NAMESPACE_TOOLS, "Terms", None),
    ]);
    if use_list {
        let mut list = wiki::WikiList::new(None);
        for link in links.iter() {
            list.add_item(link);
        }
        page.add_list(&list);
    } else {
        for link in links.iter() {
            page.add_paragraph(link);
        }
    }
}
