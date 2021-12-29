use crate::dokuwiki as wiki;
use crate::model;
use crate::connectedtext::to_model::build_model;
use crate::model::NAMESPACE_NAVIGATION;
use crate::dokuwiki::gen_from_model::GenFromModel;
use crate::dokuwiki::CategoryTree;

const PROJECT_NAME: &str = "Tools";

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
    gen_all_topics_page(&model);
    let category_tree = model.category_tree(&model.main_namespace);

    // category_tree.report_by_node_count();
    // panic!();

    gen_categories_page(&model, &category_tree);
    // gen_terms_page();
    GenFromModel::new(&model).gen();
    println!("\nDone generating wiki.");
}

fn gen_sidebar_page(model: &model::Wiki) {
    let mut page = wiki::WikiGenPage::new(&model.qualify_namespace(model::NAMESPACE_ROOT), wiki::PAGE_NAME_SIDEBAR, None);
    add_main_page_links(&mut page, model,false, true);
    add_all_topics(&mut page, &model);
    page.write();
}

fn gen_start_page(model: &model::Wiki) {
    let mut page = wiki::WikiGenPage::new(&model.qualify_namespace(model::NAMESPACE_ROOT), wiki::PAGE_NAME_START, Some(PROJECT_NAME));
    page.add_headline("Main Pages",2);
    add_main_page_links(&mut page, model, true, false);
    page.write();
}

fn gen_all_topics_page(model: &model::Wiki) {
    let mut page = wiki::WikiGenPage::new(&model.qualify_namespace(model::NAMESPACE_NAVIGATION), wiki::PAGE_NAME_ALL_TOPICS,None);
    add_all_topics(&mut page, model);
    page.write();
}

fn gen_categories_page(model: &model::Wiki, _category_tree: &CategoryTree) {
    let page = wiki::WikiGenPage::new(&model.qualify_namespace(model::NAMESPACE_NAVIGATION), wiki::PAGE_NAME_CATEGORIES,None);




    page.write();
}

fn add_main_page_links(page: &mut wiki::WikiGenPage, model: &model::Wiki, use_list: bool, include_start_page: bool) {
    let mut links = vec![];
    if include_start_page {
        links.push(wiki::page_link(model::NAMESPACE_ROOT, wiki::PAGE_NAME_START, None));
    };
    let qualified_namespace = model.qualify_namespace(NAMESPACE_NAVIGATION);
    links.append(&mut vec![
        wiki::page_link(&qualified_namespace,wiki::PAGE_NAME_MAIN, None),
        wiki::page_link(&qualified_namespace, wiki::PAGE_NAME_RECENT_TOPICS, None),
        wiki::page_link(&qualified_namespace, wiki::PAGE_NAME_ALL_TOPICS,None),
        wiki::page_link(&qualified_namespace, wiki::PAGE_NAME_CATEGORIES, None),
        wiki::page_link(&qualified_namespace, wiki::PAGE_NAME_TERMS, None),
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

fn add_all_topics(page: &mut wiki::WikiGenPage, model: &model::Wiki) {
    for topic_key in model.topic_keys_alphabetical_by_topic_name().iter() {
        //bg!(topic_key);
        let link = GenFromModel::page_link(model, topic_key);
        page.add_line_with_break(&link);
    }
}
