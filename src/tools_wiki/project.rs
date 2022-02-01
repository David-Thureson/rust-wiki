use crate::*;
use crate::dokuwiki::gen_tools_wiki::{prep_round_trip, complete_round_trip};
use util::date_time::{datetime_as_date, naive_date_to_doc_format, date_time_to_naive_date};
use std::collections::BTreeMap;
use util::format::first_cap_phrase;
use crate::model::{CATEGORY_RUST_PROJECTS, ATTRIBUTE_NAME_ADDED, ATTRIBUTE_NAME_LANGUAGE, ATTRIBUTE_NAME_PC_NAME, ATTRIBUTE_NAME_FOLDER, ATTRIBUTE_NAME_STARTED, ATTRIBUTE_NAME_UPDATED};

pub fn update_coding_project_info(compare_only: bool) {
    println!("\ndokuwiki::gen_tools_wiki::update_coding_project_info(): Start.");

    let start_from_connectedtext = false;
    let mut model = prep_round_trip(start_from_connectedtext);
    let topic_names_lower = model.get_topic_names().iter().map(|name| name.to_lowercase()).collect::<Vec<_>>();

    let project_model = manage_projects::import::build_model(true);
    // report_projects_not_in_wiki(&project_model, &topic_names_lower);
    add_missing_projects(&mut model, &project_model, &topic_names_lower);

    complete_round_trip(model, compare_only);

    println!("\ndokuwiki::gen_tools_wiki::update_coding_project_info(): Done.");
}

#[allow(dead_code)]
fn report_projects_not_in_wiki(project_model: &manage_projects::model::Model, topic_names_lower: &Vec<String>) {
    println!("\nreport_projects_not_in_wiki():");
    for pc in project_model.pcs.values() {
        for project in pc.projects.values() {
            //rintln!();
            //bg!(&project.name, ignore_project(&project.name), wiki_has_project(&project.name, topic_names_lower));
            if !ignore_project(&project.name) && !wiki_has_project(&project.name, topic_names_lower) {
                println!("\t{}: {}: {}; {} to {}", name_project(&project.name), project.name, project.path, datetime_as_date(&project.first_time()), datetime_as_date(&project.last_time()));
            }
        }
    }
}

#[allow(dead_code)]
fn add_missing_projects(model: &mut model::Model, project_model: &manage_projects::model::Model, topic_names_lower: &Vec<String>) {
    for pc in project_model.pcs.values() {
        for project in pc.projects.values() {
            if !ignore_project(&project.name) && !wiki_has_project(&project.name, topic_names_lower) {
                let topic_name = name_project(&project.name);
                let first_date = naive_date_to_doc_format(&date_time_to_naive_date(&project.first_time()));
                let last_date = naive_date_to_doc_format(&date_time_to_naive_date(&project.last_time()));
                let mut topic = model::Topic::new(model.get_main_namespace(), &topic_name);
                topic.set_category(CATEGORY_RUST_PROJECTS);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_LANGUAGE.to_string(), vec!["Rust".to_string()]);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_PC_NAME.to_string(), vec![pc.name.clone()]);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_FOLDER.to_string(), vec![project.path.clone()]);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_STARTED.to_string(), vec![first_date.clone()]);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_UPDATED.to_string(), vec![last_date]);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_ADDED.to_string(), vec![first_date]);
                //bg!(&topic.get_temp_attributes()); panic!();
                model.add_topic(topic);
            }
        }
    }

}

fn wiki_has_project(project_name: &str, topic_names_lower: &Vec<String>) -> bool {
    let proj_name_1 = project_name.to_lowercase().replace("-", " ").replace("_", " ");
    let proj_name_2 = format!("{} (coding project)", proj_name_1);
    let proj_name_3 = format!("{} (rust project)", proj_name_1);
    //bg!(&proj_name_1, topic_names_lower.contains(&proj_name_1));
    //bg!(&proj_name_2, topic_names_lower.contains(&proj_name_2));
    //bg!(&proj_name_3, topic_names_lower.contains(&proj_name_3));
    topic_names_lower.contains(&proj_name_1) || topic_names_lower.contains(&proj_name_2) || topic_names_lower.contains(&proj_name_3)
}

fn ignore_project(project_name: &str) -> bool {
    let proj_name = project_name.trim().to_lowercase();
    proj_name.ends_with("_hold")
        || proj_name.ends_with(" hold")
        || proj_name.ends_with("_old")
        || proj_name.ends_with(" old")
        || proj_name.ends_with(" copy")
        || proj_name.contains(" copy ")
        || proj_name.ends_with(" compare")
        || proj_name.ends_with(" check")
}

// fn catalog_unknown_crates_in_use(model: &model::Model) {



//}

// let dependency_project_map = get_dependency_project_map(&project_model);
//}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
fn name_project(folder_name: &str) -> String {
    if folder_name.eq("wsdl") || folder_name.eq("ddp") {
        return folder_name.to_uppercase()
    }
    let name = folder_name.replace("-", " ").replace("_", " ");
    first_cap_phrase(&name)
}