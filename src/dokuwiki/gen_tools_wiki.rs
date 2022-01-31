use crate::dokuwiki as wiki;
use crate::model;
use crate::connectedtext::to_model::build_model;
use crate::model::{ATTRIBUTE_NAME_DOMAIN, FOLDER_PREFIX_WIKI_GEN_BACKUP, FOLDER_WIKI_GEN_BACKUP, FOLDER_WIKI_COMPARE_OLD, FOLDER_WIKI_COMPARE_NEW};
use crate::dokuwiki::gen_from_model::GenFromModel;
use crate::connectedtext::PATH_CT_EXPORT_IMAGES;
use crate::dokuwiki::{PATH_MEDIA, PATH_PAGES};
use std::collections::BTreeMap;

pub(crate) const PROJECT_NAME: &str = "Tools";

pub fn main() {
    let copy_image_files = false;
    let topic_limit = None;
    gen_from_connectedtext(copy_image_files, topic_limit);
}

pub fn gen_from_connectedtext_round_trip() {
    round_trip(true);
}

pub fn dokuwiki_round_trip() {
    round_trip(false);
}

fn round_trip(start_from_connectedtext: bool) {
    println!("\nDokuWiki round trip test: Start.");

    let compare_only = false;

    let model = prep_round_trip(start_from_connectedtext);
    complete_round_trip(model, compare_only);

    println!("\nDokuWiki round trip test: Done.");
}

fn prep_round_trip(start_from_connectedtext: bool) -> model::Model {
    println!("\ndokuwiki::gen_tools_wiki::prep_round_trip(): Start.");

    let path_pages_project = path_pages_project();

    if start_from_connectedtext {
        // Back up the existing DokuWiki pages.
        if util::file::path_exists(&path_pages_project) {
            let backup_folder_start = util::file::back_up_folder_next_number_r(&path_pages_project, FOLDER_WIKI_GEN_BACKUP, FOLDER_PREFIX_WIKI_GEN_BACKUP, 4).unwrap();
            println!("backup_folder_start = \"{}\".", util::file::path_name(&backup_folder_start));
        }

        gen_from_connectedtext(false, None);
        assert!(util::file::path_exists(&path_pages_project));
    }

    // Back up the DokuWiki pages.
    let backup_folder_old = util::file::back_up_folder_next_number_r(&path_pages_project, FOLDER_WIKI_GEN_BACKUP, FOLDER_PREFIX_WIKI_GEN_BACKUP, 4).unwrap();
    println!("backup_folder_old = \"{}\".", util::file::path_name(&backup_folder_old));
    // Copy these pages to the "old" comparison folder.
    util::file::copy_folder_recursive_overwrite_r(&path_pages_project, FOLDER_WIKI_COMPARE_OLD).unwrap();

    // Create a model from the DokuWiki pages.
    let model = super::to_model::build_model(PROJECT_NAME, &PROJECT_NAME.to_lowercase(), None, get_attr_to_index());

    println!("\ndokuwiki::gen_tools_wiki::prep_round_trip(): Done.");

    model
}

pub(crate) fn complete_round_trip(model: model::Model, compare_only: bool) {

    println!("\ndokuwiki::gen_tools_wiki::complete_round_trip(): Start.");

    // Create DokuWiki pages from this new model.
    let gen_path_pages = if compare_only { FOLDER_WIKI_COMPARE_NEW } else { PATH_PAGES };
    let copy_image_files_to_local_wiki = false;
    gen_tools_project_from_model(&model, gen_path_pages, copy_image_files_to_local_wiki);

    if !compare_only {
        let path_pages_project = path_pages_project();
        let backup_folder_new = util::file::back_up_folder_next_number_r(&path_pages_project, FOLDER_WIKI_GEN_BACKUP, FOLDER_PREFIX_WIKI_GEN_BACKUP, 4).unwrap();
        println!("backup_folder_new = \"{}\".", util::file::path_name(&backup_folder_new));

        // Back up the DokuWiki pages created with a round trip from DokuWiki.
        // Copy these pages to the "new" comparison folder.
        util::file::copy_folder_recursive_overwrite_r(&path_pages_project, FOLDER_WIKI_COMPARE_NEW).unwrap();
    }

    println!("\ndokuwiki::gen_tools_wiki::complete_round_trip(): Done.");
}

fn path_pages_project() -> String {
    format!("{}/{}", PATH_PAGES, PROJECT_NAME.to_lowercase())
}

fn path_media_project() -> String {
    format!("{}/{}", PATH_MEDIA, PROJECT_NAME.to_lowercase())
}

