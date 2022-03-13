/*
Put the redaction up front, right after reading in the file.
Firt record the names and file names of the private topics, along with the original file text
of the public topics.

Then compile the full list of phrases, using the blacklist and whitelist, etc. right after that
first pass.

Then for each of the public topics, redact the file contents as a single string. Then while
refining the paragraphs for the first time, deal with the redact marker. For instance, remove list
items that are nothing but a redaction. Any link (topic, section, or URL) that contains a redaction
marker should be completely replaced with a text block that is only a redaction marker.

Also, any public topic that has a redacted phrase in its title should be removed as if it's private.
This will cover some combination topics and other cases. This is probably too complicated. Instead
throw an error in this case and manually change this topic to Private.

While refining paragraphs for a public build, ignore the Added and Visibility attributes, and
don't call the later code that might add them (from file-monitor in the case of Added).
*/

use crate::dokuwiki::MARKER_REDACTION;
use crate::*;
use crate::model::FILE_NAME_REDACT;

pub(crate) fn finalize_redacted_phrases(mut phrases: Vec<String>) -> Vec<String> {
    // The model already has a list of redacted phrases consisting of the file names and topic
    // names for private topics.
    assert!(!phrases.is_empty());

    let mut blacklist = util::file::read_file_as_lines_r(FILE_NAME_REDACT).unwrap();
    phrases.append(&mut blacklist);

    let mut whitelist = PHRASE_WHITELIST.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    // In the whitelist, add versions of the original whitelist where spaces are replaced with
    // underscores. So if "social media" is on the list, "social_media" will be as well. This will
    // cover more cases such as when a whitelisted phrase appears in a link.
    whitelist.append(&mut PHRASE_WHITELIST.iter().map(|x| x.replace(" ", "_")).collect::<Vec<_>>());
    whitelist.sort();
    whitelist.dedup();

    // Lowercase and get rid of any blank phrases and whitespace.
    phrases = phrases.iter()
        .map(|phrase| phrase.trim().to_lowercase())
        .filter(|phrase| !phrase.is_empty() && !whitelist.contains(phrase))
        .collect();
    phrases.sort();
    phrases.dedup();
    //redact_phrases.sort_by_key(|x| (Reverse(x.len()), x));
    phrases.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
    //bg!(&self.phrases); panic!();
    phrases
}

pub(crate) fn text_contains_phrase(text: &str, phrases: &Vec<String>) -> bool {
    debug_assert!(!text.is_empty());
    debug_assert!(!phrases.is_empty());
    let text_lower = text.to_lowercase();
    for phrase in phrases.iter() {
        if text_lower.contains(phrase) {
            println!("redaction::text_contains_phrase() for \"{}\": phrase found is \"{}\"", text, phrase);
            return true;
        }
    }
    false
}

pub fn redact_text(text: &str, phrases: &Vec<String>) -> Option<(String)> {
    debug_assert!(!text.is_empty());
    debug_assert!(!phrases.is_empty());
    let mut working_text = text.to_string();
    loop {
        match find_match(&working_text, phrases) {
            Some((start_index, end_index, phrase)) => {
                working_text = format!("{}{}{}", &working_text[0..start_index], MARKER_REDACTION, &working_text[end_index..working_text.len()]);
            },
            None => {
                break;
            }
        }
    }
    if working_text.ne(text) {
        Some(working_text)
    } else {
        None
    }
}

fn find_match(text: &str, phrases: &Vec<String>) -> Option<(usize, usize, String)> {
    debug_assert!(!text.is_empty());
    debug_assert!(!phrases.is_empty());
    let text_lower = text.to_lowercase();
    for phrase in phrases.iter() {
        if let Some(pos) = text_lower.find(phrase) {
            return Some((pos, pos + phrase.len(), phrase.clone()))
        }
    }
    None
}

const PHRASE_WHITELIST: [&str; 36] = ["behavioral economics", "bluehost", "bold", "domains", "grit", "health", "keto", "machines", "main", "meetings",
    "meetup", "music", "nlp", "nori", "oracle vm virtualbox", "organizations", "oracle vm virtualbox", "pcs", "philips hue", "pmwiki", "podcasts", "practices",
    "precalculus", "privacy", "queue", "rework", "sbt", "security project", "simplify", "skype", "social media platform", "to do", "twitter", "virtualbox",
    "winit", "wordpress"];
