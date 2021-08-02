extern crate regex;

pub trait RegExp {
    fn matches(&self, input: String, regex: String) -> bool;
}
