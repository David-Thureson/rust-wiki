use crate::*;
use crate::model::*;
use manage_projects::model::Model as ProjectModel;
use manage_projects::model::Dependency;
use util::date_time::{naive_date_to_doc_format, date_time_to_naive_date};
use std::collections::BTreeMap;
use util::format::first_cap_phrase;

pub(crate) fn add_project_info_to_model(model: &mut Model) {
    let project_model = manage_projects::import::build_model(true);
    let name_map = make_name_map(model, &project_model);
    model.set_projects(project_model);
    model.set_projects_name_map(name_map);
}

pub(crate) fn update_projects_and_libraries(model: &mut Model) {
    add_missing_projects(model);
    // print_name_map(&name_map, Some(&name_map_clone), "Diff from add_missing_projects()");
    // let name_map_clone = name_map.clone();

    add_missing_libraries(model);
    // print_name_map(&name_map, Some(&name_map_clone), "Diff from add_missing_libraries()");

    update_dependency_paragraphs(model);
    update_used_by_paragraphs(model);
}

#[allow(dead_code)]
fn add_missing_projects(model: &mut Model) {
    let mut name_map = model.get_projects_name_map().as_ref().unwrap().clone();
    let projects = model.get_projects().as_ref().unwrap().clone();
    for pc in projects.pcs.values() {
        for project in pc.projects.values()
                .filter(|project| !ignore_project(&project.name)) {
            if !name_map.contains_key(&project.name.to_lowercase()) {
                let topic_name = name_project(&project.name);
                let first_date = naive_date_to_doc_format(&date_time_to_naive_date(&project.first_time()));
                let last_date = naive_date_to_doc_format(&date_time_to_naive_date(&project.last_time()));
                let mut topic = Topic::new(model.get_main_namespace(), &topic_name);
                topic.set_category(CATEGORY_RUST_PROJECTS);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_LANGUAGE.to_string(), vec!["Rust".to_string()]);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_PC_NAME.to_string(), vec![pc.name.clone()]);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_FOLDER.to_string(), vec![project.path.clone()]);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_PLATFORM.to_string(), vec!["Windows".to_string()]);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_IDE.to_string(), vec!["IntelliJ IDEA".to_string()]);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_STARTED.to_string(), vec![first_date.clone()]);
                if first_date != last_date {
                    topic.add_temp_attribute_values(ATTRIBUTE_NAME_UPDATED.to_string(), vec![last_date]);
                }
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_ADDED.to_string(), vec![first_date]);
                //bg!(&topic.get_temp_attributes()); panic!();
                name_map.insert(project.name.to_lowercase(), topic.get_topic_key());
                model.add_topic(topic);
            }
        }
    }
    model.set_projects_name_map(name_map);
}

#[allow(dead_code)]
fn add_missing_libraries(model: &mut Model) {
    let dep_proj_map = get_dependency_project_map(&model.get_projects().as_ref().unwrap());
    let mut name_map = model.get_projects_name_map().as_ref().unwrap().clone();
    for (crate_name, (dep, _project_names)) in dep_proj_map.iter() {
        if !name_map.contains_key(&crate_name.to_lowercase()) {
            let topic_name = name_crate(dep);
            // We don't want references from one Rust project to another within an overall project
            // to count as crate dependencies. So if the dependency is another Rust project, ignore
            // it here.
            if !topic_name.contains("(Rust project)") {
                let mut topic = Topic::new(model.get_main_namespace(), &topic_name);
                topic.set_category(CATEGORY_RUST_CRATES);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_LANGUAGE.to_string(), vec!["Rust".to_string()]);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_ADDED.to_string(), vec![date_now_to_doc_format()]);
                //bg!(&topic.get_temp_attributes()); panic!();
                name_map.insert(crate_name.to_lowercase(), topic.get_topic_key());
                model.add_topic(topic);
            }
        }
    }
    model.set_projects_name_map(name_map);
}

