use std::collections::BTreeMap;
use crate::model::{TopicErrorList, ATTRIBUTE_NAME_DOMAIN, Model};
use crate::Itertools;

#[derive(Debug)]
pub struct DomainList {
    domains: BTreeMap<String, Domain>,
}

#[derive(Debug)]
pub struct Domain {
    name: String,
    related: BTreeMap<String, usize>,
    related_by_count: Vec<String>,
}

impl DomainList {
    pub fn new() -> Self {
        Self {
            domains: Default::default(),
        }
    }

    pub fn add_domain_optional(&mut self, domain_name: &str) {
        if !self.domains.contains_key(domain_name) {
            self.domains.insert(domain_name.to_string(), Domain::new(domain_name));
        }
    }

    pub fn get_domains(&self) -> &BTreeMap<String, Domain> {
        &self.domains
    }

    pub fn get_domain(&self, name: &str) -> Option<&Domain> {
        self.domains.get(name)
    }

    pub fn add_related_domain(&mut self, domain_name: &str, related_name: &str) {
        let domain= self.domains.get_mut(domain_name).unwrap();
        let entry = domain.related.entry(related_name.to_string()).or_insert(0);
        *entry += 1;
    }

    pub fn catalog_domains(model: &mut Model) -> TopicErrorList {
        // This must be run after catalog_attributes().
        debug_assert!(!model.attributes.attributes.is_empty());
        let errors = TopicErrorList::new();
        model.domains = DomainList::new();
        for topic in model.topics.values_mut() {
            if let Some(attribute_instance) = topic.attributes.get(ATTRIBUTE_NAME_DOMAIN) {
                if attribute_instance.values.len() == 1 {
                    model.domains.add_domain_optional(&attribute_instance.values[0]);
                } else {
                    let values = attribute_instance.values.clone();
                    for i in 0..values.len() {
                        model.domains.add_domain_optional(&attribute_instance.values[i]);
                        for j in 0..values.len() {
                            if i != j {
                                model.domains.add_related_domain(&attribute_instance.values[i], &attribute_instance.values[j]);
                            }
                        }
                    }
                }
            }
        }
        for domain in model.domains.domains.values_mut() {
            domain.fill_related_by_count();
        }
        // model.domains.print();
        // model.domains.print_strong_relationships();
        errors
    }

    pub fn print(&self) {
        println!("Domains: ({})", util::format::format_count(self.domains.len()));
        for domain in self.domains.values() {
            let related_list = domain.related_by_count.iter().join(", ");
            println!("\t{}: {}", domain.name, related_list);
        }
    }

    pub fn print_strong_relationships(&self) {
        println!("Domains: ({})", util::format::format_count(self.domains.len()));
        for domain in self.domains.values()
                .sorted_by(|a, b| a.max_related_count().cmp(&b.max_related_count()).reverse().then(a.name.cmp(&b.name))) {
            println!("\n\t{}", domain.name);
            for (related_name, related_count) in domain.related.iter()
                    .sorted_by(|a, b| a.1.cmp(&b.1).reverse().then(a.0.cmp(&b.0))) {
                println!("\t\t({}) {}", util::format::format_count(*related_count), related_name);
            }
        }
    }
}

impl Domain {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            related: Default::default(),
            related_by_count: vec![],
        }
    }

    pub fn fill_related_by_count(&mut self) {
        // Sort the related domains so that the most closely related ones (higher count) come
        // first. If there's a tie, use alphabetical order.
        let mut sorted = self.related.iter()
            .map(|(name, count)| (name.clone(), count))
            .collect::<Vec<_>>();
        sorted.sort_by(|a, b| a.1.cmp(b.1).reverse().then(a.0.cmp(&b.0)));
        self.related_by_count = sorted.drain(..).map(|(name, _count)| name).collect::<Vec<_>>();
    }

    pub fn get_related_by_count(&self) -> &Vec<String> {
        &self.related_by_count
    }

    pub fn max_related_count(&self) -> usize {
        *self.related.values().max().unwrap_or(&0)
    }
}