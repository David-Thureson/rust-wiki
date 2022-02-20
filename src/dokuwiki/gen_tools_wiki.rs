use crate::dokuwiki as wiki;
use crate::model;
use crate::connectedtext::to_model::build_model;
use crate::model::{FOLDER_PREFIX_WIKI_GEN_BACKUP, FOLDER_WIKI_GEN_BACKUP, FOLDER_WIKI_COMPARE_OLD, FOLDER_WIKI_COMPARE_NEW};
use crate::dokuwiki::gen_from_model::GenFromModel;
use crate::connectedtext::PATH_CT_EXPORT_IMAGES;
use crate::dokuwiki::{PATH_MEDIA, PATH_PAGES, FILE_MONITOR_PROJECT_NAME_DOKUWIKI, FILE_MONITOR_SCAN_MINUTES};
use file_monitor::model::Marker as FileMonitorMarker;

pub(crate) const PROJECT_NAME: &str = "Tools";

pub fn main() {
    let copy_image_files = false;
    let topic_limit = None;
    gen_from_connectedtext(copy_image_files, topic_limit);
}

pub fn gen_from_connectedtext_round_trip(compare_only: bool) {
    let start_from_connectedtext = true;
    round_trip(compare_only, start_from_connectedtext);
}

pub fn dokuwiki_round_trip(compare_only: bool) {
    let start_from_connectedtext = false;
    round_trip(compare_only, start_from_connectedtext);
}

fn round_trip(compare_only: bool, start_from_connectedtext: bool) {
    println!("\nDokuWiki round trip test: Start.");

    let model = prep_round_trip(compare_only, start_from_connectedtext);
    complete_round_trip(model, compare_only);

    println!("\nDokuWiki round trip test: Done.");
}

pub(crate) fn prep_round_trip(compare_only: bool, start_from_connectedtext: bool) -> model::Model {
    println!("\ndokuwiki::gen_tools_wiki::prep_round_trip(): Start.");

    if start_from_connectedtext {
        // Back up the existing DokuWiki pages.
        let backup_folder_start = util::file::back_up_folder_next_number_r(PATH_PAGES, FOLDER_WIKI_GEN_BACKUP, FOLDER_PREFIX_WIKI_GEN_BACKUP, 4).unwrap();
        println!("backup_folder_start = \"{}\".", util::file::path_name(&backup_folder_start));

        gen_from_connectedtext(false, None);
        // let path_pages_project = path_pages_project(PATH_PAGES);
        // assert!(util::file::path_exists(&path_pages_project));
    }

    // Back up the DokuWiki pages.
    let backup_folder_old = util::file::back_up_folder_next_number_r(PATH_PAGES, FOLDER_WIKI_GEN_BACKUP, FOLDER_PREFIX_WIKI_GEN_BACKUP, 4).unwrap();
    println!("backup_folder_old = \"{}\".", util::file::path_name(&backup_folder_old));
    // Copy these pages to the "old" comparison folder.
    util::file::copy_folder_recursive_overwrite_r(PATH_PAGES, FOLDER_WIKI_COMPARE_OLD).unwrap();

    let project = file_monitor::model::set_up_project(FILE_MONITOR_PROJECT_NAME_DOKUWIKI, FILE_MONITOR_SCAN_MINUTES);
    if !compare_only {
        project.set_marker(&FileMonitorMarker::Pause);
        project.set_marker(&FileMonitorMarker::Gen);
    }

    // Create a model from the DokuWiki pages.
    let model = super::to_model::build_model(PROJECT_NAME, &PROJECT_NAME.to_lowercase(), None, Some(project));

    println!("\ndokuwiki::gen_tools_wiki::prep_round_trip(): Done.");

    model
}

pub(crate) fn complete_round_trip(mut model: model::Model, compare_only: bool) {

    println!("\ndokuwiki::gen_tools_wiki::complete_round_trip(): Start.");

    super::to_model::complete_model(&mut model);

    // Create DokuWiki pages from this new model.
    let gen_path_pages = if compare_only { FOLDER_WIKI_COMPARE_NEW } else { PATH_PAGES };
    let copy_image_files_to_local_wiki = false;

    gen_tools_project_from_model(&model, gen_path_pages, compare_only, copy_image_files_to_local_wiki);

    if !compare_only {
        // let path_pages_project = path_pages_project(PATH_PAGES);
        let backup_folder_new = util::file::back_up_folder_next_number_r(PATH_PAGES, FOLDER_WIKI_GEN_BACKUP, FOLDER_PREFIX_WIKI_GEN_BACKUP, 4).unwrap();
        println!("backup_folder_new = \"{}\".", util::file::path_name(&backup_folder_new));

        // Back up the DokuWiki pages created with a round trip from DokuWiki.
        // Copy these pages to the "new" comparison folder.
        util::file::copy_folder_recursive_overwrite_r(PATH_PAGES, FOLDER_WIKI_COMPARE_NEW).unwrap();

        if let Some(file_monitor_project) = model.get_file_monitor_project() {
            file_monitor_project.clear_marker(&FileMonitorMarker::Pause);
        }
    }

    println!("\ndokuwiki::gen_tools_wiki::complete_round_trip(): Done.");
}

fn path_pages_project(path_pages: &str) -> String {
    format!("{}/{}", path_pages, PROJECT_NAME.to_lowercase())
}

