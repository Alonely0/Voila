pub trait RegExp {
    fn matches(&self, input: &str, regex: &str) -> bool;
}
