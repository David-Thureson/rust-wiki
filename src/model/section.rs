use super::*;
use std::rc::Rc;
use std::cell::RefCell;

pub type SectionRc = Rc<RefCell<Section>>;

pub struct Section {
    pub topic: TopicRc,
    pub parent: Option<SectionRc>,
    pub depth: usize,
    pub title: Option<String>,
    pub sections: Vec<SectionRc>,
    pub paragraphs: Vec<ParagraphRc>,
}