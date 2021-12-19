use crate::model;
use crate::dokuwiki as wiki;
use crate::model::Paragraph;

pub fn gen(wiki: &model::Wiki) {
    for topic in wiki.topics.values() {
        let mut page = wiki::WikiGenPage::new(&topic.namespace, &topic.name, None);
        add_category_optional(&mut page, &topic);
        add_paragraphs(&mut page, &topic);
        page.write();
    }
}

fn add_category_optional(page: &mut wiki::WikiGenPage, topic: &model::Topic) {
    if let Some(category) = topic.category.as_ref() {
        page.add_category(category);
    }
}

fn add_paragraphs(page: &mut wiki::WikiGenPage, topic: &model::Topic) {
    for paragraph in topic.paragraphs.iter() {
        match paragraph {
            Paragraph::Attributes => {},
            Paragraph::Breadcrumbs => {},
            Paragraph::Category => {}, // This was already added to the page.
            Paragraph::GenStart => {},
            Paragraph::GenEnd => {},
            Paragraph::List { .. } => {},
            Paragraph::Placeholder => {
            },
            Paragraph::Quote { .. } => {}
            Paragraph::SectionHeader { .. } => {}
            Paragraph::Text { .. } => {}
            Paragraph::TextUnresolved { .. } => {}
            Paragraph::Unknown { .. } => {}
        }
    }
}
