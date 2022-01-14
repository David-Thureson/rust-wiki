use crate::dokuwiki as wiki;
use crate::model;
use crate::connectedtext::to_model::build_model;
use crate::model::ATTRIBUTE_NAME_DOMAIN;
use crate::dokuwiki::gen_from_model::GenFromModel;
use crate::connectedtext::PATH_CT_EXPORT_IMAGES;
use crate::dokuwiki::PATH_MEDIA;

const PROJECT_NAME: &str = "Tools";

pub fn main() {
    gen_from_connectedtext(false, None);
}

fn gen_from_connectedtext(copy_image_files_to_local_wiki: bool, topic_limit: Option<usize>) {
    println!("\nGenerating wiki from ConnectedText...");
    let namespace_main = PROJECT_NAME.to_lowercase();
    let attr_to_index = vec!["Author", "Book", "Company", "Context", "Course", ATTRIBUTE_NAME_DOMAIN, "Domains", "Format", "Founder", "IDE", "Language", "License Type", "LinkedIn", "Narrator", "Operating System", "Organization", "PC Name", "Paradigm", "Platform", "School", "Series", "Status", "Translator"];
    let mut model = build_model(PROJECT_NAME, &namespace_main, topic_limit, attr_to_index);
    // model.interpolate_added_date();
    if copy_image_files_to_local_wiki {
        let path_to = format!("{}/{}", PATH_MEDIA, namespace_main);
        GenFromModel::copy_image_files(PATH_CT_EXPORT_IMAGES, &path_to, true);
    }
    gen_sidebar_page(&model);
    gen_start_page(&model);
    // gen_recent_topics_page();
    gen_all_topics_page(&model);

    // category_tree.report_by_node_count();
    // panic!();

    let mut gen = GenFromModel::new(&model);
    gen.gen_categories_page();
    gen.gen_subtopics_page();
    gen.gen_attr_year_page();
    gen.gen_attr_date_page();
    gen.gen_attr_page();
    gen.gen_attr_value_page();
    // gen_terms_page();
    gen.gen();
    println!("\nDone generating wiki.");
}

fn gen_sidebar_page(model: &model::Wiki) {
    let mut page = wiki::WikiGenPage::new(&model.qualify_namespace(model::NAMESPACE_ROOT), wiki::PAGE_NAME_SIDEBAR, None);
    add_main_page_links(&mut page, model,false, true);
    add_links_to_all_topics(&mut page, &model);
    page.write();
}

fn gen_start_page(model: &model::Wiki) {
    let mut page = wiki::WikiGenPage::new(&model.qualify_namespace(model::NAMESPACE_ROOT), wiki::PAGE_NAME_START, Some(PROJECT_NAME));
    page.add_headline("Main Pages",2);
    add_main_page_links(&mut page, model, true, false);
    page.write();
}

fn gen_all_topics_page(model: &model::Wiki) {
    let mut page = wiki::WikiGenPage::new(&model.qualify_namespace(&model.namespace_navigation()), wiki::PAGE_NAME_ALL_TOPICS,None);
    add_links_to_all_topics(&mut page, model);
    page.write();
}

/*
fn gen_category_subtree(page: &mut wiki::WikiGenPage, depth: usize, node: Ref<model::CategoryTreeNode>) {
    let link = page_link(&node.item.namespace, &node.item.topic_name, None);
    let topic_count = node.subtree_leaf_count();
    let line = format!("{} ({})", link, util::format::format_count(topic_count));
    page.add_list_item_unordered(depth + 1, &line);
    for child_node_rc in node.child_nodes.iter()
        .filter(|child_node_rc| !b!(child_node_rc).is_leaf()){
        gen_category_subtree(page, depth + 1, b!(child_node_rc));
    }
}
*/

fn add_main_page_links(page: &mut wiki::WikiGenPage, model: &model::Wiki, use_list: bool, include_start_page: bool) {
    let mut links = vec![];
    if include_start_page {
        links.push(wiki::page_link(model::NAMESPACE_ROOT, wiki::PAGE_NAME_START, None));
    };
    let qualified_namespace = model.qualify_namespace(&model.namespace_navigation());
    links.append(&mut vec![
        wiki::page_link(&qualified_namespace,wiki::PAGE_NAME_MAIN, None),
        wiki::page_link(&qualified_namespace, wiki::PAGE_NAME_RECENT_TOPICS, None),
        wiki::page_link(&qualified_namespace, wiki::PAGE_NAME_ALL_TOPICS,None),
        wiki::page_link(&qualified_namespace, wiki::PAGE_NAME_CATEGORIES, None),
        wiki::page_link(&qualified_namespace, wiki::PAGE_NAME_SUBTOPICS,None),
        wiki::page_link(&qualified_namespace, wiki::PAGE_NAME_ATTR,None),
        wiki::page_link(&qualified_namespace, wiki::PAGE_NAME_ATTR_VALUE,None),
        wiki::page_link(&qualified_namespace, wiki::PAGE_NAME_ATTR_YEAR,None),
        wiki::page_link(&qualified_namespace, wiki::PAGE_NAME_ATTR_DATE,None),
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

fn add_links_to_all_topics(page: &mut wiki::WikiGenPage, model: &model::Wiki) {
    for topic_key in model.topic_keys_alphabetical_by_topic_name().iter() {
        //bg!(topic_key);
        let link = GenFromModel::page_link(topic_key);
        page.add_line_with_break(&link);
    }
}
