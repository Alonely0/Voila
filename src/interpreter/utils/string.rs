pub trait Str {
    // remove leading & ending spaces
    fn trim_spaces(&self, str: &String) -> String;
}