fn update_dependency_paragraphs(model: &mut Model) {
    //bg!(&project_model);
    //bg!(&name_map);
    // If a project doesn't show up in the project_model returned from the manage-projects app, we
    // leave it alone.
    let name_map = model.get_projects_name_map().as_ref().unwrap().clone();
    let projects = model.get_projects().as_ref().unwrap().clone();
    let list_type = LIST_TYPE_DEPENDENCIES;

    for pc in projects.pcs.values() {
        for project in pc.projects.values() {
            let topic_key = name_map.get(&project.name.to_lowercase());
            if let Some(topic_key) = topic_key {
                let topic = model.get_topics_mut().get_mut(topic_key).unwrap();
                let mut dep_paragraph = match topic.remove_list_paragraph_by_type(list_type) {
                    Some((_index, paragraph)) => paragraph,
                    None => Paragraph::new_list_of_type(list_type),
                };
                let list = dep_paragraph.get_list_mut();
                list.set_type_and_header(list_type);
                list.set_is_ordered(false);
                for rust_project in project.rust_projects.values() {
                    for dep in rust_project.dependencies.values() {
                        // let dep_topic_name = name_crate(dep);
                        // let dep_topic_key = TopicKey::new(&namespace_root, &dep_topic_name);
                        match name_map.get(&dep.crate_name.to_lowercase()) {
                            Some(dep_topic_key) => {
                                list.add_item_topic_link_if_missing(1, false, dep_topic_key);
                            },
                            None => {
                                // sim is a reference in the Monkeybait Dev project to an internal
                                // Rust project. We don't handle this case yet.
                                if dep.crate_name.ne("sim") {
                                    println!("In topic \"{}\", dependency \"{}\" not found.", &topic.get_name(), &dep.crate_name);
                                }
                            }
                        }
                    }
                }
                if !list.get_items().is_empty() {
                    list.sort_items();
                    let dep_paragraph_index = topic.get_paragraph_index_end_of_first_section();
                    topic.insert_paragraph(dep_paragraph_index, dep_paragraph);
                }
            }
        }
    }
}

fn update_used_by_paragraphs(model: &mut Model) {
    let name_map = model.get_projects_name_map().as_ref().unwrap().clone();
    let list_type = LIST_TYPE_USED_BY;
    for (dep, using_project_names) in get_dependency_project_map(&model.get_projects().as_ref().unwrap()).values() {
        let topic_key = name_map.get(&dep.crate_name.to_lowercase());
        // assert!(topic_key.is_some());
        if let Some(topic_key) = topic_key {
            let topic = model.get_topics_mut().get_mut(topic_key).unwrap();
            let mut used_by_paragraph = match topic.remove_list_paragraph_by_type(list_type) {
                Some((_index, paragraph)) => paragraph,
                None => Paragraph::new_list_of_type(list_type),
            };
            let list = used_by_paragraph.get_list_mut();
            list.set_type_and_header(list_type);
            list.set_is_ordered(false);
            for using_project_name in using_project_names.iter() {
                match name_map.get(&*using_project_name.to_lowercase()) {
                    Some(using_topic_key) => {
                        list.add_item_topic_link_if_missing(1, false, &using_topic_key);
                    },
                    None => {
                        println!("In topic \"{}\", used by topic \"{}\" not found.", &topic.get_name(), using_project_name);
                    }
                }
            }
            if !list.get_items().is_empty() {
                list.sort_items();
                let used_by_paragraph_index = topic.get_paragraph_index_end_of_first_section();
                topic.insert_paragraph(used_by_paragraph_index, used_by_paragraph);
            }
        }
    }
}

