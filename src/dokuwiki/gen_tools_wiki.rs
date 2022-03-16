use crate::dokuwiki as wiki;
use crate::model;
use crate::model::{FOLDER_PREFIX_WIKI_GEN_BACKUP, FOLDER_WIKI_GEN_BACKUP, FOLDER_WIKI_COMPARE_OLD, FOLDER_WIKI_COMPARE_NEW};
use crate::dokuwiki::gen_from_model::GenFromModel;
use crate::dokuwiki::{PATH_MEDIA, PATH_PAGES, FILE_MONITOR_PROJECT_NAME_DOKUWIKI, FILE_MONITOR_SCAN_MINUTES};
use file_monitor::model::Marker as FileMonitorMarker;
use crate::dokuwiki::to_model::BuildProcess;

pub(crate) const PROJECT_NAME: &str = "Tools";

pub fn dokuwiki_round_trip(mut compare_only: bool, is_public: bool) {
    println!("\nDokuWiki round trip test: Start.");

    if is_public {
        compare_only = true;
    }

    let (model, build_process) = prep_round_trip(compare_only, is_public);
    complete_round_trip(model, build_process);

    println!("\nDokuWiki round trip test: Done.");
}

pub(crate) fn prep_round_trip(compare_only: bool, is_public: bool) -> (model::Model, BuildProcess) {
    println!("\ndokuwiki::gen_tools_wiki::prep_round_trip(): Start.");

    let project = file_monitor::model::set_up_project(FILE_MONITOR_PROJECT_NAME_DOKUWIKI, FILE_MONITOR_SCAN_MINUTES);
    if !compare_only {
        project.set_marker(&FileMonitorMarker::Pause);
        project.set_marker(&FileMonitorMarker::Gen);
    }

    // Create a model from the DokuWiki pages.
    let (model, build_process) = super::to_model::build_model(PROJECT_NAME, &PROJECT_NAME.to_lowercase(), compare_only, is_public, None, Some(project));

    // Back up the DokuWiki pages.
    let backup_folder_old = util::file::back_up_folder_next_number_r(PATH_PAGES, FOLDER_WIKI_GEN_BACKUP, FOLDER_PREFIX_WIKI_GEN_BACKUP, 4).unwrap();
    println!("backup_folder_old = \"{}\".", util::file::path_name(&backup_folder_old));
    // Copy these pages to the "old" comparison folder.
    util::file::copy_folder_recursive_overwrite_r(PATH_PAGES, FOLDER_WIKI_COMPARE_OLD).unwrap();

    println!("\ndokuwiki::gen_tools_wiki::prep_round_trip(): Done.");

    (model, build_process)
}

