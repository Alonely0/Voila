pub trait Str {
    // remove leading & ending spaces
    fn trim_spaces(&self, string: &str) -> String;
}