fn clean_up_tools_dokuwiki_files(include_images: bool) {
    let path_pages_project = path_pages_project();
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

fn create_tools_wiki_folders() {
    util::file::path_create_if_necessary_r(path_pages_project()).unwrap();
    util::file::path_create_if_necessary_r(path_media_project()).unwrap();
    for namespace in ["book", "nav"].iter() {
        let path = format!("{}/{}", path_pages_project(), namespace);
        util::file::path_create_if_necessary_r(path).unwrap();
    }
}

fn gen_from_connectedtext(copy_image_files_to_local_wiki: bool, topic_limit: Option<usize>) {
    println!("\nGenerating wiki from ConnectedText: Start.");
    let namespace_main = PROJECT_NAME.to_lowercase();
    let model = build_model(PROJECT_NAME, &namespace_main, topic_limit, get_attr_to_index());
    gen_tools_project_from_model(&model, PATH_PAGES, copy_image_files_to_local_wiki);
    println!("\nGenerating wiki from ConnectedText: Done.");
}

fn gen_tools_project_from_model(model: &model::Model, path_pages: &str, copy_image_files_to_local_wiki: bool) {
    println!("\nGenerating wiki from model: Start.");

    let namespace_main = PROJECT_NAME.to_lowercase();

    clean_up_tools_dokuwiki_files(copy_image_files_to_local_wiki);
    create_tools_wiki_folders();

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

pub(crate) fn get_attr_to_index() -> Vec<&'static str> {
    vec!["Author", "Book", "Company", "Context", "Course", ATTRIBUTE_NAME_DOMAIN, "Domains", "Format", "Founder", "IDE", "Language", "License Type", "LinkedIn", "Narrator", "Operating System", "Organization", "PC Name", "Paradigm", "Platform", "School", "Series", "Status", "Translator"]
}

pub fn update_coding_project_info(_compare_only: bool) {
    println!("\ndokuwiki::gen_tools_wiki::update_coding_project_info(): Start.");

    let start_from_connectedtext = false;
    let model = prep_round_trip(start_from_connectedtext);
    let topic_names_lower = model.get_topic_names().iter().map(|name| name.to_lowercase()).collect::<Vec<_>>();

    let project_model = manage_projects::import::build_model(true);
    report_projects_not_in_wiki(&project_model, &topic_names_lower);


    // complete_round_trip(model, compare_only);

    println!("\ndokuwiki::gen_tools_wiki::update_coding_project_info(): Done.");
}

fn report_projects_not_in_wiki(project_model: &manage_projects::model::Model, topic_names_lower: &Vec<String>) {
    println!("\nreport_projects_not_in_wiki():");
    let proj_dep_map = get_project_dependency_map(project_model);
    for proj_name in proj_dep_map.keys() {
        let proj_name_1 = proj_name.to_lowercase();
        let proj_name_2 = format!("{} (coding project)", proj_name_1);
        let proj_name_3 = format!("{} (rust project)", proj_name_1);
        if !topic_names_lower.contains(&proj_name_1) && !topic_names_lower.contains(&proj_name_2) && !topic_names_lower.contains(&proj_name_3) {
            println!("\t{}", proj_name);
        }
    }
}


// fn catalog_unknown_crates_in_use(model: &model::Model) {



    //}

    // let dependency_project_map = get_dependency_project_map(&project_model);
//}

fn get_project_dependency_map(project_model: &manage_projects::model::Model) -> BTreeMap<String, BTreeMap<String, manage_projects::model::Dependency>> {
    // This assumes that we won't find the same project name on two PCs, and that within a given
    // logical project that contains multiple Rust projects, we don't care which dependencies are
    // in which Rust project.
    let mut map = BTreeMap::new();
    for pc in project_model.pcs.values() {
        for project in pc.projects.values() {
            let project_key = project.name.clone();
            assert!(!map.contains_key(&project_key));
            let entry = map.entry(project_key).or_insert(BTreeMap::new());
            for rust_project in project.rust_projects.values() {
                for dependency in rust_project.dependencies.values() {
                    let dependency_key = dependency.to_string();
                    entry.insert(dependency_key, dependency.clone());
                }
            }
        }
    }
    map
}

fn get_dependency_project_map(project_model: &manage_projects::model::Model) -> BTreeMap<String, Vec<String>> {
    let proj_dep_map = get_project_dependency_map(project_model);
    let mut map = BTreeMap::new();
    for (project_name, dependencies) in proj_dep_map.iter() {
        for dep in dependencies.values() {
            let dep_name = dep.crate_name.to_lowercase();
            let entry = map.entry(dep_name).or_insert(vec![]);
            entry.push(project_name.to_string());
        }
    }
    for project_list in map.values_mut() {
        project_list.sort();
    }
    map
}
