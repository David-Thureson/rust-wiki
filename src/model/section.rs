use super::*;
use std::rc::Rc;
use std::cell::RefCell;

pub type SectionRc = Rc<RefCell<Section>>;

// At first keep a flat list of sections. We may not need them in a hierarchy.
pub struct Section {
    topic: TopicRc,
    depth: usize,
    title: String,
    paragraphs: Vec<ParagraphRc>,
}
