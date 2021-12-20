use crate::dokuwiki as wiki;
use crate::model;
use crate::connectedtext::to_model::build_model;
use crate::model::NAMESPACE_NAVIGATION;

pub fn main() {
    gen_from_connectedtext(true, None);
}

fn gen_from_connectedtext(_copy_image_files_to_local_wiki: bool, topic_limit: Option<usize>) {
    println!("\nGenerating wiki from ConnectedText...");
    let model = build_model(topic_limit);
    // if copy_image_files_to_local_wiki {
    //     copy_image_files(db, NaiveDate::from_ymd(1900, 3, 20), true);
    // }
    gen_sidebar_page(&model);
    gen_start_page(&model);
    // gen_recent_topics_page();
    // gen_categories_page();
    // gen_terms_page();
    crate::dokuwiki::gen_from_model::GenFromModel::new(&model).gen();
    println!("\nDone generating wiki.");
}

fn gen_sidebar_page(model: &model::Wiki) {
    let mut page = wiki::WikiGenPage::new(model::NAMESPACE_ROOT, wiki::PAGE_NAME_SIDEBAR, None);
    add_main_page_links(&mut page, model,false, true);
    page.write();
}

fn gen_start_page(model: &model::Wiki) {
    let mut page = wiki::WikiGenPage::new(model::NAMESPACE_ROOT, wiki::PAGE_NAME_START, Some("Tools"));
    page.add_headline("Main Pages",2);
    add_main_page_links(&mut page, model, true, false);
    page.write();
}

fn add_main_page_links(page: &mut wiki::WikiGenPage, model: &model::Wiki, use_list: bool, include_start_page: bool) {
    let mut links = vec![];
    if include_start_page {
        links.push(wiki::page_link(model::NAMESPACE_ROOT, wiki::PAGE_NAME_START, Some("Start")));
    };
    let qualified_namespace = model.qualify_namespace(NAMESPACE_NAVIGATION);
    links.append(&mut vec![
        wiki::page_link(&qualified_namespace,"Main", None),
        wiki::page_link(&qualified_namespace, "Recent Topics", None),
        wiki::page_link(&qualified_namespace, "Categories", None),
        wiki::page_link(&qualified_namespace, "Terms", None),
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
