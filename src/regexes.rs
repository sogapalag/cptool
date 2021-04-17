use lazy_static::lazy_static;
use regex::Regex;

pub fn is_module_def(text: &str) -> Option<&str> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^[[:space:]]*(pub )?mod (?P<name>[[:word:]]+);").unwrap();
    }
    RE.captures(text)
        .and_then(|cap| cap.name("name").map(|name| name.as_str()))
}

pub fn get_module_def(text: &str) -> Option<&str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^(?P<def>(pub )?mod ([[:word:]]+));").unwrap();
    }
    RE.captures(text)
        .and_then(|cap| cap.name("def").map(|x| x.as_str()))
}

// re export
pub fn get_use(text: &str) -> Option<&str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"pub use (?P<use>self::[[:word:]]+::)").unwrap();
    }
    RE.captures(text)
        .and_then(|cap| cap.name("use").map(|x| x.as_str()))
}
pub fn get_sub_modules(text: &str) -> Vec<&str> {
    text.lines().filter_map(|l| is_module_def(l)).collect()
}
pub fn is_inner_doc(text: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[[:space:]]*//!").unwrap();
    }
    RE.is_match(text)
}
pub fn is_outer_doc(text: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[[:space:]]*///").unwrap();
    }
    RE.is_match(text)
}
pub fn is_doc(text: &str) -> bool {
    is_inner_doc(text) || is_outer_doc(text)
}

pub fn is_comment(text: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[[:space:]]*//").unwrap();
    }
    RE.is_match(text)
}