fn make_name_map(model: &Model, project_model: &ProjectModel) -> NameTopicMap {
    let mut topic_names = BTreeMap::new();
    for topic_key in model.get_topics().keys() {
        topic_names.insert(topic_key.get_topic_name().to_lowercase(), topic_key.clone());
    }
    let mut name_map = BTreeMap::new();
    for pc in project_model.pcs.values() {
        //bg!(&pc.name);
        //bg!(&pc.projects.keys());
        for project in pc.projects.values()
                .filter(|project| !ignore_project(&project.name)) {
            // if project.name.to_lowercase().contains("rayon") { dbg!(&project.name); }
            for possible_name in project_potential_topic_names_lower(&project.name).iter() {
                if let Some(topic_key) = topic_names.get(possible_name) {
                    // if name_map.contains_key(possible_name) { dbg!(&project.name, possible_name, topic_key); }
                    // assert!(!name_map.contains_key(possible_name));
                    name_map.insert(project.name.to_lowercase(), topic_key.clone());
                    break;
                }
            }
            for rust_project in project.rust_projects.values() {
                for dependency in rust_project.dependencies.values() {
                    if dependency.crate_name.eq("geo") {
                        name_map.insert("geo".to_string(), TopicKey::new("tools", "Geo (Rust crate)"));
                    } else {
                        for possible_name in project_potential_crate_names_lower(&dependency.crate_name).iter() {
                            // let debug = possible_name.eq("sim");
                            if let Some(topic_key) = topic_names.get(possible_name) {
                                // if debug { //bg!(&topic_key); }
                                name_map.insert(dependency.crate_name.to_lowercase(), topic_key.clone());
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    name_map
}

#[allow(dead_code)]
fn get_project_dependency_map(project_model: &ProjectModel) -> BTreeMap<String, BTreeMap<String, Dependency>> {
    // This assumes that we won't find the same project name on two PCs, and that within a given
    // logical project that contains multiple Rust projects, we don't care which dependencies are
    // in which Rust project.
    let mut map = BTreeMap::new();
    for pc in project_model.pcs.values() {
        for project in pc.projects.values() {
            if !ignore_project(&project.name) {
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
    }
    map
}

#[allow(dead_code)]
fn get_dependency_project_map(project_model: &ProjectModel) -> BTreeMap<String, (Dependency, Vec<String>)> {
    let proj_dep_map = get_project_dependency_map(project_model);
    let mut map: BTreeMap<String, (Dependency, Vec<String>)> = BTreeMap::new();
    for (project_name, dependencies) in proj_dep_map.iter() {
        if !ignore_project(project_name) {
            for dep in dependencies.values() {
                let dep_name = dep.crate_name.to_lowercase();
                let entry = map.entry(dep_name).or_insert((dep.clone(), vec![]));
                entry.1.push(project_name.to_string());
            }
        }
    }
    for (_dep, project_names) in map.values_mut() {
        project_names.sort();
    }
    map
}

#[allow(dead_code)]
fn print_name_map(name_map: &NameTopicMap, subtract_map: Option<&NameTopicMap>, label: &str) {
    println!("\n{}:", label);
    for (name, topic_key) in name_map.iter() {
        if subtract_map.map_or(true, |subtract_map| !subtract_map.contains_key(name)) {
            println!("\t{} == {}", name, topic_key.get_display_text());
        }
    }
    println!();
}

/*
pub fn update_coding_project_info(compare_only: bool) {
    println!("\ndokuwiki::gen_tools_wiki::update_coding_project_info(): Start.");

    let start_from_connectedtext = false;
    let mut model = prep_round_trip(start_from_connectedtext);
    let topic_names_lower = model.get_topic_names().iter().map(|name| name.to_lowercase()).collect::<Vec<_>>();

    let project_model = manage_projects::import::build_model(true);
    // report_projects_not_in_wiki(&project_model, &topic_names_lower);
    // report_unknown_crates_in_use(&project_model, &topic_names_lower);
    // panic!();
    add_missing_crates(&mut model, &project_model, &topic_names_lower);
    add_missing_projects(&mut model, &project_model, &topic_names_lower);
    
    complete_round_trip(model, compare_only);

    println!("\ndokuwiki::gen_tools_wiki::update_coding_project_info(): Done.");
}
*/

/*
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
*/


/*
#[allow(dead_code)]
fn add_missing_crates(model: &mut model::Model, project_model: &manage_projects::model::Model) {
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
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_PLATFORM.to_string(), vec!["Windows".to_string()]);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_IDE.to_string(), vec!["IntelliJ IDEA".to_string()]);
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_STARTED.to_string(), vec![first_date.clone()]);
                if first_date != last_date {
                    topic.add_temp_attribute_values(ATTRIBUTE_NAME_UPDATED.to_string(), vec![last_date]);
                }
                topic.add_temp_attribute_values(ATTRIBUTE_NAME_ADDED.to_string(), vec![first_date]);
                //bg!(&topic.get_temp_attributes()); panic!();
                model.add_topic(topic);
            }
        }
    }

}
 */

fn project_potential_topic_names_lower(project_name: &str) -> Vec<String> {
    let name_1 = project_name.trim().to_lowercase();
    let name_2 = name_1.replace("-", " ").replace("_", " ");
    // Go from most specific to least specific, so we don't end up, for instance, thinking that
    // "Algorithms" is a rust project topic when we also have "Algorithms (Rust project)".
    vec![
        format!("{} (rust project)", name_2),
        format!("{} (rust project)", name_1),
        format!("{} (coding project)", name_2),
        format!("{} (coding project)", name_1),
        name_2.clone(),
        name_1.clone(),
    ]
}

fn project_potential_crate_names_lower(crate_name: &str) -> Vec<String> {
    let name_1 = crate_name.trim().to_lowercase();
    let name_2 = name_1.replace("-", " ").replace("_", " ");
    if name_1.eq("geo") {
        // There's an unrelated topic called "Geo".
        return vec!["Geo (Rust crate)".to_string()];
    }
    // Go from most specific to least specific, so we don't end up, for instance, thinking that
    // "Graphics" is a rust crate topic when we also have "Graphics (Rust crate)".
    vec![
        format!("{} (rust crate)", name_2),
        format!("{} (rust crate)", name_1),
        format!("{} (crate)", name_2),
        format!("{} (crate)", name_1),
        format!("{} (rust project)", name_2),
        format!("{} (rust project)", name_1),
        name_2.clone(),
        name_1.clone(),
    ]
}

/*
fn wiki_has_project(project_name: &str, topic_names_lower: &Vec<String>) -> bool {
    project_potential_topic_names_lower(project_name).iter()
        .any(|potential_name| topic_names_lower.contains())
    topic_names_lower.iter().any(|name| )
    let proj_name_1 = project_name.to_lowercase().replace("-", " ").replace("_", " ");
    let proj_name_2 = format!("{} (coding project)", proj_name_1);
    let proj_name_3 = format!("{} (rust project)", proj_name_1);
    //bg!(&proj_name_1, topic_names_lower.contains(&proj_name_1));
    //bg!(&proj_name_2, topic_names_lower.contains(&proj_name_2));
    //bg!(&proj_name_3, topic_names_lower.contains(&proj_name_3));
    topic_names_lower.contains(&proj_name_1) || topic_names_lower.contains(&proj_name_2) || topic_names_lower.contains(&proj_name_3)
}
*/

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

/*
#[allow(dead_code)]
fn report_unknown_crates_in_use(project_model: &manage_projects::model::Model, topic_names_lower: &Vec<String>) { 
    let dep_proj_map = get_dependency_project_map(project_model);
    for (crate_name, (dep, project_names)) in dep_proj_map.iter() {
        // if dep.is_local {
        //     dbg!(&dep);
        // }
        if !wiki_has_crate(crate_name, topic_names_lower) {
            // let project_names = project_names.iter().filter(|project_name| !ignore_project(project_name)).collect::<Vec<_>>();
            if !project_names.is_empty() {
                let project_list = project_names.iter().join(", ");
                println!("{}: {}: {}", name_crate(dep), crate_name, project_list);
            }
        }
    }
}

fn wiki_has_crate(crate_name: &str, topic_names_lower: &Vec<String>) -> bool {
    let crate_name_1 = crate_name.trim().to_lowercase();
    let crate_name_2 = crate_name_1.replace("-", " ").replace("_", " ");
    let crate_names = vec![
        crate_name_1.clone(),
        format!("{} (crate)", crate_name_1),
        format!("{} (rust crate)", crate_name_1),
        format!("{} (rust project)", crate_name_1),
        crate_name_2.clone(),
        format!("{} (crate)", crate_name_2),
        format!("{} (rust crate)", crate_name_2),
        format!("{} (rust project)", crate_name_2),
    ];
    crate_names.iter().any(|crate_name| topic_names_lower.contains(crate_name))
}

//}

// let dependency_project_map = get_dependency_project_map(&project_model);
//}

*/

#[allow(dead_code)]
fn name_project(folder_name: &str) -> String {
    if folder_name.eq("wsdl") || folder_name.eq("ddp") {
        return folder_name.to_uppercase()
    }
    let name = folder_name.replace("-", " ").replace("_", " ");
    if name.eq("rust asm") || name.eq("util") {
        return "Rust ASM (Rust project)".to_string();
    }
    let name = first_cap_phrase(&name);
    let name = format!("{} (Rust project)", name);
    name
}

#[allow(dead_code)]
fn name_crate(dep: &Dependency) -> String {
    let mut name = util::format::first_cap_phrase(&dep.crate_name);
    name = name.replace("-", " ").replace("_", " ");
    // if dep.is_local {
    if dep.crate_name.eq("sim") || dep.crate_name.eq("util") {
        name = format!("{} (Rust project)", name);
    } else {
        name = format!("{} (Rust crate)", name);
    }
    name
}