pub(crate) fn complete_round_trip(mut model: model::Model, mut build_process: BuildProcess) {

    println!("\ndokuwiki::gen_tools_wiki::complete_round_trip(): Start.");

    if model.is_public() {
        assert!(build_process.compare_only);
    }

    super::to_model::complete_model(&mut model);

    // Create DokuWiki pages from this new model.
    build_process.gen_path_pages = if build_process.compare_only { FOLDER_WIKI_COMPARE_NEW.to_string() } else { PATH_PAGES.to_string() };
    // if build_process.compare_only || model.is_public() {
    //     clean_up_tools_dokuwiki_files(&build_process.gen_path_pages, false);
    // }

    gen_tools_project_from_model(&model, &mut build_process);

    // At this point the standard generated files like [start.txt] and everything in the nav folder
    // have been written to disk. Now write the files for the main group of topics if they've
    // changed, and delete files that are no longer in the model, most likely because it's a public
    // build.

    build_process.write_main_topic_files();

    if !build_process.compare_only {
        // let path_pages_project = path_pages_project(PATH_PAGES);
        let backup_folder_new = util::file::back_up_folder_next_number_r(PATH_PAGES, FOLDER_WIKI_GEN_BACKUP, FOLDER_PREFIX_WIKI_GEN_BACKUP, 4).unwrap();
        println!("backup_folder_new = \"{}\".", util::file::path_name(&backup_folder_new));

        // Back up the DokuWiki pages created with a round trip from DokuWiki.
        // Copy these pages to the "new" comparison folder.
        util::file::copy_folder_recursive_overwrite_r(PATH_PAGES, FOLDER_WIKI_COMPARE_NEW).unwrap();

        if let Some(file_monitor_project) = model.get_file_monitor_project() {
            // If we're doing the public build, leave the file monitor paused since we don't want
            // to count any changes until the next private build.
            if !model.is_public() {
                file_monitor_project.clear_marker(&FileMonitorMarker::Pause);
            }
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
fn clean_up_tools_dokuwiki_files(path_pages: &str, include_images: bool) {
    let path_pages_project = path_pages_project(path_pages);
    if util::file::path_exists(&path_pages_project) {
        std::fs::remove_dir_all(&path_pages_project).unwrap();
    }

    // Delete the text files in the main DokuWiki pages folder such as start.txt and sidebar.txt.
    for result_dir_entry in std::fs::read_dir(path_pages).unwrap() {
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

fn gen_tools_project_from_model(model: &model::Model, build_process: &mut BuildProcess) {
    println!("\nGenerating wiki from model: Start.");

    if !build_process.compare_only {
        // clean_up_tools_dokuwiki_files(copy_image_files_to_local_wiki);
        util::file::path_create_if_necessary_r(path_media_project()).unwrap();
    }
    create_tools_wiki_folders(&build_process.gen_path_pages);

    let mut gen = GenFromModel::new(model, &build_process.gen_path_pages);
    gen_sidebar_page(model, &mut gen);
    gen_start_page(model, &gen);
    gen.gen_recent_topics_page();
    gen.gen_all_topics_page();
    gen.gen_categories_page();
    gen.gen_subtopics_page();
    gen.gen_attr_year_page();
    gen.gen_attr_date_page();
    gen.gen_attr_pages();
    gen.gen_attr_value_page();
    gen.gen_reports_page();
    // gen_terms_page();
    build_process.topic_dest_files = gen.gen();
    assert!(!build_process.topic_dest_files.is_empty());
    println!("\nGenerating wiki from model: Done.");
}

fn gen_sidebar_page(model: &model::Model, gen: &mut GenFromModel) {
    let mut page = wiki::WikiGenPage::new(&model.qualify_namespace(model::NAMESPACE_ROOT), wiki::PAGE_NAME_SIDEBAR, None);
    add_main_page_links(&mut page, model,false, true);
    gen.gen_topic_first_letter_links(&mut page, 6);
    page.write(gen.get_path_pages());
}

fn gen_start_page(model: &model::Model, gen: &GenFromModel) {
    let mut page = wiki::WikiGenPage::new(&model.qualify_namespace(model::NAMESPACE_ROOT), wiki::PAGE_NAME_START, Some(PROJECT_NAME));
    page.add_headline("Main Pages",2);
    add_main_page_links(&mut page, model, true, false);
    page.write(gen.get_path_pages());
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
    if !model.is_public() {
        links.push(wiki::page_link(&namespace_main, wiki::PAGE_NAME_MAIN, None));
    }
    links.append(&mut vec![
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_RECENT_TOPICS, None),
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_ALL_TOPICS,None),
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_CATEGORIES, None),
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_SUBTOPICS,None),
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_ATTR,None),
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_ATTR_VALUE,None),
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_ATTR_YEAR,None),
        wiki::page_link(&namespace_nav, wiki::PAGE_NAME_ATTR_DATE,None),
    ]);
    if !model.is_public() {
        links.push(wiki::page_link(&namespace_nav, wiki::PAGE_NAME_REPORTS, None));
        links.push(wiki::page_link(&namespace_main, wiki::PAGE_NAME_DOKUWIKI_MARKUP, None));
    }
    links.push(wiki::page_link(&namespace_main, wiki::PAGE_NAME_TERMS, None));
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