fn path_media_project() -> String {
    format!("{}/{}", PATH_MEDIA, PROJECT_NAME.to_lowercase())
}

#[allow(dead_code)]
fn clean_up_tools_dokuwiki_files(include_images: bool) {
    let path_pages_project = path_pages_project(PATH_PAGES);
    if util::file::path_exists(&path_pages_project) {
        std::fs::remove_dir_all(&path_pages_project).unwrap();
    }

    // Delete the text files in the main DokuWiki pages folder such as start.txt and sidebar.txt.
    for result_dir_entry in std::fs::read_dir(PATH_PAGES).unwrap() {
        let dir_entry = result_dir_entry.unwrap();
        if util::file::dir_entry_to_file_name(&dir_entry).to_lowercase().ends_with(".txt") {
            std::fs::remove_file(dir_entry.path()).unwrap();
        }
    }

    if include_images {
        let path_media_project = path_media_project();
        if util::file::path_exists(&path_media_project) {
            std::fs::remove_dir_all(&path_media_project).unwrap();
        }
    }
}

fn create_tools_wiki_folders(path_pages: &str) {
    util::file::path_create_if_necessary_r(path_pages_project(path_pages)).unwrap();
    for namespace in ["book", "nav"].iter() {
        let path = format!("{}/{}", path_pages_project(path_pages), namespace);
        util::file::path_create_if_necessary_r(path).unwrap();
    }
}

fn gen_from_connectedtext(copy_image_files_to_local_wiki: bool, topic_limit: Option<usize>) {
    println!("\nGenerating wiki from ConnectedText: Start.");
    let namespace_main = PROJECT_NAME.to_lowercase();
    let model = build_model(PROJECT_NAME, &namespace_main, topic_limit);
    let compare_only = false;
    gen_tools_project_from_model(&model, PATH_PAGES, compare_only, copy_image_files_to_local_wiki);
    println!("\nGenerating wiki from ConnectedText: Done.");
}

fn gen_tools_project_from_model(model: &model::Model, path_pages: &str, compare_only: bool, copy_image_files_to_local_wiki: bool) {
    println!("\nGenerating wiki from model: Start.");

    let namespace_main = PROJECT_NAME.to_lowercase();

    if !compare_only {
        // clean_up_tools_dokuwiki_files(copy_image_files_to_local_wiki);
        util::file::path_create_if_necessary_r(path_media_project()).unwrap();
    }
    create_tools_wiki_folders(path_pages);

    if copy_image_files_to_local_wiki {
        let path_to = format!("{}/{}", PATH_MEDIA, namespace_main);
        GenFromModel::copy_image_files(PATH_CT_EXPORT_IMAGES, &path_to, true);
    }

    // gen_recent_topics_page();

    let mut gen = GenFromModel::new(model, path_pages);
    gen_sidebar_page(model, &mut gen);
    gen_start_page(model, &gen);
    gen.gen_all_topics_page();
    gen.gen_categories_page();
    gen.gen_subtopics_page();
    gen.gen_attr_year_page();
    gen.gen_attr_date_page();
    gen.gen_attr_page();
    gen.gen_attr_value_page();
    // gen_terms_page();
    gen.gen();
    println!("\nGenerating wiki from model: Done.");
}

fn gen_sidebar_page(model: &model::Model, gen: &mut GenFromModel) {
    let mut page = wiki::WikiGenPage::new(&model.qualify_namespace(model::NAMESPACE_ROOT), wiki::PAGE_NAME_SIDEBAR, None);
    add_main_page_links(&mut page, model,false, true);
    gen.gen_topic_first_letter_links(&mut page, 6);
    page.write_if_changed(gen.get_path_pages(), model.get_original_pages());
}

fn gen_start_page(model: &model::Model, gen: &GenFromModel) {
    let mut page = wiki::WikiGenPage::new(&model.qualify_namespace(model::NAMESPACE_ROOT), wiki::PAGE_NAME_START, Some(PROJECT_NAME));
    page.add_headline("Main Pages",2);
    add_main_page_links(&mut page, model, true, false);
    page.write_if_changed(gen.get_path_pages(), model.get_original_pages());
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

fn add_main_page_links(page: &mut wiki::WikiGenPage, model: &model::Model, use_list: bool, include_start_page: bool) {
    let mut links = vec![];
    if include_start_page {
        links.push(wiki::page_link(model::NAMESPACE_ROOT, wiki::PAGE_NAME_START, None));
    };
    let namespace_nav = model.qualify_namespace(&model.namespace_navigation());
    let namespace_main = model.get_main_namespace();
    links.append(&mut vec![
        wiki::page_link(&namespace_main,wiki::PAGE_NAME_MAIN, None),
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_RECENT_TOPICS, None),
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_ALL_TOPICS,None),
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_CATEGORIES, None),
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_SUBTOPICS,None),
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_ATTR,None),
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_ATTR_VALUE,None),
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_ATTR_YEAR,None),
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_ATTR_DATE,None),
        wiki::page_link(&namespace_main, wiki::PAGE_NAME_TERMS, None),
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

/*
fn add_links_to_all_topics(page: &mut wiki::WikiGenPage, model: &model::Wiki) {
    for topic_key in model.topic_keys_alphabetical_by_topic_name().iter() {
        //bg!(topic_key);
        let link = GenFromModel::page_link(topic_key);
        page.add_line_with_break(&link);
    }
}
*/

